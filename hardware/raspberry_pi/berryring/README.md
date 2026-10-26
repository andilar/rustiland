# berryring

Grundgerüst für ein Raspberry-Pi-Projekt mit Dashboard-Entwürfen. Das Cargo-Projekt enthält die GPIO-Bibliothek `rppal`; `src/main.rs` gibt bislang ausschließlich „Hello, world!“ aus.

## Voraussetzungen

Für die geplante GPIO-Nutzung: Raspberry Pi mit Linux, Rust und Cargo. Die `rppal`-Abhängigkeit ist auf Raspberry Pi ausgerichtet. Zum Betrachten der HTML-Entwürfe genügt ein Browser.

## Grundgerüst starten

Auf dem Zielsystem, vom Repository-Stamm aus:

```sh
cd hardware/raspberry_pi/berryring
cargo run
```

## Dashboard-Entwürfe

Die Dateien lassen sich direkt im Browser öffnen:

- [home_automation_dashboard.html](templates/home_automation_dashboard.html): Entwurf eines Raum-/Home-Automation-Dashboards.
- [overview.html](templates/overview.html): futuristischer Übersichts-Entwurf.
- [circle_data.html](templates/circle_data.html): kreisförmiger HUD-Entwurf.

## Stand

Es gibt noch keinen Rust-Webserver, keine GPIO-Steuerung und keine Verbindung der HTML-Entwürfe zum Rust-Code. Der Projektname bleibt erhalten, bis der konkrete Anwendungszweck feststeht. Bisher keine eigenen Tests.

[Zur Projektübersicht](../../../README.md)
