# attio-cli

A command-line interface for the Attio CRM API built with Rust.

> **⚠️ Work In Progress**: This CLI is actively being developed. Currently, only the **Notes** endpoints are implemented. Additional endpoints for other Attio resources (companies, people, lists, etc.) are being added.

## Features

- 🔐 Simple authentication with API tokens
- 📝 Full note management (list, view, create, delete)
- 🎨 Interactive TUI mode for browsing notes
  - 🔍 Real-time search across cached notes (press `/`)
  - 💾 Smart caching with configurable memory limits
  - ⚡ Fetch all notes with Ctrl+A for comprehensive search
  - 🎨 Color-coded cache usage indicators
- 🌐 Open notes directly in your browser
- 📊 Clean table-formatted output
- ⚙️ Configurable settings (cache limits, etc.)

## Installation

### Quick Install (macOS)

**Recommended:** Use the install script for a seamless experience:

```bash
curl -fsSL https://raw.githubusercontent.com/zlahham/attio-cli/main/install.sh | sh
```

This script will:
- ✅ Detect your Mac architecture (Intel/Apple Silicon)
- ✅ Download the latest release
- ✅ Remove macOS quarantine flags (no security warnings!)
- ✅ Install to `/usr/local/bin`

### From Binary

Download the latest release for your platform from the [releases page](https://github.com/zlahham/attio-cli/releases).

**Available platforms:**
- Linux (x86_64) - glibc and musl variants
- macOS (x86_64 and ARM64)
- Windows (x86_64)

**macOS Installation:**
```bash
# Download the appropriate binary for your Mac:
# - attio-macos-arm64 (Apple Silicon M1/M2/M3)
# - attio-macos-amd64 (Intel)

# Remove quarantine flag and install
xattr -d com.apple.quarantine attio-macos-*
chmod +x attio-macos-*
sudo mv attio-macos-* /usr/local/bin/attio
```

**Linux Installation:**
```bash
chmod +x attio-linux-*
sudo mv attio-linux-* /usr/local/bin/attio
```

### From Source
```bash
git clone https://github.com/zlahham/attio-cli.git
cd attio-cli
cargo build --release
```

The binary will be available at `target/release/attio`.

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

### Configuration Commands

#### Set Configuration

```bash
attio config set <key> <value>
```

Set a configuration value.

**Available keys:**
- `cache-limit-mb` - Maximum cache size in megabytes (default: 50)

**Example:**
```bash
attio config set cache-limit-mb 100
```

---

#### Get Configuration

```bash
attio config get <key>
```

Get the current value of a configuration setting.

**Example:**
```bash
attio config get cache-limit-mb
```

---

#### List Configuration

```bash
attio config list
```

Display all current configuration settings.

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

**Interactive TUI Controls:**
- `←/→` - Navigate between pages
- `/` - Enter search mode
  - Type to search across all cached notes by title/content
  - `Backspace` to delete characters
  - `Esc` to exit search
- `Ctrl+A` - Fetch all notes into cache for comprehensive searching
- `Q` or `Esc` - Quit

**Features:**
- Smart caching: Notes are cached as you browse to improve search performance
- Memory management: Visual indicator shows cache usage with color coding (green/yellow/red)
- Search pagination: Navigate through search results with arrow keys
- Configurable cache limit (see `attio config set cache-limit-mb`)

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

**Configuration file format:**
```json
{
  "token": "your_api_token_here",
  "cache_limit_mb": 50
}
```

**Available settings:**
- `token` - Your Attio API token (set via `attio auth <token>`)
- `cache_limit_mb` - Maximum cache size in MB (set via `attio config set cache-limit-mb <value>`, default: 50)

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

### Creating Releases

Releases are automated via GitHub Actions. To create a new release:

1. Update the version in `Cargo.toml`
2. Commit the version change
3. Create and push a version tag:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

This will automatically:
- Build binaries for Linux (x64), macOS (x64, ARM), and Windows (x64)
- Create a GitHub release with all binaries attached
- Optionally publish to crates.io (requires `CARGO_TOKEN` secret)

## Roadmap

- [x] Notes endpoints (list, get, create, delete)
- [x] Interactive TUI with search and caching
- [x] Configuration management
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
