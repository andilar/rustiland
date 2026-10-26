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

## Stand und bekannte Einschränkung

Der Algorithmus ist implementiert, aber das Beispiel endet derzeit mit einer fehlgeschlagenen Assertion: Das enthaltene Array hat das Maximum `112332`, die Assertion erwartet `25`. Der Startbefehl demonstriert daher aktuell diesen Fehler. Für die Umstrukturierung wurde das Verhalten beibehalten.

Das Beispiel dient zum Lernen; die vielen kurzlebigen Threads sind keine Empfehlung für eine effiziente Maximumsuche in kleinen Arrays. Eigene Tests fehlen bislang.

## Herkunft

Hieß zuvor `application_rustigy` bzw. als Cargo-Paket `application-rustigy`. Die alte README beschrieb eine Anwendungserkennung; der vorhandene Code implementiert ausschließlich die Maximumsuche.

[Zur Projektübersicht](../../README.md)
