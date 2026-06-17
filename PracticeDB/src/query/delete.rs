use std::fs::File;

use crate::{
    core::{QueryObject, TableMetadataObject},
    parsing::get_table_schema,
    query::utils::parse_sequential,
};

pub fn delete_row_by_condition(query: &QueryObject) -> Result<(), String> {
    let schema: TableMetadataObject = get_table_schema(&query.table)?;
    let table_path = format!("database/tables/{}.practice", &query.table);
    let file: File = File::open(&table_path).map_err(|e| e.to_string())?;
    let file_length = file.metadata().map_err(|e| e.to_string())?.len();
    parse_sequential(query, file, file_length, schema, "delete")?;
    return Ok(());
}
