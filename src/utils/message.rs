use anyhow::{anyhow, Error, Result};
use esp_idf_hal::rmt::{config::TransmitConfig, TxRmtDriver};
use std::iter::Rev;
use std::ops::Range;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::matrix_1d::{Matrix1D, Panel, PANEL_PIXELS};
use crate::rgb::{Rgb, RgbLayout};

use super::ws2812_rmt::Ws2812Rmt;

pub type MessageRmtChannel = esp_idf_hal::rmt::CHANNEL1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Off,
    Message(String, Rgb),
    Scroll(String, Rgb, usize), // scroll rate
}

type MessageGuard = (Mutex<Message>, Condvar);
static MESSAGE_GUARD: Mutex<Option<Arc<MessageGuard>>> = Mutex::new(None);

const MESSAGE_POLL_MS: u64 = 25;

pub struct Ws2812Message<const N: usize> {
    message_thread: Option<JoinHandle<Result<(), Error>>>,
}

impl<const N: usize> Ws2812Message<N> {
    pub fn init(
        pin: esp_idf_hal::gpio::AnyOutputPin,
        channel: MessageRmtChannel,
        panels: [Panel; N],
    ) -> Result<Self> {
        // We cant pass ws2812 instance into fn due to lifetime issues
        // (needs to be 'static for thread) so we create here
        let tx = TxRmtDriver::new(channel, pin, &TransmitConfig::new().clock_divider(1))?;
        let mut ws2812 = Ws2812Rmt::new(tx, PANEL_PIXELS * N, RgbLayout::Grb);

        let guard = Arc::new((Mutex::new(Message::Off), Condvar::new()));
        // Initialise static GUARD with clone (use for TX)
        {
            let mut guard_static = MESSAGE_GUARD.lock().unwrap();
            *guard_static = Some(guard.clone());
        }
        let mut matrix = Matrix1D::<N>::from_panels(panels);

        // Move into thread
        let rx = thread::spawn(move || {
            let (update, cvar) = &*guard;
            let mut message = Message::Off;
            let mut scroll_iter: Rev<Range<i32>> = (0..0).rev();
            let mut ticks = 0_usize;
            loop {
                // Wait for CVAR timeout
                let started = update.lock().unwrap();
                let result = cvar
                    .wait_timeout(started, Duration::from_millis(MESSAGE_POLL_MS))
                    .unwrap();
                if !result.1.timed_out() {
                    log::info!("UPDATE:: {:?}", *result.0);
                    // Update status
                    message = result.0.clone();
                    match &message {
                        Message::Off => {
                            matrix.clear();
                            ws2812.set(matrix.iter())?;
                        }
                        Message::Message(s, rgb) => {
                            matrix.clear();
                            matrix.draw_str(s, *rgb, (0, 0));
                            ws2812.set(matrix.iter())?;
                        }
                        Message::Scroll(s, _, _) => {
                            matrix.clear();
                            scroll_iter = matrix.scroll_iter(s.len());
                            ticks = 0;
                        }
                    }
                }
                match message {
                    Message::Off => {}
                    Message::Message(_, _) => {}
                    Message::Scroll(ref s, rgb, t) => {
                        if ticks % t == 0 {
                            let x = if let Some(x) = scroll_iter.next() {
                                x
                            } else {
                                // Reset iterator
                                scroll_iter = matrix.scroll_iter(s.len());
                                scroll_iter.next().unwrap_or(0)
                            };
                            matrix.clear();
                            matrix.draw_str(s, rgb, (x, 0));
                            ws2812.set(matrix.iter())?;
                        }
                    }
                }
                ticks += 1;
            }
        });
        Ok(Self {
            message_thread: Some(rx),
        })
    }

    pub fn join(&mut self) -> Result<()> {
        if let Some(handle) = self.message_thread.take() {
            handle
                .join()
                .map_err(|e| anyhow!("Thread panicked: {:?}", e))?
        } else {
            anyhow::bail!("Thread not running")
        }
    }

    pub fn update(message: Message) -> Result<()> {
        // Lock the GUARD to access its contents
        let guard = MESSAGE_GUARD
            .lock()
            .map_err(|_| anyhow::anyhow!("Cant lock MESSAGE_GUARD"))?;
        // Unwrap the Option to get the Arc<(Mutex<bool>, Condvar)>
        let guard = guard
            .as_ref()
            .ok_or(anyhow::anyhow!("MESSAGE_GUARD empty"))?;
        // Dereference the Arc to get the tuple (Mutex<bool>, Condvar)
        let (update, cvar) = &**guard;
        // Lock ledstate to get contents
        let mut s = update
            .lock()
            .map_err(|_| anyhow::anyhow!("Cant lock UPDATE"))?;
        // Update status
        *s = message;
        // Notify the condvar that the value has changed.
        cvar.notify_one();
        Ok(())
    }
}
