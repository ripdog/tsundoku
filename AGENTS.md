# Tsundoku Development Guidelines

## Project Overview

Tsundoku is a Japanese web novel downloader and translator. It scrapes content from Syosetu, Kakuyomu, and Pixiv, extracts character names, and translates content using OpenAI-compatible APIs.

## Code Quality Principles

### Error Handling

- Use `thiserror` for defining custom error types in library code
- Use `anyhow` for application-level error handling in `main.rs`
- Prefer `Result<T, E>` over panics; reserve `unwrap()` for truly impossible cases
- Provide context with `.context()` or `.with_context()` from anyhow
- Create domain-specific error enums that map to user-friendly messages

```rust
// Good: Descriptive error with context
let content = fs::read_to_string(&path)
    .with_context(|| format!("Failed to read config file: {}", path.display()))?;

// Bad: Panic on error
let content = fs::read_to_string(&path).unwrap();
```

### Async Best Practices

- Use `tokio` as the async runtime
- Prefer `async fn` over manual `Future` implementations
- Use `tokio::time::sleep` for delays, never `std::thread::sleep` in async code
- Consider using `tokio::select!` for concurrent operations with cancellation
- Use `futures::stream` for processing items concurrently with backpressure

### Struct and Type Design

- Prefer strong typing over primitive obsession (e.g., `ChapterId(u32)` vs bare `u32`)
- Use the builder pattern for complex configuration structs
- Derive common traits: `Debug`, `Clone`, `PartialEq` where appropriate
- Use `#[non_exhaustive]` for public enums that may grow

```rust
// Good: Strong typing
pub struct NovelId(String);
pub struct ChapterNumber(u32);

// Bad: Stringly-typed
fn download(novel_id: String, chapter: u32) { }
```

### Module Organization

- Keep modules focused and cohesive
- Use `mod.rs` sparingly; prefer `module_name.rs` with `module_name/` subdirectory
- Public API should be minimal; use `pub(crate)` for internal visibility
- Re-export important types at crate root for convenience

```
src/
├── main.rs           # Entry point, CLI handling
├── lib.rs            # Library root, re-exports
├── config.rs         # Configuration types and loading
├── console.rs        # Terminal output utilities
├── error.rs          # Error types
├── translator.rs     # Translation logic
├── name_mapping.rs   # Name extraction and mapping
└── scrapers/
    ├── mod.rs        # Scraper trait and registry
    ├── syosetu.rs
    ├── kakuyomu.rs
    └── pixiv.rs
```

### Testing Strategy

- Write unit tests in `#[cfg(test)]` modules within source files
- Place integration tests in `tests/` directory
- Use `mockito` or `wiremock` for HTTP mocking
- Use `tempfile` for filesystem tests
- Test error paths, not just happy paths

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fetches_chapter_list() {
        // Arrange
        let server = MockServer::start().await;
        // ...
        
        // Act
        let result = scraper.get_chapters(&url).await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### Configuration

- Use `serde` for config deserialization
- Provide sensible defaults with `#[serde(default)]`
- Validate configuration at load time, not at use time
- Store config in platform-appropriate directories via `dirs` crate

### Performance Considerations

- Respect rate limits; make delays configurable
- Use streaming where possible (don't load entire responses into memory)
- Compile regex patterns once with `once_cell::Lazy` or `std::sync::LazyLock`
- Consider connection pooling (reqwest does this automatically)

### Documentation

- Document public APIs with `///` doc comments
- Include examples in doc comments where helpful
- Use `#![warn(missing_docs)]` to catch undocumented items
- Keep README.md updated with usage instructions

### Git Practices

- Write clear, concise commit messages
- Keep commits atomic and focused
- Don't commit secrets or API keys
- Use `.gitignore` appropriately

## Dependencies Policy

### Preferred Crates

| Purpose | Crate | Notes |
|---------|-------|-------|
| HTTP client | `reqwest` | With `cookies`, `json` features |
| HTML parsing | `scraper` | CSS selector support |
| Async runtime | `tokio` | Full features for CLI |
| Serialization | `serde`, `serde_json` | With derive |
| CLI parsing | `clap` | With derive feature |
| Regex | `regex` | Standard choice |
| Config dirs | `dirs` | Cross-platform |
| Error handling | `thiserror`, `anyhow` | Library/app split |
| Terminal colors | `console` or `crossterm` | TTY detection |
| Lazy statics | `once_cell` | Or std LazyLock when stable |

### Adding Dependencies

- Use `cargo add` to add dependencies
- Prefer well-maintained crates with recent updates
- Check license compatibility (MIT/Apache-2.0 preferred)
- Enable only needed features to reduce compile time

```bash
# Good: Specific features
cargo add tokio --features rt-multi-thread,macros,time

# Bad: Kitchen sink
cargo add tokio --features full
```

## Code Review Checklist

Before submitting code, verify:

- [ ] No `unwrap()` or `expect()` on fallible operations without justification
- [ ] Errors provide useful context for debugging
- [ ] Async code doesn't block the runtime
- [ ] Tests cover the new functionality
- [ ] Public APIs are documented
- [ ] No hardcoded secrets or paths
- [ ] Rate limiting is respected for network requests
- [ ] Resources (files, connections) are properly cleaned up

## Architecture Decisions

### Scraper Trait Design

Scrapers implement a common trait for polymorphism:

```rust
#[async_trait]
pub trait Scraper: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, url: &str) -> bool;
    async fn get_novel_info(&self, url: &str) -> Result<NovelInfo>;
    async fn get_chapter_list(&self, base_url: &str) -> Result<ChapterList>;
    async fn download_chapter(&self, chapter_url: &str) -> Result<String>;
}
```

### Configuration Layering

Configuration follows this precedence (highest to lowest):
1. Command-line arguments
2. Environment variables (for secrets)
3. Config file
4. Built-in defaults

### State Management

- Name mappings are persisted to JSON after each modification
- Chapter progress is tracked via filesystem (existence of translated files)
- No global mutable state; pass dependencies explicitly

## Common Pitfalls

1. **Blocking in async**: Don't use `std::fs` in async code; use `tokio::fs` or `spawn_blocking`
2. **Unbounded retries**: Always have a maximum retry count
3. **Silent failures**: Log warnings for recoverable errors
4. **Hardcoded delays**: Make rate-limiting configurable
5. **Platform assumptions**: Use `dirs` and `Path` for cross-platform compatibility
