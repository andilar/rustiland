# macos_usb_info

Kleine macOS-CLI zur Anzeige erkannter USB-Geräte und vorhandener Geschwindigkeitsinformationen. Ruft `system_profiler SPUSBHostDataType -json` auf und liest die verschachtelten Gerätedaten mit `serde_json`.

## Voraussetzungen

macOS mit `system_profiler`, Rust und Cargo. Der eigentliche Programmablauf ist macOS-spezifisch.

## Starten

Vom Repository-Stamm aus:

```sh
cd tools/macos_usb_info
cargo run
```

Die Ausgabe enthält Gerätenamen und die vom System gelieferten Felder zu Verbindungsgeschwindigkeiten. Verschachtelte Geräte werden eingerückt dargestellt. Der Aufruf von `system_profiler` kann etwas Zeit beanspruchen.

## Stand

Implementiertes CLI-Werkzeug ohne Kommandozeilenoptionen oder eigene Tests. Die verfügbaren JSON-Felder hängen von macOS und den Geräten ab. Angezeigte Geschwindigkeiten sind Systeminformationen, keine gemessene Übertragungsleistung. Fehlt `system_profiler` oder ist das JSON ungültig, bricht das Programm derzeit mit einer Fehlermeldung ab.

## Herkunft

Hieß zuvor `usbinfo`.

[Zur Projektübersicht](../../README.md)
