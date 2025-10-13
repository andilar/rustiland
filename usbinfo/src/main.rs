use std::process::Command;
use serde_json::Value;

fn main() {
    // system_profiler mit JSON-Ausgabe
    let output = Command::new("system_profiler")
        .arg("SPUSBHostDataType")
        .arg("-json")
        .output()
        .expect("Fehler beim Ausführen von system_profiler");

    if !output.status.success() {
        eprintln!("system_profiler fehlgeschlagen");
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // JSON parsen
    let data: Value = serde_json::from_str(&stdout).expect("Fehler beim Parsen des JSON");

    // Zugriff auf das USB-Array
    if let Some(items) = data.get("SPUSBHostDataType").and_then(|v| v.as_array()) {
        println!("Gefundene USB-Geräte:");
        for item in items {
            parse_usb_item(item, 0);
        }
    } else {
        println!("Keine USB-Daten gefunden.");
    }
}

fn parse_usb_item(item: &Value, indent: usize) {
    let prefix = " ".repeat(indent * 2);

    if let Some(name) = item.get("_name").and_then(|v| v.as_str()) {
        println!("{}🔌 {}", prefix, name);
    }

    if let Some(speed) = item.get("speed").and_then(|v| v.as_str()) {
        println!("{}   📊 Geschwindigkeit: {}", prefix, speed);
    }

    // Wenn das Gerät untergeordnete USB-Objekte hat
    if let Some(children) = item.get("_items").and_then(|v| v.as_array()) {
        for child in children {
            parse_usb_item(child, indent + 1);
        }
    }
}
