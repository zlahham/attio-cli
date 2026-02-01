# attio-cli

A command-line interface for the Attio CRM API built with Rust.

> **⚠️ Work In Progress**: This CLI is actively being developed. Currently, only the **Notes** endpoints are implemented. Additional endpoints for other Attio resources (companies, people, lists, etc.) are being added.

## Features

- 🔐 Simple authentication with API tokens
- 📝 Full note management (list, view, create, delete)
- 🎨 Interactive TUI mode for browsing notes
- 🌐 Open notes directly in your browser
- 📊 Clean table-formatted output

## Installation

### From Source
```bash
git clone https://github.com/zlahham/attio-cli.git
cd attio-cli
cargo build --release
```

The binary will be available at `target/release/attio`.

### From Binary
[TODO: Add instructions once releases are published]

## Authentication

Set your Attio API token using one of these methods:

1. **Config file** (recommended, persistent):
   ```bash
   attio auth <your-token>
   ```

2. **Environment variable** (temporary):
   ```bash
   export ATTIO_API_TOKEN=your_token_here
   ```

Get your API token from: https://app.attio.com/[worspace-slug]/settings/developers/access-tokens.

**Token precedence**: Config file → Environment variable

## Usage

### Authentication

```bash
attio auth <token>
```

Saves your Attio API token to the config file for persistent authentication.

**Arguments:**
- `<token>` - Your Attio API token

---

### Notes Commands

#### List Notes

```bash
# Interactive TUI mode (default)
attio notes list

# Plain text table mode
attio notes list --plain
```

Lists all notes in your workspace. By default, launches an interactive terminal UI for browsing notes. Use `--plain` for a simple table output.

**Flags:**
- `--plain` - Display notes in a non-interactive table format

---

#### Get a Note

```bash
# Display note details
attio notes get <note-id>

# Display and open in browser
attio notes get <note-id> --open-in-browser
```

Retrieves and displays details for a specific note.

**Arguments:**
- `<note-id>` - The ID of the note to retrieve

**Flags:**
- `--open-in-browser` - Open the note in your default browser after displaying it

---

#### Create a Note

```bash
attio notes create \
  --parent-object <object> \
  --parent-record-id <record-id> \
  --title <title> \
  --content <content> \
  [--format <format>] \
  [--open-in-browser]
```

Creates a new note attached to a parent record.

**Required Flags:**
- `--parent-object <object>` - The object type the note belongs to (e.g., "people", "companies")
- `--parent-record-id <record-id>` - The ID of the record to attach the note to
- `--title <title>` - The title of the note
- `--content <content>` - The content/body of the note

**Optional Flags:**
- `--format <format>` - Content format: "plaintext" or "markdown" (default: "plaintext")
- `--open-in-browser` - Open the created note in your default browser

**Example:**
```bash
attio notes create \
  --parent-object people \
  --parent-record-id 12345678-1234-1234-1234-123456789abc \
  --title "Follow-up meeting" \
  --content "Discussed Q1 goals and next steps" \
  --format plaintext
```

---

#### Delete a Note

```bash
attio notes delete <note-id>
```

Deletes a note by ID.

**Arguments:**
- `<note-id>` - The ID of the note to delete

---

## Configuration

Configuration is stored at:
- **Linux**: `~/.config/attio/config.json`
- **macOS**: `~/Library/Application Support/attio/config.json`
- **Windows**: `%APPDATA%\attio\config.json`

## Development

### Prerequisites
- Rust 1.70+ (or latest stable)
- Cargo

### Building
```bash
cargo build
```

### Running Tests
```bash
cargo test
```

### Running Locally
```bash
cargo run -- <command>

# Examples:
cargo run -- auth <token>
cargo run -- notes list
cargo run -- notes get <note-id>
```

### Docker Development
```bash
# Using docker-compose
docker-compose run app cargo build
docker-compose run app cargo run -- <command>
```

## Roadmap

- [x] Notes endpoints (list, get, create, delete)
- [ ] Companies endpoints
- [ ] People endpoints
- [ ] Lists endpoints
- [ ] Records endpoints
- [ ] Webhooks endpoints
- [ ] Additional filtering and pagination options

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

Built for use with the [Attio API](https://developers.attio.com/).
