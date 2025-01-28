use askama::Template;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::http;
use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{
    AccessPointConfiguration, AccessPointInfo, AuthMethod, Configuration, EspWifi,
};
use std::str::FromStr;
use std::sync::Mutex;
//use std::net::Ipv4Addr;
//use esp_idf_svc::hal::spi;
//use esp_idf_svc::hal::gpio;
//use smart_leds::SmartLedsWrite;
//use ws2812_spi::Ws2812;

use c3zero::nvs::{APStore, KNOWN_APS};
use c3zero::wifi::WifiConfig;
static WIFI_SCAN: Mutex<Vec<AccessPointInfo>> = Mutex::new(Vec::new());

#[derive(askama::Template)]
#[template(path = "../templates/config_page.html")]
struct ConfigPage<'a> {
    visible: Vec<(&'a str, u8, i8, &'a str)>,
    aps: Vec<&'a str>,
}

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Starting...");

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;

    let nvs_default_partition = EspDefaultNvsPartition::take()?;
    APStore::init(nvs_default_partition.clone())?;

    let mut wifi: EspWifi<'_> = EspWifi::new(
        peripherals.modem,
        sys_loop.clone(),
        Some(nvs_default_partition),
    )?;

    // Start WiFi initially with default config for scan
    let config = Configuration::Client(esp_idf_svc::wifi::ClientConfiguration {
        ..Default::default()
    });
    wifi.set_configuration(&config)?;
    wifi.start()?;

    // WiFi Scan
    wifi_scan(&mut wifi)?;

    // Find WiFi Config
    let mut wifi_config: Option<WifiConfig> = None;
    let mut known = Vec::new();
    let mut seen = Vec::new(); // We can see same SSID on multiple bands
    {
        // Only lock mutex in block
        let aps = WIFI_SCAN.lock().unwrap();
        for ap in aps.iter() {
            if !seen.contains(&ap.ssid.as_str()) {
                // Check if we have configuration in NVS (using hashed SSID)
                if let Ok(Some(config)) = APStore::get_wifi_config(&ap.ssid.as_str()) {
                    log::info!("Found AP config: {}", ap.ssid.as_str());
                    known.push(config);
                }
                seen.push(ap.ssid.as_str());
            }
        }
    }

    for config in known {
        log::info!("Trying network: {}", config.ssid);
        match connect_wifi(&mut wifi, &config, 10000) {
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

    let _server = if let Some(config) = wifi_config {
        log::info!("Connected to SSID: {}", config.ssid);
        start_http_server()?
    } else {
        log::info!("No valid config found - starting AP");
        start_access_point(&mut wifi)?;
        start_http_server()?
    };

    loop {
        FreeRtos::delay_ms(1000); // Delay for 100 milliseconds
    }
}

fn wifi_scan(wifi: &mut EspWifi) -> anyhow::Result<()> {
    log::info!("Starting WiFi scan...");
    // Note that scan will disable WiFi connection
    let scan = wifi
        .scan()?
        .into_iter()
        .inspect(|ap| {
            log::info!(
                "SSID: {:?}, Channel: {}, RSSI: {}, Auth: {:?}",
                ap.ssid,
                ap.channel,
                ap.signal_strength,
                ap.auth_method,
            )
        })
        .collect::<Vec<_>>();
    let mut aps = WIFI_SCAN.lock().unwrap();
    *aps = scan;
    Ok(())
}

fn connect_wifi(wifi: &mut EspWifi, config: &WifiConfig, timeout_ms: u32) -> anyhow::Result<bool> {
    const SLEEP_MS: u32 = 500;
    let sta_config = Configuration::Client(esp_idf_svc::wifi::ClientConfiguration {
        ssid: config.ssid.clone(),
        password: config.password.clone(),
        ..Default::default()
    });

    wifi.set_configuration(&sta_config)?;
    wifi.start()?;
    wifi.connect()?;

    let mut timer = 0;
    loop {
        match wifi.is_up()? {
            true => break,
            false => {
                log::info!(
                    "Connecting: {} [{}ms] {}",
                    config.ssid,
                    timer,
                    match wifi.is_connected()? {
                        true => "<connected>",
                        false => "",
                    }
                );
                FreeRtos::delay_ms(SLEEP_MS);
                timer += SLEEP_MS;
                if timer >= timeout_ms {
                    wifi.stop()?;
                    return Ok(false);
                }
            }
        }
    }
    log::info!(
        "Connected:  {} {:?}",
        config.ssid,
        wifi.sta_netif().get_ip_info()?
    );
    Ok(true)
}

fn start_access_point(wifi: &mut EspWifi) -> anyhow::Result<()> {
    let ssid: heapless::String<32> =
        heapless::String::from_str("ESP32C3-AP").map_err(|_| anyhow::anyhow!("SSID too long"))?;
    let password: heapless::String<64> =
        heapless::String::from_str("password").map_err(|_| anyhow::anyhow!("PW too long"))?;

    let ap_config = AccessPointConfiguration {
        ssid,
        password,
        channel: 1,
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    };

    wifi.set_configuration(&Configuration::AccessPoint(ap_config))?;
    wifi.start()?;

    println!("Access Point started. Connect to ESP32C3-AP with password 'password'");

    // Here you would implement a web server to capture the WiFi credentials
    // and save them using `save_wifi_config`

    Ok(())
}

fn start_http_server<'a>() -> anyhow::Result<EspHttpServer<'a>> {
    let config: HttpConfig = HttpConfig {
        uri_match_wildcard: true,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&config)?;

    server.fn_handler("/hello", http::Method::Get, |req| {
        let mut response = req.into_ok_response()?;
        response.write("Hello from ESP32-C3!\n".as_bytes())?;
        Ok::<(), anyhow::Error>(())
    })?;

    server.fn_handler("/", http::Method::Get, |req| {
        let aps = KNOWN_APS.lock().unwrap();
        let visible = WIFI_SCAN.lock().unwrap();
        let visible = visible
            .iter()
            .map(|ap| {
                (
                    ap.ssid.as_str(),
                    ap.channel,
                    ap.signal_strength,
                    match ap.auth_method {
                        Some(_) => "Some",
                        None => "None",
                    },
                )
            })
            .collect::<Vec<_>>();
        let config_page = ConfigPage {
            visible,
            aps: aps.0.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
        };
        let mut response = req.into_ok_response()?;
        let html = config_page.render()?;
        response.write(html.as_bytes())?;
        Ok::<(), anyhow::Error>(())
    })?;

    // Handle deleting an AP
    server.fn_handler("/delete/*", http::Method::Get, |req| {
        log::info!("DELETE: {:?}", req.uri());
        let ssid = req.uri().split('/').next_back().unwrap_or("").to_string();
        match APStore::delete_wifi_config(&ssid) {
            Ok(_) => {
                log::info!("Successfully deleted SSID: {}", ssid);
                req.into_response(302, Some("Successfully deleted SSID"), &[("Location", "/")])?;
            }
            Err(e) => {
                log::error!("Failed to delete SSID: {} - {}", ssid, e);
                req.into_response(302, Some("Failed to delete SSID"), &[("Location", "/")])?;
            }
        };
        Ok::<(), anyhow::Error>(())
    })?;

    // Handle adding a new AP
    server.fn_handler("/add", http::Method::Post, |mut req| {
        // Read the body of the request
        let mut buf = [0_u8; 256];
        let len = req.read(&mut buf)?;

        match serde_urlencoded::from_bytes(&buf[0..len]) {
            Ok(config) => {
                // Save the WiFi configuration
                match APStore::save_wifi_config(&config) {
                    Ok(_) => {
                        log::info!("Successfully saved SSID: {}", config.ssid);
                        req.into_response(
                            302,
                            Some("Successfully saved SSID"),
                            &[("Location", "/")],
                        )?;
                    }
                    Err(e) => {
                        log::error!("Failed to save SSID: {} - {}", config.ssid, e);
                        req.into_response(302, Some("Failed to save SSID"), &[("Location", "/")])?;
                    }
                }
            }
            Err(_) => {
                log::error!("Invalid form data");
                req.into_response(400, Some("Invalid form data"), &[])?;
            }
        }
        Ok::<(), anyhow::Error>(())
    })?;

    log::info!("Web server started");

    Ok(server)
}

/*
fn start_led_display() -> anyhow::Result<()> {
    let peripherals = Peripherals::take().unwrap();
    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio6;
    let mosi = peripherals.pins.gpio7;

    let spi_driver = spi::SpiDriver::new(
        spi,
        sclk,
        mosi,
        Option::<gpio::Gpio0>::None,
        &spi::SpiConfig::new().baudrate(3.MHz().into()),
    )?;

    let mut ws2812 = Ws2812::new(spi_driver);

    let colors = [smart_leds::RGB8::new(255, 0, 0); 64]; // Red color for all LEDs
    ws2812.write(colors.iter().cloned())?;

    println!("LED display started");

    Ok(())
}
*/
