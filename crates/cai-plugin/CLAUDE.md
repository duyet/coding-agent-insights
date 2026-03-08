# cai-plugin - Claude Code Instructions

## Crate Purpose

`cai-plugin` provides the Claude Code plugin interface for CAI:

- Plugin trait for skill/command registration
- C FFI exports for WASM compatibility
- Session hooks (start/end)
- Built-in CaiPlugin implementation

## Architecture

### Module Structure

```
src/
└── lib.rs  - Plugin trait, CaiPlugin, FFI exports
```

### Plugin Trait

```rust
pub trait Plugin {
    fn config(&self) -> &PluginConfig;
    fn handle_skill(&mut self, skill: &str, params: &Value) -> Result<String>;
    fn handle_command(&mut self, cmd: &str, args: &[String]) -> Result<String>;
    fn on_session_start(&mut self, ctx: &SessionContext) -> Result<()>;
    fn on_session_end(&mut self, ctx: &SessionContext) -> Result<()>;
}
```

### PluginConfig

```rust
pub struct PluginConfig {
    pub version: String,
    pub skills: Vec<String>,     // Registered skill names
    pub commands: Vec<String>,   // Registered command names
}
```

### Default Skills

- `cai.query` - Execute SQL queries
- `cai.ingest` - Ingest data from sources
- `cai.stats` - Get statistics
- `cai.tui` - Launch terminal UI
- `cai.web` - Launch web dashboard

## Common Tasks

### Adding a New Skill

1. Add to `PluginConfig::default()`:
```rust
impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            skills: vec![
                // ... existing skills
                "cai.myskill".into(),
            ],
            // ...
        }
    }
}
```

2. Add handler in `CaiPlugin::handle_skill()`:
```rust
fn handle_skill(&mut self, skill: &str, params: &serde_json::Value) -> Result<String> {
    match skill {
        // ... existing skills
        "cai.myskill" => self.handle_myskill(params),
        _ => Err(Error::Message(format!("Unknown skill: {}", skill))),
    }
}
```

3. Implement handler:
```rust
fn handle_myskill(&mut self, params: &serde_json::Value) -> Result<String> {
    let param = params["my_param"]
        .as_str()
        .ok_or_else(|| Error::Message("Missing my_param".into()))?;
    Ok(format!("My skill executed with: {}", param))
}
```

### Adding Session Hook Logic

Edit hook methods in `CaiPlugin`:

```rust
fn on_session_start(&mut self, ctx: &SessionContext) -> Result<()> {
    // Initialize storage
    // Load configuration
    // Set up analytics
    tracing::info!("Session started: {}", ctx.session_id);
    Ok(())
}

fn on_session_end(&mut self, ctx: &SessionContext) -> Result<()> {
    // Persist analytics
    // Clean up resources
    // Save state
    tracing::info!("Session ended: {}", ctx.session_id);
    Ok(())
}
```

### Creating a Custom Plugin

Implement the `Plugin` trait:

```rust
pub struct MyPlugin {
    config: PluginConfig,
    // Add your fields
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            config: PluginConfig {
                version: "0.1.0".to_string(),
                skills: vec!["my.skill".into()],
                commands: vec!["my-cmd".into()],
            },
        }
    }
}

impl Plugin for MyPlugin {
    fn config(&self) -> &PluginConfig {
        &self.config
    }

    fn handle_skill(&mut self, skill: &str, params: &Value) -> Result<String> {
        match skill {
            "my.skill" => Ok("Custom skill response".to_string()),
            _ => Err(Error::Message("Unknown skill".to_string())),
        }
    }

    fn handle_command(&mut self, cmd: &str, args: &[String]) -> Result<String> {
        match cmd {
            "my-cmd" => Ok(format!("Custom command: {:?}", args)),
            _ => Err(Error::Message("Unknown command".to_string())),
        }
    }

    fn on_session_start(&mut self, _ctx: &SessionContext) -> Result<()> {
        Ok(())
    }

    fn on_session_end(&mut self, _ctx: &SessionContext) -> Result<()> {
        Ok(())
    }
}
```

## FFI Exports

The crate exports C functions for WASM compatibility:

```rust
#[no_mangle]
pub extern "C" fn cai_plugin_create() -> *mut CaiPlugin;

#[no_mangle]
pub extern "C" fn cai_plugin_destroy(ptr: *mut CaiPlugin);
```

These allow the plugin to be loaded from C/C++ code or WASM runtimes.

## Session Context

```rust
pub struct SessionContext {
    pub session_id: String,
    pub work_dir: String,
    pub env: HashMap<String, String>,
}
```

Use context in hooks to access:
- Session identifier
- Working directory
- Environment variables

## Testing Patterns

### Test Plugin Configuration

```rust
#[test]
fn test_plugin_config() {
    let plugin = CaiPlugin::new();
    assert_eq!(plugin.config().skills.len(), 5);
    assert!(plugin.config().skills.contains(&"cai.query".to_string()));
}
```

### Test Skill Handling

```rust
#[test]
fn test_handle_query_skill() {
    let mut plugin = CaiPlugin::new();
    let params = json!({"sql": "SELECT * FROM entries"});
    let result = plugin.handle_skill("cai.query", &params);
    assert!(result.is_ok());
}
```

## Skill Definition Format

Skills are defined in the plugin as JSON parameters:

```json
{
  "sql": "SELECT * FROM entries LIMIT 10",
  "output": "table"
}
```

Each skill handler extracts its required parameters from the JSON value.

## Dependencies

- `serde_json` - JSON parameter parsing
- `cai-core` - Error types, Result

## Integration Points

The plugin integrates with:

1. **Claude Code** - Via FFI exports
2. **CLI commands** - Via `handle_command()`
3. **Skills system** - Via `handle_skill()`

## Getting Help

- See `README.md` for skill definitions
- Check `lib.rs` for plugin interface
- Review tests for implementation patterns
- Look at `plugin/skills/` for skill documentation
