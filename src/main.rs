mod cli;
mod cmd;
mod config;
use clap::Parser;
use cli::FTPCommands;
use cmd::jetdirect::{Mode, Jetdirect};
use cmd::ftp::send_file;
use anyhow::Result;
use std::collections::HashMap;

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
    let cfg = config::Cfg::new(config_path)?;


    send(cli, cfg)?;

    Ok(())
}

/// send whatever command the CLI has requested to the printer
fn send(
    cmd: cli::Args,
    cfg: config::Cfg,
) -> Result<()> {

    let style = match cfg.style.get(&cmd.style) {
        Some(s) => s,
        None => {
            let err_msg =
                anyhow::Error::msg(format!("Style {} does not exist in config", cmd.style));
            return Err(err_msg);
        }
    };

    let printer = cfg.printer.get(&cmd.printer)
    .ok_or(anyhow::Error::msg(format!("Could not find printer {} in config", &cmd.printer)))?;

    let printer = Jetdirect::new(printer.ip.clone(), printer.port);


    match &cmd.command {
        cli::Commands::File { name } => {
            let print_mode = match std::path::Path::new(name).extension(){
                Some(ext) => {
                    match ext.to_string_lossy().as_ref() {
                        "nrd" => {
                            Mode::SGD
                        }
                        _ => {
                            Mode::Print
                        }
                    }
                }
                None => Mode::Print
            };
            println!("Printing file with mode: {:?}", print_mode);
            printer.send_file(name.to_string(), print_mode)?;
        }
        cli::Commands::Message { msg, count } => {
            let print_msg = style.clone().create_zpl_message(msg.to_vec());
            for _ in 0..*count  {
                printer.send_string(print_msg.clone(), Mode::Print)?;
            }
            
        }
        cli::Commands::Raw { msg, count } => {
            for _ in 0..*count {
                printer.send_string(msg.to_string(), Mode::Print)?;
            }
            
        }
        cli::Commands::Sgd { command } => {
            printer.send_sgd_cmd(command.clone())?;
        }
        cli::Commands::Ftp { command } => {
            handle_ftp(command, printer)?;
        },
        cli::Commands::Styles => {
            print_map_keys(cfg.style)
        },
        cli::Commands::Printers => {
            print_map_keys(cfg.printer)
        }
    }

    Ok(()) 
}

fn print_map_keys<T>(style_list: HashMap<String, T>) {

    for key in style_list.keys() {
        println!("- {}", key);
    }
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
