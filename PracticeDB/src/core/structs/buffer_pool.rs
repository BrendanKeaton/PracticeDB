use crate::core::{PAGE_SIZE, Page};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

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

    fn flush_page_to_disk(&mut self, table_name: &str, page: Page) -> Result<(), String> {
        let table_state = self
            .tables
            .get_mut(table_name)
            .ok_or_else(|| "ERR: File not found in buffer table.".to_string())?;

        let mut file: &File = &table_state.file;

        let offset = page.id * PAGE_SIZE as u64;

        file.seek(SeekFrom::Start(offset))
            .map_err(|e| e.to_string())?;

        file.write_all(&page.data).map_err(|e| e.to_string())?;

        file.flush().map_err(|e| e.to_string())?;

        Ok(())
    }

    fn evict_page_from_buffer_LRU(&mut self) -> Result<(), String> {
        let mut oldest_key: (String, u64) = (String::new(), 0);
        let mut oldest_value = self.clock;
        for (key, value) in &self.pages {
            if oldest_key.0.is_empty() {
                oldest_key = key.clone();
                oldest_value = value.last_used;
                continue;
            }
            if value.last_used < oldest_value {
                oldest_value = value.last_used;
                oldest_key = key.clone();
            }
        }

        let removed_page = self
            .pages
            .remove(&oldest_key)
            .ok_or_else(|| "ERR: LRU victim missing from pages".to_string())?;

        if removed_page.dirty {
            self.flush_page_to_disk(&oldest_key.0, removed_page);
        }

        return Ok(());
    }

    pub fn get_page_from_buffer(
        &mut self,
        table_name: &str,
        page_id: u64,
    ) -> Result<&mut Page, String> {
        let key = (table_name.to_owned(), page_id);

        if !self.pages.contains_key(&key) {
            self.ensure_table_exists_in_buffer(table_name)?;
            let table = self
                .tables
                .get(table_name)
                .ok_or_else(|| "ERR: table not found in buffer or on disk.".to_string())?;
            if page_id >= table.next_page_id {
                return Err(format!(
                    "ERR: page {} out of range for table {} ({} pages)",
                    page_id, table_name, table.next_page_id
                ));
            }
            let mut file = &table.file;
            let mut page: Page = Page::default();
            file.seek(SeekFrom::Start(page_id * PAGE_SIZE as u64))
                .map_err(|e| e.to_string())?;
            file.read_exact(&mut page.data).map_err(|e| e.to_string())?;
            page.id = page_id;
            if !(self.pages.len() < self.capacity) {
                self.evict_page_from_buffer_LRU();
            }
            self.pages.insert(key.clone(), page);
        }

        self.clock += 1;
        let curr_page = self
            .pages
            .get_mut(&key)
            .ok_or_else(|| "ERR: page not found in buffer or on disk".to_string())?;
        curr_page.last_used = self.clock;
        return Ok(curr_page);
    }
}
