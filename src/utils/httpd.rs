use askama::Template;
use esp_idf_svc::http;
use esp_idf_svc::http::server::{
    Configuration as HttpConfig, EspHttpConnection, EspHttpServer, Request,
};
use esp_idf_sys as _; // Import the ESP-IDF bindings

use crate::nvs::APStore;
use crate::wifi::WIFI_SCAN;

#[derive(askama::Template)]
#[template(path = "config_page.html")]
struct ConfigPage<'a> {
    visible: Vec<(&'a str, u8, i8, &'a str)>,
    aps: Vec<&'a str>,
}

pub fn start_http_server<'a>() -> anyhow::Result<EspHttpServer<'a>> {
    log::info!("Starting HTTPD:");
    let config: HttpConfig = HttpConfig {
        uri_match_wildcard: true,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&config)?;

    server.fn_handler("/", http::Method::Get, handle_config)?;
    server.fn_handler("/hello", http::Method::Get, handle_hello)?;
    server.fn_handler("/delete/*", http::Method::Get, handle_delete)?;
    server.fn_handler("/add", http::Method::Post, handle_add)?;

    log::info!("Web server started");

    Ok(server)
}

fn handle_hello(request: Request<&mut EspHttpConnection>) -> anyhow::Result<()> {
    let mut response = request.into_ok_response()?;
    response.write("Hello from ESP32-C3!\n".as_bytes())?;
    Ok(())
}

fn handle_config(request: Request<&mut EspHttpConnection>) -> anyhow::Result<()> {
    let aps = APStore::get_known_aps()?;
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
        aps: aps.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
    };
    let mut response = request.into_ok_response()?;
    let html = config_page.render()?;
    response.write(html.as_bytes())?;
    Ok::<(), anyhow::Error>(())
}

fn handle_delete(request: Request<&mut EspHttpConnection>) -> anyhow::Result<()> {
    log::info!("DELETE: {:?}", request.uri());
    let ssid = request.uri().split('/').next_back().expect("Invalid SSID");
    let ssid = urlencoding::decode(ssid)?;
    if APStore::get_ap_config(&ssid)?.is_some() {
        match APStore::delete_ap(&ssid) {
            Ok(_) => {
                log::info!("Successfully deleted SSID: {}", ssid);
                request.into_response(
                    302,
                    Some("Successfully deleted SSID"),
                    &[("Location", "/")],
                )?;
            }
            Err(e) => {
                log::error!("Failed to delete SSID: {} - {}", ssid, e);
                request.into_response(302, Some("Failed to delete SSID"), &[("Location", "/")])?;
            }
        }
    } else {
        log::error!("Unknown SSID: {}", ssid);
        request.into_response(302, Some("Unknown SSID"), &[("Location", "/")])?;
    }
    Ok::<(), anyhow::Error>(())
}

fn handle_add(mut request: Request<&mut EspHttpConnection>) -> anyhow::Result<()> {
    // Read the body of the request
    let mut buf = [0_u8; 256];
    let len = request.read(&mut buf)?;

    match serde_urlencoded::from_bytes(&buf[0..len]) {
        Ok(config) => {
            // Save the WiFi configuration
            match APStore::add_ap(&config) {
                Ok(_) => {
                    log::info!("Successfully saved SSID: {}", config.ssid);
                    request.into_response(
                        302,
                        Some("Successfully saved SSID"),
                        &[("Location", "/")],
                    )?;
                }
                Err(e) => {
                    log::error!("Failed to save SSID: {} - {}", config.ssid, e);
                    request.into_response(
                        302,
                        Some("Failed to save SSID"),
                        &[("Location", "/")],
                    )?;
                }
            }
        }
        Err(_) => {
            log::error!("Invalid form data");
            request.into_response(400, Some("Invalid form data"), &[])?;
        }
    }
    Ok::<(), anyhow::Error>(())
}
