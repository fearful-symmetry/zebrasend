use clap::Parser;
use std::path::PathBuf;
use crate::cmd::sgd::SGDCommands;

#[derive(Parser, Clone)]
#[clap(author, version, long_about = None)]
#[clap(about = "A CLI utility for sending commands to zebra printers via Jetdirect and FTP")]
#[clap(propagate_version = true)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,
    #[clap(short, long, value_parser, value_name = "FILE")]
    pub config_file: Option<PathBuf>,
    /// Printer from the specified config to use
    #[clap(short, long, value_parser, default_value_t = String::from("default"))]
    pub printer: String,
    /// Style from the specified config to use
    #[clap(short, long, value_parser, default_value_t = String::from("default"))]
    pub style: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ArgEnum)]
pub enum PrintMode {
    Jetdirect,
    Cups,
}

#[derive(clap::Subcommand, Clone)]
pub enum Commands {
    /// Send a ZPL or NRD file to the printer
    File {
        #[clap(value_parser)]
        name: String,
    },
    /// Send a string message to the printer. Individual words will be printed to new lines, use quotes to print on a single line.
    Message {
        #[clap(value_parser)]
        msg: Vec<String>,
        /// Print the message n times
        #[clap(short, long, value_parser, default_value_t = 1)]
        count: i32,
    },
    /// Sends a raw ZPL string to the printer
    Raw {
        #[clap(value_parser)]
        msg: String,
        /// Print the message n times
        #[clap(short, long, value_parser, default_value_t = 1)]
        count: i32,
    },
    /// Send SGD commands to printer via telnet
    Sgd {
        #[clap(subcommand)]
        command: SGDCommands,
    },
    /// send and delete files via FTP
    Ftp {
        #[clap(subcommand)]
        command: FTPCommands,
    },
    /// Print a list of configured styles used by the 'message' command
    Styles,
    /// Print a list of configured printers
    Printers,
}



#[derive(clap::Subcommand, Clone)]
pub enum FTPCommands {
    Put {
        #[clap(value_parser)]
        name: String,
    }
}

