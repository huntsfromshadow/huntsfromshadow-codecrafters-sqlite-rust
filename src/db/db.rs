use std::cmp::PartialEq;
use std::fs::{File};
use std::io::{Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};
use crate::db::structs::page::{BTreePageType, Page};

pub struct Db {
    //disk_file_path: String,
    disk_file: Option<File>,
    pages: Vec<Page>,
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
    pub fn new() -> Self {
        Self {
            disk_file: None,
            pages: vec![],
        }
    }
    pub fn new_with_file(path: String) -> Self {
        Self {
            //disk_file_path: path.clone(),
            disk_file: Some(File::open(path).expect("Could not open database file")),
            pages: vec![],
        }
    }

    /**
    Some initial data gathering on the db. Overall we DO NOT load the data
    until upon need. But figuring out page idx, and a list of tables is helpful
     */
    pub fn discover_db(&mut self) {
        self.locate_pages();
        self.get_table_schemas();
    }

    /**
    Grab all the
    */
    pub fn locate_pages(&mut self) {
        if self.disk_file.is_none() {
            panic!("Disk File is empty. No DB Possible")
        }

        let mut df: &File = self.disk_file.as_ref().unwrap();
        df.rewind().expect("Could not rewind db file");

        df.seek(SeekFrom::Start(16)).expect("Error could not seek to page size pos 16");
        let db_page_size = df.read_u16::<BigEndian>().expect("Error could not read u16 for page size");

        // number of tables = number of cells on page 1
        df.seek(SeekFrom::Start(28)).expect("Error could not seek page count");
        let number_of_pages = df.read_u32::<BigEndian>().expect("Could not read u32 for number of cells");

        // Lets get the pages setup
        for i in 0..(number_of_pages as i64) {
            let mut sl: i64 = 0;
            let mut el: i64 = 0;
            if i == 0 {
                sl = 100;   // Page 1 sheet header starts at 100 (past db header)
                el = (db_page_size as i64) - 1;
            } else {
                sl = i * (db_page_size as i64);
                el = sl + (i * (db_page_size as i64)) - 1;
            }

            df.seek(SeekFrom::Start(sl as u64)).expect("Error could not seek page count");

            let pt = match df.read_u8().expect("Error could not read u8 for page type") {
                0x02 => BTreePageType::InteriorIndex,
                0x05 => BTreePageType::InteriorTable,
                0x0a => BTreePageType::LeafIndex,
                0x0d => BTreePageType::LeafTable,
                _ => BTreePageType::Error
            };

            self.pages.push(
                Page {
                    page_type: pt,
                    page_hdr_idx: sl,
                    page_end_idx: el,
                });
        }
    }

    pub fn get_table_schemas(&mut self) {
        // Table schema is always on page 1.
        //  idx 103 size 2 - Number of cells on page
        



        
            3	2	The two-byte integer at offset 3 gives the number of cells on the page.
            5	2	The two-byte integer at offset 5 designates the start of the cell content area. A zero value for this integer is interpreted as 65536.
        7	1	The one-byte integer at offset 7 gives the number of fragmented free bytes within the cell content area.
            8	4	The four-byte page number at offset 8 is the right-most pointer. This value appears in the header of interior b-tree pages only and is omitted from all other pages.

    }
}



/*
    

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
}*/

