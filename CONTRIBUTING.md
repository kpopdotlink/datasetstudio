# Contributing to Dataset Studio

Thank you for contributing to Dataset Studio!

## Development Environment Setup

### Required Tools

- **Node.js** 18.0 or later
- **pnpm** 8.0 or later
- **Rust** 1.70 or later (stable)
- **Visual Studio Code** (recommended)

### Installation

```bash
# Clone repository
git clone https://github.com/your-org/datasetstudio.git
cd datasetstudio

# Install dependencies
pnpm install

# Run development server
pnpm tauri dev
```

## Coding Style

### TypeScript/React

- Follow ESLint rules
- Use functional components
- Type definitions required

```bash
# Lint check
pnpm run lint

# Type check
pnpm run tsc --noEmit
```

### Rust

- Follow `cargo fmt` formatting
- No `cargo clippy` warnings

```bash
cd src-tauri

# Format
cargo fmt

# Clippy
cargo clippy -- -D warnings

# Test
cargo test
```

## Commit Convention

Follow Conventional Commits:

```
feat: Add new feature
fix: Bug fix
docs: Documentation changes
style: Code formatting (no functional changes)
refactor: Refactoring
test: Add/modify tests
chore: Build/tool configuration changes
```

### Commit Message Examples

```
feat: Implement chunk preview feature

- Add ChunkPreview component
- Implement preview_chunks Tauri command
- Provide real-time preview on parameter change
```

## Pull Request

### Pre-PR Checklist

- [ ] `pnpm run lint` passes
- [ ] `pnpm run tsc --noEmit` passes
- [ ] `cargo fmt` applied
- [ ] No `cargo clippy` warnings
- [ ] `cargo test` passes
- [ ] Related documentation updated

### Creating a PR

1. Create feature branch: `git checkout -b feat/my-feature`
2. Commit changes
3. Link related issue when creating PR: `Closes #123`

## Project Structure

```
datasetstudio/
├── src/                     # React frontend
│   ├── components/
│   │   ├── layout/          # Layout components
│   │   ├── sources/         # Source management
│   │   ├── chunks/          # Chunk settings
│   │   ├── review/          # Review workflow
│   │   └── export/          # Export functionality
│   ├── stores/              # Zustand stores
│   ├── api/                 # Tauri command wrappers
│   └── types/               # TypeScript types
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── commands/        # Tauri command handlers
│   │   ├── core/            # Core business logic
│   │   │   ├── chunker.rs   # Chunk generation engine
│   │   │   ├── normalize.rs # Text normalization
│   │   │   ├── tokenizer.rs # Token estimation
│   │   │   └── exporter.rs  # JSONL export
│   │   ├── db/              # Database
│   │   │   ├── models.rs    # Data models
│   │   │   ├── repository.rs # DB queries
│   │   │   └── migrations.rs # Migrations
│   │   └── jobs/            # Background jobs
│   └── migrations/          # SQL files
├── fixtures/                # Test data
└── docs/                    # Documentation
```

## Key Module Descriptions

### Chunk Engine (src-tauri/src/core/chunker.rs)

- Sliding window + overlap approach
- Deterministic output guaranteed
- Paragraph/sentence boundary preservation

### Database (src-tauri/src/db/)

- SQLite + rusqlite
- Migration-based schema management
- r2d2 connection pool

### Tauri Commands (src-tauri/src/commands/)

- Frontend-backend communication
- Async processing
- Error handling

## Testing

### Rust Tests

```bash
cd src-tauri
cargo test
```

### Test Writing Guide

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_determinism() {
        // Ensure same input produces same output
        let params = ChunkParams::default();
        let text = "Test text...";

        let chunks1 = chunk(text, &params);
        let chunks2 = chunk(text, &params);

        assert_eq!(chunks1, chunks2);
    }
}
```

## Documentation

- Update related documentation when changing code
- `docs/PRD.md`: Product requirements
- `docs/Architecture.md`: Technical architecture
- `README.md`: User guide

## Reporting Issues

When you find a bug:

1. Search existing issues
2. When creating a new issue, include:
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Environment (OS, app version)

## Questions

If you have questions, please reach out through GitHub Issues.

---

Thank you again for your contribution!
