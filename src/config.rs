use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Cfg {
    pub printer: HashMap<String, Printer>,
    pub style: HashMap<String, Style>,
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

#[derive(Deserialize, Clone)]
pub struct Style {
    #[serde(default = "Cfg::default_font_size")]
    pub font_size: i32,
    #[serde(default)]
    pub invert: bool,
    #[serde(default = "Cfg::default_font")]
    pub font: String,
    #[serde(default)]
    pub line_padding: i32,
}

impl Cfg {
    pub fn new(cfg_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
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
    fn default_font_size() -> i32 {
        35
    }
    fn default_font() -> String {
        "A".to_string()
    }
    fn default_port() -> u16 {
        9100
    }
}
