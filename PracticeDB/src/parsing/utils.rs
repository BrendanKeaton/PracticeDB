use std::fs;

use crate::core::{FieldTypesAllowed, TableMetadataObject, ValueTypes};

pub fn parse_field(s: &str) -> Result<(ValueTypes, FieldTypesAllowed, &str), String> {
    let count_colon: usize = s.chars().filter(|c| *c == ':').count();

    if count_colon == 1 {
        let (name, ty) = s
            .split_once(':')
            .ok_or("Field must be formatted as name:type")?;

        let field_name = ValueTypes::from_str(name).ok_or("Protected name for field")?;

        let field_type = FieldTypesAllowed::from_str(ty).ok_or("Unknown field type")?;

        Ok((field_name, field_type, "false"))
    } else if count_colon == 2 {
        let parts: Vec<&str> = s.splitn(3, ':').collect();
        if parts.len() != 3 {
            return Err("Field must be formatted as name:type:is_index".to_string());
        }
        let (name, ty, is_index) = (parts[0], parts[1], parts[2]);

        let field_name = ValueTypes::from_str(name).ok_or("Protected name for field")?;

        let field_type = FieldTypesAllowed::from_str(ty).ok_or("Unknown field type")?;

        Ok((field_name, field_type, is_index))
    } else {
        Err("Incorrect formatting".to_string())
    }
}

pub fn parse_values_from_token(
    token: &str,
    schema: &TableMetadataObject,
) -> Result<Vec<ValueTypes>, String> {
    let parts: Vec<&str> = token.split(':').collect();

    if parts.len() != schema.fields.len() {
        return Err(format!(
            "Column count mismatch: expected {}, got {}",
            schema.fields.len(),
            parts.len()
        ));
    }

    let mut values = Vec::new();
    for (part, _field) in parts.iter().zip(schema.fields.iter()) {
        values.push(ValueTypes::String(part.to_string()));
    }

    Ok(values)
}

pub fn get_table_schema(table_name: &str) -> Result<TableMetadataObject, String> {
    let schema_path = format!("database/catalogs/{}.json", table_name);

    let json = fs::read_to_string(&schema_path).map_err(|e| e.to_string())?;

    let schema: TableMetadataObject = serde_json::from_str(&json).map_err(|e| e.to_string())?;

    Ok(schema)
}

pub fn get_field_names(table_name: &str) -> Result<Vec<String>, &'static str> {
    let Ok(table_schema) = get_table_schema(table_name) else {
        return Err("Table does not exist.");
    };

    let mut field_names: Vec<String> = Vec::new();

    for each in table_schema.fields {
        field_names.push(each.name.to_owned());
    }

    return Ok(field_names);
}
