use anyhow::{Ok, Result};
use esp_idf_hal::rmt::{config::TransmitConfig, TxRmtDriver};
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};

use c3zero::matrix::{Matrix, Orientation};
use c3zero::rgb::{self, Rgb, RgbLayout};
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
    let mut matrix = Matrix::new(Orientation::North);

    let msg = "Hello, this is a message! ±!@£$%^&*()_+ 01234567890 {}[]:;'|<>?/\\";

    loop {
        rotate(&mut matrix, &mut ws2812, 2)?;
        matrix.set_orientation(Orientation::East);
        scroll(&mut matrix, &mut ws2812, 'Z')?;
        glyph(&mut matrix, &mut ws2812)?;
        message(&mut matrix, &mut ws2812, msg)?;
        bitmap(&mut matrix, &mut ws2812)?;
        FreeRtos::delay_ms(2000);
    }
}

fn bitmap(matrix: &mut Matrix, ws2812: &mut Ws2812Rmt) -> Result<()> {
    #[rustfmt::skip]
    let bitmap = [
        ".ABCD...",
        "....A...",
        "....B...",
        "....C...",
        "....DCBA",
        "........",
        "AB......",
        "CD......",
    ];
    let colourmap = [
        ('A', rgb::RED),
        ('B', rgb::GREEN),
        ('C', rgb::BLUE),
        ('D', Rgb::new(0, 64, 64)),
        ('.', rgb::OFF),
    ];

    matrix.draw_bitmap(&bitmap, &colourmap, 0);
    ws2812.set(matrix.iter())?;
    Ok(())
}

fn glyph(matrix: &mut Matrix, ws2812: &mut Ws2812Rmt) -> Result<()> {
    #[rustfmt::skip]
    let glyph = [
        0b00000001,
        0b00000010,
        0b00000100,
        0b00001000,
        0b00001000,
        0b00010100,
        0b00100010,
        0b01000001,
    ];

    for offset in -8..=8 {
        matrix.fill(rgb::OFF);
        matrix.draw_glyph(glyph, rgb::BLUE, offset);
        ws2812.set(matrix.iter())?;
        FreeRtos::delay_ms(50);
    }
    Ok(())
}

fn scroll(matrix: &mut Matrix, ws2812: &mut Ws2812Rmt, c: char) -> Result<()> {
    for offset in -8..=8 {
        matrix.fill(rgb::OFF);
        matrix.draw_char(c, rgb::GREEN, offset);
        ws2812.set(matrix.iter())?;
        FreeRtos::delay_ms(50);
    }
    Ok(())
}

fn message(matrix: &mut Matrix, ws2812: &mut Ws2812Rmt, msg: &str) -> Result<()> {
    for (c1, c2) in msg
        .chars()
        .zip(msg.chars().chain(std::iter::once(' ')).skip(1))
    {
        for o in 0..8 {
            matrix.fill(rgb::OFF);
            matrix.draw_char(c1, Rgb::new(64, 0, 0), -o);
            matrix.draw_char(c2, Rgb::new(64, 0, 0), 8 - o);
            ws2812.set(matrix.iter())?;
            FreeRtos::delay_ms(50);
        }
    }
    Ok(())
}

fn rotate(matrix: &mut Matrix, ws2812: &mut Ws2812Rmt, count: usize) -> Result<()> {
    // Draw arrow
    matrix.fill(rgb::OFF);
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
