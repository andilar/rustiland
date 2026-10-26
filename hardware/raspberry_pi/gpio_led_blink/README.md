# gpio_led_blink

Minimales Raspberry-Pi-Beispiel mit `rust_gpiozero`: Eine LED an BCM-GPIO `17` wird mit `led.blink(2.0, 3.0)` angesteuert, also mit zwei Sekunden Ein- und drei Sekunden Aus-Zeit.

## Voraussetzungen

- Raspberry Pi mit Linux und Zugriff auf GPIO.
- Rust und Cargo auf dem Pi oder eine passende Cross-Compilation-Umgebung.
- LED mit passendem Vorwiderstand, verbunden mit BCM-GPIO 17 und GND. BCM-GPIO 17 entspricht beim üblichen 40-Pin-Header dem physischen Pin 11.

## Starten

Auf dem Raspberry Pi, vom Repository-Stamm aus:

```sh
cd hardware/raspberry_pi/gpio_led_blink
cargo run
```

Der Benutzer benötigt die für das System passenden GPIO-Zugriffsrechte.

## Stand

Siebenzeiliges Hardware-Lernbeispiel mit der älteren Bibliothek `rust_gpiozero` 0.2. Der Lebenszyklus der Blinkfunktion und die Kompatibilität mit dem verwendeten Pi-Modell müssen auf echter Hardware überprüft werden. Bisher keine eigenen Tests.

## Herkunft

Hieß zuvor `rust_gpiozero_demo`.

[Zur Projektübersicht](../../../README.md)
