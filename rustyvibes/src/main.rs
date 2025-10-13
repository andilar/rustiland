// Für rustyvibes Projekt
use anyhow::Result;
use embedded_svc::http::Method;
use embedded_svc::io::Write;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, ClientConfiguration, Configuration as WifiConfig, EspWifi};
use esp_idf_sys as _; // Bindings to ESP-IDF
use log::*;
use serde::{Deserialize, Serialize};

// Datenstrukturen für API
#[derive(Serialize, Deserialize, Debug)]
struct ApiRequest {
    command: String,
    value: Option<i32>,
    data: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse {
    status: String,
    message: String,
    result: Option<serde_json::Value>,
}

// WiFi-Konfiguration
const SSID: &str = "YOUR_WIFI_SSID";
const PASSWORD: &str = "YOUR_WIFI_PASSWORD";

fn main() -> Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // WiFi initialisieren
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    connect_wifi(&mut wifi)?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("WiFi DHCP info: {:?}", ip_info);

    // HTTP Server starten
    let server_config = Configuration::default();
    let mut server = EspHttpServer::new(&server_config)?;

    // GET Endpunkt - Status abfragen
    server.fn_handler("/api/status", Method::Get, |request| {
        info!("GET /api/status");
        
        let response = ApiResponse {
            status: "ok".to_string(),
            message: "ESP32 REST API läuft".to_string(),
            result: Some(serde_json::json!({
                "uptime": esp_idf_sys::esp_timer_get_time() / 1000000,
                "free_heap": unsafe { esp_idf_sys::esp_get_free_heap_size() }
            })),
        };

        let json_response = serde_json::to_string(&response).unwrap();
        let mut response = request.into_ok_response()?;
        response.write_all(json_response.as_bytes())?;
        Ok(())
    })?;

    // POST Endpunkt - Befehle empfangen
    server.fn_handler("/api/command", Method::Post, |mut request| {
        info!("POST /api/command");

        // Request Body lesen
        let len = request.content_len().unwrap_or(0) as usize;
        let mut body = vec![0u8; len];
        request.read_exact(&mut body)?;
        
        let body_str = String::from_utf8(body).unwrap_or_default();
        info!("Received body: {}", body_str);

        // JSON parsen
        let api_request: Result<ApiRequest, _> = serde_json::from_str(&body_str);
        
        let response = match api_request {
            Ok(req) => {
                info!("Parsed request: {:?}", req);
                
                // Hier deine Geschäftslogik implementieren
                let result = process_command(req);
                
                ApiResponse {
                    status: "success".to_string(),
                    message: "Befehl erfolgreich verarbeitet".to_string(),
                    result: Some(result),
                }
            }
            Err(e) => {
                warn!("JSON parsing error: {:?}", e);
                ApiResponse {
                    status: "error".to_string(),
                    message: format!("Invalid JSON: {}", e),
                    result: None,
                }
            }
        };

        let json_response = serde_json::to_string(&response).unwrap();
        let mut response = request.into_ok_response()?;
        response.write_all(json_response.as_bytes())?;
        Ok(())
    })?;

    // PUT Endpunkt - Konfiguration aktualisieren
    server.fn_handler("/api/config", Method::Put, |mut request| {
        info!("PUT /api/config");

        let len = request.content_len().unwrap_or(0) as usize;
        let mut body = vec![0u8; len];
        request.read_exact(&mut body)?;
        
        let body_str = String::from_utf8(body).unwrap_or_default();
        
        // Hier Konfiguration verarbeiten
        let response = ApiResponse {
            status: "success".to_string(),
            message: "Konfiguration aktualisiert".to_string(),
            result: Some(serde_json::json!({"updated": true})),
        };

        let json_response = serde_json::to_string(&response).unwrap();
        let mut response = request.into_ok_response()?;
        response.write_all(json_response.as_bytes())?;
        Ok(())
    })?;

    info!("Server läuft auf http://{}/", ip_info.ip);
    
    // Server am Leben halten
    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> Result<()> {
    let wifi_configuration = WifiConfig::Client(ClientConfiguration {
        ssid: SSID.into(),
        password: PASSWORD.into(),
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;
    wifi.start()?;
    
    info!("Verbinde mit WiFi SSID: {}", SSID);
    wifi.connect()?;
    
    info!("Warte auf IP-Adresse...");
    wifi.wait_netif_up()?;
    
    info!("WiFi verbunden!");
    Ok(())
}

// Geschäftslogik für eingehende Befehle
fn process_command(request: ApiRequest) -> serde_json::Value {
    match request.command.as_str() {
        "led_on" => {
            info!("LED einschalten");
            // Hier GPIO für LED ansteuern
            serde_json::json!({"led_state": "on"})
        }
        "led_off" => {
            info!("LED ausschalten");
            // Hier GPIO für LED ansteuern
            serde_json::json!({"led_state": "off"})
        }
        "get_sensor" => {
            info!("Sensordaten lesen");
            // Hier Sensor auslesen
            serde_json::json!({
                "temperature": 23.5,
                "humidity": 65.2,
                "timestamp": esp_idf_sys::esp_timer_get_time() / 1000000
            })
        }
        "set_value" => {
            if let Some(value) = request.value {
                info!("Wert setzen: {}", value);
                serde_json::json!({"set_value": value, "success": true})
            } else {
                serde_json::json!({"error": "Kein Wert angegeben"})
            }
        }
        _ => {
            warn!("Unbekannter Befehl: {}", request.command);
            serde_json::json!({"error": "Unbekannter Befehl"})
        }
    }
}
