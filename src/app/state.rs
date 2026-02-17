use crate::data::{OutputFormat, SelectionMode, Table};

#[derive(Debug)]
pub struct State {
    pub selected_row: usize,
    pub selected_col: usize,
    pub active_mode: SelectionMode,
    pub available_modes: Vec<SelectionMode>,
    pub table: Table,
    pub filter_enabled: bool,
    pub filter_text: String,
    pub filtered_indices: Vec<usize>,
    pub output_format: OutputFormat,
    /// Indices of columns that are visible (not hidden). Maps visible position to actual column index.
    pub visible_columns: Vec<usize>,
}

impl State {
    pub fn visible_rows(&self) -> usize {
        self.filtered_indices.len()
    }

    pub fn num_columns(&self) -> usize {
        self.visible_columns.len()
    }

    /// Maps a visible column index to the actual table column index.
    pub fn actual_col_index(&self, visible_col: usize) -> usize {
        self.visible_columns[visible_col]
    }

    /// Returns the actual table row index for a given filtered position.
    pub fn actual_row_index(&self, filtered_pos: usize) -> usize {
        self.filtered_indices[filtered_pos]
    }

    pub fn cell_is_selected(&self, filtered_pos: usize, col: usize) -> bool {
        match self.active_mode {
            SelectionMode::Row => filtered_pos == self.selected_row,
            SelectionMode::Column => col == self.selected_col,
            SelectionMode::Cell => filtered_pos == self.selected_row && col == self.selected_col,
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

    pub fn update_filtered_indices(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_indices = (0..self.table.rows.len()).collect();
        } else {
            let query = self.filter_text.to_lowercase();
            self.filtered_indices = self
                .table
                .rows
                .iter()
                .enumerate()
                .filter(|(_, row)| {
                    row.iter()
                        .any(|cell| cell.to_lowercase().contains(&query))
                })
                .map(|(idx, _)| idx)
                .collect();
        }
    }

    pub fn init_filtered_indices(&mut self) {
        self.filtered_indices = (0..self.table.rows.len()).collect();
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
            filter_enabled: true,
            filter_text: String::new(),
            filtered_indices: Vec::new(),
            output_format: OutputFormat::Plain,
            visible_columns: Vec::new(),
        }
    }
}
