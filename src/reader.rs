use std::cell::RefCell;
use std::rc::Rc;

pub struct Reader {
    buffer: Vec<u8>,
    position: Rc<RefCell<usize>>,
}

#[allow(dead_code)]
impl Reader {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self {
            buffer,
            position: Rc::new(RefCell::new(0)),
        }
    }

    pub fn read(&self) -> u8 {
        let p: usize = *self.position.borrow();
        let b = self.buffer[p];
        *self.position.borrow_mut() += 1;
        b
    }

    pub fn read_bytes(&self, n: usize) -> &[u8] {
        let p: usize = *self.position.borrow();
        let bs = &self.buffer[p..p + n];
        *self.position.borrow_mut() += n;
        bs
    }

    pub fn read_bool(&self) -> bool {
        self.read() == 1
    }

    pub fn read_string(&self, n: usize) -> String {
        let b = self.read_bytes(n);
        std::str::from_utf8(b)
            .expect("invalid utf-8 sequence in string")
            .to_string()
    }

    pub fn read_variable_quantity(&self) -> usize {
        let mut bytes: [u8; 4] = [0; 4];
        for i in 0..4 {
            let b = self.read();
            bytes[i] = b & 0b01111111;
            if b & 0b10000000 == 0 {
                break;
            }
            // If we're in our last loop, we shouldn't make it this far:
            if i == 3 {
                panic!("More bytes than expected in a variable quantity")
            }
        }

        bytes
            .iter()
            .enumerate()
            .fold(0, |r, (i, &b)| r + ((b as usize) << (i * 7)))
    }

    pub fn pos(&self) -> usize {
        *self.position.borrow()
    }

    pub fn set_pos(&self, n: usize) {
        *self.position.borrow_mut() = n;
    }

    pub fn step_back(&self) {
        *self.position.borrow_mut() -= 1;
    }

    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn rest(&self) -> Vec<u8> {
        let p: usize = *self.position.borrow();
        self.buffer[p..].to_vec()
    }
}
