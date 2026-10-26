# esp32_rest_api

ESP32-Prototyp mit WLAN-Verbindung und HTTP-API auf Basis von `esp-idf-svc`, `esp-idf-hal` und ESP-IDF. Demonstriert Statusabfragen und den Empfang von JSON-Befehlen.

## Voraussetzungen

- ESP32-Board mit bekanntem Chip und passendem USB-/Serial-Zugang.
- Zum Chip passende Rust-Toolchain und ESP-IDF-Buildumgebung.
- Flash-Werkzeug, beispielsweise `espflash`, sowie Zugang zu einem WLAN.

## Konfigurieren und bauen

Vom Repository-Stamm aus:

```sh
cd hardware/esp32/esp32_rest_api
```

In `src/main.rs` enthalten `SSID` und `PASSWORD` Platzhalter. Vor dem Einsatz eine lokale WLAN-Konfiguration ergänzen; Zugangsdaten nicht ins Repository übernehmen.

Das Projekt enthält derzeit weder einen festgelegten Target-Triple noch eine `rust-toolchain.toml`, `.cargo/config.toml` oder `build.rs`. Chipabhängige Toolchain-, Linker- und ESP-IDF-Buildkonfiguration müssen zunächst eingerichtet und auf die vorhandenen Bibliotheksversionen abgestimmt werden.

Erst nach dieser Einrichtung bauen und flashen. Beispiel für einen klassischen ESP32 mit Xtensa-Toolchain:

```sh
cargo +esp build --target xtensa-esp32-espidf
espflash flash --monitor target/xtensa-esp32-espidf/debug/esp32_rest_api
```

Für andere ESP32-Chips können Target und Toolchain abweichen. Diese Befehle sind ein bedingtes Beispiel, keine bereits validierte Build-Anleitung für ein bestimmtes Board.

## Endpunkte

Der Server verwendet die Standardkonfiguration von `EspHttpServer`; die WLAN-IP wird beim Start protokolliert.

| Methode | Pfad | Zweck |
|---|---|---|
| GET | `/api/status` | Uptime und freier Heap |
| POST | `/api/command` | JSON-Befehl verarbeiten |
| PUT | `/api/config` | Platzhalter für Konfigurationsänderungen |

Beispiel für einen Command-Body:

```json
{"command": "set_value", "value": 42}
```

## Stand und Einschränkungen

`led_on`, `led_off` und `get_sensor` liefern bisher simulierte Ergebnisse. `set_value` bestätigt den empfangenen Wert; `PUT /api/config` speichert noch keine Konfiguration. Echte GPIO-/Sensoranbindung und ein reproduzierbarer Hardware-Build fehlen.

Die Request-Handler verwenden `read_exact`; der dazugehörige `Read`-Trait ist im aktuellen Quellcode noch nicht importiert. Weitere Anpassungen an die ESP-IDF-Bibliotheksversionen sind vor dem ersten Hardwareeinsatz zu prüfen. Bisher keine eigenen Tests.

Die vorhandene VS-Code-LLDB-Konfiguration wurde auf den neuen Paketnamen angepasst. Ein Debug-Aufbau für echte ESP32-Hardware ist damit noch nicht eingerichtet.

## Herkunft

Hieß zuvor `rustyvibes`.

[Zur Projektübersicht](../../../README.md)
