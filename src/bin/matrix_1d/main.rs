use anyhow::Result;
use esp_idf_hal::rmt::{config::TransmitConfig, TxRmtDriver};
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};

use c3zero::matrix_1d::{Matrix1D, Orientation, Panel};
use c3zero::rgb::{self, Rgb, RgbLayout, RgbTransform};
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

    loop {
        text(&mut ws2812)?;
        scroll(&mut ws2812)?;
    }
}

fn text(ws2812: &mut Ws2812Rmt) -> Result<()> {
    let (p1, p2) = (Panel::new(Orientation::East), Panel::new(Orientation::East));
    let mut matrix = Matrix1D::<2>::from_panels([p1, p2]);
    for x in -8..16 {
        matrix.clear();
        matrix.draw_char('A', Rgb::new(128, 0, 0), (x, 0));
        ws2812.set(matrix.iter())?;
        FreeRtos::delay_ms(50);
    }
    let msg = "Hello There!";
    for x in matrix.scroll_iter(msg.len()) {
        matrix.clear();
        matrix.draw_str(msg, Rgb::new(128, 0, 0), (x, 0));
        ws2812.set(matrix.iter())?;
        FreeRtos::delay_ms(50);
    }
    let msg = "This is a long message... 0123456789 ±!@£$%^&*()";
    for x in matrix.scroll_iter(msg.len()) {
        matrix.clear();
        matrix.draw_str(msg, Rgb::new(128, 0, 0), (x, 0));
        ws2812.set(matrix.iter())?;
        FreeRtos::delay_ms(25);
    }
    Ok(())
}

fn scroll(ws2812: &mut Ws2812Rmt) -> Result<()> {
    for o in [
        Orientation::North,
        Orientation::East,
        Orientation::South,
        Orientation::West,
    ] {
        println!(">> Orientation:: {o:?}");
        let (p1, p2) = (Panel::new(o), Panel::new(o));
        let mut matrix = Matrix1D::<2>::from_panels([p1, p2]);
        for y in 0..8 {
            for x in 0..16 {
                matrix.transform(
                    (0, 0),
                    (16, 8),
                    &[RgbTransform::Intensity(0.3), RgbTransform::Rotate],
                );
                matrix.transform(
                    (0, 0),
                    (16, 8),
                    &[RgbTransform::FillThreshold(Rgb::new(1, 1, 1), 0.01)],
                );

                matrix.set((x, y), rgb::RED);
                ws2812.set(matrix.iter())?;
                FreeRtos::delay_ms(50);
            }
        }
    }
    Ok(())
}
