use crate::{
    core::{
        BufferPool, PAGE_HEADER_SIZE_ON_CREATE, PAGE_HEADER_SLOT_SIZE_FOR_ROW, PAGE_SIZE,
        QueryObject,
    },
    parsing::get_table_schema,
    query::utils::build_row_byte,
};

/*
Parent method for inserting new data.
steps:
1) Determine information that needs to be known (row bytes, row_len)
2) call "find_page" the buffer pool owns all file access from here down
*/
pub fn insert_new_data(
    query: &mut QueryObject,
    buffer_pool: &mut BufferPool,
) -> Result<(), String> {
    let schema = get_table_schema(&query.table)?;
    let row_bytes = build_row_byte(&schema, &query.values)?;
    let row_len: u64 = row_bytes.len() as u64;

    // Test command is : insert profile 1:brendan:24
    find_page(&query.table, &row_bytes, row_len, buffer_pool)?;

    return Ok(());
}

fn build_new_page(
    table_name: &str,
    row_data: &[u8],
    row_len: u64,
    buffer_pool: &mut BufferPool,
) -> Result<(), String> {
    let page = buffer_pool.add_new_page(table_name)?;
    let curr_page_id = page.id;

    page.data[0] = curr_page_id as u8;

    page.data[1..3].copy_from_slice(&1u16.to_le_bytes()); // row_count - 2b

    let bytes = (row_len as u16).to_le_bytes();
    // len of current rows of data, max ~65k (over a page size, but u8 is too small). This is saved to the "free page offset"
    // To calculate the next page of offset, you would just take this value, and minus the new
    // rows length
    page.data[3..5].copy_from_slice(&bytes);

    let data_used: u16 = PAGE_HEADER_SIZE_ON_CREATE + row_len as u16;
    let space_remaining: u16 = PAGE_SIZE as u16 - data_used;
    page.data[5..7].copy_from_slice(&space_remaining.to_le_bytes()); // This is the amount of space remaining.. update on insert / delete

    // This is Last Sequence Number (for WAL recovery)... this needs to be updated as the "actual" value once WAL is created in this repo TODO
    page.data[7..9].copy_from_slice(&0u16.to_le_bytes());

    // This is a u16 of the size of the header. Including the row sizes in order
    let page_header_size_with_first_slot = PAGE_HEADER_SIZE_ON_CREATE + 4;
    page.data[9..11].copy_from_slice(&page_header_size_with_first_slot.to_le_bytes());
    let new_row_start = PAGE_SIZE - row_len as usize;
    page.data[new_row_start..PAGE_SIZE].copy_from_slice(row_data);
    // we arent setting bytes 11-12 or 13-14 for freed space and offset, because they are 0 by default
    page.data[15..17].copy_from_slice(&(row_len as u16).to_le_bytes());

    return Ok(());
}

fn find_page(
    table_name: &str,
    row_bytes: &[u8],
    row_len: u64,
    buffer_pool: &mut BufferPool,
) -> Result<(), String> {
    let num_pages = buffer_pool.page_count(table_name)?;

    for curr_page_id in 0..num_pages {
        let page = buffer_pool.get_page_from_buffer(table_name, curr_page_id)?;

        let space_remaining = u16::from_le_bytes(
            page.data[5..7]
                .try_into()
                .map_err(|_| "Corrupt page header")?,
        );

        let row_count = u16::from_le_bytes(
            page.data[1..3]
                .try_into()
                .map_err(|_| "Corrupt page header")?,
        );

        if space_remaining as u64 >= row_len + PAGE_HEADER_SLOT_SIZE_FOR_ROW && row_count < u16::MAX
        {
            // this section just updates all the bytes in the page accordingly... IE
            // the row count, free space left, adds the row to the back of page, adds slot, etc
            page.dirty = true;
            page.data[1..3].copy_from_slice(&(row_count + 1).to_le_bytes());

            let current_offset = u16::from_le_bytes(
                page.data[3..5]
                    .try_into()
                    .map_err(|_| "Corrupt page header")?,
            );

            let start_new_data: usize = PAGE_SIZE - current_offset as usize - row_len as usize;
            let end_new_data = start_new_data + row_len as usize;
            page.data[start_new_data..end_new_data].copy_from_slice(&row_bytes);
            let current_header_size = u16::from_le_bytes(
                page.data[9..11]
                    .try_into()
                    .map_err(|_| "Corrupt page header")?,
            );
            let slot_start = current_header_size as usize;
            page.data[slot_start..slot_start + 2].copy_from_slice(&current_offset.to_le_bytes());
            page.data[slot_start + 2..slot_start + 4]
                .copy_from_slice(&(row_len as u16).to_le_bytes());
            let new_header_size = current_header_size + PAGE_HEADER_SLOT_SIZE_FOR_ROW as u16;
            page.data[9..11].copy_from_slice(&new_header_size.to_le_bytes());
            let new_offset = current_offset + row_len as u16;
            page.data[3..5].copy_from_slice(&new_offset.to_le_bytes());
            let current_space_remaining = u16::from_le_bytes(
                page.data[5..7]
                    .try_into()
                    .map_err(|_| "Corrupt page header")?,
            );
            let new_space_remaining =
                current_space_remaining - row_len as u16 - PAGE_HEADER_SLOT_SIZE_FOR_ROW as u16;
            page.data[5..7].copy_from_slice(&new_space_remaining.to_le_bytes());

            return Ok(());
        }
    }
    build_new_page(table_name, row_bytes, row_len, buffer_pool)?;
    return Ok(());
}
