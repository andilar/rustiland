use actix_web::{web, App, HttpResponse, HttpServer, Result};
use std::process::Command;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};

// WAV-Datei direkt ins Binary einbetten (optional)
static MAIN_SOUND: &[u8] = include_bytes!("../audio/main_sound.wav");

// Globaler State für den ausgewählten Sound
struct AppState {
    selected_sound: Arc<Mutex<String>>,
}

#[derive(Deserialize)]
struct PlayRequest {
    file: Option<String>,
}

#[derive(Deserialize)]
struct SelectRequest {
    file: String,
}

#[derive(Serialize)]
struct SystemStatus {
    cpu_temp: String,
    cpu_usage: String,
    memory_usage: String,
    disk_usage: String,
    uptime: String,
    hostname: String,
}

#[derive(Serialize)]
struct SelectionStatus {
    selected_sound: String,
}

// Systemstatus abrufen
async fn get_status() -> Result<HttpResponse> {
    let status = SystemStatus {
        cpu_temp: get_cpu_temp(),
        cpu_usage: get_cpu_usage(),
        memory_usage: get_memory_usage(),
        disk_usage: get_disk_usage(),
        uptime: get_uptime(),
        hostname: get_hostname(),
    };
    
    Ok(HttpResponse::Ok().json(status))
}

fn get_cpu_temp() -> String {
    fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
        .ok()
        .and_then(|s| s.trim().parse::<f32>().ok())
        .map(|t| format!("{:.1}°C", t / 1000.0))
        .unwrap_or_else(|| "N/A".to_string())
}

fn get_cpu_usage() -> String {
    Command::new("sh")
        .arg("-c")
        .arg("top -bn1 | grep 'Cpu(s)' | awk '{print $2}' | cut -d'%' -f1")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| format!("{}%", s.trim()))
        .unwrap_or_else(|| "N/A".to_string())
}

fn get_memory_usage() -> String {
    Command::new("sh")
        .arg("-c")
        .arg("free -m | awk 'NR==2{printf \"%.0f/%.0fMB (%.0f%%)\", $3,$2,$3*100/$2}'")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

fn get_disk_usage() -> String {
    Command::new("sh")
        .arg("-c")
        .arg("df -h / | awk 'NR==2{printf \"%s/%s (%s)\", $3,$2,$5}'")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

fn get_uptime() -> String {
    Command::new("uptime")
        .arg("-p")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().replace("up ", ""))
        .unwrap_or_else(|| "N/A".to_string())
}

fn get_hostname() -> String {
    Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "raspberry-pi".to_string())
}

// Hauptendpunkt zum Abspielen des eingebetteten Sounds
async fn play_main() -> Result<HttpResponse> {
    let temp_path = "/tmp/main_sound.wav";
    
    match fs::File::create(temp_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(MAIN_SOUND) {
                return Ok(HttpResponse::InternalServerError()
                    .body(format!("ERROR: WRITE FAILED: {}", e)));
            }
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError()
                .body(format!("ERROR: FILE CREATE FAILED: {}", e)));
        }
    }
    
    match Command::new("aplay")
        .arg("-D")
        .arg("plughw:1,0")
        .arg(temp_path)
        .spawn()
    {
        Ok(_) => {
            println!("[PLAY MAIN]");
            Ok(HttpResponse::Ok()
                .body("PLAYING MAIN SOUND"))
        },
        Err(e) => Ok(HttpResponse::InternalServerError()
            .body(format!("ERROR: {}", e))),
    }
}

// Sound auswählen
async fn select_sound(
    data: web::Data<AppState>,
    info: web::Json<SelectRequest>,
) -> Result<HttpResponse> {
    let wav_path = format!("/home/andilar/audio/{}", info.file);
    
    if !std::path::Path::new(&wav_path).exists() {
        return Ok(HttpResponse::NotFound()
            .body(format!("FILE NOT FOUND: {}", info.file)));
    }
    
    let mut selected = data.selected_sound.lock().unwrap();
    *selected = info.file.clone();
    
    println!("[SELECT] {}", info.file);
    Ok(HttpResponse::Ok()
        .body(format!("SELECTED: {}", info.file)))
}

