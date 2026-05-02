pub mod utils;
use crate::core::ConditionsObject;
pub use crate::parsing::utils::get_table_schema;
use crate::utils::classify_token;
use crate::{
    core::{Command, LogicalConnector, QueryObject, TokenType, ValueTypes},
    parsing::utils::get_field_names,
};

pub fn handle_select(tokens: &[&str], query: &mut QueryObject) -> Result<bool, String> {
    query.command = Some(Command::SELECT);
    query.index += 1;

    while let TokenType::VALUE(val) = classify_token(tokens[query.index]) {
        if val == ValueTypes::STAR {
            query.fields.push(val);
            query.index += 1;
            return Ok(true);
        } else {
            query.fields.push(val);
            query.index += 1;

            if query.index == query.length {
                return Err(
                    "Please complete command. SELECT cannot be the final token.".to_string()
                );
            }
        }
    }
    return Ok(true);
}

pub fn handle_create(tokens: &[&str], query: &mut QueryObject) -> Result<bool, String> {
    query.command = Some(Command::CREATE);
    query.field_type = Some(Vec::new());
    query.is_field_index = Some(Vec::new());
    query.index += 1;

    let curr_token = tokens[query.index].to_owned();
    if let Ok(num) = curr_token.parse::<isize>() {
        println!(
            "Malformed request. Please do not user integer as table name. Attempted table name: {}",
            num
        );
        return Ok(false);
    }
    query.table = curr_token;
    query.index += 1;

    let field_types = query.field_type.as_mut().unwrap();
    let index_fields = query.is_field_index.as_mut().unwrap();

    while let TokenType::VALUE(ValueTypes::String(s)) = classify_token(tokens[query.index]) {
        let (field_name, field_type, is_index_str) = utils::parse_field(&s)?;

        let is_index = match is_index_str {
            "false" => false,
            "true" => true,
            _ => return Err("Invalid index flag (expected true or false)".into()),
        };

        query.fields.push(field_name);
        field_types.push(field_type);
        index_fields.push(is_index);

        query.index += 1;

        if query.index == query.length {
            return Ok(true);
        }
    }

    return Ok(true);
}

pub fn handle_insert(tokens: &[&str], query: &mut QueryObject) -> Result<bool, String> {
    query.command = Some(Command::INSERT);
    query.index += 1;

    query.table = tokens[query.index].to_owned();
    query.index += 1;

    let schema = get_table_schema(&query.table)?;

    while query.index < tokens.len() {
        if let TokenType::VALUE(ValueTypes::String(s)) = classify_token(tokens[query.index]) {
            let parsed_values = utils::parse_values_from_token(&s, &schema)?;
            query.values.extend(parsed_values);
        } else {
            return Err(format!("Unexpected token: {}", tokens[query.index]));
        }
        query.index += 1;
    }

    return Ok(true);
}

pub fn handle_where(
    tokens: &[&str],
    query: &mut QueryObject,
    cmd: &str,
) -> Result<bool, &'static str> {
    if tokens[query.index].to_owned() == "and" && query.conditions.len() == 0 {
        query.index += 1;
        return Err("Malformed request. Please use where statement before 'and'. ");
    }

    query.index += 1; // Move passed "WHERE", and dont save to query object direclty. If its "AND"/"OR", and passed above check, same deal

    let table = get_field_names(&query.table)?;

    let curr_token = tokens[query.index].to_owned();

    let mut conditions: ConditionsObject = ConditionsObject::default();

    if cmd == "or" {
        conditions.connector = Some(LogicalConnector::Or);
    } else if cmd == "and" {
        conditions.connector = Some(LogicalConnector::And);
    }

    if table.contains(&curr_token) {
        conditions.object_one_is_field = true;
    } else {
        conditions.object_one_is_field = false;
    }

    conditions.object_one = curr_token.to_owned();
    query.index += 1;

    if let TokenType::OP(op) = classify_token(tokens[query.index]) {
        conditions.operand = op.to_owned();
        query.index += 1;
    } else {
        return Err("Invalid operand type provided.");
    }

    let curr_token = tokens[query.index].to_owned();
    if table.contains(&curr_token) {
        conditions.object_two_is_field = true;
    } else {
        conditions.object_two_is_field = false;
    }

    conditions.object_two = curr_token.to_owned();
    query.index += 1;

    query.conditions.push(conditions);

    return Ok(true);
}

pub fn handle_from(tokens: &[&str], query: &mut QueryObject) -> Result<bool, &'static str> {
    query.index += 1; // Move passed "FROM", dont save to query object directly

    query.table = tokens[query.index].to_owned();
    query.index += 1;

    return Ok(true);
}

pub fn handle_delete(query: &mut QueryObject) -> Result<bool, String> {
    query.command = Some(Command::DELETE);
    query.index += 1;

    return Ok(true);
}
