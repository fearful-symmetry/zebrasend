use crate::cli;
use crate::cmd::cups::Cups;
use crate::cmd::jetdirect::Jetdirect;
use crate::config;

/// Handler for all the different protocol handlers used by the printer
/// We require some juggling here, as certain commands only work over cups, or jetdirect
pub struct PrinterHandler {
    pub jd_handler: Jetdirect,
    pub cups_handler: Cups,
    selected_handler: cli::PrintMode,
}

impl PrinterHandler {
    pub fn new(cli: cli::Args, cfg: &mut config::Cfg) -> Result<Self, Box<dyn std::error::Error>> {
        let printer: config::Printer = match cfg.printer.remove(&cli.printer) {
            Some(p) => p,
            None => {
                let err_msg: Box<dyn std::error::Error> =
                    format!("Printer {} does not exist in config", cli.printer).into();
                return Err(err_msg);
            }
        };

        let cups_handler = Cups::new(printer.clone())?;
        let jetdirect_handler = Jetdirect::new(printer.ip, printer.port);

        Ok(PrinterHandler {
            jd_handler: jetdirect_handler,
            cups_handler,
            selected_handler: cli.print_mode,
        })
    }

    pub fn send_file(&self, path: String) -> Result<String, Box<dyn std::error::Error>> {
        match self.selected_handler {
            cli::PrintMode::Jetdirect => self.jd_handler.send_file(path),
            cli::PrintMode::Cups => self.cups_handler.send_file(path),
        }
    }
    pub fn send_string(&self, data: String) -> Result<String, Box<dyn std::error::Error>> {
        match self.selected_handler {
            cli::PrintMode::Jetdirect => self.jd_handler.send_string(data),
            cli::PrintMode::Cups => self.cups_handler.send_string(data),
        }
    }
}
