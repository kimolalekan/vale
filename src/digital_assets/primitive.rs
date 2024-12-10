use chrono::Utc;

pub struct Candidates {
    values: Vec<String>,
}

pub enum DocumentVisibility {
    Private(String),
    Encrypted(Vec<u8>),
    Public(String),
}

pub struct Document {
    link: DocumentVisibility,
    visibilty: bool,
    hash: String,
    timestamp: u64,
}
