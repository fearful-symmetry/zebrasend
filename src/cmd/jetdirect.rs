use telnet::{Event, Telnet};

pub struct Jetdirect {
    addr: String,
    port: u16,
}

impl Jetdirect {
    pub fn new(addr: String, port: u16) -> Jetdirect {
        Jetdirect { addr, port }
    }
}

impl Jetdirect {
    fn send_and_get_resp(
        &self,
        payload: String,
        handle: &mut telnet::Telnet,
    ) -> Result<String, Box<dyn std::error::Error>> {
        handle.write(payload.as_bytes())?;
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
            let event = handle.read_timeout(std::time::Duration::new(5, 0))?;
            match event {
                Event::Data(data) => {
                    quote_count += data.iter().filter(|n| *n == &ascii_quote).count();
                    resp_acc.extend_from_slice(&data);
                }
                Event::TimedOut => {
                    done = true;
                }
                _ => {
                    println!("Got other jetdirect event: {:?}", event)
                }
            }
        }
        Ok(String::from_utf8(resp_acc)?)
    }
    pub fn send_file(&self, path: String) -> Result<String, Box<dyn std::error::Error>> {
        let payload = std::fs::read_to_string(path)?;
        let mut telnet = Telnet::connect((self.addr.clone(), self.port), 256)?;
        self.send_and_get_resp(payload, &mut telnet)
        //telnet.write(payload.as_bytes())
    }

    pub fn send_string(&self, data: String) -> Result<String, Box<dyn std::error::Error>> {
        let mut telnet = Telnet::connect((self.addr.clone(), self.port), 256)?;
        self.send_and_get_resp(data, &mut telnet)
    }
}

// impl Sender for Jetdirect {
//     fn send_file(&self, path: String) -> Result<(), Box<dyn std::error::Error>> {
//         let payload = std::fs::read_to_string(path)?;
//         let mut telnet = Telnet::connect((self.addr.clone(), self.port), 256)?;
//         self.send_and_get_resp(payload, telnet)?;
//         telnet.write(payload.as_bytes())?;
//         Ok(())
//     }

//     fn send_string(&self, data: String) -> Result<(), Box<dyn std::error::Error>> {
//         let mut telnet = Telnet::connect((self.addr.clone(), self.port), 256)?;
//         telnet.write(data.as_bytes())?;
//         Ok(())
//     }
// }

// pub fn send_jd<A>(raw_print: String, addr: A) -> Result<(), Box<dyn std::error::Error>>
// where
//     A: ToSocketAddrs + std::marker::Copy,
// {
//     let mut telnet = Telnet::connect(addr, 256)?;
//     telnet.write(raw_print.as_bytes())?;
//     Ok(())
// }
