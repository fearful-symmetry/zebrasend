mod cli;
mod cmd;
mod config;
use clap::Parser;
use cli::FTPCommands;
use cmd::jetdirect::{Mode, Jetdirect};
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

    let printer: config::Printer = cfg.printer.remove(&cli.printer)
    .ok_or(anyhow::Error::msg(format!("Could not find printer {} in config", &cli.printer)))?;

    let printer = Jetdirect::new(printer.ip.clone(), printer.port);

    send(printer, cli, style)?;

    Ok(())
}

/// send whatever command the CLI has requested to the printer
fn send(
    printer: Jetdirect,
    cmd: cli::Args,
    style: cmd::zpl::MessageStyle,
) -> Result<()> {
    match &cmd.command {
        cli::Commands::File { name } => {
            printer.send_file(name.to_string(), Mode::Print)?;
        }
        cli::Commands::Message { msg } => {
            let print_msg = style.create_zpl_message(msg.to_vec());
            printer.send_string(print_msg, Mode::Print)?;
        }
        cli::Commands::Sgd { command } => {
            let sgd_string = command.build_cmd();
            println!("'{}'", sgd_string);
            printer.send_string(sgd_string, Mode::SGD)?;
        }
        cli::Commands::Ftp { command } => {
            handle_ftp(command, printer)?;
        }
    }

    Ok(()) 
}

fn handle_ftp(cmd: &FTPCommands, printer: Jetdirect) -> Result<()> {

    match cmd {
        FTPCommands::Put { name } => {
            println!("IP: {}", printer.addr);
            send_file(&printer.addr, name)?;
        }
    }

    Ok(())
}
