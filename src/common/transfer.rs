use crate::common::message::TransferMessage;
use crate::common::message::TransferMessage::{RecvReady, SendReady};
use crate::common::net::{read_json, write_json};
use log::debug;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;
#[cfg(feature = "zstd")]
use zstd::{Decoder, Encoder};

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
        TransferMessage::RecvReady {
            offset,
            has_compression,
        } => {
            debug!("Receiver reports an offset of {} bytes", offset);
            // Seek to the specified offset in the file
            file.seek(SeekFrom::Start(offset))?;
            let length = length - offset;
            // Enable compression if both sides support it
            let use_compression = has_compression && cfg!(feature = "zstd");
            // Send the send ready message
            write_json(
                writer,
                SendReady {
                    length,
                    use_compression,
                },
            )?;
            // Check whether there are bytes to be sent
            if length == 0 {
                Ok(debug!("File already transferred"))
            } else {
                if use_compression {
                    debug!("Receiver is using compression");
                }
                debug!("Starting transfer of {} bytes...", length);
                // Send the file
                send_bytes(&mut BufReader::new(file), writer, length, use_compression)?;
                // Flush the writer to ensure everything is sent
                writer.flush()?;
                Ok(debug!("Transfer complete"))
            }
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unexpected message",
        )),
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
    write_json(
        writer,
        RecvReady {
            offset,
            has_compression: cfg!(feature = "zstd"),
        },
    )?;
    // Wait for the send ready message
    match read_json(reader)? {
        TransferMessage::SendReady {
            length: 0,
            use_compression: _,
        } => Ok(debug!("File already transferred")),
        TransferMessage::SendReady {
            length,
            use_compression,
        } => {
            debug!("Sender reports a length of {} bytes", length);
            if use_compression {
                debug!("Sender is using compression");
            }
            debug!("Starting transfer from byte {}...", offset);
            // Receive the file
            recv_bytes(reader, &mut BufWriter::new(file), length, use_compression)
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

/// Send an exact number of bytes, optionally using compression
fn send_bytes<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    length: u64,
    use_compression: bool,
) -> io::Result<()> {
    // Attempt to create an encoder if using compression
    let mut writer = if use_compression {
        encoder(writer)?
    } else {
        Box::from(writer)
    };
    // Copy all of the bytes from the reader
    Ok(assert_eq!(io::copy(reader, &mut writer)?, length))
}

/// Receive an exact number of bytes, optionally using compression
fn recv_bytes<R: Read, W: Write>(
    reader: &mut R,
    writer: &mut W,
    length: u64,
    use_compression: bool,
) -> io::Result<()> {
    // Attempt to create an decoder if using compression
    let reader = if use_compression {
        decoder(reader)?
    } else {
        Box::from(reader)
    };
    // Copy all of the bytes to the writer
    if io::copy(&mut reader.take(length), writer)? == length {
        Ok(())
    } else {
        Err(io::Error::from(io::ErrorKind::UnexpectedEof))
    }
}

#[cfg(feature = "zstd")]
fn encoder<'a>(writer: &'a mut impl Write) -> io::Result<Box<dyn Write + 'a>> {
    Ok(Box::from(
        Encoder::new(writer, 0).unwrap().on_finish(|_| ()),
    ))
}

#[cfg(feature = "zstd")]
fn decoder<'a>(reader: &'a mut impl Read) -> io::Result<Box<dyn Read + 'a>> {
    Ok(Box::from(Decoder::new(reader).unwrap()))
}

#[cfg(not(feature = "zstd"))]
fn encoder<'a>(_writer: &'a mut impl Write) -> io::Result<Box<dyn Write + 'a>> {
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "compression not supported",
    ))
}

#[cfg(not(feature = "zstd"))]
fn decoder<'a>(_reader: &'a mut impl Read) -> io::Result<Box<dyn Read + 'a>> {
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "compression not supported",
    ))
}
