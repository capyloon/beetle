use std::hash::Hasher;
use std::io;

use twox_hash::XxHash64;

use super::ringbuffer::RingBuffer;

pub struct Decodebuffer {
    buffer: RingBuffer,
    pub dict_content: Vec<u8>,

    pub window_size: usize,
    total_output_counter: u64,
    pub hash: XxHash64,
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DecodebufferError {
    #[error("Need {need} bytes from the dictionary but it is only {got} bytes long")]
    NotEnoughBytesInDictionary { got: usize, need: usize },
    #[error("offset: {offset} bigger than buffer: {buf_len}")]
    OffsetTooBig { offset: usize, buf_len: usize },
}

impl io::Read for Decodebuffer {
    fn read(&mut self, target: &mut [u8]) -> io::Result<usize> {
        let max_amount = self.can_drain_to_window_size().unwrap_or(0);
        let amount = max_amount.min(target.len());

        let mut written = 0;
        self.drain_to(amount, |buf| {
            target[written..][..buf.len()].copy_from_slice(buf);
            written += buf.len();
            (buf.len(), Ok(()))
        })?;
        Ok(amount)
    }
}

impl Decodebuffer {
    pub fn new(window_size: usize) -> Decodebuffer {
        Decodebuffer {
            buffer: RingBuffer::new(),
            dict_content: Vec::new(),
            window_size,
            total_output_counter: 0,
            hash: XxHash64::with_seed(0),
        }
    }

