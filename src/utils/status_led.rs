use anyhow::Result;
use esp_idf_hal::rmt::{config::TransmitConfig, FixedLengthSignal, PinState, Pulse, TxRmtDriver};
use std::time::Duration;

use crate::rgb::Rgb;

pub type Ws2812RmtChannel = esp_idf_hal::rmt::CHANNEL0;

pub struct Ws2812Rmt<'a> {
    tx: esp_idf_hal::rmt::TxRmtDriver<'a>,
}

impl Ws2812Rmt<'_> {
    pub fn new(led: esp_idf_hal::gpio::AnyOutputPin, channel: Ws2812RmtChannel) -> Result<Self> {
        let config = TransmitConfig::new().clock_divider(1);
        let tx = TxRmtDriver::new(channel, led, &config)?;
        Ok(Self { tx })
    }

    pub fn set(&mut self, rgb: Rgb) -> Result<()> {
        let color: u32 = rgb.into();
        let ticks_hz = self.tx.counter_clock()?;
        let (t0h, t0l, t1h, t1l) = (
            Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(350))?,
            Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(800))?,
            Pulse::new_with_duration(ticks_hz, PinState::High, &Duration::from_nanos(700))?,
            Pulse::new_with_duration(ticks_hz, PinState::Low, &Duration::from_nanos(600))?,
        );
        let mut signal = FixedLengthSignal::<24>::new();
        for i in (0..24).rev() {
            let p = 2_u32.pow(i);
            let bit: bool = p & color != 0;
            let (high_pulse, low_pulse) = if bit { (t1h, t1l) } else { (t0h, t0l) };
            signal.set(23 - i as usize, &(high_pulse, low_pulse))?;
        }
        self.tx.start_blocking(&signal)?;
        Ok(())
    }
}
