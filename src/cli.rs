use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Clone)]
#[clap(author, version, long_about = None)]
#[clap(about = "A CLI utility for sending commands to zebra printers via CUPS")]
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
    /// send and delete files via FTP
    Ftp {
        #[clap(subcommand)]
        command: FTPCommands,
    },
}

#[derive(clap::Subcommand, Clone)]
pub enum FTPCommands {
    Put {
        #[clap(value_parser)]
        name: String,
    }
}

#[derive(clap::Subcommand, Clone)]
pub enum SGDCommands {
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
                format!("{} setvar{} {}", prefix, gen_args(cmd.to_vec()), postfix)
            }
            SGDCommands::Get { cmd } => {
                format!("{} getvar{} {}", prefix, gen_args(cmd.to_vec()), postfix)
            }
            SGDCommands::Do { cmd } => {
                format!("{} do{} {}", prefix, gen_args(cmd.to_vec()), postfix)
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
