use heapless::String;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct WifiConfig {
    pub ssid: String<32>,
    pub password: String<64>,
}

impl WifiConfig {
    pub fn new(ssid: &str, password: &str) -> anyhow::Result<Self> {
        Ok(WifiConfig {
            ssid: ssid
                .try_into()
                .map_err(|_| anyhow::anyhow!("Failed to create SSID"))?,
            password: password
                .try_into()
                .map_err(|_| anyhow::anyhow!("Failed to create PW"))?,
        })
    }
}
