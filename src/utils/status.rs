use anyhow::{anyhow, Error, Result};
use esp_idf_sys as _;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::rgb::{Rgb, RgbLayout};
use crate::ws2812_rmt::{Ws2812RmtChannel, Ws2812RmtSingle};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LedState {
    Off,
    On(Rgb),
    Flash(Rgb, u32),
    Wheel(u32),
    Sequence(Vec<(Rgb, u32)>),
}

type StatusGuard = (Mutex<LedState>, Condvar);
static STATUS_GUARD: Mutex<Option<Arc<StatusGuard>>> = Mutex::new(None);

const STATUS_POLL_MS: u32 = 50; // Minimum CVAR wait time seems to be c.20ms

pub struct Status {
    status_thread: Option<JoinHandle<Result<(), Error>>>,
}

impl Status {
    pub fn new(
        led: esp_idf_hal::gpio::AnyOutputPin,
        channel: Ws2812RmtChannel,
        format: RgbLayout,
    ) -> Result<Self> {
        let mut led = Ws2812RmtSingle::new(led, channel, format)?;
        let guard = Arc::new((Mutex::new(LedState::Off), Condvar::new()));
        // Initialise static GUARD with clone (use for TX)
        {
            let mut guard_static = STATUS_GUARD.lock().unwrap();
            *guard_static = Some(guard.clone());
        }
        // Move guard into thread
        let rx = thread::spawn(move || {
            let (ledstate, cvar) = &*guard;
            let mut status = LedState::Off;
            let mut wheel_hue = 0_u32;
            let mut timer = 0_u32;
            let mut flash_state = false;
            let mut sequence_state = 0_usize;
            let mut start_ticks = unsafe { esp_idf_sys::xTaskGetTickCount() };
            loop {
                // Wait for CVAR timeout
                let started = ledstate.lock().unwrap();
                let result = cvar
                    .wait_timeout(started, Duration::from_millis(STATUS_POLL_MS as u64))
                    .unwrap();

                // Update tick counter
                let now = unsafe { esp_idf_sys::xTaskGetTickCount() };

                if !result.1.timed_out() {
                    log::info!("MESSAGE:: {:?} {}", *result.0, *result.0 == status);
                    // Update status
                    status = result.0.clone();
                    match status {
                        LedState::Flash(rgb, ms) => {
                            timer = ms / 2;
                            start_ticks = now;
                            flash_state = true;
                            led.set(rgb)?;
                        }
                        LedState::Sequence(ref seq) => {
                            if !seq.is_empty() {
                                timer = seq[0].1;
                                led.set(seq[0].0)?;
                                sequence_state = 0;
                                start_ticks = now;
                            }
                        }
                        LedState::Wheel(_) => wheel_hue = 0,
                        _ => {}
                    }
                }

                // Elapsed time in ms since last state change
                let elapsed = (now - start_ticks) * 1000 / esp_idf_sys::configTICK_RATE_HZ;

                // Handle LED output
                match status {
                    LedState::Off => led.set(Rgb::new(0, 0, 0))?,
                    LedState::On(rgb) => led.set(rgb)?,
                    LedState::Flash(rgb, _) => {
                        if elapsed >= timer {
                            // log::info!("FLASH: {}", elapsed);
                            start_ticks = now;
                            flash_state = !flash_state;
                            match flash_state {
                                true => led.set(rgb)?,
                                false => led.set(Rgb::new(0, 0, 0))?,
                            }
                        }
                    }
                    LedState::Sequence(ref seq) => {
                        if elapsed >= timer {
                            // log::info!("FLASH: {}", elapsed);
                            start_ticks = now;
                            sequence_state = (sequence_state + 1) % seq.len();
                            timer = seq[sequence_state].1;
                            led.set(seq[sequence_state].0)?;
                        }
                    }
                    LedState::Wheel(step) => {
                        wheel_hue = (wheel_hue + step) % 360;
                        led.set(Rgb::from_hsv(wheel_hue, 100, 20)?)?;
                    }
                }
            }
        });

        Ok(Self {
            status_thread: Some(rx),
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

    pub fn update(status: LedState) -> Result<()> {
        // Lock the GUARD to access its contents
        let guard = STATUS_GUARD
            .lock()
            .map_err(|_| anyhow::anyhow!("Cant lock STATUS_GUARD"))?;
        // Unwrap the Option to get the Arc<(Mutex<bool>, Condvar)>
        let guard = guard
            .as_ref()
            .ok_or(anyhow::anyhow!("STATUS_GUARD empty"))?;
        // Dereference the Arc to get the tuple (Mutex<bool>, Condvar)
        let (ledstate, cvar) = &**guard;
        // Lock ledstate to get contents
        let mut s = ledstate
            .lock()
            .map_err(|_| anyhow::anyhow!("Cant lock LED_STATE"))?;
        // Update status
        *s = status;
        // Notify the condvar that the value has changed.
        cvar.notify_one();
        Ok(())
    }
}
