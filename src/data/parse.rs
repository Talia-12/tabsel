use std::io::{self, IsTerminal, Read};

use anyhow::{anyhow, Result};

use super::{InputFormat, Table};

/// Read from stdin and parse into a Table.
pub fn parse_stdin(format: InputFormat, has_header: bool) -> Result<Table> {
    if io::stdin().is_terminal() {
        return Err(anyhow!("no input provided; pipe data into tabsel or redirect from a file"));
    }
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    parse_string(&input, format, has_header)
}

/// Parse a string into a Table (testable core).
pub fn parse_string(input: &str, format: InputFormat, has_header: bool) -> Result<Table> {
    match format {
        InputFormat::Csv => parse_csv(input, has_header),
        InputFormat::Json => parse_json(input),
    }
}

fn parse_csv(input: &str, has_header: bool) -> Result<Table> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(has_header)
        .flexible(true)
        .from_reader(input.as_bytes());

    let headers = if has_header {
        let hdrs = reader.headers()?.clone();
        if hdrs.is_empty() {
            None
        } else {
            Some(hdrs.iter().map(|h| h.to_string()).collect())
        }
    } else {
        None
    };

    let mut rows = Vec::new();
    for result in reader.records() {
        let record = result?;
        rows.push(record.iter().map(|field| field.to_string()).collect());
    }

    Ok(Table { headers, rows })
}

fn parse_json(input: &str) -> Result<Table> {
    let value: serde_json::Value = serde_json::from_str(input)?;

    match value {
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                return Ok(Table {
                    headers: None,
                    rows: Vec::new(),
                });
            }

            // Check if first element is an object (array of objects) or array (array of arrays)
            match &arr[0] {
                serde_json::Value::Object(_) => parse_json_objects(&arr),
                serde_json::Value::Array(_) => parse_json_arrays(&arr),
                _ => Err(anyhow!(
                    "JSON input must be an array of objects or an array of arrays"
                )),
            }
        }
        _ => Err(anyhow!("JSON input must be a top-level array")),
    }
}

fn parse_json_objects(arr: &[serde_json::Value]) -> Result<Table> {
    // Collect all unique keys in order of first appearance
    let mut headers: Vec<String> = Vec::new();
    for item in arr {
        if let serde_json::Value::Object(map) = item {
            for key in map.keys() {
                if !headers.contains(key) {
                    headers.push(key.clone());
                }
            }
        } else {
            return Err(anyhow!("Expected all elements to be objects"));
        }
    }

    let mut rows = Vec::new();
    for item in arr {
        if let serde_json::Value::Object(map) = item {
            let row: Vec<String> = headers
                .iter()
                .map(|key| match map.get(key) {
                    Some(v) => stringify_json_value(v),
                    None => String::new(),
                })
                .collect();
            rows.push(row);
        }
    }

    Ok(Table {
        headers: Some(headers),
        rows,
    })
}

fn parse_json_arrays(arr: &[serde_json::Value]) -> Result<Table> {
    let mut rows = Vec::new();
    for item in arr {
        if let serde_json::Value::Array(inner) = item {
            let row: Vec<String> = inner.iter().map(stringify_json_value).collect();
            rows.push(row);
        } else {
            return Err(anyhow!("Expected all elements to be arrays"));
        }
    }

    Ok(Table {
        headers: None,
        rows,
    })
}

