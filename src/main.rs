use anyhow::{bail, Result};
use std::fs::File;
use std::io::prelude::*;
use std::os::unix::prelude::FileExt;

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
    match command.as_str() {
        ".dbinfo" => {
            let mut file = File::open(&args[1])?;
            let mut header = [0; 100];
            file.read_exact(&mut header)?;

            // Not a very short solution but can help with new people to rust.
            // Also requires that the first page have all the data.
            
            // The page size is stored at the 16th byte offset, using 2 bytes in big-endian order
            #[allow(unused_variables)]
            let page_size = u16::from_be_bytes([header[16], header[17]]);

            #[allow(unused_variables)]
            let page_count = u32::from_be_bytes([header[28], header[29], header[30], header[31]]);
           
            // Page 0 - we know from challenges we only care about page first page
            // going to run from 0 to 3096 (4096 - 100)
            let mut page0_post_header: [u8; 3096] = [0; 3096];
            file.read_exact_at(&mut page0_post_header, 100)?;
            
            #[allow(unused_variables)]
            let page_type = u8::from_be_bytes([page0_post_header[0]]);
            
            #[allow(unused_variables)]
            let cell_count = u16::from_be_bytes([page0_post_header[3],page0_post_header[4]]); 
            
            eprintln!("{}", cell_count);
           
          
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            eprintln!("Logs from your program will appear here!");

            eprintln!("Number of pages: {}", page_count);
          


            println!("database page size: {}", page_size);
            println!("number of tables: {}", cell_count);
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
