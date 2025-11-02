use actix_web::{web, App, HttpResponse, HttpServer, Result};
use std::process::Command;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::sync::{Arc, Mutex};

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

// Sound auswählen
async fn select_sound(
    data: web::Data<AppState>,
    info: web::Json<SelectRequest>,
) -> Result<HttpResponse> {
    let wav_path = format!("/home/andilar/audio/{}", info.file);
    
    if !std::path::Path::new(&wav_path).exists() {
        return Ok(HttpResponse::NotFound()
            .body(format!("Datei nicht gefunden: {}", info.file)));
    }
    
    let mut selected = data.selected_sound.lock().unwrap();
    *selected = info.file.clone();
    
    println!("✓ Sound ausgewählt: {}", info.file);
    Ok(HttpResponse::Ok()
        .body(format!("Sound ausgewählt: {}", info.file)))
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
            .body("Kein Sound ausgewählt! Bitte wähle zuerst einen Sound aus."));
    }
    
    let wav_path = format!("/home/andilar/audio/{}", selected);
    
    if !std::path::Path::new(&wav_path).exists() {
        return Ok(HttpResponse::NotFound()
            .body(format!("Datei nicht gefunden: {}", selected)));
    }
    
    match Command::new("aplay")
        .arg("-D")
        .arg("plughw:1,0")
        .arg(&wav_path)
        .spawn()
    {
        Ok(_) => {
            println!("⚡ Spiele ausgewählten Sound ab: {}", selected);
            Ok(HttpResponse::Ok()
                .body(format!("⚡ Spiele ab: {}", selected)))
        },
        Err(e) => Ok(HttpResponse::InternalServerError()
            .body(format!("Fehler: {}", e))),
    }
}

