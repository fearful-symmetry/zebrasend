use crate::cmd::zpl::MessageStyle;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;
use anyhow::Result;

#[derive(Deserialize)]
pub struct Cfg {
    pub printer: HashMap<String, Printer>,
    pub style: HashMap<String, MessageStyle>,
}

#[derive(Deserialize, Clone)]
pub struct Printer {
    pub uri: String,
    #[serde(default)]
    pub ip: String,
    #[serde(default = "Cfg::default_port")]
    pub port: u16,
    #[serde(default = "Cfg::default_user_name")]
    pub user_name: String,
    #[serde(default = "Cfg::default_print_mode")]
    pub ze_print_mode: String,
}

impl Cfg {
    pub fn new(cfg_path: PathBuf) -> Result<Self> {
        let cfg_str = read_to_string(cfg_path)?;
        let config: Cfg = toml::from_str(&cfg_str)?;

        Ok(config)
    }

    fn default_user_name() -> String {
        String::from("user")
    }
    fn default_print_mode() -> String {
        String::from("Peel")
    }
    fn default_port() -> u16 {
        9100
    }
}
