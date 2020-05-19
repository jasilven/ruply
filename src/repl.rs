use super::Result;
use std::io::{Read, Write};
use std::net::TcpStream;

use super::bail;
use super::nrepl::Nrepl;
use super::prepl::Prepl;

pub enum Response {
    Done(Option<String>),
    Exception(String),
    StdOut(String),
    StdErr(String),
    Other(String),
}

pub trait Repl {
    fn send(&mut self, s: &str) -> Result<()>;
    fn namespace(&self) -> &str;
    fn recv(&mut self) -> Result<Response>;
    fn quit(&mut self) -> Result<()>;
    fn name(&self) -> &str;
}

pub fn get_repl(host: &str, port: usize) -> Result<Box<dyn Repl>> {
    let mut stream = TcpStream::connect(format!("{}:{}", host, port))?;
    let _ = stream.write_all(b"d4:code7:(+ 1 1)2:op4:evale\n")?;
    stream.flush()?;

    let mut buf = [0u8; 1];
    stream.read_exact(&mut buf)?;

    // restart connection from clean state
    stream.shutdown(std::net::Shutdown::Both)?;
    stream = TcpStream::connect(format!("{}:{}", host, port))?;

    match buf[0] {
        123 => {
            let repl = Prepl::new(stream)?;
            return Ok(Box::new(repl));
        }
        100 => {
            let repl = Nrepl::new(stream)?;
            return Ok(Box::new(repl));
        }
        _ => bail!("Unable to identify nREPL/pREPL"),
    }
}

