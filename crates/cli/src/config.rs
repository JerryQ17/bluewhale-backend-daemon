use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub addr: IpAddr,
    pub port: u16,
}
