use std::process::Command;
use serde_json::Value;

fn main() {
    // Run system_profiler with JSON output
    let output = Command::new("system_profiler")
        .arg("SPUSBHostDataType")
        .arg("-json")
        .output()
        .expect("Failed to run system_profiler");

    if !output.status.success() {
        eprintln!("system_profiler command failed");
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let data: Value = serde_json::from_str(&stdout).expect("Failed to parse JSON");

    if let Some(items) = data.get("SPUSBHostDataType").and_then(|v| v.as_array()) {
        println!("🔍 Detected USB connections:\n");
        for item in items {
            parse_usb_item(item, 0);
        }
    } else {
        println!("No USB data found.");
    }
}

fn parse_usb_item(item: &Value, indent: usize) {
    let prefix = " ".repeat(indent * 2);

    if let Some(name) = item.get("_name").and_then(|v| v.as_str()) {
        println!("{}🔌 {}", prefix, name);
    }

    // All possible fields that may contain link speed info on macOS
    let possible_speed_keys = [
        "speed",
        "device_speed",
        "controller_speed",
        "current_speed",
        "link_speed",
        "bus_speed",
        "USBDeviceKeyLinkSpeed", // ✅ found in your JSON
        "USBDeviceKeyCurrentSpeed",
        "USBDeviceKeyBusSpeed"
    ];

    for key in &possible_speed_keys {
        if let Some(speed) = item.get(*key).and_then(|v| v.as_str()) {
            println!("{}   📊 Data rate: {} (from \"{}\")", prefix, speed, key);
        }
    }

    // Recursively handle nested USB devices
    if let Some(children) = item.get("_items").and_then(|v| v.as_array()) {
        for child in children {
            parse_usb_item(child, indent + 1);
        }
    }
}
