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

    let mut status = Status::new(led, channel)?;

    for (state, delay) in [
        (LedState::On(Rgb::new(255, 0, 0)), 500),
        (LedState::On(Rgb::new(0, 255, 0)), 500),
        (LedState::On(Rgb::new(0, 0, 255)), 500),
        (LedState::Off, 500),
        (LedState::Flash(Rgb::new(255, 0, 0), 500), 5000),
        (LedState::Off, 500),
        (LedState::Flash(Rgb::new(0, 255, 0), 100), 5000),
        (LedState::Off, 500),
        (LedState::Flash(Rgb::new(0, 0, 255), 1000), 5000),
        (LedState::Off, 500),
        (LedState::Wheel(10), 2000),
        (LedState::Off, 2000),
        (
            LedState::Sequence(vec![
                (Rgb::new(255, 0, 0), 250),
                (Rgb::new(0, 0, 0), 250),
                (Rgb::new(0, 255, 0), 500),
                (Rgb::new(0, 0, 0), 250),
                (Rgb::new(0, 0, 255), 1000),
                (Rgb::new(0, 0, 0), 250),
            ]),
            10000,
        ),
    ] {
        log::info!(">> {:?} [{}ms]", state, delay);
        Status::update(state)?;
        FreeRtos::delay_ms(delay);
    }

    Status::update(LedState::Sequence(vec![
        (Rgb::new(255, 0, 0), 500),
        (Rgb::new(0, 255, 0), 250),
        (Rgb::new(0, 0, 255), 250),
    ]))?;

    status.join()?;
    Ok(())
}
