use crate::data::{SelectionMode, Table};

#[derive(Debug)]
pub struct State {
    pub selected_row: usize,
    pub selected_col: usize,
    pub active_mode: SelectionMode,
    pub available_modes: Vec<SelectionMode>,
    pub table: Table,
}

impl State {
    pub fn visible_rows(&self) -> usize {
        self.table.rows.len()
    }

    pub fn num_columns(&self) -> usize {
        self.table
            .headers
            .as_ref()
            .map(|h| h.len())
            .unwrap_or_else(|| self.table.rows.first().map_or(0, |r| r.len()))
    }

    pub fn cell_is_selected(&self, row: usize, col: usize) -> bool {
        match self.active_mode {
            SelectionMode::Row => row == self.selected_row,
            SelectionMode::Column => col == self.selected_col,
            SelectionMode::Cell => row == self.selected_row && col == self.selected_col,
        }
    }

    pub fn cycle_mode(&mut self) {
        if self.available_modes.len() <= 1 {
            return;
        }
        let current_idx = self
            .available_modes
            .iter()
            .position(|m| *m == self.active_mode)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.available_modes.len();
        self.active_mode = self.available_modes[next_idx];
    }

    pub fn clamp_col(&mut self) {
        let num_cols = self.num_columns();
        if num_cols > 0 && self.selected_col >= num_cols {
            self.selected_col = num_cols - 1;
        }
    }
}

impl Default for State {
    fn default() -> Self {
        State {
            selected_row: 0,
            selected_col: 0,
            active_mode: SelectionMode::Row,
            available_modes: vec![SelectionMode::Row],
            table: Table {
                headers: None,
                rows: Vec::new(),
            },
        }
    }
}
