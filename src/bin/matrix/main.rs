use anyhow::Result;
use esp_idf_hal::rmt::{config::TransmitConfig, TxRmtDriver};
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};

use c3zero::rgb::{Rgb, RgbLayout, OFF};
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
    ws2812_board.set(OFF)?;

    let led = peripherals.pins.gpio0.downgrade_output();
    let channel = peripherals.rmt.channel1;
    let config = TransmitConfig::new().clock_divider(1);
    let tx = TxRmtDriver::new(channel, led, &config)?;
    let mut ws2812 = Ws2812Rmt::new(tx, 64, RgbLayout::Grb);
    let mut matrix = Ws2812Matrix::new(Orientation::North);

    rotate(&mut matrix, &mut ws2812, 2)?;
    matrix.set_orientation(Orientation::North);

    let msg = "Hello There";
    loop {
        message(&mut matrix, &mut ws2812, msg)?;
        FreeRtos::delay_ms(500);
    }
}

fn message(matrix: &mut Ws2812Matrix, ws2812: &mut Ws2812Rmt, msg: &str) -> Result<()> {
    for (c1, c2) in msg.chars().zip(msg.chars().skip(1)) {
        for o in 0..8 {
            matrix.fill(OFF);
            matrix.draw_char(c1, Rgb::new(128, 0, 0), -o);
            matrix.draw_char(c2, Rgb::new(128, 0, 0), 8 - o);
            ws2812.set(matrix.iter())?;
            FreeRtos::delay_ms(100);
        }
    }
    Ok(())
}

fn rotate(matrix: &mut Ws2812Matrix, ws2812: &mut Ws2812Rmt, count: usize) -> Result<()> {
    // Draw arrow
    for p in [(2, 1), (1, 2), (0, 3)] {
        matrix.set(p, Rgb::new(128, 0, 0));
    }
    for p in [(5, 1), (6, 2), (7, 3)] {
        matrix.set(p, Rgb::new(0, 0, 128));
    }
    for y in 0..8 {
        let c = 16 * y as u8;
        matrix.set((3, y), Rgb::new(128 - c, c, 0));
        matrix.set((4, y), Rgb::new(0, c, 128 - c));
    }

    for _ in 0..count {
        for o in [
            Orientation::North,
            Orientation::East,
            Orientation::South,
            Orientation::West,
        ] {
            matrix.set_orientation(o);
            println!(">> ORIENTATION:: {:?}", o);
            ws2812.set(matrix.iter())?;
            FreeRtos::delay_ms(500);
        }
    }
    Ok(())
}
