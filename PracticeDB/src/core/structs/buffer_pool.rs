use std::collections::HashMap;

pub struct BufferPool {
    pages: HashMap<u64, Page>,
    capacity: usize,
}
