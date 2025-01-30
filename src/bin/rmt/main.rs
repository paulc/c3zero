use anyhow::Result;
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};

use c3zero::rgb::Rgb;
use c3zero::status_led::Ws2812Rmt;

pub fn main() -> Result<()> {
    esp_idf_hal::sys::link_patches();

    let peripherals = Peripherals::take()?;
    // Onboard RGB LED pin
    let led = peripherals.pins.gpio10.downgrade_output();
    let channel = peripherals.rmt.channel0;
    let mut status_led = Ws2812Rmt::new(led, channel)?;

    // infinite rainbow loop at 20% brightness
    (0..360).cycle().try_for_each(|hue| {
        FreeRtos::delay_ms(2);
        let rgb = Rgb::from_hsv(hue, 100, 20)?;
        status_led.set(rgb)
    })
}
