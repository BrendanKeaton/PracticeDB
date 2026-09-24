pub mod create;
pub mod delete;
pub mod insert;
pub mod select;
pub mod utils;

use crate::core::{BufferPool, Command, QueryObject};

pub fn handle_query(
    query: &mut QueryObject,
    buffer_pool: &mut BufferPool,
) -> Result<bool, String> {
    match query.command {
        Some(Command::SELECT) => select::read_data(query, buffer_pool)?,
        Some(Command::CREATE) => create::create_new_table(query)?,
        Some(Command::INSERT) => insert::insert_new_data(query, buffer_pool)?,
        Some(Command::DELETE) => delete::delete_row_by_condition(query, buffer_pool)?,
        Some(Command::EXIT) => {
            return Ok(false);
        }

        None => todo!(),
    }
    return Ok(true);
}
