//! CAI Plugin - Claude Code plugin interface
//!
//! This crate provides the WASM plugin interface for Claude Code integration.

#![warn(missing_docs, unused_crate_dependencies)]

use std::collections::HashMap;

pub use cai_core::Result;

/// Plugin configuration
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// Plugin version
    pub version: String,
    /// Supported skills
    pub skills: Vec<String>,
    /// Supported commands
    pub commands: Vec<String>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            skills: vec![
                "cai.query".into(),
                "cai.ingest".into(),
                "cai.stats".into(),
                "cai.tui".into(),
                "cai.web".into(),
            ],
            commands: vec![
                "cai.query".into(),
                "cai.ingest".into(),
                "cai.stats".into(),
                "cai.tui".into(),
                "cai.web".into(),
            ],
        }
    }
}

/// Plugin trait for Claude Code integration
pub trait Plugin {
    /// Get plugin configuration
    fn config(&self) -> &PluginConfig;

    /// Handle a skill invocation
    fn handle_skill(&mut self, skill: &str, params: &serde_json::Value) -> Result<String>;

    /// Handle a command invocation
    fn handle_command(&mut self, cmd: &str, args: &[String]) -> Result<String>;

    /// Session start hook
    fn on_session_start(&mut self, ctx: &SessionContext) -> Result<()>;

    /// Session end hook
    fn on_session_end(&mut self, ctx: &SessionContext) -> Result<()>;
}

/// Session context for hooks
#[derive(Debug, Clone)]
pub struct SessionContext {
    /// Session ID
    pub session_id: String,
    /// Working directory
    pub work_dir: String,
    /// Environment variables
    pub env: HashMap<String, String>,
}

/// Default plugin implementation
pub struct CaiPlugin {
    config: PluginConfig,
}

impl Default for CaiPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl CaiPlugin {
    /// Create a new plugin instance
    pub fn new() -> Self {
        Self {
            config: PluginConfig::default(),
        }
    }

    /// Get plugin info as JSON
    pub fn info(&self) -> serde_json::Value {
        serde_json::json!({
            "name": "cai",
            "version": self.config.version,
            "skills": self.config.skills,
            "commands": self.config.commands,
        })
    }
}

impl Plugin for CaiPlugin {
    fn config(&self) -> &PluginConfig {
        &self.config
    }

    fn handle_skill(&mut self, skill: &str, params: &serde_json::Value) -> Result<String> {
        match skill {
            "cai.query" => self.handle_query(params),
            "cai.ingest" => self.handle_ingest(params),
            "cai.stats" => self.handle_stats(params),
            "cai.tui" => self.handle_tui(params),
            "cai.web" => self.handle_web(params),
            _ => Err(cai_core::Error::Message(format!("Unknown skill: {}", skill))),
        }
    }

    fn handle_command(&mut self, cmd: &str, args: &[String]) -> Result<String> {
        // Delegate to CLI
        match cmd {
            "cai.query" | "cai.ingest" | "cai.stats" | "cai.tui" | "cai.web" => {
                Ok(format!("Command delegated: {} {:?}", cmd, args))
            }
            _ => Err(cai_core::Error::Message(format!("Unknown command: {}", cmd))),
        }
    }

    fn on_session_start(&mut self, _ctx: &SessionContext) -> Result<()> {
        // Initialize data store
        Ok(())
    }

    fn on_session_end(&mut self, _ctx: &SessionContext) -> Result<()> {
        // Persist analytics
        Ok(())
    }
}

impl CaiPlugin {
    fn handle_query(&mut self, params: &serde_json::Value) -> Result<String> {
        let sql = params["sql"]
            .as_str()
            .ok_or_else(|| cai_core::Error::Message("Missing 'sql' parameter".into()))?;
        let output = params["output"].as_str().unwrap_or("table");
        Ok(format!("Query: {} (output: {})", sql, output))
    }

    fn handle_ingest(&mut self, params: &serde_json::Value) -> Result<String> {
        let source = params["source"]
            .as_str()
            .ok_or_else(|| cai_core::Error::Message("Missing 'source' parameter".into()))?;
        Ok(format!("Ingest from: {}", source))
    }

    fn handle_stats(&mut self, _params: &serde_json::Value) -> Result<String> {
        Ok("Statistics".to_string())
    }

    fn handle_tui(&mut self, _params: &serde_json::Value) -> Result<String> {
        Ok("Launch TUI".to_string())
    }

    fn handle_web(&mut self, _params: &serde_json::Value) -> Result<String> {
        Ok("Launch web dashboard".to_string())
    }
}

/// Create a new CAI plugin instance
///
/// Returns a pointer to a heap-allocated `CaiPlugin` that must be freed
/// using `cai_plugin_destroy` to avoid memory leaks.
///
/// # Safety
///
/// - The returned pointer must be freed with `cai_plugin_destroy`
/// - The pointer is valid until `cai_plugin_destroy` is called
#[no_mangle]
pub extern "C" fn cai_plugin_create() -> *mut CaiPlugin {
    Box::into_raw(Box::new(CaiPlugin::new()))
}

/// Destroy a CAI plugin instance
///
/// # Safety
///
/// - `ptr` must be a valid pointer returned by `cai_plugin_create` or null
/// - This function should only be called once per plugin instance
/// - The pointer becomes invalid after this call
#[no_mangle]
pub unsafe extern "C" fn cai_plugin_destroy(ptr: *mut CaiPlugin) {
    unsafe {
        if !ptr.is_null() {
            let _ = Box::from_raw(ptr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_config() {
        let plugin = CaiPlugin::new();
        assert_eq!(plugin.config().skills.len(), 5);
        assert!(plugin.config().skills.contains(&"cai.query".to_string()));
    }

    #[test]
    fn test_handle_query() {
        let mut plugin = CaiPlugin::new();
        let params = serde_json::json!({"sql": "SELECT * FROM entries"});
        let result = plugin.handle_skill("cai.query", &params);
        assert!(result.is_ok());
    }
}
