use anyhow::Result;
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};

use c3zero::rgb::Rgb;
use c3zero::ws2812_rmt::Ws2812Rmt;

pub fn main() -> Result<()> {
    esp_idf_hal::sys::link_patches();

    let peripherals = Peripherals::take()?;
    // Onboard RGB LED pin
    let led = peripherals.pins.gpio10.downgrade_output();
    let channel = peripherals.rmt.channel0;

    let mut ws2812 = Ws2812Rmt::new(led, channel)?;

    loop {
        ws2812.set(Rgb::new(255, 0, 0))?;
        FreeRtos::delay_ms(500);
        ws2812.set(Rgb::new(0, 255, 0))?;
        FreeRtos::delay_ms(500);
        ws2812.set(Rgb::new(0, 0, 255))?;
        FreeRtos::delay_ms(500);
        ws2812.set(Rgb::new(0, 0, 0))?;
        FreeRtos::delay_ms(2000);
    }
}
