use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io;
use std::io::{BufRead, Write};

/// Read and deserialize a newline-delimited JSON object
pub(crate) fn read_json<T: DeserializeOwned>(reader: &mut impl BufRead) -> io::Result<T> {
    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;
    Ok(serde_json::from_str(&buffer)?)
}

/// Serialize and write a newline-delimited JSON object
pub(crate) fn write_json(writer: &mut impl Write, object: impl Serialize) -> io::Result<()> {
    let serialized = serde_json::to_string(&object)?;
    writeln!(writer, "{}", serialized)?;
    writer.flush()
}
