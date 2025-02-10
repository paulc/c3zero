use anyhow::Result;
use esp_idf_hal::rmt::{config::TransmitConfig, TxRmtDriver};
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};

use c3zero::rgb::{Rgb, RgbLayout};
use c3zero::ws2812_rmt::{Ws2812Rmt, Ws2812RmtSingle};

#[cfg(feature = "led_128")]
const LEDS: usize = 128;
#[cfg(not(feature = "led_128"))]
const LEDS: usize = 64;

fn main() -> Result<()> {
    esp_idf_hal::sys::link_patches();

    let peripherals = Peripherals::take()?;

    // C3-Zero onboard RGB LED pin = GPIO10
    let led = peripherals.pins.gpio10.downgrade_output();
    let channel = peripherals.rmt.channel0;
    let mut ws2812_board = Ws2812RmtSingle::new(led, channel, RgbLayout::Rgb)?;

    if cfg!(feature = "ws2812_matrix") {
        println!(">> Using WS2812 matrix");

        // Turn off onboard LED
        ws2812_board.set(Rgb::new(0, 0, 0))?;

        // Matrix
        let led = peripherals.pins.gpio0.downgrade_output();
        let channel = peripherals.rmt.channel1;
        let config = TransmitConfig::new().clock_divider(1);
        let tx = TxRmtDriver::new(channel, led, &config)?;
        let mut ws2812 = Ws2812Rmt::new(tx, LEDS, RgbLayout::Grb);
        loop {
            for c in [
                Rgb::new(255, 0, 0),
                Rgb::new(0, 255, 0),
                Rgb::new(0, 0, 255),
            ] {
                println!("Colour: {:?}", c);
                for i in 0..LEDS {
                    let mut display = [Rgb::new(0, 0, 0); LEDS];
                    display[i] = c;
                    ws2812.set(display)?;
                    ws2812_board.set(c)?;
                    // FreeRtos::delay_ms(1);
                }
                let display = [Rgb::new(0, 0, 0); LEDS];
                ws2812.set(display)?;
            }
            FreeRtos::delay_ms(1000);
        }
    } else {
        println!(">> Using builtin LED");
        loop {
            for c in [
                Rgb::new(255, 0, 0),
                Rgb::new(0, 255, 0),
                Rgb::new(0, 0, 255),
            ] {
                println!("Colour: {:?}", c);
                ws2812_board.set(c)?;
                FreeRtos::delay_ms(1000);
            }
            FreeRtos::delay_ms(2000);
        }
    }
}
