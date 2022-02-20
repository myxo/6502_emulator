use crate::bus::Device;

pub struct Ram {
    memory: Vec<u8>
}

pub struct SetMemoryError{}

impl Ram {
    pub fn new(size: u16) -> Self {
        Ram { memory: vec![0; size as usize] }
    }

    pub fn set_memory(&mut self, data: &Vec<u8>, offset: u16) -> Result<(), SetMemoryError> {
        let offset = offset as usize;
        if offset + data.len() > self.memory.len(){
            return Err(SetMemoryError{})
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
}


#[cfg(test)]
mod tests {
    use assert::*;
    use super::*;

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
