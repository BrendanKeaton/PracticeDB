use crate::core::FieldTypesAllowed;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldObject {
    pub name: String,
    pub data_type: FieldTypesAllowed,
    pub is_indexed: bool,
}
