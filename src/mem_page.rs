/*use std::cmp::PartialEq;
use std::io::{Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt};
use crate::db_file::DbFile;
use crate::util::ReadSQLiteBigEndianVarint;

#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
pub enum BTreePageType {
    InteriorIndexPage,
    InteriorTablePage,
    LeafIndexPage,
    LeafTablePage,
    UnknownPageType,
}

/// This is a page in memory.  There is a boolean to let people know if this is a page zero
///   if this is false some variables will be left at default value (noted in the block)
///   these pages are loaded Upon need (except page 1)
#[derive(Debug, Copy, Clone)]
pub struct MemPage {
    page_number: u64,
    loaded: bool,
    page_type: BTreePageType,
    first_freeblock_on_page_offset: u16,
    number_of_cells_on_page: u16,
    start_of_cell_content_area_offset: u16,
    fragmented_free_bytes_in_cell_content_number: u8,
}

impl MemPage {
    pub fn new() -> Self {
        Self {
            page_number: 0,
            loaded: false,
            page_type: BTreePageType::UnknownPageType,
            first_freeblock_on_page_offset: 0,
            number_of_cells_on_page: 0,
            start_of_cell_content_area_offset: 0,
            fragmented_free_bytes_in_cell_content_number: 0,
        }
    }

    pub fn new_page_with_number(db: &mut DbFile, page_number: u64) -> Self {
        let mut retval = Self {
            page_number: page_number,
            loaded: false,
            page_type: BTreePageType::UnknownPageType,
            first_freeblock_on_page_offset: 0,
            number_of_cells_on_page: 0,
            start_of_cell_content_area_offset: 0,
            fragmented_free_bytes_in_cell_content_number: 0,
        };

        let mut df = &db.db_file;

        df.rewind().expect("Could not rewind db file");

        if(retval.page_number == 1) {
            // Page type may need a fast-forward if page 1
            // Normally our position function would handle this but lets avoid a seek on other pages if we could
            df.seek(SeekFrom::Start(100)).expect("Could not sync to spot 100");
        }
        retval.page_type = match (df.read_u8().expect("Could not read u8 for page type")) {
            0x05 => BTreePageType::InteriorTablePage,
            0x02 => BTreePageType::InteriorIndexPage,
            0x0a => BTreePageType::LeafIndexPage,
            0x0d => BTreePageType::LeafTablePage,
            _ => { panic!("Unknown page type") }
        };

        retval.first_freeblock_on_page_offset = df.read_u16::<BigEndian>()
            .expect(&format!("Could not read first start of freeblock for page {}", retval.page_number));

        retval.number_of_cells_on_page = df.read_u16::<BigEndian>()
            .expect(&format!("Could not read number of cells for page {}", retval.page_number));

        retval.start_of_cell_content_area_offset =  df.read_u16::<BigEndian>()
            .expect(&format!("Could not read start of cell content area {}", retval.page_number));

        retval.fragmented_free_bytes_in_cell_content_number =  df.read_u8()
            .expect(&format!("Could not number of fragmented free bytes in content {}", retval.page_number));

        if retval.page_type != BTreePageType::LeafTablePage {
            panic!("Unexpected page type - mem-page.rs");
        }

        let mut cell_pointers: Vec<u16> = vec![];
        for x in 0..retval.number_of_cells_on_page {
            let cp = df.read_u16::<BigEndian>().expect("Error reading cell pointer");
            cell_pointers.push(cp);
        }

        println!("cell pointers: {:?}", cell_pointers);

        /*    page_zero.database_page_size = crate::util::read_u16(&mut file);

            file.seek(SeekFrom::Start(28)).unwrap();
            page_zero.number_of_pages = crate::util::read_u32(&mut file);

            file.seek(SeekFrom::Start(103)).unwrap();
            page_zero.number_of_tables = crate::util::read_u16(&mut file);

            // Now we need the cell pointer list
            let mut ptr_vec: Vec<u16> = Vec::with_capacity(page_zero.number_of_tables as usize);
            file.seek(SeekFrom::Start(108)).unwrap();
            for i in 0..page_zero.number_of_tables {
                let ptr_val = file.read_u16::<BigEndian>().unwrap();
                ptr_vec.push(ptr_val);
            }
            ptr_vec.reverse();
        }*/


        retval
    }
}*/