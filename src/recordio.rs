use std::io::{self, Error, ErrorKind, Write};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::str::FromStr;
use std::u64;

use protobuf::{self, Message};

use proto::scheduler::{Event};

pub struct RecordIOCodec {
    len_buf: Option<Vec<u8>>,
    buf: Option<Vec<u8>>,
    send: Sender<Event>,
}

impl Write for RecordIOCodec {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        for byte in input {
            if self.buf.is_none() {
                // need to parse length
                if *byte == 0x20 {
                    // we've reached the recordio size delimiter
                    if self.len_buf.is_none() {
                        // empty message
                        continue;
                    }
                    let len = try!(parse(self.len_buf.take().unwrap()));
                    self.buf = Some(Vec::with_capacity(len as usize));
                } else {
                    // non-terminator, hopefully ascii 0x30-0x39 (numbers)
                    if *byte < 0x30 || *byte > 0x39 {
                        return Err(
                            Error::new(
                                ErrorKind::InvalidData,
                                "received invalid bytes representing the size of a recordio frame"
                            )
                        );
                    }
                    let mut len_buf = self.len_buf.take().unwrap_or(vec![]);
                    len_buf.push(*byte);
                    self.len_buf = Some(len_buf);
                }
            } else {
                // we've already read a length, now we need to
                // read that many bytes.
                let mut buf = self.buf.take().unwrap();
                buf.push(*byte);
                if buf.capacity() - buf.len() == 0 {
                    // we've read an entire message, send it
                    let event: Event =
                        protobuf::parse_from_bytes(&*buf).unwrap();
                    self.send.send(event);
                } else {
                    self.buf = Some(buf);
                }
            }
        }
        Ok(input.len())
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}


#[inline]
fn parse(bytes: Vec<u8>) -> io::Result<u64> {
    let mut sum: u64 = 0;
    for byte in bytes {
        if byte < 0x30 || byte > 0x39 {
            return Err(
                Error::new(
                    ErrorKind::InvalidData,
                    "received invalid bytes representing the size of a recordio frame"
                )
            );
        }
        else {
            sum = (sum * 10) + (byte - 0x30) as u64;
        }
    }
    Ok(sum)
}
