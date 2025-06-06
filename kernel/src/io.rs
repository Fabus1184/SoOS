pub trait Write {
    fn write(&mut self, buffer: &[u8]) -> Result<usize, WriteError>;
}

#[derive(thiserror::Error, Debug)]
pub enum WriteError {
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
    fn write(&mut self, buffer: &[u8]) -> Result<usize, WriteError> {
        if buffer.len() == 0 {
            return Ok(0);
        }

        let len = buffer.len().min(self.buffer.len() - self.position);

        if len == 0 {
            return Ok(0);
        }

        self.buffer[self.position..self.position + len].copy_from_slice(&buffer[..len]);
        self.position += len;

        Ok(len)
    }
}
