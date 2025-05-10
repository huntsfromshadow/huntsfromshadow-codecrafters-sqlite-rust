use std::fs::File;
use std::io::{Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};
use crate::util::ReadSQLiteBigEndianVarint;

/// DB File Given the nature of the way this works only using structs to pass data not to isolate down
///  code exception is the DB object mainly to hold the file

pub struct DbInfoResult {
    pub number_of_tables: u16,
    pub database_page_size: u16,
}

pub struct TablesInfoResult {

}

pub struct Db {
    disk_file_path: String,
    disk_file: File
}

impl Db {
    pub fn new_with_file(path: String) -> Self {
        Self {
            disk_file_path: path.clone(),
            disk_file: File::open(path).expect("Could not open database file"),
        }
    }

    pub fn cmd_get_db_info(&mut self) -> DbInfoResult {

        let mut df = &self.disk_file; // used just to save typing
        df.rewind().expect("Error could not rewind");

        // database page size is at page 1 offset 16
        df.seek(SeekFrom::Start(16)).expect("Error could not seek to page size pos 16");
        let dps = df.read_u16::<BigEndian>().expect("Error could not read u16 for page size");

        // number of tables = number of cells on page 1
        df.seek(SeekFrom::Start(103)).expect("Error could not seek to cell count on page 0 - pos 103");
        let num_tab = df.read_u16::<BigEndian>().expect("Could not read u16 for number of cells");

        DbInfoResult {
            number_of_tables: num_tab,
            database_page_size: dps,
        }
    }

    pub fn cmd_get_tables_info(&mut self) -> TablesInfoResult {
        // Okay first thing we really need is the various sizes
        let dbinfo = self.cmd_get_db_info();

        let mut df = &mut self.disk_file; // used just to save typing
        df.rewind().expect("Error could not rewind");

        // Tables are on page 1 in the cell content area. Let's grab those offsets the offset array
        //   is starts at 108 and goes to the number of tables (aka num cells on page 1)
        df.seek(SeekFrom::Start(108)).expect("Error could not seek to page 1, pos 108");
        let mut cell_offsets = vec![];
        for i in 0..dbinfo.number_of_tables {
            let dat = df.read_u16::<BigEndian>()
                .expect(&format!("Error could not read u16 cell offset ptr loop: {}", i));
            println!("{:04x?}", dat);
            cell_offsets.push(dat);
        }
        // Reverse the vec as it's in right to left order
        println!("Cell Pointers - {:04x?}", cell_offsets);

        // Get the table data - We are doing this in idx 1 due to offset multiplying
        for cell_offset in cell_offsets {
            // Jump to start of block
            df.seek(SeekFrom::Start(cell_offset as u64)).expect(&format!("Could not sync to offset {}", cell_offset));

            // First varint payload size in bytes including overflow
            let _ = df.read_sqlite_be_varint().expect("Error reading sqlvarint for cell payload size");
            let _ = df.read_sqlite_be_varint().expect("Error reading sqlvarint for row id");
            let total_bytes_in_header = df.read_sqlite_be_varint().expect("Error reading sqlvarint for total bytes in header");

            eprintln!("total bytes: {:?}", total_bytes_in_header);
            
            let mut coldata: Vec<(i64, usize)> = vec![];

            let mut i = total_bytes_in_header.1 as i64;
            while i < total_bytes_in_header.0 {
                let d = df.read_sqlite_be_varint().expect("Error reading sqlvarint for row id");
                let sz = match d.0 {
                    0 => 0,
                    1 => 1,
                    2 => 2,
                    3 => 3,
                    4 => 4,
                    5 => 6,
                    6 => 8,
                    7 => 8,
                    8 => 0,
                    9 => 0,
                    oval => {
                        if oval == 10 || oval == 1 {
                            panic!("Record serial type code is 10 or 11. Should not occur");
                        }
                        else {
                            if oval >= 12 && oval % 2 == 0 {
                                ( oval - 12 ) / 2
                            } else if oval >= 13 && oval % 2 == 1 {
                                ( oval - 13 ) / 2
                            }
                            else {
                                panic!("Error reading record serial type code");
                            }
                        }
                    }
                };
                coldata.push( (d.0, sz as usize) );
                i = i + d.1 as i64;;
            }
            print!("{:?}", coldata);

            // Now lets grab the actual column data
            for col in coldata {
                
            }

            


            break;

        }

        TablesInfoResult {

        }
    }
}

