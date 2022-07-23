use std::net::ToSocketAddrs;
use telnet::{Event, Telnet};
pub struct Sgd<A> {
    telnet_addr: A,
}

pub enum SgdCmd {
    Set,
    Get,
    Do,
}

impl From<SgdCmd> for String {
    fn from(cmd: SgdCmd) -> Self {
        match cmd {
            SgdCmd::Set => "setvar".to_string(),
            SgdCmd::Get => "getvar".to_string(),
            SgdCmd::Do => "do".to_string(),
        }
    }
}

impl<A: ToSocketAddrs + Copy> Sgd<A> {
    pub fn new(addr: A) -> Self
    where
        A: ToSocketAddrs + std::marker::Copy,
    {
        Sgd { telnet_addr: addr }
    }

    pub fn create_cmd(
        &self,
        cmd_type: SgdCmd,
        args: Vec<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.send_sgd_string(build_sgd_cmd(cmd_type.into(), args))
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

fn build_sgd_cmd(cmd_type: String, args: Vec<String>) -> String {
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
