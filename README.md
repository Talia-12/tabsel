# tabsel

A dmenu-like table selector for Wayland. Reads tabular data from stdin (CSV or JSON), displays it as an interactive table, and outputs the selection to stdout.

Built with [iced](https://github.com/hecrj/iced/).

## Install

### From source

```bash
git clone https://github.com/Talia-12/tabsel.git
cd tabsel
cargo build --release --locked
sudo cp target/release/tabsel /usr/bin/tabsel
```

### Nix

```bash
nix run github:Talia-12/tabsel
```

Or add to your flake inputs and use the home-manager module (see below).

## Usage

Pipe tabular data into tabsel:

```bash
# CSV with headers (default)
echo -e "name,age,city\nAlice,30,NYC\nBob,25,LA\nCarol,35,Chicago" | tabsel

# JSON array of objects
echo '[{"name":"Alice","age":30},{"name":"Bob","age":25}]' | tabsel --format json

# CSV without headers
echo -e "Alice,30\nBob,25" | tabsel --header false
```

### Key bindings

| Key              | Action                                  |
|:-----------------|:----------------------------------------|
| Arrow Up/Down    | Move row selection                      |
| Arrow Left/Right | Move column selection (column/cell mode)|
| Enter            | Confirm selection, output to stdout     |
| Escape           | Cancel (exit code 1)                    |
| Shift+Tab        | Cycle selection mode                    |
| Type text        | Filter rows (when filter bar is enabled)|

### CLI reference

```
tabsel [OPTIONS]

Options:
  -f, --format <FORMAT>              Input format: csv or json [default: csv]
      --header <HEADER>              Whether the CSV input has a header row [default: true]
  -m, --mode <MODE>                  Selection mode: row, column, cell.
                                     Repeat for multiple [default: row]
  -o, --output-format <FORMAT>       Output format: plain, json, csv [default: plain]
      --no-filter                    Disable the filter bar
  -t, --theme <PATH>                 Path to an alternate theme file
  -s, --scale <SCALE>                Scale factor for the theme
  -h, --help                         Print help
```

### Selection modes

Use `--mode` to control what gets selected and output:

```bash
# Row mode (default): outputs the entire selected row
echo -e "name,age\nAlice,30" | tabsel --mode row
# Output: Alice,30

# Cell mode: outputs a single cell value
echo -e "name,age\nAlice,30" | tabsel --mode cell
# Output: Alice

# Column mode: outputs the column header name (or index)
echo -e "name,age\nAlice,30" | tabsel --mode column
# Output: name

# Multiple modes: Shift+Tab cycles between them
echo -e "name,age\nAlice,30" | tabsel --mode row --mode cell
```

### Output formats

```bash
# Plain (default): comma-separated for rows, raw value for cells
echo -e "name,age\nAlice,30" | tabsel

# JSON: structured output
echo -e "name,age\nAlice,30" | tabsel --output-format json
# Row output: {"name":"Alice","age":"30"}
# Cell output: {"value":"Alice","row":0,"column":"name"}

# CSV: properly quoted CSV
echo -e "name,age\nAlice,30" | tabsel --output-format csv
```

### Exit codes

- **0**: Selection confirmed (output written to stdout)
- **1**: Cancelled (Escape), empty input, or error

## Theming

Tabsel looks for a theme file at `$XDG_CONFIG_HOME/tabsel/theme.scss` (typically `~/.config/tabsel/theme.scss`). Use `--theme` to specify an alternate file.

See [docs/examples/](docs/examples/) for example themes.

### Theme structure

```scss
.tabsel {
  // Window properties
  font-size: 16px;
  width: 600px;
  height: 400px;
  background: #1e1e2e;
  color: #cdd6f4;
  border-color: #585b70;
  border-width: 2px;
  border-radius: 8%;
  padding: 8px;
  --font-family: "monospace";
  --exit-unfocused: false;

  .container {
    background: #181825;
    padding: 4px;

    .search {
      // Filter bar container
      background: #313244;
      padding: 6px;
      --height: 40px;

      .input {
        // Text input inside filter bar
        background: #45475a;
        color: #cdd6f4;
        --placeholder-color: #6c7086;
        --selection-color: #89b4fa;
        font-size: 15px;
      }
    }

    .rows {
      // Table rows container
      --column-spacing: 12px;

      .header {
        // Header row
        background: #313244;
        color: #89b4fa;
        font-size: 15px;
        --separator-color: #585b70;
        --separator-width: 2px;
      }

      .row {
        // Default (unselected) data row
        background: #1e1e2e;
        color: #cdd6f4;

        .title { font-size: 14px; }
      }

      .row-selected {
        // Selected data row
        background: #45475a;
        color: #f5e0dc;
        border-color: #89b4fa;
        border-width: 1px;

        .title { color: #f5e0dc; font-size: 14px; }
      }
    }

    .scrollable {
      .scroller {
        color: #585b70;
        width: 6px;
      }
    }
  }
}
```

## Home-Manager module

Add tabsel to your flake inputs, then use the home-manager module:

```nix
{
  inputs.tabsel.url = "github:Talia-12/tabsel";

  # In your home-manager configuration:
  imports = [ inputs.tabsel.homeManagerModules.default ];

  programs.tabsel = {
    enable = true;
    style = ''
      .tabsel {
        background: #1e1e2e;
        color: #cdd6f4;
        // ...
      }
    '';
  };
}
```

## License

MIT
