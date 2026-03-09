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
    /// Query type for schema queries
    pub query_type: QueryType,
}

/// Type of SQL query
#[derive(Debug, Clone, Default, PartialEq)]
pub enum QueryType {
    /// Standard SELECT query
    #[default]
    Select,
    /// SHOW TABLES query
    ShowTables,
    /// DESCRIBE table query
    DescribeTable(String),
}

/// Parse a SQL query string
pub fn parse(sql: &str) -> QueryResult<ParsedQuery> {
    let sql_upper = sql.trim().to_uppercase();

    // Handle SHOW TABLES
    if sql_upper == "SHOW TABLES" || sql_upper.starts_with("SHOW TABLES;") {
        return Ok(ParsedQuery {
            query_type: QueryType::ShowTables,
            ..Default::default()
        });
    }

    // Handle DESCRIBE table
    if sql_upper.starts_with("DESCRIBE ") || sql_upper.starts_with("DESC ") {
        let keyword = if sql_upper.starts_with("DESCRIBE ") {
            "DESCRIBE "
        } else {
            "DESC "
        };
        let table_name = sql[keyword.len()..].trim().to_string();
        let table_name = table_name.trim_end_matches(';').trim().to_string();

        if table_name.to_lowercase() != "entries" {
            return Err(QueryError::InvalidTable(table_name));
        }

        return Ok(ParsedQuery {
            query_type: QueryType::DescribeTable("entries".to_string()),
            table: Some("entries".to_string()),
            ..Default::default()
        });
    }

    // Handle PRAGMA table_info (SQLite-style)
    if sql_upper.starts_with("PRAGMA TABLE_INFO(") {
        // Extract table name from PRAGMA table_info(entries)
        let start = sql_upper.find('(').unwrap() + 1;
        let end = sql_upper.find(')').unwrap();
        let table_name = sql[start..end].trim().to_string();

        if table_name.to_lowercase() != "entries" {
            return Err(QueryError::InvalidTable(table_name));
        }

        return Ok(ParsedQuery {
            query_type: QueryType::DescribeTable("entries".to_string()),
            table: Some("entries".to_string()),
            ..Default::default()
        });
    }

    // Validate it's a SELECT statement
    if !sql_upper.starts_with("SELECT") {
        return Err(QueryError::ParseError(
            "Only SELECT, SHOW TABLES, and DESCRIBE statements are supported".to_string(),
        ));
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
        let table_end = table_part
            .find(|c: char| c.is_whitespace())
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
        let limit_end = limit_str
            .find(|c: char| c.is_whitespace())
            .or_else(|| limit_str.find(';'))
            .unwrap_or(limit_str.len());
        limit_str[..limit_end].trim().parse::<usize>().ok()
    } else {
        None
    };

    // Check for WHERE
    let where_sql = if sql_upper.contains("WHERE ") {
        let where_idx = sql_upper.find("WHERE ").unwrap() + 6;
        let where_end = sql_upper[where_idx..]
            .find(" GROUP BY")
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
        query_type: QueryType::Select,
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
        assert_eq!(parsed.query_type, QueryType::Select);
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

    #[test]
    fn test_parse_show_tables() {
        let result = parse("SHOW TABLES");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.query_type, QueryType::ShowTables);
    }

    #[test]
    fn test_parse_show_tables_with_semicolon() {
        let result = parse("SHOW TABLES;");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.query_type, QueryType::ShowTables);
    }

    #[test]
    fn test_parse_describe_entries() {
        let result = parse("DESCRIBE entries");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(
            parsed.query_type,
            QueryType::DescribeTable("entries".to_string())
        );
    }

    #[test]
    fn test_parse_desc_entries() {
        let result = parse("DESC entries");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(
            parsed.query_type,
            QueryType::DescribeTable("entries".to_string())
        );
    }

    #[test]
    fn test_parse_pragma_table_info() {
        let result = parse("PRAGMA table_info(entries)");
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(
            parsed.query_type,
            QueryType::DescribeTable("entries".to_string())
        );
    }

    #[test]
    fn test_parse_describe_invalid_table() {
        let result = parse("DESCRIBE invalid_table");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), QueryError::InvalidTable(_)));
    }
}
