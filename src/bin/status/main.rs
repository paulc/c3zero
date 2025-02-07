use anyhow::Result;
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};

use c3zero::rgb::Rgb;
use c3zero::status::{LedState, Status};

fn main() -> Result<()> {
    esp_idf_hal::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Starting...");

    let peripherals = Peripherals::take()?;
    // Onboard RGB LED pin
    let led = peripherals.pins.gpio10.downgrade_output();
    let channel = peripherals.rmt.channel0;

    let _status = Status::new(led, channel)?;

    loop {
        for (state, delay) in [
            (LedState::On(Rgb::new(255, 0, 0)), 500),
            (LedState::On(Rgb::new(0, 255, 0)), 500),
            (LedState::On(Rgb::new(0, 0, 255)), 500),
            (LedState::Off, 1000),
            (LedState::Flash(Rgb::new(255, 0, 0), 500), 5000),
            (LedState::Off, 1000),
            (LedState::Flash(Rgb::new(0, 255, 0), 100), 5000),
            (LedState::Off, 1000),
            (LedState::Flash(Rgb::new(0, 0, 255), 1000), 5000),
            (LedState::Off, 1000),
            (LedState::Wheel(10), 5000),
            (LedState::Off, 1000),
        ] {
            log::info!(">> {:?} [{}ms]", state, delay);
            Status::update(state)?;
            FreeRtos::delay_ms(delay);
        }
    }
}
