use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;

/// Read and deserialize a newline-delimited JSON object
pub(crate) fn read_json<R, T>(reader: &mut R) -> io::Result<T>
where
    R: BufRead,
    T: DeserializeOwned,
{
    let mut buffer = String::new();
    reader.read_line(&mut buffer)?;
    Ok(serde_json::from_str(&buffer)?)
}

/// Serialize and write a newline-delimited JSON object
pub(crate) fn write_json<W, T>(writer: &mut W, object: T) -> io::Result<()>
where
    W: Write,
    T: Serialize,
{
    let serialized = serde_json::to_string(&object)?;
    writeln!(writer, "{}", serialized)?;
    writer.flush()
}

/// Read data into a file
pub(crate) fn read_file<R>(reader: &mut R, output_file: &Path, length: u64) -> io::Result<()>
where
    R: Read,
{
    // Open the output file
    let mut writer = BufWriter::new(
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(output_file)?,
    );
    // Copy the data
    let written = io::copy(&mut reader.take(length), &mut writer)?;
    dbg!(length, written);
    Ok(())
}

/// Write data from a file
pub(crate) fn write_file<W>(writer: &mut W, input_file: &Path) -> io::Result<()>
where
    W: Write,
{
    // Open the input file
    let mut reader = BufReader::new(OpenOptions::new().read(true).open(input_file)?);
    // Copy the data
    io::copy(&mut reader, writer)?;
    Ok(())
}