// Aktuell ausgewählten Sound abrufen
async fn get_selected(data: web::Data<AppState>) -> Result<HttpResponse> {
    let selected = data.selected_sound.lock().unwrap();
    let status = SelectionStatus {
        selected_sound: selected.clone(),
    };
    Ok(HttpResponse::Ok().json(status))
}

// Ausgewählten Sound abspielen (für Taster)
async fn play_selected(data: web::Data<AppState>) -> Result<HttpResponse> {
    let selected = data.selected_sound.lock().unwrap().clone();
    
    if selected.is_empty() {
        return Ok(HttpResponse::BadRequest()
            .body("ERROR: NO SOUND SELECTED"));
    }
    
    let wav_path = format!("/home/andilar/audio/{}", selected);
    
    if !std::path::Path::new(&wav_path).exists() {
        return Ok(HttpResponse::NotFound()
            .body(format!("FILE NOT FOUND: {}", selected)));
    }
    
    match Command::new("aplay")
        .arg("-D")
        .arg("plughw:1,0")
        .arg(&wav_path)
        .spawn()
    {
        Ok(_) => {
            println!("[PLAY] {}", selected);
            Ok(HttpResponse::Ok()
                .body(format!("PLAYING: {}", selected)))
        },
        Err(e) => Ok(HttpResponse::InternalServerError()
            .body(format!("ERROR: {}", e))),
    }
}

// Beliebigen Sound direkt abspielen
async fn play_custom(info: web::Query<PlayRequest>) -> Result<HttpResponse> {
    let filename = match &info.file {
        Some(f) => f,
        None => return Ok(HttpResponse::BadRequest()
            .body("ERROR: NO FILENAME SPECIFIED")),
    };
    
    let wav_path = format!("/home/andilar/audio/{}", filename);
    
    if !std::path::Path::new(&wav_path).exists() {
        return Ok(HttpResponse::NotFound()
            .body(format!("FILE NOT FOUND: {}", filename)));
    }
    
    match Command::new("aplay")
        .arg("-D")
        .arg("plughw:1,0")
        .arg(&wav_path)
        .spawn()
    {
        Ok(_) => {
            println!("[PLAY] {}", filename);
            Ok(HttpResponse::Ok()
                .body(format!("PLAYING: {}", filename)))
        },
        Err(e) => Ok(HttpResponse::InternalServerError()
            .body(format!("ERROR: {}", e))),
    }
}

// Endpunkt zum Auflisten verfügbarer Sounds
async fn list_sounds() -> Result<HttpResponse> {
    let audio_dir = "/home/andilar/audio";
    
    match std::fs::read_dir(audio_dir) {
        Ok(entries) => {
            let mut files: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.path()
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|s| s.eq_ignore_ascii_case("wav"))
                        .unwrap_or(false)
                })
                .filter_map(|e| e.file_name().into_string().ok())
                .collect();
            
            files.sort();
            Ok(HttpResponse::Ok().json(files))
        }
        Err(_) => {
            Ok(HttpResponse::Ok().json(Vec::<String>::new()))
        }
    }
}

// Weboberfläche im Spaceship-Style
async fn index() -> Result<HttpResponse> {
    // HTML aus separater Datei einbetten (wird beim Kompilieren eingebunden)
    let html = include_str!("../templates/index.html");
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        selected_sound: Arc::new(Mutex::new(String::new())),
    });
    
    println!("=== DOORBELL CONTROL SYSTEM ===");
    println!("Server running on 0.0.0.0:8080");
    println!("Web interface: http://localhost:8080");
    println!();
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route("/play-main", web::get().to(play_main))
            .route("/laugh", web::get().to(play_main))
            .route("/select", web::post().to(select_sound))
            .route("/selected", web::get().to(get_selected))
            .route("/play-selected", web::get().to(play_selected))
            .route("/play", web::get().to(play_custom))
            .route("/sounds", web::get().to(list_sounds))
            .route("/status", web::get().to(get_status))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
