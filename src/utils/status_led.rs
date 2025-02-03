use anyhow::{anyhow, Error, Result};
use esp_idf_hal::delay::FreeRtos;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

use crate::rgb::Rgb;
use crate::ws2812_rmt::{Ws2812Rmt, Ws2812RmtChannel};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LedState {
    Off,
    On(Rgb),
    Flash(Rgb, usize),
    Wheel(usize),
}

pub static STATUS: Mutex<LedState> = Mutex::new(LedState::Off);

pub struct StatusLed {
    status_thread: Option<JoinHandle<Result<(), Error>>>,
}

impl StatusLed {
    pub fn new(led: esp_idf_hal::gpio::AnyOutputPin, channel: Ws2812RmtChannel) -> Result<Self> {
        let mut led = Ws2812Rmt::new(led, channel)?;
        let mut status = LedState::Off;
        let status_thread = thread::spawn(move || loop {
            let _ = STATUS.try_lock().map(|s| status = (*s).clone());
            match status {
                LedState::Off => led.set(Rgb::new(0, 0, 0))?,
                LedState::On(rgb) => led.set(rgb)?,
                LedState::Flash(_, _) => led.set(Rgb::new(0, 0, 0))?,
                LedState::Wheel(_) => led.set(Rgb::new(0, 0, 0))?,
            };
            FreeRtos::delay_ms(100);
        });

        Ok(Self {
            status_thread: Some(status_thread),
        })
    }

    pub fn join(&mut self) -> Result<()> {
        if let Some(handle) = self.status_thread.take() {
            handle
                .join()
                .map_err(|e| anyhow!("Thread panicked: {:?}", e))?
        } else {
            anyhow::bail!("Thread not running")
        }
    }

    pub fn update(state: LedState) {
        let mut status = STATUS.lock().unwrap();
        *status = state
    }
}
