use anyhow::Result;
use esp_idf_hal::rmt::{config::TransmitConfig, TxRmtDriver};
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};

use c3zero::matrix_1d::{Matrix1D, Orientation};
use c3zero::rgb::{self, RgbLayout};
use c3zero::ws2812_rmt::{Ws2812Rmt, Ws2812RmtSingle};

fn main() -> Result<()> {
    esp_idf_hal::sys::link_patches();

    let peripherals = Peripherals::take()?;

    // C3-Zero onboard RGB LED pin = GPIO10
    let led = peripherals.pins.gpio10.downgrade_output();
    let channel = peripherals.rmt.channel0;
    let mut ws2812_board = Ws2812RmtSingle::new(led, channel, RgbLayout::Rgb)?;
    // Turn off onboard LED
    ws2812_board.set(rgb::OFF)?;

    let led = peripherals.pins.gpio0.downgrade_output();
    let channel = peripherals.rmt.channel1;
    let config = TransmitConfig::new().clock_divider(1);
    let tx = TxRmtDriver::new(channel, led, &config)?;
    let mut ws2812 = Ws2812Rmt::new(tx, 64, RgbLayout::Grb);
    let mut matrix = Matrix1D::<2>::new(Orientation::North);

    loop {
        for o in [
            //Orientation::North,
            Orientation::East,
            //Orientation::South,
            //Orientation::West,
        ] {
            println!(">> Orientation:: {o:?}");
            matrix.set_orientation(o);
            for y in 0..8 {
                for x in 0..16 {
                    matrix.clear();
                    matrix.set((x, y), rgb::BLUE)?;
                    ws2812.set(matrix.iter())?;
                    FreeRtos::delay_ms(50);
                }
            }
        }
    }
}
