use crate::data::Table;

#[derive(Debug)]
pub struct State {
    pub selected_row: usize,
    pub table: Table,
}

impl Default for State {
    fn default() -> Self {
        State {
            selected_row: 0,
            table: Table {
                headers: None,
                rows: Vec::new(),
            },
        }
    }
}