// Beliebigen Sound direkt abspielen
async fn play_custom(info: web::Query<PlayRequest>) -> Result<HttpResponse> {
    let filename = match &info.file {
        Some(f) => f,
        None => return Ok(HttpResponse::BadRequest()
            .body("Bitte Dateinamen angeben")),
    };
    
    let wav_path = format!("/home/andilar/audio/{}", filename);
    
    if !std::path::Path::new(&wav_path).exists() {
        return Ok(HttpResponse::NotFound()
            .body(format!("Datei nicht gefunden: {}", filename)));
    }
    
    match Command::new("aplay")
        .arg("-D")
        .arg("plughw:1,0")
        .arg(&wav_path)
        .spawn()
    {
        Ok(_) => {
            println!("⚡ Spiele ab: {}", filename);
            Ok(HttpResponse::Ok()
                .body(format!("Spiele ab: {}", filename)))
        },
        Err(e) => Ok(HttpResponse::InternalServerError()
            .body(format!("Fehler: {}", e))),
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

// Weboberfläche im Avengers-Style
async fn index() -> Result<HttpResponse> {
    let html = r#"
<!DOCTYPE html>
<html lang="de">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>⚡ AVENGERS SOUNDBOARD</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: 'Arial', sans-serif;
            background: linear-gradient(135deg, #0a0e27 0%, #1a1f3a 50%, #0a0e27 100%);
            color: #ffffff;
            min-height: 100vh;
            padding: 20px;
            position: relative;
            overflow-x: hidden;
        }
        
        body::before {
            content: '';
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: 
                radial-gradient(circle at 20% 50%, rgba(220, 20, 60, 0.1) 0%, transparent 50%),
                radial-gradient(circle at 80% 50%, rgba(0, 150, 255, 0.1) 0%, transparent 50%);
            pointer-events: none;
            z-index: 0;
        }
        
        .container {
            max-width: 1400px;
            margin: 0 auto;
            position: relative;
            z-index: 1;
        }
        
        h1 {
            text-align: center;
            font-size: 3.5em;
            margin: 30px 0;
            background: linear-gradient(90deg, #dc143c, #4169e1, #ffd700);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            font-weight: 900;
            letter-spacing: 3px;
            text-shadow: 0 0 30px rgba(220, 20, 60, 0.5);
            animation: pulse 3s ease-in-out infinite;
        }
        
        @keyframes pulse {
            0%, 100% { transform: scale(1); }
            50% { transform: scale(1.02); }
        }
        
        .subtitle {
            text-align: center;
            font-size: 1.2em;
            color: #4169e1;
            margin-bottom: 40px;
            letter-spacing: 2px;
        }
        
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
            gap: 25px;
            margin-bottom: 30px;
        }
        
        .card {
            background: linear-gradient(135deg, rgba(25, 30, 60, 0.9), rgba(15, 20, 45, 0.9));
            border: 2px solid rgba(65, 105, 225, 0.3);
            border-radius: 15px;
            padding: 25px;
            box-shadow: 
                0 8px 32px rgba(0, 0, 0, 0.4),
                inset 0 1px 0 rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(10px);
            transition: all 0.3s ease;
        }
        
        .card:hover {
            transform: translateY(-5px);
            border-color: rgba(65, 105, 225, 0.6);
            box-shadow: 
                0 12px 40px rgba(65, 105, 225, 0.3),
                inset 0 1px 0 rgba(255, 255, 255, 0.1);
        }
        
        .card h2 {
            margin-bottom: 20px;
            color: #4169e1;
            border-bottom: 2px solid rgba(65, 105, 225, 0.3);
            padding-bottom: 12px;
            font-size: 1.5em;
            display: flex;
            align-items: center;
            gap: 10px;
        }
        
        .status-item {
            display: flex;
            justify-content: space-between;
            padding: 12px 0;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        }
        
        .status-label {
            color: #a8b8d8;
            font-weight: 500;
        }
        
        .status-value {
            color: #4169e1;
            font-weight: bold;
            font-family: 'Courier New', monospace;
        }
        
        button {
            background: linear-gradient(135deg, #dc143c, #b8112e);
            color: white;
            border: none;
            padding: 18px 35px;
            font-size: 1.1em;
            border-radius: 10px;
            cursor: pointer;
            font-weight: bold;
            box-shadow: 
                0 5px 20px rgba(220, 20, 60, 0.4),
                inset 0 1px 0 rgba(255, 255, 255, 0.2);
            transition: all 0.3s ease;
            width: 100%;
            margin: 10px 0;
            text-transform: uppercase;
            letter-spacing: 1px;
            position: relative;
            overflow: hidden;
        }
        
        button::before {
            content: '';
            position: absolute;
            top: 50%;
            left: 50%;
            width: 0;
            height: 0;
            border-radius: 50%;
            background: rgba(255, 255, 255, 0.2);
            transform: translate(-50%, -50%);
            transition: width 0.6s, height 0.6s;
        }
        
        button:hover::before {
            width: 300px;
            height: 300px;
        }
        
        button:hover {
            transform: translateY(-2px);
            box-shadow: 
                0 8px 30px rgba(220, 20, 60, 0.6),
                inset 0 1px 0 rgba(255, 255, 255, 0.3);
        }
        
        button:active {
            transform: translateY(0);
        }
        
        .primary-button {
            background: linear-gradient(135deg, #4169e1, #1e3a8a);
            font-size: 1.3em;
            padding: 22px 40px;
        }
        
        .primary-button:hover {
            box-shadow: 
                0 8px 30px rgba(65, 105, 225, 0.6),
                inset 0 1px 0 rgba(255, 255, 255, 0.3);
        }
        
        .sound-list {
            max-height: 350px;
            overflow-y: auto;
            scrollbar-width: thin;
            scrollbar-color: #4169e1 rgba(255, 255, 255, 0.1);
        }
        
        .sound-list::-webkit-scrollbar {
            width: 8px;
        }
        
        .sound-list::-webkit-scrollbar-track {
            background: rgba(255, 255, 255, 0.05);
            border-radius: 4px;
        }
        
        .sound-list::-webkit-scrollbar-thumb {
            background: #4169e1;
            border-radius: 4px;
        }
        
        .sound-item {
            background: linear-gradient(135deg, rgba(65, 105, 225, 0.1), rgba(65, 105, 225, 0.05));
            padding: 15px;
            margin: 8px 0;
            border-radius: 8px;
            cursor: pointer;
            border: 1px solid rgba(65, 105, 225, 0.2);
            transition: all 0.3s ease;
            display: flex;
            align-items: center;
            gap: 10px;
            justify-content: space-between;
        }
        
        .sound-item:hover {
            background: linear-gradient(135deg, rgba(65, 105, 225, 0.2), rgba(65, 105, 225, 0.1));
            border-color: rgba(65, 105, 225, 0.5);
            transform: translateX(5px);
        }
        
        .sound-item.selected {
            background: linear-gradient(135deg, rgba(255, 215, 0, 0.3), rgba(220, 20, 60, 0.2));
            border: 2px solid #ffd700;
            transform: translateX(5px);
        }
        
        .sound-item-name {
            flex: 1;
        }
        
        .sound-item-actions {
            display: flex;
            gap: 8px;
        }
        
        .small-button {
            padding: 8px 15px;
            font-size: 0.85em;
            width: auto;
            margin: 0;
        }
        
        .message {
            text-align: center;
            font-size: 1.3em;
            padding: 20px;
            margin: 20px 0;
            border-radius: 10px;
            background: linear-gradient(135deg, rgba(65, 105, 225, 0.2), rgba(220, 20, 60, 0.2));
            border: 1px solid rgba(65, 105, 225, 0.3);
            display: none;
            backdrop-filter: blur(10px);
        }
        
        .message.show {
            display: block;
            animation: slideIn 0.5s ease;
        }
        
        @keyframes slideIn {
            from { 
                opacity: 0; 
                transform: translateY(-30px);
            }
            to { 
                opacity: 1; 
                transform: translateY(0);
            }
        }
        
        .loading {
            color: #4169e1;
            text-align: center;
        }
        
        .info-box {
            background: rgba(65, 105, 225, 0.1);
            border: 1px solid rgba(65, 105, 225, 0.3);
            border-radius: 8px;
            padding: 12px;
            margin: 10px 0;
            font-size: 0.9em;
            color: #a8b8d8;
        }
        
        .selection-box {
            background: linear-gradient(135deg, rgba(255, 215, 0, 0.2), rgba(220, 20, 60, 0.1));
            border: 2px solid #ffd700;
            border-radius: 10px;
            padding: 15px;
            margin: 15px 0;
            text-align: center;
            font-size: 1.1em;
            font-weight: bold;
        }
        
        .hero-section {
            text-align: center;
            margin: 40px 0;
            padding: 40px;
            background: linear-gradient(135deg, rgba(220, 20, 60, 0.1), rgba(65, 105, 225, 0.1));
            border-radius: 20px;
            border: 2px solid rgba(65, 105, 225, 0.2);
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>⚡ AVENGERS SOUNDBOARD ⚡</h1>
        <div class="subtitle">EARTH'S MIGHTIEST AUDIO SYSTEM</div>
        
        <div id="message" class="message"></div>
        
        <div class="hero-section">
            <div class="selection-box">
                🎯 Ausgewählt für Taster: <span id="selectedSound">Kein Sound ausgewählt</span>
            </div>
            <button class="primary-button" onclick="playSelected()">▶ AUSGEWÄHLTEN SOUND ABSPIELEN</button>
            <div class="info-box">
                💡 Wähle einen Sound aus der Liste und klicke auf "Auswählen". 
                Dieser Sound wird dann beim Taster-Druck abgespielt!
            </div>
        </div>
        
        <div class="grid">
            <div class="card">
                <h2>🔊 Sound Library</h2>
                <button onclick="loadSounds()">Sounds neu laden</button>
                <div id="soundList" class="sound-list"></div>
            </div>
            
            <div class="card">
                <h2>📊 System Status</h2>
                <div id="status" class="loading">Lade Status...</div>
                <button onclick="loadStatus()">Status aktualisieren</button>
            </div>
        </div>
    </div>
    
    <script>
        let currentSelectedSound = '';
        
        function showMessage(text, duration = 3000) {
            const msg = document.getElementById('message');
            msg.textContent = text;
            msg.classList.add('show');
            setTimeout(() => msg.classList.remove('show'), duration);
        }
        
        function updateSelectedDisplay() {
            document.getElementById('selectedSound').textContent = 
                currentSelectedSound || 'Kein Sound ausgewählt';
        }
        
        async function selectSound(filename) {
            try {
                const response = await fetch('/select', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({ file: filename })
                });
                const text = await response.text();
                currentSelectedSound = filename;
                updateSelectedDisplay();
                showMessage('✓ ' + text);
                loadSounds(); // Neu laden um Auswahl hervorzuheben
            } catch (error) {
                showMessage('❌ Fehler beim Auswählen!');
            }
        }
        
        async function playSelected() {
            try {
                const response = await fetch('/play-selected');
                const text = await response.text();
                showMessage(text);
            } catch (error) {
                showMessage('❌ Fehler beim Abspielen!');
            }
        }
        
        async function playSound(filename) {
            try {
                const response = await fetch(`/play?file=${encodeURIComponent(filename)}`);
                const text = await response.text();
                showMessage(text);
            } catch (error) {
                showMessage('❌ Fehler beim Abspielen!');
            }
        }
        
        async function loadSelected() {
            try {
                const response = await fetch('/selected');
                const data = await response.json();
                currentSelectedSound = data.selected_sound;
                updateSelectedDisplay();
            } catch (error) {
                console.error('Fehler beim Laden der Auswahl:', error);
            }
        }
        
        async function loadSounds() {
            try {
                const response = await fetch('/sounds');
                const sounds = await response.json();
                const list = document.getElementById('soundList');
                
                if (sounds.length === 0) {
                    list.innerHTML = '<div class="info-box">Keine WAV-Dateien in /home/andilar/audio gefunden</div>';
                    return;
                }
                
                list.innerHTML = sounds.map(sound => 
                    `<div class="sound-item ${sound === currentSelectedSound ? 'selected' : ''}" id="sound-${sound}">
                        <span class="sound-item-name">🎵 ${sound}</span>
                        <div class="sound-item-actions">
                            <button class="small-button" onclick="event.stopPropagation(); playSound('${sound}')">▶ Play</button>
                            <button class="small-button" onclick="event.stopPropagation(); selectSound('${sound}')">✓ Auswählen</button>
                        </div>
                    </div>`
                ).join('');
            } catch (error) {
                document.getElementById('soundList').innerHTML = 
                    '<p>❌ Fehler beim Laden der Sounds</p>';
            }
        }
        
        async function loadStatus() {
            try {
                const response = await fetch('/status');
                const status = await response.json();
                
                document.getElementById('status').innerHTML = `
                    <div class="status-item">
                        <span class="status-label">Hostname:</span>
                        <span class="status-value">${status.hostname}</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">CPU Temp:</span>
                        <span class="status-value">${status.cpu_temp}</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">CPU Usage:</span>
                        <span class="status-value">${status.cpu_usage}</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">RAM:</span>
                        <span class="status-value">${status.memory_usage}</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">Disk:</span>
                        <span class="status-value">${status.disk_usage}</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">Uptime:</span>
                        <span class="status-value">${status.uptime}</span>
                    </div>
                `;
            } catch (error) {
                document.getElementById('status').innerHTML = 
                    '<p>❌ Fehler beim Laden des Status</p>';
            }
        }
        
        // Auto-load on start
        loadSelected();
        loadStatus();
        loadSounds();
        
        // Auto-refresh
        setInterval(loadStatus, 5000);
        setInterval(loadSelected, 2000);
    </script>
</body>
</html>
    "#;
    
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        selected_sound: Arc::new(Mutex::new(String::new())),
    });
    
    println!("⚡ === AVENGERS SOUNDBOARD === ⚡");
    println!("Server running on 0.0.0.0:8080");
    println!("Web interface: http://localhost:8080");
    println!();
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
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
