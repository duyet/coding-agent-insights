//! SQL query parser

use crate::error::{QueryError, QueryResult};

/// Parsed query representation
#[derive(Debug, Clone, Default)]
pub struct ParsedQuery {
    /// Columns to select (empty means all)
    pub select_wildcard: bool,
    /// Selected column names
    pub columns: Vec<String>,
    /// Table name (must be "entries")
    pub table: Option<String>,
    /// WHERE clause as SQL string (for simple cases)
    pub where_sql: Option<String>,
    /// GROUP BY columns
    pub group_by: Vec<String>,
    /// ORDER BY columns
    pub order_by: Vec<(String, bool)>, // (column, asc)
    /// LIMIT value
    pub limit: Option<usize>,
    /// Has aggregate functions
    pub has_aggregates: bool,
}

/// Parse a SQL query string
pub fn parse(sql: &str) -> QueryResult<ParsedQuery> {
    let sql_upper = sql.to_uppercase();
    
    // Validate it's a SELECT statement
    if !sql_upper.trim().starts_with("SELECT") {
        return Err(QueryError::ParseError("Only SELECT statements are supported".to_string()));
    }
    
    // Check for FROM entries
    if !sql_upper.contains("FROM") {
        return Err(QueryError::ParseError("Missing FROM clause".to_string()));
    }

    // Extract table name
    let table = if sql_upper.contains("FROM ENTRIES") {
        Some("entries".to_string())
    } else {
        // Try to find what comes after FROM
        let from_idx = sql_upper.find("FROM ").unwrap() + 5;
        let table_part = &sql[from_idx..];
        let table_end = table_part.find(|c: char| c.is_whitespace())
            .or_else(|| table_part.find(';'))
            .unwrap_or(table_part.len());
        let table_name = table_part[..table_end].trim().to_string();
        // Validate table name
        if table_name.to_lowercase() != "entries" {
            return Err(QueryError::InvalidTable(table_name));
        }
        Some("entries".to_string())
    };
    
    // Check for LIMIT
    let limit = if let Some(limit_idx) = sql_upper.find("LIMIT ") {
        let limit_str = &sql[limit_idx + 6..];
        let limit_end = limit_str.find(|c: char| c.is_whitespace())
            .or_else(|| limit_str.find(';'))
            .unwrap_or(limit_str.len());
        limit_str[..limit_end].trim().parse::<usize>().ok()
    } else {
        None
    };
    
    // Check for WHERE
    let where_sql = if sql_upper.contains("WHERE ") {
        let where_idx = sql_upper.find("WHERE ").unwrap() + 6;
        let where_end = sql_upper[where_idx..].find(" GROUP BY")
            .or_else(|| sql_upper[where_idx..].find(" ORDER BY"))
            .or_else(|| sql_upper[where_idx..].find(" LIMIT"))
            .or_else(|| sql_upper[where_idx..].find(';'))
            .unwrap_or(sql_upper[where_idx..].len());
        Some(sql[where_idx..where_idx + where_end].trim().to_string())
    } else {
        None
    };
    
    // Check for wildcard
    let select_wildcard = sql_upper.contains("SELECT *");
    
    Ok(ParsedQuery {
        select_wildcard,
        columns: vec![],
        table,
        where_sql,
        group_by: vec![],
        order_by: vec![],
        limit,
        has_aggregates: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_select() {
        let result = parse("SELECT * FROM entries");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.select_wildcard);
        assert_eq!(parsed.table, Some("entries".to_string()));
    }

    #[test]
    fn test_parse_select_with_limit() {
        let result = parse("SELECT * FROM entries LIMIT 10");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.limit, Some(10));
    }

    #[test]
    fn test_parse_select_with_where() {
        let result = parse("SELECT * FROM entries WHERE source = 'Claude'");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(parsed.where_sql.is_some());
    }
}
