use crate::core::PAGE_SIZE;

#[derive(Debug, Clone)]
pub struct Page {
    pub id: u64,
    pub dirty: bool,
    pub data: [u8; PAGE_SIZE],
    pub last_used: u64,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            id: 0,
            data: [0; PAGE_SIZE],
            dirty: false,
            last_used: 0,
        }
    }
}
