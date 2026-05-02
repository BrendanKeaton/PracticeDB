use crate::core::FieldObject;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TableMetadataObject {
    pub table_name: String,
    pub fields: Vec<FieldObject>,
}
