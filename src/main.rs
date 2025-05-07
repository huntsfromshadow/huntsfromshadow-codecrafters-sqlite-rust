mod util;

use anyhow::{bail, Result};
use std::fs::File;
use crate::util::{parse_page_zero, PageZero};

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

    let file = File::open(&args[1])?;
    let mut pz = PageZero::default();

    parse_page_zero(file, &mut pz);

    match command.as_str() {
        ".dbinfo" => {
            //println!("Number of pages: {}", pz.number_of_pages);
            //println!("database page size: {}", pz.database_page_size);
            println!("number of tables: {}", pz.number_of_tables);
        }
        ".tables" => {
            //eprintln!("Number of pages: {}", pz.number_of_pages);
            //eprintln!("database page size: {}", pz.database_page_size);
            //eprintln!("number of tables: {}", pz.number_of_tables);
            println!("{:?}", pz.table_names);
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
