mod cli;
mod cmd;
mod config;
mod handler;
use clap::Parser;
use cli::FTPCommands;
use ipp::attribute::*;
use cmd::jetdirect::Mode;
use cmd::ftp::send_file;
use anyhow::Result;

fn main() -> Result<()> {
    let cli = cli::Args::parse();
    let config_path = match cli.config_file.clone() {
        Some(c) => c,
        None => {
            let mut home =
                dirs::home_dir().ok_or(anyhow::Error::msg("Could not find home dir and no config path set"))?;
            home.push(".zebrasend.toml");
            home
        }
    };
    let mut cfg = config::Cfg::new(config_path)?;
    let style = match cfg.style.remove(&cli.style) {
        Some(s) => s,
        None => {
            let err_msg =
                anyhow::Error::msg(format!("Style {} does not exist in config", cli.style));
            return Err(err_msg);
        }
    };

    let printer = handler::PrinterHandler::new(cli.clone(), &mut cfg)?;

    send(printer, cli, style)?;

    Ok(())
}

/// send whatever command the CLI has requested to the printer
fn send(
    printer: handler::PrinterHandler,
    cmd: cli::Args,
    style: cmd::zpl::MessageStyle,
) -> Result<()> {
    match &cmd.command {
        cli::Commands::File { name } => {
            printer.send_file(name.to_string())?;
        }
        cli::Commands::Message { msg } => {
            let print_msg = style.create_zpl_message(msg.to_vec());
            printer.send_string(print_msg)?;
        }
        cli::Commands::Sgd { command } => {
            let sgd_string = command.build_cmd();
            println!("'{}'", sgd_string);
            printer.jd_handler.send_string(sgd_string, Mode::SGD)?;
        }
        cli::Commands::Options => {
            let attrs = printer.cups_handler.get_attrs()?;
            print_attrs(attrs);
        },
        cli::Commands::Ftp { command } => {
            handle_ftp(command, printer)?;
        }
    }

    Ok(()) 
}

fn handle_ftp(cmd: &FTPCommands, printer: handler::PrinterHandler) -> Result<()> {

    match cmd {
        FTPCommands::Put { name } => {
            println!("IP: {}", printer.printer_ip);
            send_file(&printer.printer_ip, name)?;
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
