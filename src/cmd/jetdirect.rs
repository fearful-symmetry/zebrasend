use std::io::Write;

use telnet::{Event, Telnet};

pub struct Jetdirect {
    addr: String,
    port: u16,
}

#[derive(PartialEq)]
pub enum Mode {
    SGD,
    Print
}

impl Jetdirect {
    pub fn new(addr: String, port: u16) -> Jetdirect {
        Jetdirect { addr, port }
    }
}

impl Jetdirect {
    fn send_command_and_print(
        &self,
        payload: String,
        handle: &mut telnet::Telnet,
        mode: Mode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        handle.write(payload.as_bytes())?;
        // As far as I can tell, there's no way to detect the end of an SGD command response.
        // There can be any number of double-quotes; there's no terminating control character, newline, etc.
        // Only thing we can really do is print lines as we get them, and wait for a timeout.

        let timeout = match mode {
            Mode::Print => std::time::Duration::new(2, 0),
            Mode::SGD => std::time::Duration::new(4, 0)
        };
        
        loop {
            let event = handle.read_timeout(timeout)?;
            match event {
                Event::Data(data) => {
                        if mode == Mode::SGD {
                            let resp_part = String::from_utf8_lossy(&data);
                            print!("{}",resp_part);
                            std::io::stdout().flush()?;
                            if return_early(&data) {
                                println!("");
                                break
                            }
                        }

                }
                Event::TimedOut => {
                    // We don't get the linebreak at the end of a response, usually
                    println!("");
                    break
                }
                _ => {
                    println!("Got other jetdirect event: {:?}", event)
                }
            }
        }
        Ok(())
    }

    pub fn send_file(&self, path: String, mode: Mode) -> Result<(), Box<dyn std::error::Error>> {
        let payload = std::fs::read_to_string(path)?;
        let mut telnet = Telnet::connect((self.addr.clone(), self.port), 512)?;
        self.send_command_and_print(payload, &mut telnet, mode)
    }

    pub fn send_string(&self, data: String, mode: Mode) -> Result<(), Box<dyn std::error::Error>> {
        let mut telnet = Telnet::connect((self.addr.clone(), self.port), 512)?;
        self.send_command_and_print(data, &mut telnet, mode)
    }
}


// some heuristics to guess if we can return early from listening on the socket
fn return_early(payload :&[u8]) -> bool {
    let err = r#""?""#.as_bytes();
    if payload == err {
        return true
    };
    let quote: u8 = 34;
    let nl: u8 = 10;
    let tab: u8 = 9;
    // if only have two quotes, but no whitespace formatting, we probably got back a basic repsonse, can return
    let quote_count = payload.iter().filter(|x| *x == &quote).count();
    let ws_count = payload.iter().filter(|x| *x == &nl || *x == &tab).count();

    if ws_count == 0 && quote_count == 2 {
        return true
    }

    return false
}