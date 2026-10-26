# restor

Axum-Webserver mit einer kleinen JSON-API, HTML-Startseite, HTML-Fehlerseiten und strukturiertem Logging. Nutzt Tokio, Tower-Middleware, CORS und Tracing.

## Voraussetzungen

Rust und Cargo; Port `3000` muss verfügbar sein. Aus dem Projektordner starten, da Templates und Logs über relative Pfade geladen bzw. geschrieben werden.

## Starten

Vom Repository-Stamm aus:

```sh
cd web/restor
cargo run
```

Im Browser: <http://localhost:3000/>. Der Server bindet an `0.0.0.0:3000` und ist damit bei passender Netzwerkkonfiguration auch über das Netzwerk erreichbar.

Ausführlicheres Logging:

```sh
RUST_LOG=debug cargo run
```

## Endpunkte

| Methode | Pfad | Zweck |
|---|---|---|
| GET | `/` | HTML-Startseite |
| GET | `/api/hello` | JSON-Begrüßung mit Paketversion |
| GET | `/api/health` | Health-Check |

Middleware protokolliert Anfragen und ersetzt Antworten mit Status 404, 405 und 500 durch HTML-Fehlerseiten. Die CORS-Konfiguration ist derzeit permissiv.

## Dateien

- `templates/index.html`: Startseite.
- `templates/errors/`: Fehlerseiten.
- `logs/`: täglich rotierende JSON-Logdateien; vorhandene Beispieldateien wurden bei der Umstrukturierung erhalten.

## Stand und Tests

Implementierter Web-Prototyp mit Tests für Startseite, 404-Seite und API-Endpunkte:

```sh
cargo test
```

Die JSON-Tests verwenden `ApiResponse` und `Message` zur Deserialisierung, beide Typen leiten derzeit aber nur `Serialize` ab. Die Tests benötigen daher noch eine Korrektur. Cargo-Metadaten zu Autor und Repository enthalten ebenfalls Platzhalter.

[Zur Projektübersicht](../../README.md)
