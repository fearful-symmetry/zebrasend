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
                            // if we got a whole error string ("?") just return now.
                            if resp_part == String::from(r#""?""#) {
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
        let mut telnet = Telnet::connect((self.addr.clone(), self.port), 256)?;
        self.send_command_and_print(payload, &mut telnet, mode)
    }

    pub fn send_string(&self, data: String, mode: Mode) -> Result<(), Box<dyn std::error::Error>> {
        let mut telnet = Telnet::connect((self.addr.clone(), self.port), 256)?;
        self.send_command_and_print(data, &mut telnet, mode)
    }
}
