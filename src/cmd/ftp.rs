use ftp::FtpStream;
use std::path::Path;
use std::fs::File;
use anyhow::{Result, Error};

pub fn send_file(addr: &str, fpath: &str) -> Result<()> {
    // Zebra's FTP is pretty simplistic; we really only have one path (E:)
    // So just extract the file name and use that as the destination.
    let fname = Path::new(addr).file_name().ok_or(Error::msg("no filename found in source path"))?.to_string_lossy();
    let mut reader = File::open(fpath)?;

    let conn_pair = (addr, 21);
    let mut ftp_stream = FtpStream::connect(conn_pair)?;
    // The zebra FTP doesn't seem to have any user/password pairs, can just be whatever
    ftp_stream.login("user", "")?;
    ftp_stream.put(&fname, &mut reader)?;
    

    Ok(())
}