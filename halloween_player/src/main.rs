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
    let html = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DOORBELL CONTROL</title>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=Share+Tech+Mono&display=swap');
        
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: 'Share Tech Mono', monospace;
            background: #000000;
            color: #00ff00;
            min-height: 100vh;
            padding: 20px;
            background-image: 
                repeating-linear-gradient(0deg, rgba(0, 255, 0, 0.03) 0px, transparent 1px, transparent 2px, rgba(0, 255, 0, 0.03) 3px),
                repeating-linear-gradient(90deg, rgba(0, 255, 0, 0.03) 0px, transparent 1px, transparent 2px, rgba(0, 255, 0, 0.03) 3px);
        }
        
        .scan-line {
            position: fixed;
            top: 0;
            left: 0;
            width: 100%;
            height: 2px;
            background: rgba(0, 255, 0, 0.5);
            animation: scan 4s linear infinite;
            pointer-events: none;
            z-index: 1000;
        }
        
        @keyframes scan {
            0% { top: 0; }
            100% { top: 100%; }
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        
        .header {
            text-align: center;
            margin: 40px 0;
            border: 2px solid #00ff00;
            padding: 20px;
            background: rgba(0, 255, 0, 0.05);
            box-shadow: 0 0 20px rgba(0, 255, 0, 0.3);
        }
        
        h1 {
            font-size: 2.5em;
            letter-spacing: 8px;
            text-shadow: 0 0 10px #00ff00;
            animation: flicker 3s infinite;
        }
        
        @keyframes flicker {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.95; }
            75% { opacity: 0.98; }
        }
        
        .subtitle {
            font-size: 0.9em;
            letter-spacing: 3px;
            margin-top: 10px;
            opacity: 0.7;
        }
        
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        
        .panel {
            border: 2px solid #00ff00;
            padding: 20px;
            background: rgba(0, 255, 0, 0.02);
            box-shadow: inset 0 0 20px rgba(0, 255, 0, 0.1);
        }
        
        .panel-header {
            font-size: 1.2em;
            margin-bottom: 15px;
            padding-bottom: 10px;
            border-bottom: 1px solid #00ff00;
            letter-spacing: 2px;
        }
        
        .status-line {
            display: flex;
            justify-content: space-between;
            padding: 8px 0;
            border-bottom: 1px solid rgba(0, 255, 0, 0.2);
            font-size: 0.9em;
        }
        
        .status-label {
            opacity: 0.7;
        }
        
        .status-value {
            font-weight: bold;
            text-align: right;
        }
        
        button {
            background: transparent;
            color: #00ff00;
            border: 2px solid #00ff00;
            padding: 12px 20px;
            font-size: 1em;
            font-family: 'Share Tech Mono', monospace;
            cursor: pointer;
            transition: all 0.3s;
            width: 100%;
            margin: 8px 0;
            letter-spacing: 2px;
            text-transform: uppercase;
        }
        
        button:hover {
            background: rgba(0, 255, 0, 0.2);
            box-shadow: 0 0 15px rgba(0, 255, 0, 0.5);
        }
        
        button:active {
            background: rgba(0, 255, 0, 0.4);
        }
        
        .primary-button {
            font-size: 1.2em;
            padding: 18px 25px;
            border-width: 3px;
            animation: pulse-border 2s infinite;
        }
        
        @keyframes pulse-border {
            0%, 100% { box-shadow: 0 0 5px rgba(0, 255, 0, 0.5); }
            50% { box-shadow: 0 0 20px rgba(0, 255, 0, 0.8); }
        }
        
        .sound-list {
            max-height: 400px;
            overflow-y: auto;
            scrollbar-width: thin;
            scrollbar-color: #00ff00 #000000;
        }
        
        .sound-list::-webkit-scrollbar {
            width: 8px;
        }
        
        .sound-list::-webkit-scrollbar-track {
            background: #000000;
        }
        
        .sound-list::-webkit-scrollbar-thumb {
            background: #00ff00;
        }
        
        .sound-item {
            border: 1px solid rgba(0, 255, 0, 0.3);
            padding: 12px;
            margin: 8px 0;
            display: flex;
            justify-content: space-between;
            align-items: center;
            background: rgba(0, 255, 0, 0.02);
            transition: all 0.2s;
        }
        
        .sound-item:hover {
            background: rgba(0, 255, 0, 0.1);
            border-color: #00ff00;
        }
        
        .sound-item.selected {
            background: rgba(0, 255, 0, 0.2);
            border: 2px solid #00ff00;
            box-shadow: 0 0 10px rgba(0, 255, 0, 0.5);
        }
        
        .sound-item-name {
            flex: 1;
            font-size: 0.9em;
        }
        
        .sound-item-actions {
            display: flex;
            gap: 8px;
        }
        
        .small-button {
            padding: 6px 12px;
            font-size: 0.75em;
            width: auto;
            margin: 0;
        }
        
        .message {
            text-align: center;
            font-size: 1.1em;
            padding: 15px;
            margin: 20px 0;
            border: 2px solid #00ff00;
            background: rgba(0, 255, 0, 0.1);
            display: none;
            letter-spacing: 2px;
        }
        
        .message.show {
            display: block;
            animation: blink 0.5s;
        }
        
        @keyframes blink {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
        
        .loading {
            text-align: center;
            opacity: 0.7;
            animation: pulse 1.5s infinite;
        }
        
        @keyframes pulse {
            0%, 100% { opacity: 0.5; }
            50% { opacity: 1; }
        }
        
        .info-box {
            border: 1px solid rgba(0, 255, 0, 0.3);
            padding: 10px;
            margin: 10px 0;
            font-size: 0.85em;
            opacity: 0.8;
            background: rgba(0, 255, 0, 0.02);
        }
        
        .selection-panel {
            border: 3px solid #00ff00;
            padding: 20px;
            margin: 20px 0;
            background: rgba(0, 255, 0, 0.05);
            text-align: center;
            box-shadow: 0 0 20px rgba(0, 255, 0, 0.3);
        }
        
        .selection-display {
            font-size: 1.3em;
            margin: 15px 0;
            letter-spacing: 2px;
        }
        
        .timestamp {
            position: fixed;
            top: 10px;
            right: 10px;
            font-size: 0.8em;
            opacity: 0.5;
            letter-spacing: 1px;
        }
    </style>
</head>
<body>
    <div class="scan-line"></div>
    <div class="timestamp" id="timestamp"></div>
    
    <div class="container">
        <div class="header">
            <h1>DOORBELL</h1>
            <div class="subtitle">CONTROL SYSTEM v1.0</div>
        </div>
        
        <div id="message" class="message"></div>
        
        <div class="selection-panel">
            <div class="panel-header">[ACTIVE SOUND SELECTION]</div>
            <div class="selection-display" id="selectedSound">-- NO SOUND SELECTED --</div>
            <button class="primary-button" onclick="playSelected()">► EXECUTE PLAYBACK</button>
            <div class="info-box">
                SELECT SOUND FROM AUDIO LIBRARY BELOW<br>
                BUTTON TRIGGER WILL EXECUTE SELECTED SOUND
            </div>
        </div>
        
        <div class="grid">
            <div class="panel">
                <div class="panel-header">[AUDIO LIBRARY]</div>
                <button onclick="loadSounds()">REFRESH LIBRARY</button>
                <div id="soundList" class="sound-list"></div>
            </div>
            
            <div class="panel">
                <div class="panel-header">[SYSTEM STATUS]</div>
                <div id="status" class="loading">LOADING SYSTEM DATA...</div>
                <button onclick="loadStatus()">REFRESH STATUS</button>
            </div>
        </div>
    </div>
    
    <script>
        let currentSelectedSound = '';
        
        function updateTimestamp() {
            const now = new Date();
            const timestamp = now.toISOString().replace('T', ' ').substr(0, 19);
            document.getElementById('timestamp').textContent = timestamp;
        }
        
        setInterval(updateTimestamp, 1000);
        updateTimestamp();
        
        function showMessage(text, duration = 3000) {
            const msg = document.getElementById('message');
            msg.textContent = text;
            msg.classList.add('show');
            setTimeout(() => msg.classList.remove('show'), duration);
        }
        
        function updateSelectedDisplay() {
            document.getElementById('selectedSound').textContent = 
                currentSelectedSound || '-- NO SOUND SELECTED --';
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
                showMessage('[OK] ' + text);
                loadSounds();
            } catch (error) {
                showMessage('[ERROR] SELECTION FAILED');
            }
        }
        
        async function playSelected() {
            try {
                const response = await fetch('/play-selected');
                const text = await response.text();
                showMessage('[EXEC] ' + text);
            } catch (error) {
                showMessage('[ERROR] PLAYBACK FAILED');
            }
        }
        
        async function playSound(filename) {
            try {
                const response = await fetch(`/play?file=${encodeURIComponent(filename)}`);
                const text = await response.text();
                showMessage('[PLAY] ' + text);
            } catch (error) {
                showMessage('[ERROR] PLAYBACK FAILED');
            }
        }
        
        async function loadSelected() {
            try {
                const response = await fetch('/selected');
                const data = await response.json();
                currentSelectedSound = data.selected_sound;
                updateSelectedDisplay();
            } catch (error) {
                console.error('[ERROR] Failed to load selection:', error);
            }
        }
        
        async function loadSounds() {
            try {
                const response = await fetch('/sounds');
                const sounds = await response.json();
                const list = document.getElementById('soundList');
                
                if (sounds.length === 0) {
                    list.innerHTML = '<div class="info-box">NO WAV FILES FOUND IN /home/andilar/audio</div>';
                    return;
                }
                
                list.innerHTML = sounds.map(sound => 
                    `<div class="sound-item ${sound === currentSelectedSound ? 'selected' : ''}" id="sound-${sound}">
                        <span class="sound-item-name">> ${sound}</span>
                        <div class="sound-item-actions">
                            <button class="small-button" onclick="event.stopPropagation(); playSound('${sound}')">PLAY</button>
                            <button class="small-button" onclick="event.stopPropagation(); selectSound('${sound}')">SELECT</button>
                        </div>
                    </div>`
                ).join('');
            } catch (error) {
                document.getElementById('soundList').innerHTML = 
                    '<p>[ERROR] FAILED TO LOAD AUDIO LIBRARY</p>';
            }
        }
        
        async function loadStatus() {
            try {
                const response = await fetch('/status');
                const status = await response.json();
                
                document.getElementById('status').innerHTML = `
                    <div class="status-line">
                        <span class="status-label">HOSTNAME:</span>
                        <span class="status-value">${status.hostname}</span>
                    </div>
                    <div class="status-line">
                        <span class="status-label">CPU TEMP:</span>
                        <span class="status-value">${status.cpu_temp}</span>
                    </div>
                    <div class="status-line">
                        <span class="status-label">CPU LOAD:</span>
                        <span class="status-value">${status.cpu_usage}</span>
                    </div>
                    <div class="status-line">
                        <span class="status-label">MEMORY:</span>
                        <span class="status-value">${status.memory_usage}</span>
                    </div>
                    <div class="status-line">
                        <span class="status-label">STORAGE:</span>
                        <span class="status-value">${status.disk_usage}</span>
                    </div>
                    <div class="status-line">
                        <span class="status-label">UPTIME:</span>
                        <span class="status-value">${status.uptime}</span>
                    </div>
                `;
            } catch (error) {
                document.getElementById('status').innerHTML = 
                    '<p>[ERROR] FAILED TO LOAD SYSTEM STATUS</p>';
            }
        }
        
        // Initialize
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
    
    println!("=== DOORBELL CONTROL SYSTEM ===");
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
