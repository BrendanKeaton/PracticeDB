use crate::{
    core::{Command, Filter, Operand, TokenType, ValueTypes, structs::QueryObject},
    parsing::{
        handle_create, handle_delete, handle_from, handle_insert, handle_select, handle_where,
    },
};

pub fn handle_sql_parsing(input: &str, query: &mut QueryObject) -> bool {
    let tokens: Vec<&str> = input.split_whitespace().collect();

    query.length = tokens.len();
    query.index = 0;

    while query.index < query.length {
        let curr_token: TokenType = classify_token(tokens[query.index]);
        match curr_token {
            TokenType::CMD(command) => {
                let cmd_result = match_command(&command, &tokens, query);
                if let Err(e) = &cmd_result {
                    println!("Malformed Request. {}", e);
                }
                if cmd_result == Ok(false) {
                    return false;
                }
            }
            TokenType::OP(operand) => {
                println!(
                    "Malformed Request. Please rewrite and try again. {}",
                    operand
                );
                return false;
            }
            TokenType::FILTER(filter) => {
                let filter_result = match_filter(&filter, &tokens, query);
                if let Err(e) = &filter_result {
                    println!("Malformed Request. {}", e);
                }
                if filter_result == Ok(false) {
                    return false;
                }
            }
            TokenType::VALUE(value_types) => {
                println!(
                    "Malformed Request. Please rewrite and try again {}",
                    value_types
                );
                return false;
            }
        }
    }

    return true;
}

pub fn classify_token(token: &str) -> TokenType {
    Command::from_str(token)
        .map(TokenType::CMD)
        .or_else(|| Filter::from_str(token).map(TokenType::FILTER))
        .or_else(|| Operand::from_str(token).map(TokenType::OP))
        .or_else(|| ValueTypes::from_str(token).map(TokenType::VALUE))
        .unwrap_or_else(|| return TokenType::CMD(Command::EXIT))
}

fn match_command(
    command: &Command,
    tokens: &[&str],
    query: &mut QueryObject,
) -> Result<bool, String> {
    match command {
        Command::SELECT => handle_select(tokens, query),
        Command::INSERT => handle_insert(tokens, query),
        Command::CREATE => handle_create(tokens, query),
        Command::DELETE => handle_delete(query),
        Command::EXIT => return Ok(false),
    }
}

fn match_filter(
    filter: &Filter,
    tokens: &[&str],
    query: &mut QueryObject,
) -> Result<bool, &'static str> {
    match filter {
        Filter::FROM => handle_from(tokens, query),
        Filter::WHERE => handle_where(tokens, query, "where"),
        Filter::AND => handle_where(tokens, query, "and"),
        Filter::OR => handle_where(tokens, query, "or"),
    }
}
