//! Built-in SQL functions for CAI queries

use crate::error::{QueryError, QueryResult};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Type alias for SQL function implementation
type SqlFunction = Box<dyn Fn(&[FunctionArg]) -> QueryResult<FunctionArg> + Send + Sync>;

/// Function registry for SQL functions
pub struct FunctionRegistry {
    functions: HashMap<String, SqlFunction>,
}

impl Default for FunctionRegistry {
    fn default() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };

        // Register built-in functions
        registry.register("date_format", date_format);
        registry.register("concat", concat);
        registry.register("length", length);
        registry.register("upper", upper);
        registry.register("lower", lower);
        registry.register("substring", substring);
        registry.register("coalesce", coalesce);
        registry.register("now", now);

        registry
    }
}

impl FunctionRegistry {
    /// Register a new function
    pub fn register<F>(&mut self, name: &str, func: F)
    where
        F: Fn(&[FunctionArg]) -> QueryResult<FunctionArg> + Send + Sync + 'static,
    {
        self.functions.insert(name.to_lowercase(), Box::new(func));
    }

    /// Call a function by name
    pub fn call(&self, name: &str, args: &[FunctionArg]) -> QueryResult<FunctionArg> {
        match self.functions.get(&name.to_lowercase()) {
            Some(func) => func(args),
            None => Err(QueryError::ParseError(format!(
                "Unknown function: {}",
                name
            ))),
        }
    }

    /// Check if a function exists
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(&name.to_lowercase())
    }
}

/// Function argument value
#[derive(Debug, Clone, PartialEq)]
pub enum FunctionArg {
    /// String value
    String(String),
    /// Integer number
    Number(i64),
    /// Floating-point number
    Float(f64),
    /// Boolean value
    Boolean(bool),
    /// Null value
    Null,
}

impl From<&str> for FunctionArg {
    fn from(s: &str) -> Self {
        FunctionArg::String(s.to_string())
    }
}

impl From<String> for FunctionArg {
    fn from(s: String) -> Self {
        FunctionArg::String(s)
    }
}

impl From<i64> for FunctionArg {
    fn from(n: i64) -> Self {
        FunctionArg::Number(n)
    }
}

impl From<f64> for FunctionArg {
    fn from(n: f64) -> Self {
        FunctionArg::Float(n)
    }
}

impl From<bool> for FunctionArg {
    fn from(b: bool) -> Self {
        FunctionArg::Boolean(b)
    }
}

// ============================================================================
// Built-in Functions
// ============================================================================

/// Format a DateTime to a string representation
///
/// Supports formats: "iso", "date", "time", "unix", "year", "month", "day", "hour", "minute", "ymd"
pub fn date_format(args: &[FunctionArg]) -> QueryResult<FunctionArg> {
    if args.len() != 2 {
        return Err(QueryError::ParseError(
            "date_format requires 2 arguments: timestamp and format".to_string(),
        ));
    }

    // For now, we'll work with string timestamps
    let timestamp_str = match &args[0] {
        FunctionArg::String(s) => s.as_str(),
        _ => {
            return Err(QueryError::ParseError(
                "date_format first argument must be a string".to_string(),
            ))
        }
    };

    let format = match &args[1] {
        FunctionArg::String(s) => s.as_str(),
        _ => {
            return Err(QueryError::ParseError(
                "date_format second argument must be a string".to_string(),
            ))
        }
    };

    // Parse ISO 8601 timestamp
    let dt = DateTime::parse_from_rfc3339(timestamp_str)
        .map_err(|_| QueryError::ParseError(format!("Invalid timestamp: {}", timestamp_str)))?;

    let result = match format {
        "iso" => dt.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        "date" => dt.format("%Y-%m-%d").to_string(),
        "time" => dt.format("%H:%M:%S").to_string(),
        "unix" => dt.timestamp().to_string(),
        "year" => dt.format("%Y").to_string(),
        "month" => dt.format("%m").to_string(),
        "day" => dt.format("%d").to_string(),
        "hour" => dt.format("%H").to_string(),
        "minute" => dt.format("%M").to_string(),
        "ymd" => dt.format("%Y%m%d").to_string(),
        _ => {
            return Err(QueryError::ParseError(format!(
                "Unknown format: {}",
                format
            )))
        }
    };

    Ok(FunctionArg::String(result))
}