    pub fn reset(&mut self, window_size: usize) {
        self.window_size = window_size;
        self.buffer.clear();
        self.buffer.reserve(self.window_size);
        self.dict_content.clear();
        self.total_output_counter = 0;
        self.hash = XxHash64::with_seed(0);
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn push(&mut self, data: &[u8]) {
        self.buffer.extend(data);
        self.total_output_counter += data.len() as u64;
    }

    pub fn repeat(&mut self, offset: usize, match_length: usize) -> Result<(), DecodebufferError> {
        if offset > self.buffer.len() {
            if self.total_output_counter <= self.window_size as u64 {
                // at least part of that repeat is from the dictionary content
                let bytes_from_dict = offset - self.buffer.len();

                if bytes_from_dict > self.dict_content.len() {
                    return Err(DecodebufferError::NotEnoughBytesInDictionary {
                        got: self.dict_content.len(),
                        need: bytes_from_dict,
                    });
                }

                if bytes_from_dict < match_length {
                    let dict_slice =
                        &self.dict_content[self.dict_content.len() - bytes_from_dict..];
                    self.buffer.extend(dict_slice);

                    self.total_output_counter += bytes_from_dict as u64;
                    return self.repeat(self.buffer.len(), match_length - bytes_from_dict);
                } else {
                    let low = self.dict_content.len() - bytes_from_dict;
                    let high = low + match_length;
                    let dict_slice = &self.dict_content[low..high];
                    self.buffer.extend(dict_slice);
                }
            } else {
                return Err(DecodebufferError::OffsetTooBig {
                    offset,
                    buf_len: self.buffer.len(),
                });
            }
        } else {
            let buf_len = self.buffer.len();
            let start_idx = buf_len - offset;
            let end_idx = start_idx + match_length;

            self.buffer.reserve(match_length);
            if end_idx > buf_len {
                // We need to copy in chunks.
                // We have at max offset bytes in one chunk, the last one can be smaller
                let mut start_idx = start_idx;
                let mut copied_counter_left = match_length;
                // TODO this can  be optimized further I think.
                // Each time we copy a chunk we have a repetiton of length 'offset', so we can copy offset * iteration many bytes from start_idx
                while copied_counter_left > 0 {
                    let chunksize = usize::min(offset, copied_counter_left);

                    // SAFETY:
                    // we know that start_idx <= buf_len and start_idx + offset == buf_len and we reserverd match_length space
                    unsafe {
                        self.buffer
                            .extend_from_within_unchecked(start_idx, chunksize)
                    };
                    copied_counter_left -= chunksize;
                    start_idx += chunksize;
                }
            } else {
                // can just copy parts of the existing buffer
                // SAFETY:
                // we know that start_idx and end_idx <= buf_len and we reserverd match_length space
                unsafe {
                    self.buffer
                        .extend_from_within_unchecked(start_idx, match_length)
                };
            }

            self.total_output_counter += match_length as u64;
        }

        Ok(())
    }

    // Check if and how many bytes can currently be drawn from the buffer
    pub fn can_drain_to_window_size(&self) -> Option<usize> {
        if self.buffer.len() > self.window_size {
            Some(self.buffer.len() - self.window_size)
        } else {
            None
        }
    }

    //How many bytes can be drained if the window_size does not have to be maintained
    pub fn can_drain(&self) -> usize {
        self.buffer.len()
    }

    //drain as much as possible while retaining enough so that decoding si still possible with the required window_size
    //At best call only if can_drain_to_window_size reports a 'high' number of bytes to reduce allocations
    pub fn drain_to_window_size(&mut self) -> Option<Vec<u8>> {
        //TODO investigate if it is possible to return the std::vec::Drain iterator directly without collecting here
        match self.can_drain_to_window_size() {
            None => None,
            Some(can_drain) => {
                let mut vec = Vec::with_capacity(can_drain);
                self.drain_to(can_drain, |buf| {
                    vec.extend_from_slice(buf);
                    (buf.len(), Ok(()))
                })
                .ok()?;
                Some(vec)
            }
        }
    }

    pub fn drain_to_window_size_writer(&mut self, mut sink: impl io::Write) -> io::Result<usize> {
        match self.can_drain_to_window_size() {
            None => Ok(0),
            Some(can_drain) => {
                self.drain_to(can_drain, |buf| write_all_bytes(&mut sink, buf))?;
                Ok(can_drain)
            }
        }
    }

    //drain the buffer completely
    pub fn drain(&mut self) -> Vec<u8> {
        let (slice1, slice2) = self.buffer.as_slices();
        self.hash.write(slice1);
        self.hash.write(slice2);

        let mut vec = Vec::with_capacity(slice1.len() + slice2.len());
        vec.extend_from_slice(slice1);
        vec.extend_from_slice(slice2);
        self.buffer.clear();
        vec
    }

    pub fn drain_to_writer(&mut self, mut sink: impl io::Write) -> io::Result<usize> {
        let len = self.buffer.len();
        self.drain_to(len, |buf| write_all_bytes(&mut sink, buf))?;

        Ok(len)
    }

    pub fn read_all(&mut self, target: &mut [u8]) -> io::Result<usize> {
        let amount = self.buffer.len().min(target.len());

        let mut written = 0;
        self.drain_to(amount, |buf| {
            target[written..][..buf.len()].copy_from_slice(buf);
            written += buf.len();
            (buf.len(), Ok(()))
        })?;
        Ok(amount)
    }

    /// Semantics of write_bytes:
    /// Should dump as many of the provided bytes as possible to whatever sink until no bytes are left or an error is encountered
    /// Return how many bytes have actually been dumped to the sink.
    fn drain_to(
        &mut self,
        amount: usize,
        mut write_bytes: impl FnMut(&[u8]) -> (usize, io::Result<()>),
    ) -> io::Result<()> {
        if amount == 0 {
            return Ok(());
        }

        struct DrainGuard<'a> {
            buffer: &'a mut RingBuffer,
            amount: usize,
        }

        impl<'a> Drop for DrainGuard<'a> {
            fn drop(&mut self) {
                if self.amount != 0 {
                    self.buffer.drop_first_n(self.amount);
                }
            }
        }

        let mut drain_guard = DrainGuard {
            buffer: &mut self.buffer,
            amount: 0,
        };

        let (slice1, slice2) = drain_guard.buffer.as_slices();
        let n1 = slice1.len().min(amount);
        let n2 = slice2.len().min(amount - n1);

        if n1 != 0 {
            let (written1, res1) = write_bytes(&slice1[..n1]);
            self.hash.write(&slice1[..written1]);
            drain_guard.amount += written1;

            // Apparently this is what clippy thinks is the best way of expressing this
            res1?;

            // Only if the first call to write_bytes was not a partial write we can continue with slice2
            // Partial writes SHOULD never happen without res1 being an error, but lets just protect against it anyways.
            if written1 == n1 && n2 != 0 {
                let (written2, res2) = write_bytes(&slice2[..n2]);
                self.hash.write(&slice2[..written2]);
                drain_guard.amount += written2;

                // Apparently this is what clippy thinks is the best way of expressing this
                res2?;
            }
        }

        // Make sure we don't accidentally drop `DrainGuard` earlier.
        drop(drain_guard);

        Ok(())
    }
}

/// Like Write::write_all but returns partial write length even on error
fn write_all_bytes(mut sink: impl io::Write, buf: &[u8]) -> (usize, io::Result<()>) {
    let mut written = 0;
    while written < buf.len() {
        match sink.write(&buf[written..]) {
            Ok(w) => written += w,
            Err(e) => return (written, Err(e)),
        }
    }
    (written, Ok(()))
}

#[cfg(test)]
mod tests {
    use super::Decodebuffer;
    use std::io::Write;

