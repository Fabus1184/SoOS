pub trait Write {
    fn write(&mut self, buffer: &[u8]) -> Result<usize, WriteError>;

    fn ignore_next(&mut self, count: usize);
}

#[derive(thiserror::Error, Debug)]
pub enum WriteError {
    #[error("Write operation failed due to an invalid offset")]
    InvalidOffset,
}

pub struct Cursor<'a> {
    position: usize,
    ignore: usize,
    buffer: &'a mut [u8],
}

impl<'a> Cursor<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Cursor {
            ignore: 0,
            position: 0,
            buffer,
        }
    }
}

impl Write for Cursor<'_> {
    fn write(&mut self, buffer: &[u8]) -> Result<usize, WriteError> {
        if buffer.is_empty() {
            return Ok(0);
        }

        let mut len = buffer.len().min(self.buffer.len() - self.position);

        if len == 0 {
            return Ok(0);
        }

        if len <= self.ignore {
            self.ignore -= len;
            return Ok(0);
        }

        len -= self.ignore;
        self.ignore = 0;

        self.buffer[self.position..self.position + len].copy_from_slice(&buffer[..len]);
        self.position += len;

        Ok(len)
    }

    fn ignore_next(&mut self, count: usize) {
        self.ignore += count;
    }
}
