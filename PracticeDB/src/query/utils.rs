use std::fs::File;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::core::{
    ConditionsObject, FieldTypesAllowed, LogicalConnector, Operand, PAGE_HEADER_SLOT_SIZE_FOR_ROW,
    PAGE_SIZE, QueryObject, TableMetadataObject, ValueTypes, VariableReturn,
};

pub fn get_selected_column_ids(
    query: &QueryObject,
    schema: &TableMetadataObject,
) -> Result<Vec<u8>, String> {
    let mut return_vec: Vec<u8> = Vec::new();

    if query.fields.iter().any(|f| matches!(f, ValueTypes::STAR)) {
        return Ok((0..schema.fields.len()).map(|i| i as u8).collect());
    }

    for (index, field) in schema.fields.iter().enumerate() {
        if query
            .fields
            .iter()
            .any(|qf| qf.as_str().map_or(false, |name| name == field.name))
        {
            return_vec.push(index as u8);
        }
    }
    Ok(return_vec)
}

pub fn get_selected_column_ids_in_conditional(
    query: &QueryObject,
    schema: &TableMetadataObject,
) -> Result<Vec<u8>, String> {
    let mut return_vec: Vec<u8> = Vec::new();

    if query.fields.iter().any(|f| matches!(f, ValueTypes::STAR)) {
        return Ok((0..schema.fields.len()).map(|i| i as u8).collect());
    }

    for (index, field) in schema.fields.iter().enumerate() {
        if query.conditions.iter().any(|qf| {
            (qf.object_one_is_field && qf.object_one == field.name)
                || (qf.object_two_is_field && qf.object_two == field.name)
        }) {
            return_vec.push(index as u8);
        }
    }
    Ok(return_vec)
}

pub fn parse_sequential(
    query: &QueryObject,
    mut file: File,
    file_len: u64,
    schema: TableMetadataObject,
    action: &str,
) -> Result<(), String> {
    let mut page_data: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
    let num_pages: u64 = file_len / PAGE_SIZE as u64;

    let selected_column_ids = get_selected_column_ids(query, &schema)?;
    let mut page_modified = false;

    for curr_page_id in 0..num_pages {
        file.seek(SeekFrom::Start(curr_page_id * PAGE_SIZE as u64))
            .map_err(|e| e.to_string())?;

        file.read_exact(&mut page_data).map_err(|e| e.to_string())?;

        if curr_page_id as u8 != page_data[0] {
            return Err("Page ID mismatch".to_string());
        }

        let row_count = page_data[1];

        for row in (0..row_count).rev() {
            let curr_slot_offset = row * 4 + 11;
            let row_offset = u16::from_le_bytes(
                page_data[curr_slot_offset as usize..(curr_slot_offset as usize + 2)]
                    .try_into()
                    .unwrap(),
            );
            let row_length = u16::from_le_bytes(
                page_data[curr_slot_offset as usize + 2..(curr_slot_offset as usize + 4)]
                    .try_into()
                    .unwrap(),
            );

            let row_start = PAGE_SIZE - row_length as usize - row_offset as usize;
            let row_end = row_start + row_length as usize;
            let row_bytes = &page_data[row_start..row_end];
            let decoded_row: Vec<String>;
            if action == "delete" {
                let selected_conditional_column_ids =
                    get_selected_column_ids_in_conditional(query, &schema)?;
                let should_delete: bool =
                    should_delete_row(row_bytes, &query, selected_conditional_column_ids)?;
                if should_delete {
                    delete_row(&mut page_data, row, row_length)?;
                    page_modified = true;
                }
            } else {
                decoded_row = decode_row(row_bytes, &schema, &selected_column_ids)?;
                println!("{:?}", decoded_row);
            }
        }

        if page_modified {
            let schema_path = format!("database/tables/{}.practice", &query.table);
            let mut write_file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&schema_path)
                .map_err(|e| e.to_string())?;

            write_file
                .seek(SeekFrom::Start(curr_page_id * PAGE_SIZE as u64))
                .map_err(|e| e.to_string())?;
            write_file
                .write_all(&page_data)
                .map_err(|e| e.to_string())?;
            write_file.flush().map_err(|e| e.to_string())?;

            page_modified = false;
        }
    }

    Ok(())
}

