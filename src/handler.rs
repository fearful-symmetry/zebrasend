use std::fmt::format;

use crate::cli;
use crate::cmd::cups::Cups;
use crate::cmd::jetdirect::{Jetdirect, Mode};
use crate::config;
use anyhow::Result;

/// Handler for all the different protocol handlers used by the printer
/// We require some juggling here, as certain commands only work over cups, or jetdirect
pub struct PrinterHandler {
    pub jd_handler: Jetdirect,
    pub cups_handler: Cups,
    selected_handler: cli::PrintMode,
    pub printer_ip: String,
}

impl PrinterHandler {
    pub fn new(cli: cli::Args, cfg: &mut config::Cfg) -> Result<Self> {
        let printer: config::Printer = cfg.printer.remove(&cli.printer).ok_or(anyhow::Error::msg(format!("Could not find printer {} in config", &cli.printer)))?;

        let cups_handler = Cups::new(printer.clone())?;
        let jetdirect_handler = Jetdirect::new(printer.ip.clone(), printer.port);

        Ok(PrinterHandler {
            jd_handler: jetdirect_handler,
            cups_handler,
            selected_handler: cli.print_mode,
            printer_ip: printer.ip,
        })
    }

    pub fn send_file(&self, path: String) -> Result<()> {
        match self.selected_handler {
            cli::PrintMode::Jetdirect => self.jd_handler.send_file(path, Mode::Print),
            cli::PrintMode::Cups => self.cups_handler.send_file(path),
        }
    }
    pub fn send_string(&self, data: String) -> Result<()> {
        match self.selected_handler {
            cli::PrintMode::Jetdirect => self.jd_handler.send_string(data, Mode::Print),
            cli::PrintMode::Cups => self.cups_handler.send_string(data),
        }
    }
}
