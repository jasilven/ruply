use super::repl::Repl;
use super::Result;
use std::collections::HashMap;
use std::io::BufReader;
use std::net::TcpStream;

use super::bail;
use super::util::write_and_flush;
use super::Response;

pub struct Nrepl {
    name: String,
    namespace: String,
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl Nrepl {
    pub fn new(stream: TcpStream) -> Result<Nrepl> {
        let nrepl = Nrepl {
            name: "nREPL".into(),
            namespace: "user".into(),
            writer: stream.try_clone()?,
            reader: BufReader::new(stream),
        };
        Ok(nrepl)
    }
}

impl Repl for Nrepl {
    fn quit(&mut self) -> Result<()> {
        Ok(())
    }

    fn namespace(&self) -> &str {
        &self.namespace
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn send(&mut self, s: &str) -> Result<()> {
        let mut map: HashMap<&str, &str> = HashMap::new();

        map.insert("op", "eval");
        map.insert("code", s);

        write_and_flush(&mut self.writer, &bencode_rs::Value::from(map).to_bencode())?;

        Ok(())
    }

    fn recv(&mut self) -> Result<Response> {
        match bencode_rs::parse_bencode(&mut self.reader) {
            Ok(Some(bencode_rs::Value::Map(map))) => {
                if let Some(bencode_rs::Value::Str(s)) =
                    map.get(&bencode_rs::Value::Str("ns".into()))
                {
                    self.namespace = s.into();
                }
                if let Some(bencode_rs::Value::Str(s)) =
                    map.get(&bencode_rs::Value::Str("err".into()))
                {
                    Ok(Response::StdErr(s.into()))
                } else if let Some(bencode_rs::Value::Str(s)) =
                    map.get(&bencode_rs::Value::Str("out".into()))
                {
                    Ok(Response::StdOut(s.into()))
                } else if let Some(bencode_rs::Value::Str(s)) =
                    map.get(&bencode_rs::Value::Str("value".into()))
                {
                    Ok(Response::StdOut(format!("{}\n", s)))
                } else if let Some(bencode_rs::Value::List(list)) =
                    map.get(&bencode_rs::Value::Str("status".into()))
                {
                    if list.contains(&bencode_rs::Value::Str("done".into())) {
                        Ok(Response::Done(None))
                    } else {
                        Ok(Response::Other("".into()))
                    }
                } else {
                    bail!("Unexpected response message from nREPL: {:?}", map);
                }
            }
            Ok(None) => bail!("nREPL died?"),
            Ok(_) => bail!("Unexpected response from nREPL"),
            Err(e) => bail!("Error: {} ", e),
        }
    }
}
