#[derive(Debug)]
pub struct BoundedAtomicFifo<T, const SIZE: usize> {
    data: [T; SIZE],
    read: usize,
    write: usize,
}

impl<T, const SIZE: usize> BoundedAtomicFifo<T, SIZE> {
    pub fn new() -> Self {
        Self {
            data: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            read: 0,
            write: 0,
        }
    }
}

impl<T, const SIZE: usize> BoundedAtomicFifo<T, SIZE>
where
    T: Copy,
{
    pub fn push(&mut self, value: T) -> Result<(), &'static str> {
        let next_write = (self.write + 1) % SIZE;
        if next_write == self.read {
            return Err("FIFO is full!");
        }

        self.data[self.write] = value;
        self.write = next_write;

        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.read == self.write {
            return None;
        }

        let value = self.data[self.read];
        self.read = (self.read + 1) % SIZE;

        Some(value)
    }
}