fn stringify_json_value(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        // For non-string primitives and nested structures, use JSON representation
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // --- CSV tests ---

    #[test]
    fn csv_with_header() {
        let input = "name,age\nAlice,30\nBob,25";
        let table = parse_string(input, InputFormat::Csv, true).unwrap();

        assert_eq!(
            table.headers,
            Some(vec!["name".to_string(), "age".to_string()])
        );
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0], vec!["Alice", "30"]);
        assert_eq!(table.rows[1], vec!["Bob", "25"]);
    }

    #[test]
    fn csv_without_header() {
        let input = "Alice,30\nBob,25";
        let table = parse_string(input, InputFormat::Csv, false).unwrap();

        assert_eq!(table.headers, None);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0], vec!["Alice", "30"]);
        assert_eq!(table.rows[1], vec!["Bob", "25"]);
    }

    #[test]
    fn csv_empty() {
        let input = "";
        let table = parse_string(input, InputFormat::Csv, true).unwrap();

        assert_eq!(table.headers, None);
        assert_eq!(table.rows.len(), 0);
    }

    #[test]
    fn csv_single_column() {
        let input = "item\napple\nbanana\ncherry";
        let table = parse_string(input, InputFormat::Csv, true).unwrap();

        assert_eq!(table.headers, Some(vec!["item".to_string()]));
        assert_eq!(table.rows.len(), 3);
        assert_eq!(table.rows[0], vec!["apple"]);
        assert_eq!(table.rows[1], vec!["banana"]);
        assert_eq!(table.rows[2], vec!["cherry"]);
    }

    #[test]
    fn csv_quoted_fields_with_commas_and_newlines() {
        let input = "name,bio\nAlice,\"likes cats, dogs\"\nBob,\"line1\nline2\"";
        let table = parse_string(input, InputFormat::Csv, true).unwrap();

        assert_eq!(
            table.headers,
            Some(vec!["name".to_string(), "bio".to_string()])
        );
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0], vec!["Alice", "likes cats, dogs"]);
        assert_eq!(table.rows[1], vec!["Bob", "line1\nline2"]);
    }

    #[test]
    fn csv_ragged_rows() {
        // csv crate pads short rows and allows long rows by default
        let input = "a,b,c\n1,2\n3,4,5,6";
        let table = parse_string(input, InputFormat::Csv, true).unwrap();

        assert_eq!(
            table.headers,
            Some(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
        // Short row: missing fields become empty
        assert_eq!(table.rows[0].len(), 2);
        // Long row: extra fields are included
        assert_eq!(table.rows[1].len(), 4);
    }

    // --- JSON tests ---

    #[test]
    fn json_array_of_objects() {
        let input = r#"[{"name":"Alice","age":30},{"name":"Bob","age":25}]"#;
        let table = parse_string(input, InputFormat::Json, false).unwrap();

        assert_eq!(
            table.headers,
            Some(vec!["name".to_string(), "age".to_string()])
        );
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0], vec!["Alice", "30"]);
        assert_eq!(table.rows[1], vec!["Bob", "25"]);
    }

    #[test]
    fn json_array_of_arrays() {
        let input = r#"[["Alice",30],["Bob",25]]"#;
        let table = parse_string(input, InputFormat::Json, false).unwrap();

        assert_eq!(table.headers, None);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0], vec!["Alice", "30"]);
        assert_eq!(table.rows[1], vec!["Bob", "25"]);
    }

    #[test]
    fn json_empty_array() {
        let input = "[]";
        let table = parse_string(input, InputFormat::Json, false).unwrap();

        assert_eq!(table.headers, None);
        assert_eq!(table.rows.len(), 0);
    }

    #[test]
    fn json_nested_values_stringified() {
        let input = r#"[{"name":"Alice","meta":{"x":1}},{"name":"Bob","meta":[1,2]}]"#;
        let table = parse_string(input, InputFormat::Json, false).unwrap();

        assert_eq!(
            table.headers,
            Some(vec!["name".to_string(), "meta".to_string()])
        );
        assert_eq!(table.rows[0], vec!["Alice", r#"{"x":1}"#]);
        assert_eq!(table.rows[1], vec!["Bob", "[1,2]"]);
    }

    #[test]
    fn json_invalid_input() {
        let input = "not valid json";
        let result = parse_string(input, InputFormat::Json, false);
        assert!(result.is_err());
    }

    #[test]
    fn json_not_array() {
        let input = r#"{"key":"value"}"#;
        let result = parse_string(input, InputFormat::Json, false);
        assert!(result.is_err());
    }

    #[test]
    fn json_null_values() {
        let input = r#"[{"name":"Alice","age":null},{"name":"Bob","age":25}]"#;
        let table = parse_string(input, InputFormat::Json, false).unwrap();

        assert_eq!(table.rows[0], vec!["Alice", ""]);
        assert_eq!(table.rows[1], vec!["Bob", "25"]);
    }

    #[test]
    fn json_objects_with_different_keys() {
        let input = r#"[{"a":1,"b":2},{"b":3,"c":4}]"#;
        let table = parse_string(input, InputFormat::Json, false).unwrap();

        assert_eq!(
            table.headers,
            Some(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
        assert_eq!(table.rows[0], vec!["1", "2", ""]);
        assert_eq!(table.rows[1], vec!["", "3", "4"]);
    }
}
