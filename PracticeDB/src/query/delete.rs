use crate::{
    core::{BufferPool, QueryObject, TableMetadataObject},
    parsing::get_table_schema,
    query::utils::parse_sequential,
};

pub fn delete_row_by_condition(
    query: &QueryObject,
    buffer_pool: &mut BufferPool,
) -> Result<(), String> {
    let schema: TableMetadataObject = get_table_schema(&query.table)?;
    parse_sequential(query, buffer_pool, schema, "delete")?;
    return Ok(());
}
