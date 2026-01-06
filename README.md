# Tsundoku

A Japanese web novel downloader and translator supporting Syosetu, Kakuyomu, and Pixiv platforms.

## Features

- Downloads web novels from multiple Japanese platforms:
  - Syosetu (ncode.syosetu.com, novel18.syosetu.com)
  - Kakuyomu (kakuyomu.jp)
  - Pixiv (pixiv.net/novel)
- Automatic character name extraction and mapping using LLM
- Translation using OpenAI-compatible APIs
- Persistent name mapping with vote-based consensus
- Incremental progress saving and resume capability
- Streaming translation with real-time progress display

## Installation

### From AUR (Arch Linux)

```bash
yay -S tsundoku
# or
paru -S tsundoku
```

### From Source

Requires Rust 1.80+ (edition 2024 support)

```bash
git clone https://github.com/ripdog/Tsundoku.git
cd Tsundoku
cargo build --release
cargo install --path .
```

## Configuration

On first run, Tsundoku will create a default configuration file:

- Linux: `~/.config/Tsundoku/config.toml`
- macOS: `~/Library/Application Support/Tsundoku/config.toml`
- Windows: `%APPDATA%\Tsundoku\config.toml`

Edit the configuration file to add your OpenAI-compatible API key and customize settings.

### Required Configuration

At minimum, you must set your API key:

```toml
[api]
key = "your_api_key_here"
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
```

### Optional Configuration

#### Separate Scout API

You can use a different (cheaper) model for character name extraction:

```toml
[scout_api]
key = "your_scout_api_key"
base_url = "https://api.openai.com/v1"
model = "gpt-4o-mini"
```

#### Editor for Name Review

Specify which editor to use when reviewing name mappings:

```toml
[paths]
editor_command = "kate"  # or "vim", "nano", "code", "notepad", etc.
```

If not specified, Tsundoku will auto-detect a suitable editor based on your platform.

## Usage

Download and translate a novel:

```bash
tsundoku https://ncode.syosetu.com/n1234ab/
```

### Options

- `--start N`: Start downloading from chapter N (1-based)
- `--end N`: Stop downloading at chapter N (1-based, inclusive)
- `--no-name-pause`: Skip manual name mapping review pause

### Examples

Download chapters 5-10 only:

```bash
tsundoku --start 5 --end 10 https://ncode.syosetu.com/n1234ab/
```

Download without pausing for name review:

```bash
tsundoku --no-name-pause https://kakuyomu.jp/works/1234567890
```

## How It Works

1. **Download**: Scrapes the novel chapters from the source website
2. **Name Scout**: Extracts character names using LLM and builds a mapping
3. **Review** (optional): Pause to let you manually review/edit name mappings
4. **Translate**: Applies name mappings and translates content using LLM
5. **Save**: Stores translated chapters as text files

### Resume Capability

Tsundoku is fully resumable at every stage:

- **Already downloaded chapters are skipped** - Original text files are checked before downloading
- **Already translated chapters are skipped** - Translated files are checked before translation
- **Name scouting tracks coverage** - Chapters that have been scanned for names won't be scanned again
- **Progress is saved incrementally** - Name mappings are saved after each successful API call

This means you can:
- Stop and restart the program at any time
- Re-run the same command to continue where you left off
- Add new chapters to an existing series by running with a wider `--end` range
- Retry failed translations without redoing successful ones

Simply run the same command again, and Tsundoku will intelligently skip completed work.

### Output Structure

Multi-chapter novels:
```
[syosetu: n1234ab] Novel Title/
├── Original/
│   ├── 001 - Chapter 1 Title.txt
│   ├── 002 - Chapter 2 Title.txt
│   └── ...
├── 001 - Chapter 1 Title.txt
├── 002 - Chapter 2 Title.txt
└── ...
```

One-shot stories:
```
[pixiv: 12345678] Story Title/
├── original.txt
└── oneshot.txt
```

## Name Mapping System

Tsundoku automatically extracts character names and builds a persistent mapping database. The system uses a voting mechanism to determine the best English rendering of each Japanese name, with mappings stored in:

- Linux: `~/.local/share/Tsundoku/names/`
- macOS: `~/Library/Application Support/Tsundoku/names/`
- Windows: `%APPDATA%\Tsundoku\names\`

You can manually edit the JSON files to correct name mappings. The format is:

```json
{
  "names": {
    "太郎": {
      "part": "given",
      "votes": {
        "Taro": 5,
        "Tarou": 2
      },
      "english": "Taro",
      "count": 5
    }
  },
  "coverage": [1, 2, 3]
}
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Running

```bash
cargo run -- https://ncode.syosetu.com/n1234ab/
```

## License

Licensed under the GNU General Public License v3.0 or later. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

Inspired by the original Python-based syosetu_grabber project.
