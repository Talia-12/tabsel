use crate::data::Table;

#[derive(Debug)]
pub struct State {
    pub selected_row: usize,
    pub table: Table,
}

impl State {
    /// Returns the number of visible data rows.
    pub fn visible_rows(&self) -> usize {
        self.table.rows.len()
    }

    /// Returns the number of columns in the table.
    pub fn num_columns(&self) -> usize {
        self.table
            .headers
            .as_ref()
            .map(|h| h.len())
            .unwrap_or_else(|| self.table.rows.first().map_or(0, |r| r.len()))
    }
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
