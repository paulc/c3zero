use anyhow::Result;
use esp_idf_hal::rmt::{config::TransmitConfig, TxRmtDriver};
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};

use c3zero::rgb::Rgb;
use c3zero::ws2812_rmt::{Ws2812Rmt, Ws2812RmtSingle};

const RMT_SINGLE: bool = cfg!(feature = "rmt_single");

pub fn main() -> Result<()> {
    esp_idf_hal::sys::link_patches();

    let peripherals = Peripherals::take()?;
    // Onboard RGB LED pin
    let led = peripherals.pins.gpio10.downgrade_output();
    let channel = peripherals.rmt.channel0;

    if RMT_SINGLE {
        println!(">> RMT_SINGLE");
        let mut ws2812 = Ws2812RmtSingle::new(led, channel)?;

        loop {
            ws2812.set(Rgb::new(255, 0, 0))?;
            FreeRtos::delay_ms(500);
            ws2812.set(Rgb::new(0, 255, 0))?;
            FreeRtos::delay_ms(500);
            ws2812.set(Rgb::new(0, 0, 255))?;
            FreeRtos::delay_ms(500);
            ws2812.set(Rgb::new(0, 0, 0))?;
            FreeRtos::delay_ms(2000);
        }
    } else {
        println!(">> RMT_MULTIPLE");

        let config = TransmitConfig::new().clock_divider(1);
        let tx = TxRmtDriver::new(channel, led, &config)?;
        let mut ws2812 = Ws2812Rmt::new(tx, 1);

        loop {
            ws2812.set([Rgb::new(255, 0, 0)])?;
            FreeRtos::delay_ms(500);
            ws2812.set([Rgb::new(0, 255, 0)])?;
            FreeRtos::delay_ms(500);
            ws2812.set([Rgb::new(0, 0, 255)])?;
            FreeRtos::delay_ms(500);
            ws2812.set([Rgb::new(0, 0, 0)])?;
            FreeRtos::delay_ms(2000);
        }
    }
}
