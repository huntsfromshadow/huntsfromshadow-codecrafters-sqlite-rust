use std::fs::File;
use std::io::{Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};
use crate::db_file::Instruction::SKIP;
use crate::mem_page::MemPage;
// Core data of just the db file itself
// also methods to get at other data

//Header stuff
#[derive(Debug, PartialEq, Copy, Clone)]
enum Instruction {
    SKIP,
    SAVE
}
const HEADER_DEF: [(i32, i32, Instruction); 23] = [
    (0, 16, Instruction::SKIP), // Header String
    (16, 2, Instruction::SAVE), // The database page size in bytes. 1 = 65536
    (18, 1, Instruction::SKIP), //	File format write version. 1 for legacy; 2 for WAL.
    (19, 1, Instruction::SKIP), //	File format read version. 1 for legacy; 2 for WAL.
    (20, 1, Instruction::SKIP), //	Bytes of unused "reserved" space at the end of each page. Usually 0.
    (21, 1, Instruction::SKIP), //	Maximum embedded payload fraction. Must be 64.
    (22, 1, Instruction::SKIP), //	Minimum embedded payload fraction. Must be 32.
    (23, 1, Instruction::SKIP), //	Leaf payload fraction. Must be 32.
    (24, 4, Instruction::SKIP), //	File change counter.
    (28, 4, Instruction::SAVE), //	Size of the database file in pages. The "in-header database size".
    (32, 4, Instruction::SAVE), //	Page number of the first freelist trunk page.
    (36, 4, Instruction::SAVE), //	Total number of freelist pages.
    (40, 4, Instruction::SKIP), //	The schema cookie.
    (44, 4, Instruction::SKIP), //	The schema format number. Supported schema formats are 1, 2, 3, and 4.
    (48, 4, Instruction::SKIP), //	Default page cache size.
    (52, 4, Instruction::SKIP), //	The page number of the largest root b-tree page when in auto-vacuum or incremental-vacuum modes, or zero otherwise.
    (56, 4, Instruction::SAVE), //	The database text encoding. A value of 1 means UTF-8. A value of 2 means UTF-16le. A value of 3 means UTF-16be.
    (60, 4, Instruction::SKIP), //	The "user version" as read and set by the user_version pragma.
    (64, 4, Instruction::SKIP), //	True (non-zero) for incremental-vacuum mode. False (zero) otherwise.
    (68, 4, Instruction::SKIP), //	The "Application ID" set by PRAGMA application_id.
    (72, 2, Instruction::SKIP), //  0	Reserved for expansion. Must be zero.
    (92, 4, Instruction::SKIP), //	The version-valid-for number.
    (96, 4, Instruction::SKIP), //	SQLITE_VERSION_NUMBER
];

#[derive(Debug)]
pub struct DbFile {
    // Most of this class really is just moving the seek head around,
    // but there are some fields that we should keep
    pub database_page_size: u16,
    pub db_size_in_pages: u32,
    pub page_number_first_freelist_trunk: u32,
    pub number_freelist_pages: u32,
    pub db_text_encoding: u32,

    pub db_file: File,
    pub page_one: MemPage,
}

impl DbFile {
    // Static Stuff
    pub fn init_process_file(db_path: String) {
        let mut retval = Self {
            database_page_size: 0,
            db_size_in_pages: 0,
            page_number_first_freelist_trunk: 0,
            number_freelist_pages: 0,
            db_text_encoding: 0,

            db_file: File::open(db_path).expect("Could not open database file"),
            page_one: MemPage::new(), // Just putting something here will replace later
        };
        retval.process_db_header();
        retval.page_one = MemPage::new_for_page_one(&mut retval);
    }

    fn process_db_header(&mut self) {
        HEADER_DEF.iter().filter(|hfield: &&(i32, i32, Instruction)| {
            hfield.2 == Instruction::SAVE
        })
        .copied()
        .for_each(|hfield: (i32, i32, Instruction)| {
            self.db_file.seek(SeekFrom::Start(hfield.0 as u64))
                .expect(&format!("Can't Seek for {:?}", hfield));

            let mut tmp_val: u32 = 0;

            // We are up converting all the values to u32, and we
            // will shrink them down to the size they need
            match (hfield.1) {
                2 => {
                    let t = self.db_file.read_u8()
                        .expect(&format!("Can't read from {:?}", hfield));
                    tmp_val = t as u32;
                }
                4 => {
                    let t = self.db_file.read_u32::<BigEndian>()
                        .expect(&format!("Can't read from {:?}", hfield));
                    tmp_val = t;
                }
                _ => {
                    panic!("Unknown size field {}", hfield.1);
                }
            }

            match (hfield.0) {
                16 => {
                    self.database_page_size = tmp_val as u16;
                }
                28 => {
                    self.db_size_in_pages = tmp_val;
                }
                32 => {
                    self.page_number_first_freelist_trunk = tmp_val;
                }
                36 => {
                    self.number_freelist_pages = tmp_val;
                }
                56 => {
                    self.db_text_encoding = tmp_val;
                }
                _ => {
                    panic!("Unknown data to save in the struct {:?}", hfield);
                }
            }
        });
    }
}