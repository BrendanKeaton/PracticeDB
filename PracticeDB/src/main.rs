mod core;
mod parsing;
mod query;
mod utils;

use colored::*;
use query::handle_query;
use std::io::{self, Write};
use utils::handle_sql_parsing;

use crate::{core::BufferPool, core::QueryObject};

fn main() {
    let mut buffer_pool: BufferPool = BufferPool::default();
    run(&mut buffer_pool);
}

fn run(buffer_pool: &mut BufferPool) {
    loop {
        print!("Enter command: ");
        if io::stdout().flush().is_err() {
            println!("{}", "Failed to flush stdout. Retrying".red());
            continue;
        }
        let mut cmd = String::new();
        match io::stdin().read_line(&mut cmd) {
            Ok(_) => {
                let cmd = cmd.trim().to_lowercase();
                let mut query: QueryObject = QueryObject::default();
                if !(handle_sql_parsing(&cmd.as_str(), &mut query)) {
                    break;
                }
                println!("{}", query);
                match handle_query(&mut query, buffer_pool) {
                    Ok(true) => {}
                    Ok(false) => break,
                    Err(e) => println!("{}", e.red()),
                }
            }
            Err(_) => {
                println!("{}", "Failed to read input, please try again.".red());
                continue;
            }
        }
    }

    if let Err(e) = buffer_pool.flush_all_pages_to_disk() {
        println!("{}", format!("Failed to flush pages on exit: {}", e).red());
    }
}
