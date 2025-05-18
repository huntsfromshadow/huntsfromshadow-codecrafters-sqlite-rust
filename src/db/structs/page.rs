#[derive(Debug, PartialOrd, PartialEq)]
pub enum BTreePageType {
    InteriorIndex,
    InteriorTable,
    LeafIndex,
    LeafTable,
    Error
}

#[derive(Debug)]
pub struct Page {
    pub page_type: BTreePageType,
    pub page_hdr_idx: i64,
    pub page_end_idx: i64,
}