pub fn decode_row(
    row_bytes: &[u8],
    schema: &TableMetadataObject,
    selected_column_ids: &Vec<u8>,
) -> Result<Vec<String>, String> {
    let header_size = row_bytes[0] as usize;
    let num_columns = row_bytes[1] as usize;
    let col_lengths = &row_bytes[2..2 + num_columns];
    let mut data_offset = header_size;

    let mut values: Vec<String> = Vec::new();

    for (i, field) in schema.fields.iter().enumerate() {
        let len = col_lengths[i] as usize;
        let field_bytes = &row_bytes[data_offset..data_offset + len];

        if selected_column_ids.contains(&(i as u8)) {
            let value = match field.data_type {
                FieldTypesAllowed::I8 => (field_bytes[0] as i8).to_string(),

                FieldTypesAllowed::I32 => {
                    let arr: [u8; 4] = field_bytes.try_into().map_err(|_| "corrupt I32")?;
                    i32::from_le_bytes(arr).to_string()
                }

                FieldTypesAllowed::String => String::from_utf8(field_bytes.to_vec())
                    .map_err(|_| format!("invalid UTF-8 in field '{}'", field.name))?,
            };

            values.push(value);
        }

        data_offset += len;
    }

    Ok(values)
}

pub fn build_row_byte(
    schema: &TableMetadataObject,
    values: &[ValueTypes],
) -> Result<Vec<u8>, String> {
    let num_columns: usize = schema.fields.len();

    let mut row_header: Vec<u8> = Vec::new();
    let mut row_data: Vec<u8> = Vec::new();

    for (field, value) in schema.fields.iter().zip(values.iter()) {
        let raw: &str = value
            .as_str()
            .ok_or_else(|| format!("value for field '{}' is None", field.name))?;

        match field.data_type {
            FieldTypesAllowed::I8 => {
                let v = raw
                    .parse::<i8>()
                    .map_err(|_| format!("invalid I8 for field '{}'", field.name))?;
                row_data.push(v as u8);
                row_header.push(1);
            }
            FieldTypesAllowed::I32 => {
                let v = raw
                    .parse::<i32>()
                    .map_err(|_| format!("invalid I32 for field '{}'", field.name))?;
                row_data.extend_from_slice(&v.to_le_bytes());
                row_header.push(4);
            }
            FieldTypesAllowed::String => {
                let max_len = 255;
                let bytes = raw.as_bytes();
                let byte_len = bytes.len();

                if byte_len > max_len {
                    return Err(format!(
                        "string byte length for field '{}' exceeds maximum of {}",
                        field.name, max_len
                    ));
                }
                row_data.extend_from_slice(bytes);
                row_header.push(byte_len as u8);
            }
        }
    }

    let mut result: Vec<u8> = Vec::new();
    let row_header_size = 2 + num_columns;

    result.push(row_header_size as u8);
    result.push(num_columns as u8);
    result.extend(row_header);
    result.extend(row_data);

    Ok(result)
}

pub fn delete_row(
    page_data: &mut [u8; PAGE_SIZE],
    row_index: u8,
    row_length: u16,
) -> Result<(), String> {
    page_data[1] -= 1;

    let current_freed = u16::from_le_bytes(page_data[9..11].try_into().unwrap());
    let new_freed = current_freed + row_length;
    page_data[9..11].copy_from_slice(&new_freed.to_le_bytes());

    let current_header_size = u16::from_le_bytes(page_data[7..9].try_into().unwrap());

    let slot_start = 11 + (row_index as usize) * 4;
    let slots_end = current_header_size as usize;

    if slot_start + 4 < slots_end {
        page_data.copy_within(slot_start + 4..slots_end, slot_start);
    }

    let new_slots_end = slots_end - 4;
    page_data[new_slots_end..slots_end].fill(0);

    let new_header_size = current_header_size - PAGE_HEADER_SLOT_SIZE_FOR_ROW as u16;
    page_data[7..9].copy_from_slice(&new_header_size.to_le_bytes());

    let current_space = u16::from_le_bytes(page_data[4..6].try_into().unwrap());
    let new_space = current_space + PAGE_HEADER_SLOT_SIZE_FOR_ROW as u16;
    page_data[4..6].copy_from_slice(&new_space.to_le_bytes());

    Ok(())
}

