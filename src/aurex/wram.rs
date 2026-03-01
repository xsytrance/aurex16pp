pub struct Wram {
    memory: Box<[u8; 512 * 1024]>,
}

impl Wram {
    pub fn new() -> Self {
        Self {
            memory: Box::new([0; 512 * 1024]),
        }
    }
}
