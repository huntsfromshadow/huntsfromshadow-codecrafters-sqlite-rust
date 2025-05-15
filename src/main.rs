mod util;
mod db;
mod sqlite_varint_processing;
mod db_interface;

use anyhow::{bail, Result};
use crate::db::Db;

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();

    let command: String;
    let fname: String;
    if args.len() == 1 && args[0] == "target/debug/codecrafters-sqlite" {
        // Set up the variables so we can debug the current test
        command = "SELECT COUNT(*) FROM apples".to_string();
        fname = "sample.db".to_string();
    } else {
        match args.len() {
            0 | 1 => bail!("Missing <database path> and <command>"),
            2 => bail!("Missing <command>"),
            _ => {}
        }

        // Parse command and act accordingly
        //let command = &args[2];
        command = args[2].to_string();
        fname = args[1].to_string();
    }

    let mut db = Db::new_with_file(fname.clone());

    match command.as_str() {
        ".dbinfo" => {
            let dbinfo = db.cmd_get_db_info();
            println!("number of tables: {}", dbinfo.number_of_tables);
            println!("database page size: {}", dbinfo.database_page_size);
        }
        ".tables" => {
            let dat = db.cmd_get_tables_info();
            println!("{:?}", dat.table_names);
        }
        "SELECT COUNT(*) FROM apples" => {
            // For now lets just manually do the query. Then we can abstract from that.
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
