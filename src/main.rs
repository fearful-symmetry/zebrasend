mod cmd;
mod config;
use clap::Parser;
use cmd::cups::Sender;
use cmd::sgd::{Sgd, SgdCmd};
use ipp::attribute::*;
use std::path::PathBuf;

#[derive(Parser)]
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
}

#[derive(clap::Subcommand)]
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

#[derive(clap::Subcommand)]
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();
    let config_path = match cli.config_file {
        Some(c) => c,
        None => {
            let mut home =
                dirs::home_dir().ok_or("Could not find home dir and no config path set")?;
            home.push(".zebrasend.toml");
            home
        }
    };
    let mut cfg = config::Cfg::new(config_path)?;

    let printer: config::Printer = match cfg.printer.remove(&cli.printer) {
        Some(p) => p,
        None => {
            let err_msg: Box<dyn std::error::Error> =
                format!("Printer {} does not exist in config", cli.printer).into();
            return Err(err_msg);
        }
    };

    let style = match cfg.style.remove(&cli.style) {
        Some(s) => s,
        None => {
            let err_msg: Box<dyn std::error::Error> =
                format!("Style {} does not exist in config", cli.style).into();
            return Err(err_msg);
        }
    };

    let client = Sender::new(printer.clone())?;
    match &cli.command {
        Commands::File { name } => {
            client.print_zpl_file(name.to_string()).await?;
        }
        Commands::Message { msg } => {
            client
                .print_zpl_string(style.create_zpl_message(msg.to_vec()))
                .await?;
        }
        Commands::Sgd { command } => {
            let sgd_handler = Sgd::new((printer.ip.as_str(), printer.port));
            let resp: String = match command {
                SGDCommands::Get { cmd } => sgd_handler.create_cmd(SgdCmd::Get, cmd.to_vec())?,
                SGDCommands::Set { cmd } => sgd_handler.create_cmd(SgdCmd::Set, cmd.to_vec())?,
                SGDCommands::Do { cmd } => sgd_handler.create_cmd(SgdCmd::Do, cmd.to_vec())?,
            };

            println!("{}", resp);
        }
        Commands::Options => {
            let attrs = client.get_attrs().await?;
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
