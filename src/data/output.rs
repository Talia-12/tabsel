use super::{OutputFormat, Table};

pub fn format_row(table: &Table, format: OutputFormat, row_idx: usize) -> String {
    let row = &table.rows[row_idx];
    match format {
        OutputFormat::Plain => row.join(","),
        OutputFormat::Csv => csv_encode_row(row),
        OutputFormat::Json => {
            if let Some(headers) = &table.headers {
                let obj: serde_json::Map<String, serde_json::Value> = headers
                    .iter()
                    .enumerate()
                    .map(|(i, h)| {
                        let val = row.get(i).cloned().unwrap_or_default();
                        (h.clone(), serde_json::Value::String(val))
                    })
                    .collect();
                serde_json::to_string(&obj).unwrap()
            } else {
                let arr: Vec<serde_json::Value> = row
                    .iter()
                    .map(|v| serde_json::Value::String(v.clone()))
                    .collect();
                serde_json::to_string(&arr).unwrap()
            }
        }
    }
}

pub fn format_column(table: &Table, format: OutputFormat, col_idx: usize) -> String {
    let col_name = table
        .headers
        .as_ref()
        .and_then(|h| h.get(col_idx).cloned());

    match format {
        OutputFormat::Plain => col_name.unwrap_or_else(|| col_idx.to_string()),
        OutputFormat::Csv => col_name.unwrap_or_else(|| col_idx.to_string()),
        OutputFormat::Json => {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "column".to_string(),
                serde_json::Value::String(col_name.unwrap_or_else(|| col_idx.to_string())),
            );
            serde_json::to_string(&obj).unwrap()
        }
    }
}

pub fn format_cell(
    table: &Table,
    format: OutputFormat,
    row_idx: usize,
    col_idx: usize,
) -> String {
    let value = table.rows[row_idx]
        .get(col_idx)
        .cloned()
        .unwrap_or_default();

    match format {
        OutputFormat::Plain => value,
        OutputFormat::Csv => csv_encode_row(&[value]),
        OutputFormat::Json => {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "value".to_string(),
                serde_json::Value::String(value),
            );
            obj.insert(
                "row".to_string(),
                serde_json::Value::Number(serde_json::Number::from(row_idx)),
            );
            let col_name = table
                .headers
                .as_ref()
                .and_then(|h| h.get(col_idx).cloned())
                .unwrap_or_else(|| col_idx.to_string());
            obj.insert(
                "column".to_string(),
                serde_json::Value::String(col_name),
            );
            serde_json::to_string(&obj).unwrap()
        }
    }
}

fn csv_encode_row(fields: &[String]) -> String {
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(Vec::new());
    wtr.write_record(fields).unwrap();
    wtr.flush().unwrap();
    let bytes = wtr.into_inner().unwrap();
    String::from_utf8(bytes).unwrap().trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn table_with_headers() -> Table {
        Table {
            headers: Some(vec!["name".to_string(), "age".to_string()]),
            rows: vec![
                vec!["Alice".to_string(), "30".to_string()],
                vec!["Bob".to_string(), "25".to_string()],
            ],
        }
    }

    fn table_without_headers() -> Table {
        Table {
            headers: None,
            rows: vec![
                vec!["Alice".to_string(), "30".to_string()],
                vec!["Bob".to_string(), "25".to_string()],
            ],
        }
    }

    // --- Row output ---

    #[test]
    fn row_plain_with_headers() {
        let t = table_with_headers();
        assert_eq!(format_row(&t, OutputFormat::Plain, 0), "Alice,30");
    }

    #[test]
    fn row_plain_without_headers() {
        let t = table_without_headers();
        assert_eq!(format_row(&t, OutputFormat::Plain, 1), "Bob,25");
    }

    #[test]
    fn row_json_with_headers() {
        let t = table_with_headers();
        assert_eq!(
            format_row(&t, OutputFormat::Json, 0),
            r#"{"name":"Alice","age":"30"}"#
        );
    }

    #[test]
    fn row_json_without_headers() {
        let t = table_without_headers();
        assert_eq!(
            format_row(&t, OutputFormat::Json, 0),
            r#"["Alice","30"]"#
        );
    }

    #[test]
    fn row_csv_with_headers() {
        let t = table_with_headers();
        assert_eq!(format_row(&t, OutputFormat::Csv, 0), "Alice,30");
    }

    #[test]
    fn row_csv_with_commas() {
        let t = Table {
            headers: Some(vec!["name".to_string(), "bio".to_string()]),
            rows: vec![vec!["Alice".to_string(), "likes cats, dogs".to_string()]],
        };
        assert_eq!(
            format_row(&t, OutputFormat::Csv, 0),
            r#"Alice,"likes cats, dogs""#
        );
    }

    // --- Column output ---

    #[test]
    fn column_plain_with_headers() {
        let t = table_with_headers();
        assert_eq!(format_column(&t, OutputFormat::Plain, 0), "name");
        assert_eq!(format_column(&t, OutputFormat::Plain, 1), "age");
    }

    #[test]
    fn column_plain_without_headers() {
        let t = table_without_headers();
        assert_eq!(format_column(&t, OutputFormat::Plain, 0), "0");
        assert_eq!(format_column(&t, OutputFormat::Plain, 1), "1");
    }

    #[test]
    fn column_json_with_headers() {
        let t = table_with_headers();
        assert_eq!(
            format_column(&t, OutputFormat::Json, 0),
            r#"{"column":"name"}"#
        );
    }

    #[test]
    fn column_json_without_headers() {
        let t = table_without_headers();
        assert_eq!(
            format_column(&t, OutputFormat::Json, 1),
            r#"{"column":"1"}"#
        );
    }

    // --- Cell output ---

    #[test]
    fn cell_plain() {
        let t = table_with_headers();
        assert_eq!(format_cell(&t, OutputFormat::Plain, 0, 0), "Alice");
        assert_eq!(format_cell(&t, OutputFormat::Plain, 1, 1), "25");
    }

    #[test]
    fn cell_json_with_headers() {
        let t = table_with_headers();
        assert_eq!(
            format_cell(&t, OutputFormat::Json, 0, 0),
            r#"{"value":"Alice","row":0,"column":"name"}"#
        );
    }

    #[test]
    fn cell_json_without_headers() {
        let t = table_without_headers();
        assert_eq!(
            format_cell(&t, OutputFormat::Json, 0, 1),
            r#"{"value":"30","row":0,"column":"1"}"#
        );
    }

    #[test]
    fn cell_csv() {
        let t = table_with_headers();
        assert_eq!(format_cell(&t, OutputFormat::Csv, 0, 0), "Alice");
    }

    // --- Edge cases ---

    #[test]
    fn single_column_row() {
        let t = Table {
            headers: Some(vec!["item".to_string()]),
            rows: vec![vec!["apple".to_string()]],
        };
        assert_eq!(format_row(&t, OutputFormat::Plain, 0), "apple");
        assert_eq!(
            format_row(&t, OutputFormat::Json, 0),
            r#"{"item":"apple"}"#
        );
    }

    #[test]
    fn single_row_column() {
        let t = Table {
            headers: Some(vec!["x".to_string()]),
            rows: vec![vec!["val".to_string()]],
        };
        assert_eq!(format_column(&t, OutputFormat::Plain, 0), "x");
    }
}
