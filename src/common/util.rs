use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io;
use std::io::{BufRead, Write};

/// Read and deserialize a newline-delimited JSON object
pub fn read_json<R, T>(reader: &mut R) -> io::Result<T>
where
    R: BufRead,
    T: DeserializeOwned,
{
    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;
    Ok(serde_json::from_str(&buffer)?)
}

/// Serialize and write a newline-delimited JSON object
pub fn write_json<W, T>(writer: &mut W, object: T) -> io::Result<()>
where
    W: Write,
    T: Serialize,
{
    let serialized = serde_json::to_string(&object)?;
    writeln!(writer, "{}", serialized)?;
    writer.flush()
}
