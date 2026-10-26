# parallel_max

Lernbeispiel für eine parallele Maximumsuche mit `crossbeam::scope`. Ein Zahlenarray wird rekursiv geteilt; Threads berechnen die Teilmaxima und führen sie zusammen. Teilarrays mit höchstens zwei Elementen werden direkt ausgewertet.

## Voraussetzungen

Rust und Cargo; keine besondere Hardware.

## Starten

Vom Repository-Stamm aus:

```sh
cd learning/parallel_max
cargo run
```

Erwartete Ausgabe:

```text
Maximum: 112332
```

## Tests

```sh
cargo test
```

Fünf Tests prüfen das Beispielarray, leere Eingaben, Einzelwerte und Paare, ausschließlich negative Zahlen sowie ungerade Längen, doppelte Maxima und die Grenzen von `i32`. Leere Eingaben liefern `None`.

## GitHub Actions

Der [Rust-Workflow](../../.github/workflows/rust.yml) prüft das Projekt bei Pushes und Pull Requests in einem eigenen Job auf Ubuntu 24.04 mit der stabilen Rust-Toolchain:

```sh
cargo fmt --check
cargo build --release
cargo test --verbose
cargo run --release
```

Nach dem Push sind die Ergebnisse im GitHub-Reiter **Actions** unter **Rust** sichtbar. Dort kann der Workflow über **Run workflow** auch manuell für einen Branch gestartet werden, sobald die Workflow-Datei im Standardbranch vorhanden ist. Der Schritt **Run example** protokolliert bei erfolgreichem Lauf `Maximum: 112332`.

## Stand und Einschränkung

Die ursprünglich falsche Assertion auf `25` wurde durch eine Ausgabe des tatsächlichen Maximums ersetzt. Ein Regressionstest sichert für das Beispielarray den Wert `112332` ab.

Das Beispiel dient zum Lernen; die vielen kurzlebigen Threads sind keine Empfehlung für eine effiziente Maximumsuche in kleinen Arrays.

## Herkunft

Hieß zuvor `application_rustigy` bzw. als Cargo-Paket `application-rustigy`. Die alte README beschrieb eine Anwendungserkennung; der vorhandene Code implementiert ausschließlich die Maximumsuche.

[Zur Projektübersicht](../../README.md)
