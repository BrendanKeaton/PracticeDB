use crate::{
    core::{QueryObject, TableMetadataObject},
    parsing::get_table_schema,
    query::utils::parse_sequential,
};
use std::fs::File;

pub fn read_data(query: &mut QueryObject) -> Result<(), String> {
    let schema: TableMetadataObject = get_table_schema(&query.table)?;
    let schema_path = format!("database/tables/{}.practice", &query.table);
    let file: File = File::open(&schema_path).map_err(|e| e.to_string())?;
    let file_length = file.metadata().map_err(|e| e.to_string())?.len();
    let _ = parse_sequential(query, file, file_length, schema, "select");
    Ok(())
}
