use crate::bus::Device;

pub struct Ram {
    memory: Vec<u8>,
}

#[derive(Debug)]
pub struct SetMemoryError {}

impl Ram {
    pub fn new(size: usize) -> Self {
        Ram {
            memory: vec![0; size],
        }
    }

    pub fn set_memory(&mut self, data: &[u8], offset: u16) -> Result<(), SetMemoryError> {
        let offset = offset as usize;
        if offset + data.len() > self.memory.len() {
            return Err(SetMemoryError {});
        }
        for i in 0..data.len() {
            self.memory[i + offset] = data[i]
        }
        Ok(())
    }
}

impl Device for Ram {
    fn set_byte(&mut self, byte: u8, offset: u16) {
        self.memory[offset as usize] = byte;
    }

    fn get_byte(&self, offset: u16) -> u8 {
        self.memory[offset as usize]
    }

    fn get_bytes_slice(&self, from: u16, to: u16) -> Vec<u8> {
        self.memory[from as usize..to as usize].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert::*;

    #[test]
    fn set_memory_to_begin() {
        let mut ram = Ram::new(8);
        let card = vec![1, 2, 3, 4];
        let expexted = vec![1, 2, 3, 4, 0, 0, 0, 0];
        assert_ok!(ram.set_memory(&card, 0));
        assert_eq!(ram.memory, expexted);
    }

    #[test]
    fn set_memory_to_middle() {
        let mut ram = Ram::new(8);
        let card = vec![1, 2, 3, 4];
        let expexted = vec![0, 0, 1, 2, 3, 4, 0, 0];
        assert_ok!(ram.set_memory(&card, 2));
        assert_eq!(ram.memory, expexted);
    }

    #[test]
    fn set_memory_to_end() {
        let mut ram = Ram::new(8);
        let card = vec![1, 2, 3, 4];
        let expexted = vec![0, 0, 0, 0, 1, 2, 3, 4];
        assert_ok!(ram.set_memory(&card, 4));
        assert_eq!(ram.memory, expexted);
    }
    #[test]
    fn set_memory_to_pass_end() {
        let mut ram = Ram::new(8);
        let card = vec![1, 2, 3, 4];
        assert_err!(ram.set_memory(&card, 5));
    }
}
