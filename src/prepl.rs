use super::anyhow;
use super::bail;
use super::format_err;
use super::repl::Repl;
use super::util::write_and_flush;
use super::Response;
use super::Result;
use edn::parser::Parser;
use std::collections::BTreeMap;
use std::io::BufRead;
use std::io::BufReader;
use std::net::TcpStream;

fn get_value(key: &str, map: &BTreeMap<edn::Value, edn::Value>) -> Option<String> {
    match map.get(&edn::Value::Keyword(key.into())) {
        Some(edn::Value::String(value)) => {
            return Some(value.into());
        }
        Some(edn::Value::Keyword(value)) => {
            return Some(value.into());
        }
        Some(edn::Value::Boolean(value)) => {
            return Some(value.to_string());
        }
        _ => None,
    }
}

pub struct Prepl {
    name: String,
    namespace: String,
    reader: BufReader<TcpStream>,
    writer: TcpStream,
}

impl Prepl {
    pub fn new(stream: TcpStream) -> Result<Prepl> {
        let prepl = Prepl {
            name: "pREPL".into(),
            namespace: "user".into(),
            writer: stream.try_clone()?,
            reader: BufReader::new(stream),
        };
        Ok(prepl)
    }
}

impl Repl for Prepl {
    fn quit(&mut self) -> Result<()> {
        write_and_flush(&mut self.writer, ":repl/quit\n")?;
        Ok(())
    }

    fn namespace(&self) -> &str {
        self.namespace.as_str()
    }

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn send(&mut self, s: &str) -> Result<()> {
        write_and_flush(&mut self.writer, &s)?;
        Ok(())
    }

    fn recv(&mut self) -> Result<Response> {
        let mut buf = String::from("");
        self.reader.read_line(&mut buf)?;
        let mut parser = Parser::new(&buf);
        let response = parser
            .read()
            .ok_or(format_err!("Unexpected 'None'-response from pREPL"))?;

        match response {
            Ok(edn::Value::Map(map)) => {
                let val = get_value("val", &map).ok_or(anyhow!("'val' not found in response"))?;
                let tag = get_value("tag", &map).ok_or(anyhow!("'tag' not found in response"))?;
                self.namespace = get_value("ns", &map).unwrap_or(self.namespace().to_string());
                match tag.as_str() {
                    "err" => {
                        return Ok(Response::StdErr(val.into()));
                    }
                    "out" => {
                        return Ok(Response::StdOut(val.into()));
                    }
                    "ret" => {
                        if get_value("exception", &map).is_some() {
                            let mut parser = Parser::new(val.as_str());
                            if let Ok(edn::Value::Map(emap)) = parser
                                .read()
                                .ok_or(anyhow!("Unable to parse exception '{}'", val))?
                            {
                                match get_value("cause", &emap) {
                                    Some(cause) => return Ok(Response::Exception(cause)),
                                    None => bail!("Unable to parse error '{}'", val),
                                }
                            } else {
                                bail!("Unable to parse error '{}'", val);
                            }
                        } else {
                            return Ok(Response::Done(Some(val)));
                        }
                    }
                    _ => bail!("Unknown tag in response '{:?}'", tag),
                }
            }
            Ok(_) => bail!("Unexpected pREPL-response '{:?}'", response),
            Err(e) => bail!("Parse error: {}", e.message),
        }
    }
}
