# cai-core - Claude Code Instructions

## Crate Purpose

`cai-core` is the foundation of the CAI workspace. It provides:
- Core domain types (`Entry`, `Source`, `Metadata`)
- Shared error types (`Error`, `Result`)
- Common utilities used across all crates

## Design Principles

### Keep It Minimal

This crate should remain **lean and focused**. Only include types that are:
1. Used by 2+ crates in the workspace
2. Fundamental to the domain model
3. Unlikely to change frequently

### Use Non-Exhaustive Enums

All public enums should be `#[non_exhaustive]` to allow adding variants without breaking changes:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum Source {
    Claude,
    Codex,
    Git,
    // Future variants can be added without breaking
}
```

### Serde Compatibility

All public structs must derive `Serialize` and `Deserialize`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Entry {
    pub id: String,
    // ...
}
```

## Working with Entries

### Creating Entries

```rust
use cai_core::{Entry, Source, Metadata};
use chrono::Utc;

// Minimal entry
let entry = Entry {
    id: uuid::Uuid::new_v4().to_string(),
    source: Source::Claude,
    timestamp: Utc::now(),
    prompt: "help me refactor".to_string(),
    response: "here's how".to_string(),
    metadata: Metadata::default(),
};

// Entry with metadata
let mut metadata = Metadata::default();
metadata.file_path = Some("src/main.rs".to_string());
metadata.language = Some("Rust".to_string());

let entry = Entry {
    id: uuid::Uuid::new_v4().to_string(),
    source: Source::Git,
    timestamp: Utc::now(),
    prompt: "commit message".to_string(),
    response: "diff content".to_string(),
    metadata,
};
```

### Serializing Entries

```rust
use cai_core::Entry;
use serde_json;

let entry: Entry = /* ... */;
let json = serde_json::to_string(&entry)?;
let pretty = serde_json::to_string_pretty(&entry)?;
```

## Error Handling

### Creating Errors

```rust
use cai_core::{Error, Result};

// Use Io for I/O errors
fn read_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(Error::from)
}

// Use Json for parsing errors
fn parse_json(s: &str) -> Result<Entry> {
    serde_json::from_str(s).map_err(Error::from)
}

// Use Message for custom errors
fn validate(entry: &Entry) -> Result<()> {
    if entry.prompt.is_empty() {
        return Err(Error::Message("prompt cannot be empty".to_string()));
    }
    Ok(())
}
```

### Propagating Errors

```rust
use cai_core::Result;

async fn process_entry(entry: Entry) -> Result<String> {
    validate(&entry)?;
    let json = serde_json::to_string(&entry)?;
    Ok(json)
}
```

## Adding New Types

When adding new types to `cai-core`:

1. **Check necessity** - Is this used by multiple crates?
2. **Document thoroughly** - Include examples in doc comments
3. **Add tests** - Unit tests for serialization/deserialization
4. **Consider compatibility** - Use `#[non_exhaustive]` for enums

Example of adding a new type:

```rust
/// Represents the type of coding interaction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InteractionType {
    /// Question-answer interaction
    Qa,
    /// Code generation request
    Generation,
    /// Refactoring request
    Refactor,
    /// Other interaction type
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interaction_type_serialization() {
        let t = InteractionType::Qa;
        let json = serde_json::to_string(&t).unwrap();
        let deserialized: InteractionType = serde_json::from_str(&json).unwrap();
        assert_eq!(t, deserialized);
    }
}
```

## Testing Guidelines

### Unit Tests

Write unit tests for:
- Serialization/deserialization roundtrips
- Default implementations
- Edge cases (empty strings, None values)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entry_serialization() {
        let entry = Entry {
            id: "test".to_string(),
            source: Source::Claude,
            timestamp: Utc::now(),
            prompt: "test".to_string(),
            response: "response".to_string(),
            metadata: Metadata::default(),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: Entry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_metadata_default() {
        let metadata = Metadata::default();
        assert!(metadata.file_path.is_none());
        assert!(metadata.extra.is_empty());
    }
}
```

### Property-Based Tests

Use `proptest` for testing invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_entry_roundtrip(
        id in "[a-z0-9-]+",
        prompt in ".*",
        response in ".*"
    ) {
        let entry = Entry {
            id,
            source: Source::Claude,
            timestamp: Utc::now(),
            prompt,
            response,
            metadata: Metadata::default(),
        };

        let json = serde_json::to_string(&entry)?;
        let deserialized: Entry = serde_json::from_str(&json)?;
        prop_assert_eq!(entry, deserialized);
    }
}
```

## Common Patterns

### Builder Pattern

For complex types, consider a builder:

```rust
impl Entry {
    pub fn builder() -> EntryBuilder {
        EntryBuilder::default()
    }
}

pub struct EntryBuilder {
    id: Option<String>,
    source: Option<Source>,
    // ...
}

impl EntryBuilder {
    pub fn id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    pub fn source(mut self, source: Source) -> Self {
        self.source = Some(source);
        self
    }

    pub fn build(self) -> Result<Entry> {
        Ok(Entry {
            id: self.id.ok_or_else(|| Error::Message("id required".to_string()))?,
            source: self.source.ok_or_else(|| Error::Message("source required".to_string()))?,
            // ...
        })
    }
}
```

### Validation

Add validation methods:

```rust
impl Entry {
    pub fn validate(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(Error::Message("id cannot be empty".to_string()));
        }
        if self.prompt.is_empty() && self.response.is_empty() {
            return Err(Error::Message("prompt or response required".to_string()));
        }
        Ok(())
    }
}
```

## Release Notes

When making changes to `cai-core`:

1. **Bump version** - This affects all dependent crates
2. **Update CHANGELOG** - Document breaking changes
3. **Notify dependents** - Other crates may need updates
4. **Run full test suite** - Ensure workspace compatibility

## Dependencies

`cai-core` has minimal dependencies:
- `serde` - Serialization
- `serde_json` - JSON support
- `chrono` - DateTime handling
- `thiserror` - Error derives

Avoid adding new dependencies unless absolutely necessary.

## Getting Help

- See crate `README.md` for usage examples
- Check lib.rs doc comments for API details
- Review tests in `src/lib.rs` for patterns
