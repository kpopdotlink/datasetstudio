# Dataset Studio

A local-first desktop app for creating text datasets for LLM training.

Split long texts into "overlapping chunks", review/edit/approve them, and export to JSONL datasets ready for training pipelines.

## Key Features

- **Source Management**: Register folders/files or directly input text
- **Chunk Creation**: Deterministic chunk splitting with sliding window + overlap
- **Review Workflow**: Keyboard-centric fast review/edit/approve
- **JSONL Export**: Various preset support (Text-only, Prompt/Completion, Instruction/Context/Response)
- **Reproducibility**: Same input + settings = Same output guaranteed

## Tech Stack

| Category | Technology |
|----------|------------|
| App Framework | Tauri 2.x |
| Frontend | React + Vite + TypeScript |
| UI | Tailwind CSS + Radix UI |
| State Management | Zustand |
| Backend | Rust |
| Database | SQLite (rusqlite) |

## System Requirements

- **macOS**: 11.0 (Big Sur) or later
- **Windows**: 10 or later
- **Node.js**: 18.0 or later
- **Rust**: 1.70 or later
- **pnpm**: 8.0 or later

## Installation & Build

### Prerequisites

```bash
# Install Rust (https://rustup.rs/)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js and pnpm
# macOS (Homebrew)
brew install node pnpm

# Windows (winget)
winget install OpenJS.NodeJS.LTS
npm install -g pnpm
```

### Development Environment

```bash
# Clone repository
git clone https://github.com/your-org/datasetstudio.git
cd datasetstudio

# Install dependencies
pnpm install

# Run development server
pnpm tauri dev
```

### Production Build

```bash
# macOS (.app, .dmg)
pnpm tauri build

# Windows (.msi)
pnpm tauri build
```

Build outputs are generated in the `src-tauri/target/release/bundle/` directory.

## Project Structure

```
datasetstudio/
├── src/                     # React frontend
│   ├── components/          # UI components
│   ├── stores/              # Zustand state management
│   ├── api/                 # Tauri command calls
│   └── types/               # TypeScript type definitions
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── commands/        # Tauri commands
│   │   ├── core/            # Core logic (chunker, tokenizer, etc.)
│   │   ├── db/              # Database (schema, migrations)
│   │   └── jobs/            # Background jobs
│   └── migrations/          # SQL migration files
├── fixtures/                # Test sample data
└── docs/                    # Documentation
```

## Usage

### 1. Create Project

Launch the app and click "New Project" button to create a project.

### 2. Register Sources

- **Folder**: Drag and drop or select a folder containing text files
- **Direct Input**: Add documents by directly entering text

### 3. Chunk Settings

- `max_len`: Maximum chunk length (tokens or characters)
- `overlap_len`: Overlap length
- Boundary rules: Options to preserve paragraph/sentence boundaries

### 4. Review

Quick review with keyboard shortcuts:

| Key | Action |
|-----|--------|
| `Enter` | Approve |
| `E` | Edit mode |
| `S` | Skip |
| `B` | Previous chunk |
| `N` | Next chunk |
| `Cmd/Ctrl + Z` | Undo |

### 5. Export

Export approved chunks to JSONL format:

- **Text-only**: `{"text": "..."}`
- **Prompt/Completion**: `{"prompt": "...", "completion": "..."}`
- **Instruction**: `{"instruction": "...", "context": "...", "response": "..."}`

## Development

### Code Quality Checks

```bash
# TypeScript type check
pnpm run tsc --noEmit

# ESLint
pnpm run lint

# Rust format
cd src-tauri && cargo fmt

# Rust Clippy
cd src-tauri && cargo clippy
```

### Tests

```bash
# Rust tests
cd src-tauri && cargo test

# Frontend tests (to be added)
pnpm test
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT License - See [LICENSE](LICENSE)
