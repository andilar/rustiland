# rustiland

Sammelbecken kleiner Rust-Projekte: Lernbeispiele, Webserver, Hardware-Prototypen und Systemwerkzeuge. Die Projekte sind nach Thema gruppiert; Hardware-Projekte zusätzlich nach Plattform.

## Projekte

| Bereich | Projekt | Zweck | Plattform | Stand |
|---|---|---|---|---|
| Lernen | [hello_ferris](learning/hello_ferris/README.md) | Ferris-Begrüßung in der Konsole | Desktop | Lernbeispiel |
| Lernen | [parallel_max](learning/parallel_max/README.md) | Parallele Maximumsuche mit Crossbeam | Desktop | Lernbeispiel mit Regressionstest und Randfalltests |
| Web | [http_server_from_scratch](web/http_server_from_scratch/README.md) | HTTP-Server ohne Framework | Desktop/Linux | Lernserver |
| Web | [restor](web/restor/README.md) | Axum-API, HTML-Seiten und Logging | Desktop/Linux | Web-Prototyp; JSON-Tests benötigen Korrektur |
| Hardware | [gpio_led_blink](hardware/raspberry_pi/gpio_led_blink/README.md) | LED über GPIO blinken lassen | Raspberry Pi/Linux | Hardware-Lernbeispiel |
| Hardware | [halloween_player](hardware/raspberry_pi/halloween_player/README.md) | Soundplayer und Türklingel mit Weboberfläche | Raspberry Pi/Linux | Hardware-Prototyp |
| Hardware | [berryring](hardware/raspberry_pi/berryring/README.md) | Projektgerüst mit Dashboard-Entwürfen | Raspberry Pi/Linux geplant | Grundgerüst |
| Hardware | [esp32_rest_api](hardware/esp32/esp32_rest_api/README.md) | WLAN und HTTP-API auf ESP-IDF | ESP32 | Prototyp; Buildkonfiguration offen |
| Werkzeuge | [macos_usb_info](tools/macos_usb_info/README.md) | USB-Geräte und Verbindungsinformationen anzeigen | macOS | CLI-Werkzeug |

Die Statusangaben beschreiben den vorhandenen Code. Hardwarefunktionen müssen auf dem jeweiligen Zielsystem geprüft werden.

## Ordnerstruktur

```text
rustiland/
├── learning/
│   ├── hello_ferris/
│   └── parallel_max/
├── web/
│   ├── http_server_from_scratch/
│   └── restor/
├── hardware/
│   ├── raspberry_pi/
│   │   ├── gpio_led_blink/
│   │   ├── halloween_player/
│   │   └── berryring/
│   └── esp32/
│       └── esp32_rest_api/
└── tools/
    └── macos_usb_info/
```

## Arbeiten mit den Projekten

Jedes Projekt hat eine eigene `Cargo.toml` und wird eigenständig gebaut. Es gibt keinen gemeinsamen Cargo-Workspace, da Plattformen und Toolchains unterschiedlich sind.

Beispiel, vom Repository-Stamm aus:

```sh
cd learning/hello_ferris
cargo run
```

Die jeweilige Projekt-README beschreibt Voraussetzungen, Startbefehle und Einschränkungen. Insbesondere `restor` muss aus seinem Projektordner gestartet werden, damit seine relativen Template-Pfade stimmen. Raspberry-Pi- und ESP32-Projekte benötigen ihre jeweilige Zielumgebung.

## Konventionen

- Projektordner und Cargo-Paketnamen verwenden `snake_case`: Kleinbuchstaben und Unterstriche, keine Bindestriche.
- Themenordner beschreiben den Bereich; Projektnamen beschreiben den Zweck oder behalten einen etablierten Eigennamen.
- Jedes Projekt erhält eine README mit Zweck, Voraussetzungen, Startanleitung und Stand.
- Neue Projekte werden in der Tabelle oben verlinkt.
- Der Entwicklungsstand wird dokumentiert; die thematische Zuordnung bleibt bei Fortschritten bestehen.

## Frühere Namen

| Bisher | Neuer Pfad und Paketname |
|---|---|
| `hello_cargo` | `learning/hello_ferris` |
| `application_rustigy` / `application-rustigy` | `learning/parallel_max` |
| `server` | `web/http_server_from_scratch` |
| `restor` | `web/restor` |
| `rust_gpiozero_demo` | `hardware/raspberry_pi/gpio_led_blink` |
| `halloween_player` | `hardware/raspberry_pi/halloween_player` |
| `berryring` | `hardware/raspberry_pi/berryring` |
| `rustyvibes` | `hardware/esp32/esp32_rest_api` |
| `usbinfo` | `tools/macos_usb_info` |

## Automatische Prüfung

[GitHub Actions](.github/workflows/rust.yml) führt bei Pushes und Pull Requests zwei unabhängige Jobs auf Ubuntu mit der stabilen Rust-Toolchain aus:

- `http_server_from_scratch`: Release-Build und Tests.
- `parallel_max`: Formatprüfung, Release-Build, fünf Tests und ein Probelauf des Beispiels.

Der Workflow lässt sich außerdem im GitHub-Reiter **Actions** über **Rust → Run workflow** manuell starten. Die übrigen Projekte sind derzeit nicht Teil dieser CI-Prüfung.
