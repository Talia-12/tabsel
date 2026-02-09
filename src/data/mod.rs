pub mod parse;

#[derive(Debug, Clone)]
pub struct Table {
    pub headers: Option<Vec<String>>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputFormat {
    Csv,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    Row,
    Column,
    Cell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Plain,
    Json,
    Csv,
}
