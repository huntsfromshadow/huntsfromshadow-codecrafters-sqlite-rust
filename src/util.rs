use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub struct PageZero {
    pub database_page_size: u16,
    pub number_of_pages: u32,
    pub number_of_tables: u16,
}

impl PageZero {
    pub fn default() -> Self {
        Self {
            number_of_pages: 0,
            database_page_size: 0,
            number_of_tables: 0,
        }
    }
}

pub fn parse_page_zero(mut file: File, page_zero: &mut PageZero)  {

    file.seek(SeekFrom::Start(16)).unwrap();
    page_zero.database_page_size = read_u16(&mut file);
    
    file.seek(SeekFrom::Start(28)).unwrap();
    page_zero.number_of_pages = read_u32(&mut file);

    file.seek(SeekFrom::Start(103)).unwrap();
    page_zero.number_of_tables = read_u16(&mut file);


    // Now lets parse the table itself
}

fn read_u16(file: &mut File) -> u16 {
    let mut rd_buf = [0; 2];
    file.read_exact(&mut rd_buf).unwrap();

    u16::from_be_bytes(rd_buf)
}

fn read_u32(file: &mut File) -> u32 {
    let mut rd_buf = [0; 4];
    file.read_exact(&mut rd_buf).unwrap();

    u32::from_be_bytes(rd_buf)
}