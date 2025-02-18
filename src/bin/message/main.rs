use anyhow::Result;
use askama::Template;
use esp_idf_hal::{delay::FreeRtos, gpio::OutputPin, prelude::Peripherals};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::http;
use esp_idf_svc::http::server::{EspHttpConnection, Request};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::EspWifi;
use serde::Deserialize;

use c3zero::httpd;
use c3zero::matrix_1d::{Orientation, Panel};
use c3zero::message::{Message, Ws2812Message};
use c3zero::nvs::APStore;
use c3zero::rgb::{self, Rgb, RgbLayout};
use c3zero::status::{LedState, Status};
use c3zero::wifi::{self, APConfig};

const STARTING: LedState = LedState::On(rgb::BLUE);
const WIFI_SCAN: LedState = LedState::Flash(rgb::BLUE, 100);
const WIFI_CONNECT: LedState = LedState::Flash(rgb::BLUE, 250);
const WIFI_CONNECTED: LedState = LedState::On(rgb::GREEN);
const WIFI_AP_ACTIVE: LedState = LedState::Wheel(1);

fn main() -> Result<()> {
    esp_idf_hal::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Starting...");

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs_default_partition = EspDefaultNvsPartition::take()?;

    // C3-Zero onboard RGB LED pin = GPIO10
    let led = peripherals.pins.gpio10.downgrade_output();
    let channel = peripherals.rmt.channel0;
    // Create status thread
    let _status_thread = Status::new(led, channel, RgbLayout::Rgb)?;

    Status::update(STARTING)?;

    let pin = peripherals.pins.gpio0.downgrade_output();
    let channel = peripherals.rmt.channel1;
    let panels = [Panel::new(Orientation::East), Panel::new(Orientation::East)];
    // Create message thread
    let _message_thread = Ws2812Message::init(pin, channel, panels);

    // Initislise NVS APStore
    APStore::init(nvs_default_partition.clone())?;

    // Initialise WiFi
    let mut wifi: EspWifi<'_> = EspWifi::new(
        peripherals.modem,
        sys_loop.clone(),
        Some(nvs_default_partition.clone()),
    )?;
    wifi::wifi_init(&mut wifi)?;

    // Initial scan
    Status::update(WIFI_SCAN)?;
    wifi::wifi_scan(&mut wifi)?;

    let mut wifi_config: Option<APConfig> = None;
    for config in wifi::find_known_aps() {
        Status::update(WIFI_CONNECT)?;
        log::info!("Trying network: {}", config.ssid);
        match wifi::connect_wifi(&mut wifi, &config, 10000) {
            Ok(true) => {
                log::info!("Connected to Wifi: {}", config.ssid);
                wifi_config = Some(config);
                break;
            }
            Ok(false) => {
                log::info!("Failed to connect to Wifi: {}", config.ssid);
            }
            Err(e) => {
                log::info!("Wifi Error: {} [{}]", config.ssid, e);
            }
        }
    }

    let mut server = if let Some(config) = wifi_config {
        Status::update(WIFI_CONNECTED)?;
        Ws2812Message::<2>::update(Message::Scroll(
            format!("{} : {}", config.ssid, wifi.sta_netif().get_ip_info()?.ip),
            rgb::BLUE,
            1,
        ))?;
        log::info!("Connected to SSID: {}", config.ssid);
        httpd::start_http_server()?
    } else {
        Status::update(WIFI_AP_ACTIVE)?;
        Ws2812Message::<2>::update(Message::Scroll("Starting AP:".to_string(), rgb::BLUE, 1))?;
        log::info!("No valid config found - starting AP");
        wifi::start_access_point(&mut wifi)?;
        httpd::start_http_server()?
    };

    // Add local handlers
    server.fn_handler("/message", http::Method::Get, handle_message_form)?;
    server.fn_handler("/message", http::Method::Post, handle_message_post)?;

    loop {
        FreeRtos::delay_ms(1000); // Delay for 100 milliseconds
    }
}

#[derive(askama::Template)]
#[template(path = "message_page.html")]
struct MessagePage {}

fn handle_message_form(request: Request<&mut EspHttpConnection>) -> anyhow::Result<()> {
    let message_page = MessagePage {};
    let mut response = request.into_ok_response()?;
    let html = message_page.render()?;
    response.write(html.as_bytes())?;
    Ok::<(), anyhow::Error>(())
}

fn handle_message_post(mut request: Request<&mut EspHttpConnection>) -> anyhow::Result<()> {
    #[derive(Deserialize, Default)]
    struct MessageParams {
        mode: u8,
        #[serde(default)]
        message: String,
        #[serde(default)]
        r: u8,
        #[serde(default)]
        g: u8,
        #[serde(default)]
        b: u8,
        #[serde(default = "default_delay")]
        delay: u8,
    }

    fn default_delay() -> u8 {
        1
    }

    let mut buf = [0_u8; 1024];
    let len = request.read(&mut buf)?;
    log::info!("Req:: {}", std::str::from_utf8(&buf[..len])?);
    let params: MessageParams = serde_urlencoded::from_bytes(&buf[..len])?;

    let message = match params.mode {
        0 => Message::Off,
        1 => Message::Message(params.message, Rgb::new(params.r, params.g, params.b)),
        2 => Message::Scroll(
            params.message,
            Rgb::new(params.r, params.g, params.b),
            params.delay as usize,
        ),
        _ => return Err(anyhow::anyhow!("Invalid mode")),
    };

    Ws2812Message::<2>::update(message)?;
    request.into_response(302, None, &[("Location", "/message")])?;

    Ok(())
}
