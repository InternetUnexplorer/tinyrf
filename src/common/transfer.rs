use crate::common::message::TransferMessage;
use crate::common::message::TransferMessage::{RecvReady, SendReady};
use crate::common::net::{read_json, write_json};
use log::debug;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// Send a file
pub(crate) fn send_file(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    file: &Path,
) -> io::Result<()> {
    debug!("Sending file \"{}\"...", &file.display());
    // Open the source file for reading
    let (mut file, length) = open_file(file, OpenOptions::new().read(true))?;
    // Wait for the receive ready message
    match read_json(reader)? {
        TransferMessage::RecvReady { offset } => {
            debug!("Receiver reports an offset of {} bytes", length);
            // Seek to the specified offset in the file
            file.seek(SeekFrom::Start(offset))?;
            // Send the send ready message
            let length = length - offset;
            write_json(writer, SendReady { length })?;
            // Check whether there are bytes to be sent
            if length == 0 {
                Ok(debug!("File already transferred"))
            } else {
                debug!("Starting transfer of {} bytes...", length);
                // Copy all of the bytes from file to the writer
                assert_eq!(io::copy(&mut BufReader::new(file), writer)?, length);
                // Flush the writer to ensure the bytes are sent
                writer.flush()?;
                Ok(debug!("Transfer complete"))
            }
        }
        // Unexpected message
        _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
    }
}

/// Receive a file
pub(crate) fn recv_file(
    reader: &mut impl BufRead,
    writer: &mut impl Write,
    file: &Path,
) -> io::Result<()> {
    debug!("Receiving file \"{}\"...", &file.display());
    // Open the destination file for writing
    let (file, offset) = open_file(file, OpenOptions::new().create(true).append(true))?;
    // Send the receive ready message
    write_json(writer, RecvReady { offset })?;
    // Wait for the send ready message
    match read_json(reader)? {
        TransferMessage::SendReady { length: 0 } => Ok(debug!("File already transferred")),
        TransferMessage::SendReady { length } => {
            debug!("Sender reports a length of {} bytes", length);
            debug!("Starting transfer from byte {}...", offset);
            // Copy all of the bytes from the reader into the file
            if io::copy(&mut reader.take(length), &mut BufWriter::new(file))? == length {
                // All of the bytes were copied
                Ok(debug!("Transfer complete"))
            } else {
                // EOF was encountered before all of the bytes could be copied
                Err(io::Error::from(io::ErrorKind::UnexpectedEof))
            }
        }
        // Unexpected message
        _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
    }
}

/// Open a file and get its length
fn open_file(file: &Path, options: &OpenOptions) -> io::Result<(File, u64)> {
    let file = options.open(file)?;
    let length = file.metadata()?.len();
    Ok((file, length))
}
