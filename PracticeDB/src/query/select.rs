use crate::{
    core::{BufferPool, QueryObject, TableMetadataObject},
    parsing::get_table_schema,
    query::utils::parse_sequential,
};

pub fn read_data(query: &mut QueryObject, buffer_pool: &mut BufferPool) -> Result<(), String> {
    let schema: TableMetadataObject = get_table_schema(&query.table)?;
    parse_sequential(query, buffer_pool, schema, "select")?;
    Ok(())
}
