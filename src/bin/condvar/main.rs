use anyhow::{anyhow, Error, Result};
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};
use esp_idf_sys as _;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use c3zero::rgb::Rgb;
use c3zero::ws2812_rmt::{Ws2812Rmt, Ws2812RmtChannel};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LedState {
    Off,
    On(Rgb),
    Flash(Rgb, u32),
    Wheel(u32),
}

type StatusGuard = (Mutex<LedState>, Condvar);
static STATUS_GUARD: Mutex<Option<Arc<StatusGuard>>> = Mutex::new(None);

const STATUS_POLL_MS: u32 = 50; // Minimum CVAR wait time seems to be c.20ms

pub struct StatusLed {
    status_thread: Option<JoinHandle<Result<(), Error>>>,
}

impl StatusLed {
    pub fn new(led: esp_idf_hal::gpio::AnyOutputPin, channel: Ws2812RmtChannel) -> Result<Self> {
        let mut led = Ws2812Rmt::new(led, channel)?;
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
            let mut flash_timer = 0_u32;
            let mut flash_state = false;
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
                        LedState::Flash(_, ms) => {
                            flash_timer = ms / 2;
                            start_ticks = now;
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
                        if elapsed >= flash_timer {
                            log::info!("FLASH: {}", elapsed);
                            start_ticks = now;
                            flash_state = !flash_state;
                        }
                        match flash_state {
                            true => led.set(rgb)?,
                            false => led.set(Rgb::new(0, 0, 0))?,
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

fn main() -> Result<()> {
    esp_idf_hal::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Starting...");

    let peripherals = Peripherals::take()?;
    // Onboard RGB LED pin
    let led = peripherals.pins.gpio10.downgrade_output();
    let channel = peripherals.rmt.channel0;

    let _status = StatusLed::new(led, channel)?;

    loop {
        for (state, delay) in [
            (LedState::On(Rgb::new(255, 0, 0)), 500),
            (LedState::On(Rgb::new(0, 255, 0)), 500),
            (LedState::On(Rgb::new(0, 0, 255)), 500),
            (LedState::Off, 1000),
            (LedState::Flash(Rgb::new(255, 0, 0), 500), 5000),
            (LedState::Off, 1000),
            (LedState::Flash(Rgb::new(0, 255, 0), 100), 5000),
            (LedState::Off, 1000),
            (LedState::Flash(Rgb::new(0, 0, 255), 1000), 5000),
            (LedState::Off, 1000),
            (LedState::Wheel(10), 5000),
            (LedState::Off, 1000),
        ] {
            log::info!(">> {:?} [{}ms]", state, delay);
            StatusLed::update(state)?;
            FreeRtos::delay_ms(delay);
        }
    }
}
