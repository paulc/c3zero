use anyhow::Result;
use esp_idf_hal::rmt::{config::TransmitConfig, TxRmtDriver};
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};

use c3zero::rgb::{Rgb, RgbLayout};
use c3zero::ws2812_matrix::{Orientation, Ws2812Matrix};
use c3zero::ws2812_rmt::{Ws2812Rmt, Ws2812RmtSingle};

fn main() -> Result<()> {
    esp_idf_hal::sys::link_patches();

    let peripherals = Peripherals::take()?;

    // C3-Zero onboard RGB LED pin = GPIO10
    let led = peripherals.pins.gpio10.downgrade_output();
    let channel = peripherals.rmt.channel0;
    let mut ws2812_board = Ws2812RmtSingle::new(led, channel, RgbLayout::Rgb)?;
    // Turn off onboard LED
    ws2812_board.set(Rgb::new(0, 0, 0))?;

    let led = peripherals.pins.gpio0.downgrade_output();
    let channel = peripherals.rmt.channel1;
    let config = TransmitConfig::new().clock_divider(1);
    let tx = TxRmtDriver::new(channel, led, &config)?;
    let mut ws2812 = Ws2812Rmt::new(tx, 64, RgbLayout::Grb);

    loop {
        for o in [
            Orientation::North,
            Orientation::East,
            Orientation::South,
            Orientation::West,
        ] {
            let mut matrix = Ws2812Matrix::new(o);
            matrix.fill(Rgb::new(0, 0, 0));
            ws2812.set(matrix.iter())?;
            FreeRtos::delay_ms(500);

            println!(">> ORIENTATION:: {:?}", o);
            matrix.set((0, 0), Rgb::new(255, 0, 0));
            matrix.set((1, 1), Rgb::new(0, 255, 0));
            matrix.set((2, 2), Rgb::new(0, 0, 255));
            matrix.set((0, 4), Rgb::new(63, 127, 63));
            ws2812.set(matrix.iter())?;
            FreeRtos::delay_ms(5000);
        }
    }
}
