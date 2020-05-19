use anyhow::Result;
use std::io::Write;

pub fn write_and_flush(w: &mut dyn Write, data: &str) -> Result<()> {
    w.write_all(data.as_bytes())?;
    w.flush()?;

    Ok(())
}