pub fn should_delete_row(
    row_bytes: &[u8],
    query: &QueryObject,
    column_ids: Vec<u8>,
) -> Result<bool, String> {
    let mut result = true;
    let mut col_index: usize = 0;

    for (i, condition) in query.conditions.iter().enumerate() {
        let matches = evaluate_condition(condition, row_bytes, &column_ids, &mut col_index)?;

        if i == 0 {
            result = matches;
        } else {
            match &condition.connector {
                Some(LogicalConnector::Or) => result = result || matches,
                Some(LogicalConnector::And) | None => result = result && matches,
            }
        }
    }

    Ok(result)
}

fn evaluate_condition(
    condition: &ConditionsObject,
    row_bytes: &[u8],
    column_ids: &[u8],
    col_index: &mut usize,
) -> Result<bool, String> {
    if condition.object_one_is_field && condition.object_two_is_field {
        let curr_col_1 = column_ids[*col_index];
        *col_index += 1;
        let curr_col_2 = column_ids[*col_index];
        *col_index += 1;

        let value_1: VariableReturn = get_value_by_column_id(row_bytes, curr_col_1)?;
        let value_2: VariableReturn = get_value_by_column_id(row_bytes, curr_col_2)?;

        return Ok(compare_values(&value_1, &value_2, &condition.operand));
    }

    if !condition.object_one_is_field && !condition.object_two_is_field {
        return Err("At least one side of a where statement needs to be a column".to_owned());
    }

    let curr_col = column_ids[*col_index];
    *col_index += 1;
    let value: VariableReturn = get_value_by_column_id(row_bytes, curr_col)?;

    let literal_str = if condition.object_one_is_field {
        &condition.object_two
    } else {
        &condition.object_one
    };

    let literal = match &value {
        VariableReturn::I8(_) => VariableReturn::I8(
            literal_str
                .parse::<i8>()
                .map_err(|_| format!("'{}' is not a valid i8", literal_str))?,
        ),
        VariableReturn::I32(_) => VariableReturn::I32(
            literal_str
                .parse::<i32>()
                .map_err(|_| format!("'{}' is not a valid i32", literal_str))?,
        ),
        VariableReturn::String(_) => VariableReturn::String(literal_str.clone()),
    };

    if condition.object_one_is_field {
        Ok(compare_values(&value, &literal, &condition.operand))
    } else {
        Ok(compare_values(&literal, &value, &condition.operand))
    }
}

fn compare_values(left: &VariableReturn, right: &VariableReturn, operand: &Operand) -> bool {
    match operand {
        Operand::EQ => left == right,
        Operand::NQ => left != right,
        Operand::GT => left > right,
        Operand::LT => left < right,
        Operand::GTE => left >= right,
        Operand::LTE => left <= right,
    }
}

pub fn get_value_by_column_id(row_bytes: &[u8], curr_col: u8) -> Result<VariableReturn, String> {
    let header_size = row_bytes[0] as usize;
    let num_columns = row_bytes[1] as usize;

    let col = curr_col as usize;
    if col >= num_columns {
        return Err(format!("Column {} out of range ({})", col, num_columns));
    }

    let col_len = row_bytes[2 + col] as usize;

    let data_start = header_size
        + row_bytes[2..2 + col]
            .iter()
            .map(|&b| b as usize)
            .sum::<usize>();

    let col_bytes = &row_bytes[data_start..data_start + col_len];

    match col_len {
        1 => Ok(VariableReturn::I8(col_bytes[0] as i8)),
        4 => Ok(VariableReturn::I32(i32::from_le_bytes(
            col_bytes.try_into().unwrap(),
        ))),
        _ => Ok(VariableReturn::String(
            String::from_utf8(col_bytes.to_vec()).map_err(|e| format!("Invalid UTF-8: {}", e))?,
        )),
    }
}
