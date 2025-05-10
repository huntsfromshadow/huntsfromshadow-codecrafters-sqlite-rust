use std::fs::File;
use std::io;
use byteorder::ReadBytesExt;
use crate::sqlite_varint_processing::process_sqlite_varint;

pub trait ReadSQLiteBigEndianVarint {
    fn read_sqlite_be_varint(&mut self) -> io::Result<(i64, usize)>;
}

impl ReadSQLiteBigEndianVarint for File {
    fn read_sqlite_be_varint(&mut self) -> Result<(i64, usize), std::io::Error> {
        let mut data: Vec<u8> = vec![];
        loop {
            let d = self.read_u8()?;
            data.push(d);

            if (d < 0x80) {
                break;
            }
        }
        
        let res = process_sqlite_varint(data);
        match res {
            Ok((val1, val2)) => Ok((val1, val2)),
            Err(e) => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid Varint data")),
        }
    }
}
