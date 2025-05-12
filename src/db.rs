use std::cmp::PartialEq;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};
use crate::util::ReadSQLiteBigEndianVarint;

/// DB File Given the nature of the way this works only using structs to pass data not to isolate down
///  code exception is the DB object mainly to hold the file

pub struct DbInfoResult {
    pub number_of_tables: u16,
    pub database_page_size: u16,
}

pub struct TablesInfoResult {
    pub table_names: Vec<String>,
}

pub struct Db {
    //disk_file_path: String,
    disk_file: File
}

#[derive(Debug, PartialOrd, PartialEq)]
pub enum ColumnType {
    Null,
    EightBitInteger,
    SixteenBitInteger,
    TwentyFourBitInteger,
    ThirtyTwoBitInteger,
    FortyEightBitInteger,
    SixtyFourBitInteger,
    Ieee64BitInteger,
    SchemaFour0,
    SchemaFour1,
    ReservedInternal,
    Blob(usize),
    String(usize),
    Error,
}


impl Db {
    pub fn new_with_file(path: String) -> Self {
        Self {
            //disk_file_path: path.clone(),
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

        let df = &mut self.disk_file; // used just to save typing
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

        let mut table_names: Vec<String> = vec![];

        // Get the table data - We are doing this in idx 1 due to offset multiplying
        for cell_offset in cell_offsets {
            // Jump to start of block
            df.seek(SeekFrom::Start(cell_offset as u64)).expect(&format!("Could not sync to offset {}", cell_offset));

            // First varint payload size in bytes including overflow
            let _ = df.read_sqlite_be_varint().expect("Error reading sqlvarint for cell payload size");
            let _ = df.read_sqlite_be_varint().expect("Error reading sqlvarint for row id");
            let total_bytes_in_header = df.read_sqlite_be_varint().expect("Error reading sqlvarint for total bytes in header");

            eprintln!("total bytes: {:?}", total_bytes_in_header);
            
            let mut colheaders: Vec<ColumnType> = vec![];

            let mut i = total_bytes_in_header.1 as i64;
            while i < total_bytes_in_header.0 {
                let d = df.read_sqlite_be_varint().expect("Error reading sqlvarint for row id");
                colheaders.push(match d.0 {
                    0 => ColumnType::Null,
                    1 => ColumnType::EightBitInteger,
                    2 => ColumnType::SixteenBitInteger,
                    3 => ColumnType::TwentyFourBitInteger,
                    4 => ColumnType::ThirtyTwoBitInteger,
                    5 => ColumnType::FortyEightBitInteger,
                    6 => ColumnType::SixtyFourBitInteger,
                    7 => ColumnType::Ieee64BitInteger,
                    8 => ColumnType::SchemaFour0,
                    9 => ColumnType::SchemaFour1,
                    10 | 11 => ColumnType::ReservedInternal,
                    val if val >= 12 && val % 2 == 0 => ColumnType::Blob((val as usize - 12) / 2),
                    val if val >= 23 && val % 2 == 1 => ColumnType::String((val as usize - 13) / 2),
                    _ => ColumnType::Error
                });
                i = i + d.1 as i64;
            }
            print!("{:?}", colheaders);

            let mut coldata:Vec<(ColumnType, Vec<u8>)> = vec![];
            // Now lets grab the actual column data
            for col in colheaders {
                match col {
                    ColumnType::EightBitInteger => {
                        let d = df.read_u8().expect("Error reading u8 for EightBitInteger");
                        coldata.push((ColumnType::EightBitInteger, vec![d]));
                    }
                    ColumnType::String(val) => {
                        let mut buf = vec![0u8; val];
                        df.read_exact(&mut buf).unwrap();
                        coldata.push((ColumnType::String(val),buf));
                    }
                    _ => panic!("Have not implemented column parser")
                }
            }
            
            println!("Column data: {:?}", coldata);
            
            // Read out data
            let mut cnt = 0;
            for col in coldata {
                match col.0 {
                    ColumnType::EightBitInteger => {
                        let x = col.1.first().expect("Error reading u8 for EightBitInteger").clone() as i8;
                        eprintln!("{:?} -- {}", col.0, x);
                    }
                    ColumnType::String(_) => {
                        let end_str = String::from_utf8(col.1).expect("Error reading String");
                        eprintln!("{:?} -- {:?}", col.0, end_str);

                        if cnt == 2 {
                            table_names.push(end_str);
                        }
                    }
                    _ => panic!("Have not implemented column parser")
                }
                cnt = cnt + 1;
            }
        }

        eprintln!("{:?}", table_names);

        TablesInfoResult {
            table_names: table_names.clone()
        }
    }
}