/// Concatenate strings
pub fn concat(args: &[FunctionArg]) -> QueryResult<FunctionArg> {
    if args.is_empty() {
        return Ok(FunctionArg::String(String::new()));
    }

    let mut result = String::new();
    for arg in args {
        match arg {
            FunctionArg::String(s) => result.push_str(s),
            FunctionArg::Number(n) => result.push_str(&n.to_string()),
            FunctionArg::Float(f) => result.push_str(&f.to_string()),
            FunctionArg::Boolean(b) => result.push_str(&b.to_string()),
            FunctionArg::Null => result.push_str("NULL"),
        }
    }

    Ok(FunctionArg::String(result))
}

/// Get string length
pub fn length(args: &[FunctionArg]) -> QueryResult<FunctionArg> {
    if args.len() != 1 {
        return Err(QueryError::ParseError(
            "length requires 1 argument".to_string(),
        ));
    }

    let len = match &args[0] {
        FunctionArg::String(s) => s.len(),
        FunctionArg::Number(n) => n.to_string().len(),
        FunctionArg::Float(f) => f.to_string().len(),
        FunctionArg::Boolean(b) => b.to_string().len(),
        FunctionArg::Null => 4, // "NULL"
    };

    Ok(FunctionArg::Number(len as i64))
}

/// Convert string to uppercase
pub fn upper(args: &[FunctionArg]) -> QueryResult<FunctionArg> {
    if args.len() != 1 {
        return Err(QueryError::ParseError(
            "upper requires 1 argument".to_string(),
        ));
    }

    let result = match &args[0] {
        FunctionArg::String(s) => s.to_uppercase(),
        FunctionArg::Number(n) => n.to_string().to_uppercase(),
        FunctionArg::Float(f) => f.to_string().to_uppercase(),
        FunctionArg::Boolean(b) => b.to_string().to_uppercase(),
        FunctionArg::Null => String::from("NULL"),
    };

    Ok(FunctionArg::String(result))
}

/// Convert string to lowercase
pub fn lower(args: &[FunctionArg]) -> QueryResult<FunctionArg> {
    if args.len() != 1 {
        return Err(QueryError::ParseError(
            "lower requires 1 argument".to_string(),
        ));
    }

    let result = match &args[0] {
        FunctionArg::String(s) => s.to_lowercase(),
        FunctionArg::Number(n) => n.to_string().to_lowercase(),
        FunctionArg::Float(f) => f.to_string().to_lowercase(),
        FunctionArg::Boolean(b) => b.to_string().to_lowercase(),
        FunctionArg::Null => String::from("null"),
    };

    Ok(FunctionArg::String(result))
}

/// Extract substring (1-indexed start position)
pub fn substring(args: &[FunctionArg]) -> QueryResult<FunctionArg> {
    if args.len() < 2 || args.len() > 3 {
        return Err(QueryError::ParseError(
            "substring requires 2 or 3 arguments: string, start, [length]".to_string(),
        ));
    }

    let s = match &args[0] {
        FunctionArg::String(s) => s.as_str(),
        _ => {
            return Err(QueryError::ParseError(
                "substring first argument must be a string".to_string(),
            ))
        }
    };

    let start = match &args[1] {
        FunctionArg::Number(n) => *n as usize,
        _ => {
            return Err(QueryError::ParseError(
                "substring second argument must be a number".to_string(),
            ))
        }
    };

    let result = if args.len() == 3 {
        let length = match &args[2] {
            FunctionArg::Number(n) => *n as usize,
            _ => {
                return Err(QueryError::ParseError(
                    "substring third argument must be a number".to_string(),
                ))
            }
        };
        // Convert 1-indexed to 0-indexed
        let start_idx = start.saturating_sub(1);
        s.chars().skip(start_idx).take(length).collect()
    } else {
        // Convert 1-indexed to 0-indexed
        let start_idx = start.saturating_sub(1);
        s.chars().skip(start_idx).collect()
    };

    Ok(FunctionArg::String(result))
}

/// Return first non-null argument
pub fn coalesce(args: &[FunctionArg]) -> QueryResult<FunctionArg> {
    for arg in args {
        if arg != &FunctionArg::Null {
            return Ok(arg.clone());
        }
    }
    Ok(FunctionArg::Null)
}

