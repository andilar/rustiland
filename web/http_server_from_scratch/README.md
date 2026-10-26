# http_server_from_scratch

HTTP-Lernserver ohne Webframework und ohne externe Cargo-Abhängigkeiten. Verwendet `std::net::TcpListener`, verarbeitet HTTP/1.1-Request-Zeilen, Methoden und Query-Strings und liefert Dateien aus `public/` aus.

## Voraussetzungen

Rust und Cargo; Port `8080` muss lokal verfügbar sein.

## Starten

Vom Repository-Stamm aus:

```sh
cd web/http_server_from_scratch
cargo run
```

Im Browser: <http://127.0.0.1:8080/>; die Beispielseite liegt unter <http://127.0.0.1:8080/hello>.

Das Standardverzeichnis für Webseiten wird aus `CARGO_MANIFEST_DIR` abgeleitet. Ein alternatives Verzeichnis lässt sich als absoluter Pfad setzen:

```sh
PUBLIC_PATH=/absoluter/pfad/zu/webseiten cargo run
```

## Aufbau

- `src/server.rs`: TCP-Verbindungen und Verarbeitungsschleife.
- `src/http/`: Requests, Responses, Methoden, Statuscodes und Query-Strings.
- `src/website_handler.rs`: Zuordnung von Anfragen zu Dateien.
- `public/`: Beispielseiten und CSS.

## Stand

Lernimplementierung mit sequenzieller Verarbeitung und einem festen Lesepuffer von 1024 Bytes. Kein vollständiger HTTP-Server für den Produktivbetrieb. Bisher keine eigenen Tests. Der GitHub-Actions-Workflow baut und testet dieses Projekt auf Ubuntu.

## Prüfen

```sh
cargo build --release
cargo test --verbose
```

## Herkunft

Hieß zuvor `server`.

[Zur Projektübersicht](../../README.md)
