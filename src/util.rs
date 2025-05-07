use anyhow::{bail, Result};
use byteorder::ByteOrder;
use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::ptr::write;

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

pub struct Cell {
    pub total_bytes_payload: u64,
    pub row_id: u64,
}

impl Cell {
    pub fn default() -> Self {
        Self {
            total_bytes_payload: 0,
            row_id: 0,
        }
    }
}

#[derive(Debug)]
struct Record {
    pub size_record_header_len: u64,
    pub column_data: Vec<u64>,
}

pub fn parse_page_zero(mut file: File, page_zero: &mut PageZero) {
    file.seek(SeekFrom::Start(16)).unwrap();
    page_zero.database_page_size = read_u16(&mut file);

    file.seek(SeekFrom::Start(28)).unwrap();
    page_zero.number_of_pages = read_u32(&mut file);

    file.seek(SeekFrom::Start(103)).unwrap();
    page_zero.number_of_tables = read_u16(&mut file);

    // Now we need the cell pointer list
    let mut ptr_vec: Vec<u16> = Vec::with_capacity(page_zero.number_of_tables as usize);
    file.seek(SeekFrom::Start(108)).unwrap();
    for i in 0..page_zero.number_of_tables {
        let ptr_val = file.read_u16::<BigEndian>().unwrap();
        ptr_vec.push(ptr_val);
    }
    ptr_vec.reverse();

    // Okay now lets parse the cells.
    for ptr_rel in ptr_vec {
        // Jump to cell
        file.seek(SeekFrom::Start(ptr_rel as u64)).unwrap();

        // Build our cell struct
        let mut c = Cell::default();
        c.total_bytes_payload = read_varint(&mut file).0;
        c.row_id = read_varint(&mut file).0;

        let mut r = Record {
            size_record_header_len: 0,
            column_data: vec![],
        };
        let siz = read_varint(&mut file);
        r.size_record_header_len = siz.0;
        //eprintln!("{:?}", siz);
        let start = siz.1 as u64;
        //eprintln!("start {}", start);

        let mut read_bytes = siz.1 as u64;
        while (read_bytes < r.size_record_header_len) {
            // Read next varint
            let rs = read_varint(&mut file);
            //eprintln!("rbyte: {:?}, res: {:?} /", read_bytes, rs);
            read_bytes = read_bytes + rs.1 as u64;
            r.column_data.push(rs.0);
        }
        //eprintln!("{:?}", r.column_data);

        // Finally get the data from the columns
        let mut col_cnt = 0;
        for cd in r.column_data {
            if (cd >= 23 && cd % 2 == 1) {
                // Value is a string in the text encoding and (N-13)/2 bytes in length. The nul terminator is not stored.
                let siz = (cd - 13) / 2;
                //eprintln!("siz {}", siz);

                let mut buffer = vec![0u8; siz as usize];
                //eprintln!("{:?}", buffer);
                file.read_exact(&mut buffer).unwrap();

                //eprintln!("{:0x?}", buffer);
                let ns = std::str::from_utf8(buffer.as_slice()).unwrap();
                //eprintln!("ns {}", ns);

                if (col_cnt == 2) {
                    // It's a table
                    println!("{}", ns);
                }

                col_cnt = col_cnt + 1;
            } else if cd == 1 {
                col_cnt = col_cnt + 1;
                // Two complements number
            } else {
                panic!("Unknown Type: {}", cd);
            }
        }
    }

    /*r.st_schema_type = read_varint(&mut file);
    r.st_schema_name = read_varint(&mut file);
    r.st_table_name = read_varint(&mut file);
    r.st_table_rootpage = read_varint(&mut file);
    r.st_table_sql = read_varint(&mut file);*/

    ////eprintln!("{:?}", r);
}

#[derive(Debug, PartialEq, Eq)]
pub enum VarintError {
    /// Indicates that the input byte slice did not contain enough bytes
    /// to form a complete varint (e.g., a byte indicated continuation
    /// but no more bytes followed).
    IncompleteVarint,
}

fn read_varint(file: &mut File) -> (u64, usize) {
    let mut wrk: Vec<u8> = Vec::new();
    loop {
        let v = file.read_u8().unwrap();
        wrk.push(v);
        if (v < 0x80) {
            break;
        }
    }

    if (wrk.len() == 1) {
        for i in 0..8 - wrk.len() {
            wrk.insert(0, 0u8);
        }
        let x = wrk.as_slice();
        (BigEndian::read_u64(x), 1)
    } else {
        let z = wrk.len();
        let x = decode_sqlite_varint(&wrk).unwrap();
        x
    }
}

pub fn decode_sqlite_varint(data: &[u8]) -> Result<(u64, usize), VarintError> {
    let mut result: u64 = 0;
    let mut bytes_read: usize = 0;

    if data.is_empty() {
        return Err(VarintError::IncompleteVarint);
    }

    for i in 0..9 {
        // A varint is at most 9 bytes long
        if i >= data.len() {
            // We need more bytes, but the input slice is exhausted.
            // This happens if a previous byte had its MSB set, indicating continuation.
            return Err(VarintError::IncompleteVarint);
        }

        let byte = data[i];
        bytes_read += 1;

        if i < 8 {
            // For the first 8 bytes, the lower 7 bits are payload.
            let payload = byte & 0x7F;
            result = (result << 7) | (payload as u64);

            if (byte & 0x80) == 0 {
                // MSB is 0, so this is the last byte of the varint.
                return Ok((result as u64, bytes_read));
            }
            // MSB is 1, continue to the next byte.
        } else {
            // This is the 9th byte. It uses all 8 bits for payload.
            // The `result` already contains 56 bits from the first 8 bytes (8 * 7 bits).
            // We shift it by 8 to make space for the full 9th byte.
            result = (result << 8) | (byte as u64);
            return Ok((result as u64, bytes_read));
        }
    }

    // This part should ideally not be reached if the varint is correctly formed
    // and fits within 9 bytes, as the loop's conditions and returns cover all cases.
    // If the 8th byte had MSB set, the 9th byte path is taken and returns.
    // If any byte < 8th had MSB cleared, it would have returned earlier.
    // For safety, though practically unreachable with current logic:
    Err(VarintError::IncompleteVarint) // Or a more specific error if possible
}

/*wrk.reverse();
//eprintln!("Final Read int arr: {:?}", wrk);
for i in 0..8 - wrk.len() {
    wrk.push(0);
}
//eprintln!("Final Read int arr: {:?}", wrk);
wrk.reverse();
//eprintln!("Final Read int arr: {:?}", wrk);*/

pub fn calculate_u8_be(v1: u8, v2: u8) -> u16 {
    ((v1 as u16) << 8) | v2 as u16
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
