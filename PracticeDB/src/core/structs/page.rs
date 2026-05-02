use crate::core::PAGE_SIZE;

#[derive(Debug, Copy, Clone)]
pub struct Page {
    pub id: u64,
    pub dirty: bool,
    pub pin_count: usize,
    pub data: [u8; PAGE_SIZE],
}

impl Default for Page {
    fn default() -> Self {
        Self {
            id: 0,
            data: [0; PAGE_SIZE],
            dirty: true,
            pin_count: 1,
        }
    }
}
