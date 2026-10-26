use rust_gpiozero::*;

fn main() {
    // Create a new LED attached to Pin 17
    let mut led = LED::new(17);
    // blink the LED
    led.blink(2.0, 3.0);
}