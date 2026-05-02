use crate::core::Command;
use crate::core::ConditionsObject;
use crate::core::FieldTypesAllowed;
use crate::core::ValueTypes;
use std::fmt;

#[derive(Default, Debug)]
pub struct QueryObject {
    pub(crate) command: Option<Command>,
    pub(crate) table: String,
    pub(crate) fields: Vec<ValueTypes>,
    pub(crate) values: Vec<ValueTypes>,
    pub(crate) conditions: Vec<ConditionsObject>,
    pub(crate) index: usize,
    pub(crate) length: usize, // max length of query is 255 for now.
    pub(crate) field_type: Option<Vec<FieldTypesAllowed>>,
    pub(crate) is_field_index: Option<Vec<bool>>,
}

impl fmt::Display for QueryObject {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cmd: String = match &self.command {
            Some(c) => format!("{}", c),
            None => "None".to_string(),
        };
        let fields: Vec<String> = self
            .fields
            .iter()
            .map(|v: &ValueTypes| format!("{}", v))
            .collect();
        let values: Vec<String> = self
            .values
            .iter()
            .map(|v: &ValueTypes| format!("{}", v))
            .collect();
        let field_types = match &self.field_type {
            Some(ft) => ft
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<_>>()
                .join(", "),
            None => "None".to_string(),
        };
        let indexed_fields = match &self.is_field_index {
            Some(ft) => ft
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<_>>()
                .join(", "),
            None => "None".to_string(),
        };

        write!(
            f,
            "Command: {}, Table: {}, Fields: [{}], Values: [{}], Conditions: {:?}, Index: {}, Length: {}, Field_Types[{}], Indexed_Fields[{}]",
            cmd,
            self.table,
            fields.join(", "),
            values.join(", "),
            self.conditions,
            self.index,
            self.length,
            field_types,
            indexed_fields,
        )
    }
}
