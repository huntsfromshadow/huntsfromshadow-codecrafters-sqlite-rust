mod db;
mod util;
mod sqlite_varint_processing;

use anyhow::{bail, Result};
use std::fs::File;
use crate::db::Db;

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    // Parse command and act accordingly
    let command = &args[2];

    let mut db = Db::new_with_file(args[1].clone());

    match command.as_str() {
        ".dbinfo" => {
            let dbinfo = db.cmd_get_db_info();
            println!("number of tables: {}", dbinfo.number_of_tables);
            println!("database page size: {}", dbinfo.database_page_size);
        }
        ".tables" => {
            let table_info = db.cmd_get_tables_info();
           
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
