use actix_web::{web, App, HttpResponse, HttpServer, Result};
use std::process::Command;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;

// WAV-Datei direkt ins Binary einbetten
static EVIL_LAUGH_WAV: &[u8] = include_bytes!("../audio/evil_laugh.wav");

#[derive(Deserialize)]
struct PlayRequest {
    file: Option<String>,
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

// Hauptendpunkt zum Abspielen des eingebetteten gruseligen Lachens
async fn play_laugh() -> Result<HttpResponse> {
    // Temporäre Datei erstellen
    let temp_path = "/tmp/evil_laugh.wav";
    
    match fs::File::create(temp_path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(EVIL_LAUGH_WAV) {
                return Ok(HttpResponse::InternalServerError()
                    .body(format!("Fehler beim Schreiben: {}", e)));
            }
        }
        Err(e) => {
            return Ok(HttpResponse::InternalServerError()
                .body(format!("Fehler beim Erstellen der Datei: {}", e)));
        }
    }
    
    // WAV-Datei abspielen
    match Command::new("aplay")
        .arg(temp_path)
        .spawn()
    {
        Ok(_) => {
            println!("🎃 Spiele gruseliges Lachen ab!");
            Ok(HttpResponse::Ok()
                .body("🎃 Muahahaha!"))
        },
        Err(e) => Ok(HttpResponse::InternalServerError()
            .body(format!("Fehler: {}", e))),
    }
}

// Flexibler Endpunkt zum Abspielen beliebiger Dateien aus /home/pi/audio
async fn play_custom(info: web::Query<PlayRequest>) -> Result<HttpResponse> {
    let filename = match &info.file {
        Some(f) => f,
        None => return Ok(HttpResponse::BadRequest()
            .body("Bitte Dateinamen angeben")),
    };
    
    let wav_path = format!("/home/pi/audio/{}", filename);
    
    if !std::path::Path::new(&wav_path).exists() {
        return Ok(HttpResponse::NotFound()
            .body(format!("Datei nicht gefunden: {}", filename)));
    }
    
    match Command::new("aplay")
        .arg(&wav_path)
        .spawn()
    {
        Ok(_) => {
            println!("🎃 Spiele ab: {}", filename);
            Ok(HttpResponse::Ok()
                .body(format!("Spiele ab: {}", filename)))
        },
        Err(e) => Ok(HttpResponse::InternalServerError()
            .body(format!("Fehler: {}", e))),
    }
}

// Endpunkt zum Auflisten verfügbarer Halloween-Sounds aus /home/pi/audio
async fn list_sounds() -> Result<HttpResponse> {
    let audio_dir = "/home/pi/audio";
    
    match std::fs::read_dir(audio_dir) {
        Ok(entries) => {
            let files: Vec<String> = entries
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
            
            Ok(HttpResponse::Ok().json(files))
        }
        Err(_) => {
            // Wenn Ordner nicht existiert, leere Liste zurückgeben
            Ok(HttpResponse::Ok().json(Vec::<String>::new()))
        }
    }
}

