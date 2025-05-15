use crate::db::Db;

pub struct DbInterface {
    pub db: Db
}

impl DbInterface {
    pub fn new() -> Self {
        Self {
            db: Db::new()
        }
    }
    
    pub fn open_db(path: String) -> Self {
        let db = Db::new_with_file(path.clone());
        let mut retval = Self {
            db
        };
        
        
        retval
    }
    
    
}
