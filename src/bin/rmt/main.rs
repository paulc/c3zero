use anyhow::Result;
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};

use c3zero::rgb::Rgb;
use c3zero::status_led::{LedState, StatusLed};
//use c3zero::ws2812_rmt::Ws2812Rmt;

pub fn main() -> Result<()> {
    esp_idf_hal::sys::link_patches();

    let peripherals = Peripherals::take()?;
    // Onboard RGB LED pin
    let led = peripherals.pins.gpio10.downgrade_output();
    let channel = peripherals.rmt.channel0;

    let _status = StatusLed::new(led, channel)?;

    loop {
        StatusLed::update(LedState::On(Rgb::new(255, 0, 0)));
        FreeRtos::delay_ms(500);
        StatusLed::update(LedState::On(Rgb::new(0, 255, 0)));
        FreeRtos::delay_ms(500);
        StatusLed::update(LedState::On(Rgb::new(0, 0, 255)));
        FreeRtos::delay_ms(500);
        StatusLed::update(LedState::Off);
        FreeRtos::delay_ms(2000);
    }

    /*
    let mut status_led = Ws2812Rmt::new(led, channel)?;
    // infinite rainbow loop at 20% brightness
    (0..360).cycle().step_by(5).try_for_each(|hue| {
        FreeRtos::delay_ms(1);
        let rgb = Rgb::from_hsv(hue, 100, 20)?;
        status_led.set(rgb)
    })
    */
}