// Weboberfläche
async fn index() -> Result<HttpResponse> {
    let html = r#"
<!DOCTYPE html>
<html lang="de">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🎃 Halloween Player</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: 'Courier New', monospace;
            background: linear-gradient(135deg, #1a0000 0%, #330000 50%, #1a0000 100%);
            color: #ff6600;
            min-height: 100vh;
            padding: 20px;
        }
        
        .container {
            max-width: 1200px;
            margin: 0 auto;
        }
        
        h1 {
            text-align: center;
            font-size: 3em;
            margin: 30px 0;
            text-shadow: 0 0 20px #ff6600, 0 0 40px #ff3300;
            animation: flicker 3s infinite;
        }
        
        @keyframes flicker {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.8; }
            75% { opacity: 0.9; }
        }
        
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }
        
        .card {
            background: rgba(0, 0, 0, 0.6);
            border: 2px solid #ff6600;
            border-radius: 10px;
            padding: 20px;
            box-shadow: 0 0 20px rgba(255, 102, 0, 0.3);
        }
        
        .card h2 {
            margin-bottom: 15px;
            color: #ff9933;
            border-bottom: 1px solid #ff6600;
            padding-bottom: 10px;
        }
        
        .status-item {
            display: flex;
            justify-content: space-between;
            padding: 8px 0;
            border-bottom: 1px solid #442200;
        }
        
        .status-label {
            color: #ff9933;
        }
        
        .status-value {
            color: #ffcc00;
            font-weight: bold;
        }
        
        button {
            background: linear-gradient(135deg, #ff6600, #ff3300);
            color: white;
            border: none;
            padding: 15px 30px;
            font-size: 1.2em;
            border-radius: 5px;
            cursor: pointer;
            font-family: 'Courier New', monospace;
            font-weight: bold;
            box-shadow: 0 0 20px rgba(255, 102, 0, 0.5);
            transition: all 0.3s;
            width: 100%;
            margin: 10px 0;
        }
        
        button:hover {
            transform: scale(1.05);
            box-shadow: 0 0 30px rgba(255, 102, 0, 0.8);
        }
        
        button:active {
            transform: scale(0.95);
        }
        
        .sound-list {
            max-height: 300px;
            overflow-y: auto;
        }
        
        .sound-item {
            background: rgba(255, 102, 0, 0.1);
            padding: 10px;
            margin: 5px 0;
            border-radius: 5px;
            cursor: pointer;
            border: 1px solid #663300;
            transition: all 0.3s;
        }
        
        .sound-item:hover {
            background: rgba(255, 102, 0, 0.2);
            border-color: #ff6600;
        }
        
        .message {
            text-align: center;
            font-size: 1.5em;
            padding: 15px;
            margin: 20px 0;
            border-radius: 5px;
            display: none;
        }
        
        .message.show {
            display: block;
            animation: fadeIn 0.5s;
        }
        
        @keyframes fadeIn {
            from { opacity: 0; transform: translateY(-20px); }
            to { opacity: 1; transform: translateY(0); }
        }
        
        .loading {
            color: #ffcc00;
        }
        
        .info-box {
            background: rgba(255, 153, 0, 0.1);
            border: 1px solid #ff9933;
            border-radius: 5px;
            padding: 10px;
            margin: 10px 0;
            font-size: 0.9em;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎃 HALLOWEEN PLAYER 🎃</h1>
        
        <div id="message" class="message"></div>
        
        <div class="grid">
            <div class="card">
                <h2>👻 Sounds abspielen</h2>
                <button onclick="playLaugh()">GRUSELIGES LACHEN (eingebettet)</button>
                <div class="info-box">
                    ⚡ Das gruselige Lachen ist direkt im Programm eingebettet!
                </div>
                <button onclick="loadSounds()">Externe Sounds aktualisieren</button>
                <div id="soundList" class="sound-list"></div>
            </div>
            
            <div class="card">
                <h2>📊 Raspberry Pi Status</h2>
                <div id="status" class="loading">Lade Status...</div>
                <button onclick="loadStatus()">Status aktualisieren</button>
            </div>
        </div>
    </div>
    
    <script>
        function showMessage(text, duration = 3000) {
            const msg = document.getElementById('message');
            msg.textContent = text;
            msg.classList.add('show');
            setTimeout(() => msg.classList.remove('show'), duration);
        }
        
        async function playLaugh() {
            try {
                const response = await fetch('/laugh');
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
        
        async function loadSounds() {
            try {
                const response = await fetch('/sounds');
                const sounds = await response.json();
                const list = document.getElementById('soundList');
                
                if (sounds.length === 0) {
                    list.innerHTML = '<div class="info-box">Keine externen WAV-Dateien in /home/pi/audio gefunden</div>';
                    return;
                }
                
                list.innerHTML = sounds.map(sound => 
                    `<div class="sound-item" onclick="playSound('${sound}')">
                        🔊 ${sound}
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
                        <span class="status-label">CPU Temperatur:</span>
                        <span class="status-value">${status.cpu_temp}</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">CPU Auslastung:</span>
                        <span class="status-value">${status.cpu_usage}</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">RAM:</span>
                        <span class="status-value">${status.memory_usage}</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">Festplatte:</span>
                        <span class="status-value">${status.disk_usage}</span>
                    </div>
                    <div class="status-item">
                        <span class="status-label">Laufzeit:</span>
                        <span class="status-value">${status.uptime}</span>
                    </div>
                `;
            } catch (error) {
                document.getElementById('status').innerHTML = 
                    '<p>❌ Fehler beim Laden des Status</p>';
            }
        }
        
        // Automatisches Laden beim Start
        loadStatus();
        loadSounds();
        
        // Status alle 5 Sekunden aktualisieren
        setInterval(loadStatus, 5000);
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
    println!("🎃 === Halloween Player Webserver === 🎃");
    println!("Server läuft auf 0.0.0.0:8080");
    println!("Weboberfläche: http://localhost:8080");
    println!("WAV-Datei ist eingebettet: {} bytes", EVIL_LAUGH_WAV.len());
    println!();
    
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/laugh", web::get().to(play_laugh))
            .route("/play", web::get().to(play_custom))
            .route("/sounds", web::get().to(list_sounds))
            .route("/status", web::get().to(get_status))
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