    #[test]
    fn short_writer() {
        struct ShortWriter {
            buf: Vec<u8>,
            write_len: usize,
        }

        impl Write for ShortWriter {
            fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, std::io::Error> {
                if buf.len() > self.write_len {
                    self.buf.extend_from_slice(&buf[..self.write_len]);
                    Ok(self.write_len)
                } else {
                    self.buf.extend_from_slice(buf);
                    Ok(buf.len())
                }
            }

            fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
                Ok(())
            }
        }

        let mut short_writer = ShortWriter {
            buf: vec![],
            write_len: 10,
        };

        let mut decode_buf = Decodebuffer::new(100);
        decode_buf.push(b"0123456789");
        decode_buf.repeat(10, 90).unwrap();
        let repeats = 1000;
        for _ in 0..repeats {
            assert_eq!(decode_buf.len(), 100);
            decode_buf.repeat(10, 50).unwrap();
            assert_eq!(decode_buf.len(), 150);
            decode_buf
                .drain_to_window_size_writer(&mut short_writer)
                .unwrap();
            assert_eq!(decode_buf.len(), 100);
        }

        assert_eq!(short_writer.buf.len(), repeats * 50);
        decode_buf.drain_to_writer(&mut short_writer).unwrap();
        assert_eq!(short_writer.buf.len(), repeats * 50 + 100);
    }

    #[test]
    fn wouldblock_writer() {
        struct WouldblockWriter {
            buf: Vec<u8>,
            last_blocked: usize,
            block_every: usize,
        }

        impl Write for WouldblockWriter {
            fn write(&mut self, buf: &[u8]) -> std::result::Result<usize, std::io::Error> {
                if self.last_blocked < self.block_every {
                    self.buf.extend_from_slice(buf);
                    self.last_blocked += 1;
                    Ok(buf.len())
                } else {
                    self.last_blocked = 0;
                    Err(std::io::Error::from(std::io::ErrorKind::WouldBlock))
                }
            }

            fn flush(&mut self) -> std::result::Result<(), std::io::Error> {
                Ok(())
            }
        }

        let mut short_writer = WouldblockWriter {
            buf: vec![],
            last_blocked: 0,
            block_every: 5,
        };

        let mut decode_buf = Decodebuffer::new(100);
        decode_buf.push(b"0123456789");
        decode_buf.repeat(10, 90).unwrap();
        let repeats = 1000;
        for _ in 0..repeats {
            assert_eq!(decode_buf.len(), 100);
            decode_buf.repeat(10, 50).unwrap();
            assert_eq!(decode_buf.len(), 150);
            loop {
                match decode_buf.drain_to_window_size_writer(&mut short_writer) {
                    Ok(written) => {
                        if written == 0 {
                            break;
                        }
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            continue;
                        } else {
                            panic!("Unexpected error {:?}", e);
                        }
                    }
                }
            }
            assert_eq!(decode_buf.len(), 100);
        }

        assert_eq!(short_writer.buf.len(), repeats * 50);
        loop {
            match decode_buf.drain_to_writer(&mut short_writer) {
                Ok(written) => {
                    if written == 0 {
                        break;
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        continue;
                    } else {
                        panic!("Unexpected error {:?}", e);
                    }
                }
            }
        }
        assert_eq!(short_writer.buf.len(), repeats * 50 + 100);
    }
}