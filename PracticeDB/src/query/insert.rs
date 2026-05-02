use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::{
    core::{
        PAGE_HEADER_SIZE_ON_CREATE, PAGE_HEADER_SLOT_SIZE_FOR_ROW, PAGE_SIZE, Page, QueryObject,
    },
    parsing::get_table_schema,
    query::utils::build_row_byte,
};

/*
Parent method for inserting new data.
steps:
1) Determine information that needs to be known (file, file_length, page, row_len of new row, etc)
2) call "find_page"
*/
pub fn insert_new_data(query: &mut QueryObject) -> Result<(), String> {
    let schema = get_table_schema(&query.table)?;
    let row_bytes = build_row_byte(&schema, &query.values)?;
    let row_len: u64 = row_bytes.len() as u64;
    let schema_path = format!("database/tables/{}.practice", &query.table);
    let file: File = File::open(&schema_path).map_err(|e| e.to_string())?;
    let file_length = file.metadata().map_err(|e| e.to_string())?.len();

    // println!("Inserting row bytes: {:?}", row_bytes); // Test command is : insert profile 1:brendan:24
    let _page = find_page(&query.table, &row_bytes, row_len, file_length, file)
        .map_err(|e| e.to_string())?;

    return Ok(());
}

fn build_new_page(
    row_data: &[u8],
    table_name: &str,
    row_len: u64,
    file_len: u64,
) -> Result<Page, String> {
    let mut page: Page = Page::default();

    let curr_page_id = file_len / PAGE_SIZE as u64;

    page.id = curr_page_id;

    page.data[0] = (file_len / PAGE_SIZE as u64) as u8;

    // TODO guard against 255 row overflow
    page.data[1] = 1 as u8; // row_count - 1b

    let bytes = (row_len as u16).to_le_bytes();
    // len of current rows of data, max ~65k (over a page size, but u8 is too small). This is saved to the "free page offset"
    // To calculate the next page of offset, you would just take this value, and minus the new
    // rows length
    page.data[2..4].copy_from_slice(&bytes);

    let data_used: u16 = PAGE_HEADER_SIZE_ON_CREATE + row_len as u16;
    let space_remaining: u16 = PAGE_SIZE as u16 - data_used;
    page.data[4..6].copy_from_slice(&space_remaining.to_le_bytes()); // This is the amount of space remaining.. update on insert / delete

    // This is Last Sequence Number (for WAL recovery)... this needs to be updated as the "actual" value once WAL is created in this repo TODO
    page.data[6] = 0 as u8;

    // This is a u16 of the size of the header. Including the row sizes in order
    let page_header_size_with_first_slot = PAGE_HEADER_SIZE_ON_CREATE + 4;
    page.data[7..9].copy_from_slice(&page_header_size_with_first_slot.to_le_bytes());
    let new_row_start = PAGE_SIZE - row_len as usize;
    page.data[new_row_start..PAGE_SIZE].copy_from_slice(row_data);
    // we arent setting bytes 9-10 or 11-12 for freed space and offset, because they are 0 by default
    page.data[13..15].copy_from_slice(&(row_len as u16).to_le_bytes());

    let _ = insert_page(table_name, &page);

    return Ok(page);
}

fn find_page(
    table_name: &str,
    row_bytes: &[u8],
    row_len: u64,
    file_len: u64,
    mut file: File,
) -> Result<Page, String> {
    let mut page: Page = Page::default();

    if file_len == 0 {
        let page = build_new_page(&row_bytes, table_name, row_len, file_len)?;
        return Ok(page);
    }

    for curr_page_id in 0..(file_len / PAGE_SIZE as u64) {
        file.seek(SeekFrom::Start(curr_page_id * PAGE_SIZE as u64))
            .map_err(|e| e.to_string())?;

        file.read_exact(&mut page.data).map_err(|e| e.to_string())?;

        let space_remaining = u16::from_le_bytes(
            page.data[4..6]
                .try_into()
                .map_err(|_| "Corrupt page header")?,
        );

        if space_remaining as u64 >= row_len + PAGE_HEADER_SLOT_SIZE_FOR_ROW {
            // this section just updates all the bytes in the page accordingly... IE
            // the row count, free space left, adds the row to the back of page, adds slot, etc
            page.pin_count += 1;
            page.dirty = true;
            page.id = curr_page_id;
            page.data[1] += 1;

            let current_offset = u16::from_le_bytes(
                page.data[2..4]
                    .try_into()
                    .map_err(|_| "Corrupt page header")?,
            );

            let start_new_data: usize = PAGE_SIZE - current_offset as usize - row_len as usize;
            let end_new_data = start_new_data + row_len as usize;
            page.data[start_new_data..end_new_data].copy_from_slice(&row_bytes);
            let current_header_size = u16::from_le_bytes(
                page.data[7..9]
                    .try_into()
                    .map_err(|_| "Corrupt page header")?,
            );
            let slot_start = current_header_size as usize;
            page.data[slot_start..slot_start + 2].copy_from_slice(&current_offset.to_le_bytes());
            page.data[slot_start + 2..slot_start + 4]
                .copy_from_slice(&(row_len as u16).to_le_bytes());
            let new_header_size = current_header_size + PAGE_HEADER_SLOT_SIZE_FOR_ROW as u16;
            page.data[7..9].copy_from_slice(&new_header_size.to_le_bytes());
            let new_offset = current_offset + row_len as u16;
            page.data[2..4].copy_from_slice(&new_offset.to_le_bytes());
            let current_space_remaining = u16::from_le_bytes(
                page.data[4..6]
                    .try_into()
                    .map_err(|_| "Corrupt page header")?,
            );
            let new_space_remaining =
                current_space_remaining - row_len as u16 - PAGE_HEADER_SLOT_SIZE_FOR_ROW as u16;
            page.data[4..6].copy_from_slice(&new_space_remaining.to_le_bytes());

            let _ = insert_page(table_name, &page);
            return Ok(page);
        }
    }
    page = build_new_page(&row_bytes, &table_name, row_len, file_len).map_err(|e| e.to_string())?;
    return Ok(page);
}

fn insert_page(table_name: &str, page: &Page) -> Result<(), String> {
    let schema_path = format!("database/tables/{}.practice", table_name);

    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&schema_path)
        .map_err(|e| e.to_string())?;

    let offset = page.id * PAGE_SIZE as u64;

    file.seek(SeekFrom::Start(offset))
        .map_err(|e| e.to_string())?;

    file.write_all(&page.data).map_err(|e| e.to_string())?;

    file.flush().map_err(|e| e.to_string())?;

    Ok(())
}
