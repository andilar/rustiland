# halloween_player

Webgesteuerter Soundplayer für eine Halloween-Installation oder Türklingel auf dem Raspberry Pi. Die Actix-Web-Oberfläche bietet Soundauswahl, WAV-Upload, Wiedergabe und Systemstatus.

## Voraussetzungen

- Raspberry Pi oder passendes Linux-System mit Rust und Cargo.
- ALSA-Werkzeug `aplay` und ein angeschlossenes Audiogerät.
- Linux-Werkzeuge `sh`, `top`, `free`, `df`, `uptime` und `hostname` für den Systemstatus.
- Port `8080` und ein beschreibbares Audioverzeichnis `/home/andilar/audio`.

Die CPU-Temperatur wird aus `/sys/class/thermal/thermal_zone0/temp` gelesen. Für fehlende Statusdaten gibt es Fallbacks.

## Vorbereiten und starten

Vom Repository-Stamm aus, auf dem Zielsystem:

```sh
cd hardware/raspberry_pi/halloween_player
mkdir -p /home/andilar/audio
cp audio/*.wav /home/andilar/audio/
cargo run
```

Der Benutzer benötigt Schreibrechte für das Audioverzeichnis. Im Browser: <http://localhost:8080/> oder die IP-Adresse des Raspberry Pi mit Port 8080. Der Server bindet an `0.0.0.0:8080`.

Die Wiedergabe verwendet fest `aplay -D plughw:1,0`. Mit `aplay -l` prüfen, ob dieses Gerät zum eigenen Aufbau passt, und gegebenenfalls die Geräteangabe in `src/main.rs` anpassen.

## Funktionen und Endpunkte

| Methode | Pfad | Zweck |
|---|---|---|
| GET | `/` | Weboberfläche |
| GET | `/play-main`, `/ring` | Eingebetteten Hauptsound abspielen |
| POST | `/select` | Sound auswählen; JSON mit Feld `file` |
| GET | `/selected` | Aktuelle Auswahl lesen |
| GET | `/play-selected` | Ausgewählten Sound abspielen |
| GET | `/play?file=DATEI.wav` | WAV-Datei direkt abspielen |
| GET | `/sounds` | WAV-Dateien im Audioverzeichnis auflisten |
| GET | `/status` | Systemstatus abrufen |
| POST | `/upload` | WAV-Datei per Multipart hochladen |

## Dateien und Stand

`audio/main_sound.wav` und `templates/index.html` werden beim Kompilieren eingebettet. Andere Sounds werden aus dem fest eingestellten Verzeichnis `/home/andilar/audio` geladen. Der eingebettete Hauptsound wird zur Wiedergabe nach `/tmp/main_sound.wav` geschrieben. Die mitgelieferte MP3-Datei wird von der WAV-Soundliste nicht berücksichtigt.

Implementierter Hardware-Prototyp; Audiogerät und Pfade sind noch fest im Code hinterlegt. Bisher keine eigenen Tests. Die Wiedergabe muss auf dem Zielsystem geprüft werden.

[Zur Projektübersicht](../../../README.md)
