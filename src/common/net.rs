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

/// Read a length-delimited file from the reader
pub(crate) fn read_file(reader: &mut impl Read, output_file: &Path) -> io::Result<()> {
    // Open the file for writing
    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output_file)?;
    let mut writer = BufWriter::new(file);
    // Read the length
    let mut length: [u8; 8] = [0; 8];
    reader.read_exact(&mut length)?;
    let length = u64::from_le_bytes(length);
    // Copy the data
    assert!(io::copy(&mut reader.take(length), &mut writer)? == length);
    Ok(())
}

/// Write a length-delimited file to the writer
pub(crate) fn write_file(writer: &mut impl Write, input_file: &Path) -> io::Result<()> {
    // Open the input file and get its length
    let file = OpenOptions::new().read(true).open(input_file)?;
    let length = file.metadata()?.len();
    let mut reader = BufReader::new(file);
    // Send the length
    writer.write_all(&u64::to_le_bytes(length))?;
    // Copy the data
    io::copy(&mut reader, writer)?;
    writer.flush()
}
