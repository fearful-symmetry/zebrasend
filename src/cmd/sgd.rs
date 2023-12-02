use std::str::FromStr;

use anyhow::{anyhow, Ok};
use clap::Subcommand;

#[derive(Subcommand, Clone, PartialEq, Debug)]
pub enum SGDCommands {
    ///! U1 getvar
    Get {
        #[clap(value_parser)]
        cmd: Vec<String>,
    },
    ///! U1 setvar
    Set {
        #[clap(value_parser)]
        cmd: Vec<String>,
    },
    ///! U1 do
    Do {
        #[clap(value_parser)]
        cmd: Vec<String>,
    },
}


impl FromStr for SGDCommands {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let cmds: Vec<String> =  s.split_ascii_whitespace().map(|s| s.to_string()).collect();
        if cmds.len() < 3 {
            return Err(anyhow!("{} does not appear to be a full SGD command", s));
        };

        let final_cmds: Vec<String> = cmds[1..cmds.len()].to_vec();

        match cmds[0].as_ref() {
            "setvar" =>{
               Ok(SGDCommands::Set { cmd: final_cmds })
            },
            "getvar" => {
                Ok(SGDCommands::Get { cmd: final_cmds })
            }, 
            "do" => {
                Ok(SGDCommands::Do { cmd: final_cmds })
            },
            _ =>{
                Err(anyhow!("first verb must be one of 'setvar', 'getvar', or 'do'"))
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::SGDCommands;

    #[test]
    fn test_parse_sgd() {
        let test_str: anyhow::Result<SGDCommands> = "setvar ezpl.print_width 200".parse();
        let res = test_str.unwrap();
        assert_eq!(SGDCommands::Set { cmd: vec!["ezpl.print_width".to_string(), "200".to_string()] }, res)
    }

}