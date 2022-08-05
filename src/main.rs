mod cmd;
mod config;
use clap::Parser;
use cmd::cups::Cups;
use cmd::jetdirect::Jetdirect;
use ipp::attribute::*;
use std::path::PathBuf;

#[derive(Parser, Clone)]
#[clap(author, version, long_about = None)]
#[clap(about = "A CLI utility for sending commands to zebra printers via CUPS")]
#[clap(propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
    #[clap(short, long, value_parser, value_name = "FILE")]
    config_file: Option<PathBuf>,
    /// Printer from the specified config to use
    #[clap(short, long, value_parser, default_value_t = String::from("default"))]
    printer: String,
    /// Style from the specified config to use
    #[clap(short, long, value_parser, default_value_t = String::from("default"))]
    style: String,

    /// print mode to use for file and message subcommands
    #[clap(short='m', long, arg_enum, value_parser, default_value_t = PrintMode::Cups)]
    print_mode: PrintMode,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ArgEnum)]
enum PrintMode {
    Jetdirect,
    Cups,
}

#[derive(clap::Subcommand, Clone)]
enum Commands {
    /// Send a ZPL file to the printer
    File {
        #[clap(value_parser)]
        name: String,
    },
    /// Send a string message to the printer. Individual words will be printed to new lines, use quotes to print on a single line.
    Message {
        #[clap(value_parser)]
        msg: Vec<String>,
    },
    /// Send SGD commands to printer via telnet
    Sgd {
        #[clap(subcommand)]
        command: SGDCommands,
    },
    /// Return the options known to the printer
    Options,
}

#[derive(clap::Subcommand, Clone)]
enum SGDCommands {
    Get {
        #[clap(value_parser)]
        cmd: Vec<String>,
    },
    Set {
        #[clap(value_parser)]
        cmd: Vec<String>,
    },
    Do {
        #[clap(value_parser)]
        cmd: Vec<String>,
    },
}

impl SGDCommands {
    /// Turn an SGD instance into the full command needed by the printer
    pub fn build_cmd(&self) -> String {
        let prefix = "! U1";
        let postfix = std::str::from_utf8(&[13]).unwrap();
        match self {
            SGDCommands::Set { cmd } => {
                format!("{} setvar {} {}", prefix, gen_args(cmd.to_vec()), postfix)
            }
            SGDCommands::Get { cmd } => {
                format!("{} getvar {} {}", prefix, gen_args(cmd.to_vec()), postfix)
            }
            SGDCommands::Do { cmd } => {
                format!("{} do {} {}", prefix, gen_args(cmd.to_vec()), postfix)
            }
        }
    }
}

/// Small helper function for build_cmd() to properly format SGD args
fn gen_args(args: Vec<String>) -> String {
    let mut arg_str = String::new();
    for elem in args.iter() {
        arg_str = format!("{} \"{}\"", arg_str, elem)
    }
    arg_str
}

/// Handler for all the different protocol handlers used by the printer
/// We require some juggling here, as certain commands only work over cups, or jetdirect
struct PrinterHandler {
    pub jd_handler: Jetdirect,
    pub cups_handler: Cups,
    selected_handler: PrintMode,
}

impl PrinterHandler {
    fn new(cli: Args, cfg: &mut config::Cfg) -> Result<Self, Box<dyn std::error::Error>> {
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

    fn send_file(&self, path: String) -> Result<String, Box<dyn std::error::Error>> {
        match self.selected_handler {
            PrintMode::Jetdirect => self.jd_handler.send_file(path),
            PrintMode::Cups => self.cups_handler.send_file(path),
        }
    }
    fn send_string(&self, data: String) -> Result<String, Box<dyn std::error::Error>> {
        match self.selected_handler {
            PrintMode::Jetdirect => self.jd_handler.send_string(data),
            PrintMode::Cups => self.cups_handler.send_string(data),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();
    let config_path = match cli.config_file.clone() {
        Some(c) => c,
        None => {
            let mut home =
                dirs::home_dir().ok_or("Could not find home dir and no config path set")?;
            home.push(".zebrasend.toml");
            home
        }
    };
    let mut cfg = config::Cfg::new(config_path)?;
    let style = match cfg.style.remove(&cli.style) {
        Some(s) => s,
        None => {
            let err_msg: Box<dyn std::error::Error> =
                format!("Style {} does not exist in config", cli.style).into();
            return Err(err_msg);
        }
    };

    let printer = PrinterHandler::new(cli.clone(), &mut cfg)?;

    send(printer, cli, style)?;

    Ok(())
}

/// send whatever command the CLI has requested to the printer
fn send(
    printer: PrinterHandler,
    cmd: Args,
    style: cmd::zpl::MessageStyle,
) -> Result<(), Box<dyn std::error::Error>> {
    match &cmd.command {
        Commands::File { name } => {
            printer.send_file(name.to_string())?;
        }
        Commands::Message { msg } => {
            let print_msg = style.create_zpl_message(msg.to_vec());
            printer.send_string(print_msg)?;
        }
        Commands::Sgd { command } => {
            let sgd_string = command.build_cmd();
            let resp = printer.jd_handler.send_string(sgd_string)?;
            println!("{}", resp);
        }
        Commands::Options => {
            let attrs = printer.cups_handler.get_attrs()?;
            print_attrs(attrs);
        }
    }

    Ok(())
}

// helper, print the attributes from other API calls
fn print_attrs(printer_attrs: IppAttributes) {
    let groups = printer_attrs.groups();
    for group in groups {
        let attr_map = group.attributes();
        for (key, val) in attr_map {
            println!("Attr: {:?}, Data: {}={}", key, val.name(), val.value());
        }
    }
}
