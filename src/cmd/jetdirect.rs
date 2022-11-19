use std::io::{Write, ErrorKind};
use anyhow::{Result, Context};
use crate::cmd::sgd;
use std::time::Duration;
use std::env;
use telnet::{Event, Telnet};

use std::io::prelude::*;
use std::net::TcpStream;

pub struct Jetdirect {
    pub addr: String,
    port: u16,
}

#[derive(PartialEq, Debug)]
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
    fn send_cmd_net_raw(
        &self,
        payload: String,
        timeout: Duration
    ) -> Result<()> {

        let mut stream = TcpStream::connect("192.168.1.180:9100").with_context(|| "error connecting to printer")?;
        stream.write(payload.as_bytes()).with_context(|| "error writing message")?;
        stream.set_nonblocking(false).with_context(|| "error setting nonblocking mode")?;
        stream.set_read_timeout(Some(timeout)).with_context(|| "error setting read timeout")?;
        let mut resp_vec: Vec<u8> = Vec::new();
        loop {
            // There's no way to tell if we're done reading a response as far as I can tell
            // so read the stupid thing in one chunk at a time
            let mut read_buf = [0; 1];
            // if we get an error, it could be a timeout and we're done reading
            let read_bytes = match stream.read(&mut read_buf) {
                Ok(b) => b,
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock {
                        continue
                    }
                    let quote: u8 = 34;
                    let quotes = resp_vec.iter().filter(|x| *x == &quote).count();
                    if quotes >= 2 {
                        0
                    } else {
                        println!("Error in read call:");
                        return Err(e.into());
                    }
                }
            };
            resp_vec.append(&mut read_buf.to_vec());
            //print!("{}", resp);
            if read_bytes == 0 || return_early(&resp_vec) {
                println!("{}", String::from_utf8_lossy(&resp_vec));
                break
            }
        }
            
        Ok(())
    }
    fn send_command_and_print(
        &self,
        payload: String,
        handle: &mut telnet::Telnet,
        mode: Mode,
        timeout: Duration
    ) -> Result<()> {
        handle.write(payload.as_bytes())?;
        // As far as I can tell, there's no way to detect the end of an SGD command response.
        // There can be any number of double-quotes; there's no terminating control character, newline, etc.
        // Only thing we can really do is print lines as we get them, and wait for a timeout.

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
                    // We don't get the line break at the end of a response, usually
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

    pub fn send_file(&self, path: String, mode: Mode) -> Result<()> {
        let payload = std::fs::read_to_string(path)?;
        let mut telnet = Telnet::connect((self.addr.clone(), self.port), 512)?;
        self.send_command_and_print(payload, &mut telnet, mode, Duration::new(2, 0))
    }

    pub fn send_string(&self, data: String, mode: Mode) -> Result<()> {
        let mut telnet = Telnet::connect((self.addr.clone(), self.port), 512)?;
        self.send_command_and_print(data, &mut telnet, mode, Duration::new(2, 0))
    }

   pub fn send_sgd_cmd(&self, data: sgd::SGDCommands) -> Result<()> {
        let sgd_string = data.build_cmd();
        let mut telnet = Telnet::connect((self.addr.clone(), self.port), 512)?;
        // Set different timeouts depending on what command type we're doing.
        let timeout = match data {
            sgd::SGDCommands::Get { cmd: _ } => Duration::new(2, 0),
            sgd::SGDCommands::Set { cmd: _ } => Duration::new(0, 100),
            sgd::SGDCommands::Do { cmd: _ } => Duration::new(2, 0),
        };
        match env::var_os("SGD_NO_TELNET") {
            Some(_) =>  self.send_cmd_net_raw(sgd_string, timeout),
            None => self.send_command_and_print(sgd_string, &mut telnet, Mode::SGD, timeout)
        }
        
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
    // if only have two quotes, but no whitespace formatting, we probably got back a basic response, can return
    let quote_count = payload.iter().filter(|x| *x == &quote).count();
    let ws_count = payload.iter().filter(|x| *x == &nl || *x == &tab).count();

    if ws_count == 0 && quote_count == 2 {
        return true
    }

    return false
}