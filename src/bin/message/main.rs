use anyhow::Result;
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};

fn get_free_heap_size() -> usize {
    unsafe { esp_idf_sys::xPortGetFreeHeapSize() as usize }
}

use c3zero::matrix_1d::{Orientation, Panel};
use c3zero::message::{Message, Ws2812Message};
use c3zero::rgb::{self, RgbLayout};
use c3zero::status::{LedState, Status};

fn main() -> Result<()> {
    esp_idf_hal::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Starting...");

    let peripherals = Peripherals::take()?;

    // C3-Zero onboard RGB LED pin = GPIO10
    let led = peripherals.pins.gpio10.downgrade_output();
    let channel = peripherals.rmt.channel0;
    // Create status thread
    let _status_thread = Status::new(led, channel, RgbLayout::Rgb)?;

    let pin = peripherals.pins.gpio0.downgrade_output();
    let channel = peripherals.rmt.channel1;
    let panels = [Panel::new(Orientation::East), Panel::new(Orientation::East)];
    // Create message thread
    let _message_thread = Ws2812Message::init(pin, channel, panels);

    loop {
        for (m, led, t) in [
            (
                Message::Message("Start".to_string(), rgb::GREEN),
                LedState::Flash(rgb::GREEN, 250),
                2000,
            ),
            (
                Message::Scroll("Hello".to_string(), rgb::BLUE, 1),
                LedState::Flash(rgb::BLUE, 500),
                5000,
            ),
            (
                Message::Scroll("There".to_string(), rgb::RED, 4),
                LedState::Flash(rgb::RED, 1000),
                10000,
            ),
            (Message::Off, LedState::Off, 1000),
        ] {
            Ws2812Message::<2>::update(m)?;
            Status::update(led)?;
            log::info!(">> Heap Free: {}", get_free_heap_size());
            FreeRtos::delay_ms(t);
        }
    }
}
