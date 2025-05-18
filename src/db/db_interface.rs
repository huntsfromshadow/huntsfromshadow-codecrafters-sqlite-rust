use crate::db::db::Db;

pub struct DbInterface {
    pub db: Db,
}

impl DbInterface {
    pub fn new() -> Self {
        Self {
            db: Db::new()
        }
    }
    
    pub fn open_db(path: String) -> Self {
        let mut db = Db::new_with_file(path.clone());
        db.discover_db();
        let mut retval = Self {
            db
        };
        retval
    }
}

// Result obj
pub struct DbInfoResult {
    pub number_of_tables: u16,
    pub database_page_size: u16,
}

pub struct TablesInfoResult {
    pub table_names: Vec<String>,
}
