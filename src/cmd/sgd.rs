use std::net::ToSocketAddrs;
use telnet::{Event, Telnet};
pub struct Sgd<A> {
    telnet_addr: A,
}

pub enum SgdCmd {
    Get(Vec<String>),
    Set(Vec<String>),
    Do(Vec<String>),
}

impl<A: ToSocketAddrs + Copy> Sgd<A> {
    pub fn new(addr: A) -> Self
    where
        A: ToSocketAddrs + std::marker::Copy,
    {
        Sgd { telnet_addr: addr }
    }
    //let test = r#"! U1 getvar "ip.addr" "#;
    //let cmd_bytes = format!("{}{}", test, );
    pub fn command(&self, cmd_type: SgdCmd) -> Result<String, Box<dyn std::error::Error>> {
        let string_resp: String = match cmd_type {
            SgdCmd::Get(cmd) => self.send_sgd_string(build_sgd_cmd("getvar", cmd))?,
            SgdCmd::Set(cmd) => self.send_sgd_string(build_sgd_cmd("setvar", cmd))?,
            SgdCmd::Do(cmd) => self.send_sgd_string(build_sgd_cmd("do", cmd))?,
        };
        Ok(string_resp)
    }

    fn send_sgd_string(&self, cmd: String) -> Result<String, Box<dyn std::error::Error>> {
        let mut telnet = Telnet::connect(self.telnet_addr, 256)?;
        telnet.write(cmd.as_bytes())?;
        let ascii_quote: u8 = 34;
        let mut done = false;
        // not sure of a better way to check if a SGD command response is done,
        // since it doesn't send any ascii control characters
        let mut quote_count = 0;
        let mut resp_acc = Vec::new();
        loop {
            if quote_count >= 2 || done {
                break;
            }
            let event = telnet.read_timeout(std::time::Duration::new(5, 0))?;
            match event {
                Event::Data(data) => {
                    quote_count += data.iter().filter(|n| *n == &ascii_quote).count();
                    resp_acc.extend_from_slice(&data);
                }
                Event::TimedOut => {
                    println!("Connection timed out");
                    done = true;
                }
                _ => {
                    println!("Got other event: {:?}", event)
                }
            }
        }
        Ok(String::from_utf8(resp_acc)?)
    }
}

fn build_sgd_cmd(cmd_type: &str, args: Vec<String>) -> String {
    let prefix = "! U1";
    let postfix = std::str::from_utf8(&[13]).unwrap();

    format!(
        "{} {} {} {}",
        prefix,
        cmd_type,
        format_sgd_args(args),
        postfix
    )
}

fn format_sgd_args(args: Vec<String>) -> String {
    let mut arg_str = String::new();
    for elem in args.iter() {
        arg_str = format!("{} \"{}\"", arg_str, elem)
    }
    arg_str
}

// Telnet would be:
// if data[data.len() - 3..data.len()] == [3, 13, 10] {
//     println!("Done getting response.");
//     done = true
// }
