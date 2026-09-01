use std::collections::HashMap;
use std::fs::{File, OpenOptions};

use crate::core::{PAGE_SIZE, Page};
pub struct BufferPool {
    base_dir: String,
    tables: HashMap<String, TableState>,
    pages: HashMap<(String, u64), Page>,
    capacity: usize,
    clock: u64,
}

struct TableState {
    file: File,
    next_page_id: u64,
}

impl Default for BufferPool {
    fn default() -> Self {
        Self {
            base_dir: "database/tables/".to_owned(),
            tables: HashMap::new(),
            pages: HashMap::new(),
            capacity: 16,
            clock: 0,
        }
    }
}

impl BufferPool {
    fn ensure_table_exists_in_buffer(&mut self, table_name: &str) -> Result<(), String> {
        if !self.tables.contains_key(table_name) {
            let table_path: String = format!("{}{}.practice", self.base_dir, table_name);
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&table_path)
                .map_err(|e| e.to_string())?;
            let file_len = file.metadata().map_err(|e| e.to_string())?.len();
            let next_page_id = file_len / PAGE_SIZE as u64;
            let new_table_state = TableState { file, next_page_id };
            self.tables.insert(table_name.to_owned(), new_table_state);
        }
        return Ok(());
    }
}
