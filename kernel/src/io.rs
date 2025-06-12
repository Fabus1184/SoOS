pub trait Write {
    fn write(&mut self, bytes: &[u8]) -> Result<usize, WriterError>;
}

/// Ignorer is a writer that ignores the first `count` bytes written to it,
/// then writes the rest to the underlying writer.
pub struct Ignorer<T: Write> {
    count: usize,
    writer: T,
}

impl<T: Write> Ignorer<T> {
    pub fn ignoring(count: usize, writer: T) -> Self {
        Ignorer { count, writer }
    }
}

impl<T: Write> Write for Ignorer<T> {
    fn write(&mut self, bytes: &[u8]) -> Result<usize, WriterError> {
        if self.count > 0 {
            let to_ignore = bytes.len().min(self.count);
            self.count -= to_ignore;
            if to_ignore == bytes.len() {
                return Ok(0);
            }
            // Ignore the first `to_ignore` bytes
            self.writer.write(&bytes[to_ignore..])
        } else {
            // If nothing to ignore, write the whole buffer
            self.writer.write(bytes)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum WriterError {
    #[error("Write operation failed due to an invalid offset")]
    InvalidOffset,
}

pub struct Cursor<'a> {
    position: usize,
    buffer: &'a mut [u8],
}

impl<'a> Cursor<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Cursor {
            position: 0,
            buffer,
        }
    }
}

impl Write for Cursor<'_> {
    fn write(&mut self, bytes: &[u8]) -> Result<usize, WriterError> {
        if bytes.is_empty() {
            return Ok(0);
        }

        let len = bytes.len().min(self.buffer.len() - self.position);

        if self.position + len > self.buffer.len() {
            return Err(WriterError::InvalidOffset);
        }

        self.buffer[self.position..self.position + len].copy_from_slice(&bytes[..len]);

        self.position += len;
        Ok(len)
    }
}
