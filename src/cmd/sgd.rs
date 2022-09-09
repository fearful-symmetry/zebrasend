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