/// Get current timestamp
pub fn now(args: &[FunctionArg]) -> QueryResult<FunctionArg> {
    if !args.is_empty() {
        return Err(QueryError::ParseError(
            "now requires no arguments".to_string(),
        ));
    }

    let now: DateTime<Utc> = Utc::now();
    Ok(FunctionArg::String(
        now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    ))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_registry_default() {
        let registry = FunctionRegistry::default();
        assert!(registry.has_function("date_format"));
        assert!(registry.has_function("concat"));
        assert!(registry.has_function("length"));
        assert!(registry.has_function("upper"));
        assert!(registry.has_function("lower"));
        assert!(registry.has_function("substring"));
        assert!(registry.has_function("coalesce"));
        assert!(registry.has_function("now"));
    }

    #[test]
    fn test_date_format() {
        let args = vec![
            FunctionArg::String("2024-01-15T10:30:00Z".to_string()),
            FunctionArg::String("iso".to_string()),
        ];
        let result = date_format(&args).unwrap();
        assert_eq!(
            result,
            FunctionArg::String("2024-01-15T10:30:00Z".to_string())
        );

        let args = vec![
            FunctionArg::String("2024-01-15T10:30:00Z".to_string()),
            FunctionArg::String("date".to_string()),
        ];
        let result = date_format(&args).unwrap();
        assert_eq!(result, FunctionArg::String("2024-01-15".to_string()));
    }

    #[test]
    fn test_concat() {
        let args = vec![
            FunctionArg::String("Hello".to_string()),
            FunctionArg::String(" ".to_string()),
            FunctionArg::String("World".to_string()),
        ];
        let result = concat(&args).unwrap();
        assert_eq!(result, FunctionArg::String("Hello World".to_string()));

        // With numbers
        let args = vec![
            FunctionArg::String("Count: ".to_string()),
            FunctionArg::Number(42),
        ];
        let result = concat(&args).unwrap();
        assert_eq!(result, FunctionArg::String("Count: 42".to_string()));
    }

    #[test]
    fn test_length() {
        let args = vec![FunctionArg::String("hello".to_string())];
        let result = length(&args).unwrap();
        assert_eq!(result, FunctionArg::Number(5));

        let args = vec![FunctionArg::String("".to_string())];
        let result = length(&args).unwrap();
        assert_eq!(result, FunctionArg::Number(0));
    }

    #[test]
    fn test_upper_lower() {
        let args = vec![FunctionArg::String("Hello".to_string())];
        let result = upper(&args).unwrap();
        assert_eq!(result, FunctionArg::String("HELLO".to_string()));

        let args = vec![FunctionArg::String("HELLO".to_string())];
        let result = lower(&args).unwrap();
        assert_eq!(result, FunctionArg::String("hello".to_string()));
    }

    #[test]
    fn test_substring() {
        let args = vec![
            FunctionArg::String("hello".to_string()),
            FunctionArg::Number(2),
            FunctionArg::Number(3),
        ];
        let result = substring(&args).unwrap();
        assert_eq!(result, FunctionArg::String("ell".to_string()));

        // Without length
        let args = vec![
            FunctionArg::String("hello".to_string()),
            FunctionArg::Number(2),
        ];
        let result = substring(&args).unwrap();
        assert_eq!(result, FunctionArg::String("ello".to_string()));
    }

    #[test]
    fn test_coalesce() {
        let args = vec![
            FunctionArg::Null,
            FunctionArg::String("default".to_string()),
            FunctionArg::String("other".to_string()),
        ];
        let result = coalesce(&args).unwrap();
        assert_eq!(result, FunctionArg::String("default".to_string()));

        // All null
        let args = vec![FunctionArg::Null, FunctionArg::Null];
        let result = coalesce(&args).unwrap();
        assert_eq!(result, FunctionArg::Null);
    }

    #[test]
    fn test_now() {
        let result = now(&[]).unwrap();
        match result {
            FunctionArg::String(s) => {
                // Should be a valid ISO 8601 timestamp
                assert!(DateTime::parse_from_rfc3339(s.as_str()).is_ok());
            }
            _ => panic!("now() should return a string"),
        }
    }

    #[test]
    fn test_function_registry_call() {
        let registry = FunctionRegistry::default();

        // Test calling upper function
        let args = vec![FunctionArg::String("hello".to_string())];
        let result = registry.call("upper", &args).unwrap();
        assert_eq!(result, FunctionArg::String("HELLO".to_string()));

        // Test calling unknown function
        let args = vec![FunctionArg::String("test".to_string())];
        let result = registry.call("unknown", &args);
        assert!(result.is_err());
    }
}
