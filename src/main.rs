mod util;
mod db;
mod sqlite_varint_processing;

use anyhow::{bail, Result};
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
    //let command = &args[2];
    let command = &args[2];
    let fname = &args[1];
    
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
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
