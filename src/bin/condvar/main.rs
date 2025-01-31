use anyhow::Result;
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

use c3zero::rgb::Rgb;
use c3zero::status_led::{LedState, StatusLed};

type GuardU8 = (Mutex<u8>, Condvar);
static GUARD: Mutex<Option<Arc<GuardU8>>> = Mutex::new(None);

fn main() -> Result<()> {
    esp_idf_hal::sys::link_patches();

    let guard = Arc::new((Mutex::new(0), Condvar::new()));

    // Initialise static GUARD (use in TX thread)
    {
        let mut guard_static = GUARD.lock().unwrap();
        *guard_static = Some(guard.clone());
    }

    let _tx = thread::spawn(move || {
        // Lock the GUARD to access its contents
        let guard = GUARD.lock().unwrap();
        // Unwrap the Option to get the Arc<(Mutex<bool>, Condvar)>
        let guard = guard.as_ref().unwrap();
        // Dereference the Arc to get the tuple (Mutex<bool>, Condvar)
        let (lock, cvar) = &**guard;
        loop {
            {
                let mut counter = lock.lock().unwrap();
                *counter += 1;
                // We notify the condvar that the value has changed.
                println!(">> Notify");
                cvar.notify_one();
            }
            thread::sleep(Duration::from_millis(1000));
        }
    });

    // We move this guard into the RX thread
    let guard_thread = guard.clone();
    let _rx = thread::spawn(move || {
        let (lock, cvar) = &*guard_thread;
        loop {
            let started = lock.lock().unwrap();
            let result = cvar
                .wait_timeout(started, Duration::from_millis(200))
                .unwrap();
            if !result.1.timed_out() {
                println!(">> Wake -- Counter: {}", result.0);
            } else {
                println!(">> Timeout");
            }
        }
    });

    let peripherals = Peripherals::take()?;
    // Onboard RGB LED pin
    let led = peripherals.pins.gpio10.downgrade_output();
    let channel = peripherals.rmt.channel0;

    let _status = StatusLed::new(led, channel)?;

    loop {
        StatusLed::update(LedState::On(Rgb::new(255, 0, 0)));
        FreeRtos::delay_ms(500);
        StatusLed::update(LedState::On(Rgb::new(0, 255, 0)));
        FreeRtos::delay_ms(500);
        StatusLed::update(LedState::On(Rgb::new(0, 0, 255)));
        FreeRtos::delay_ms(500);
        StatusLed::update(LedState::Off);
        FreeRtos::delay_ms(2000);
    }
}
