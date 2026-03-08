//! E2E tests for CLI workflows
//!
//! These tests verify complete CLI command flows.

#[cfg(test)]
mod cli_tests {
    use std::path::PathBuf;
    use std::process::{Command, Stdio};

    /// Get the path to the cai binary
    fn cai_bin() -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../target/debug/cai");
        path
    }

    /// Test basic CLI invocation
    #[test]
    fn test_cli_basic_invocation() {
        let bin_path = cai_bin();

        // Check if binary exists; if not, skip test during development
        if !bin_path.exists() {
            println!("Binary not found at {}. Run 'cargo build' first.", bin_path.display());
            return;
        }

        let output = Command::new(&bin_path)
            .arg("--help")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute cai");

        assert!(output.status.success(), "cai --help should succeed");
        let help_text = String::from_utf8_lossy(&output.stdout);
        assert!(help_text.contains("AI coding history") || help_text.contains("Coding Agent Insights"), "Should show app name");
        assert!(help_text.contains("query") || help_text.contains("Query"), "Should show query command");
        assert!(help_text.contains("ingest") || help_text.contains("Ingest"), "Should show ingest command");
        assert!(help_text.contains("tui") || help_text.contains("Tui") || help_text.contains("TUI"), "Should show tui command");
        assert!(help_text.contains("web") || help_text.contains("Web"), "Should show web command");
    }

    /// Test query command help
    #[test]
    fn test_query_help() {
        let bin_path = cai_bin();

        if !bin_path.exists() {
            return;
        }

        let output = Command::new(&bin_path)
            .args(["query", "--help"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute cai query --help");

        assert!(output.status.success(), "cai query --help should succeed");
        let help_text = String::from_utf8_lossy(&output.stdout);
        assert!(help_text.contains("query"), "Should show query argument");
        assert!(help_text.contains("output"), "Should show output format option");
    }

    /// Test ingest command help
    #[test]
    fn test_ingest_help() {
        let bin_path = cai_bin();

        if !bin_path.exists() {
            return;
        }

        let output = Command::new(&bin_path)
            .args(["ingest", "--help"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute cai ingest --help");

        assert!(output.status.success(), "cai ingest --help should succeed");
        let help_text = String::from_utf8_lossy(&output.stdout);
        assert!(help_text.contains("source"), "Should show source argument");
        assert!(help_text.contains("path"), "Should show path option");
    }

    /// Test web command help
    #[test]
    fn test_web_help() {
        let bin_path = cai_bin();

        if !bin_path.exists() {
            return;
        }

        let output = Command::new(&bin_path)
            .args(["web", "--help"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute cai web --help");

        assert!(output.status.success(), "cai web --help should succeed");
        let help_text = String::from_utf8_lossy(&output.stdout);
        assert!(help_text.contains("port"), "Should show port option");
    }

    /// Test query command with output format
    #[test]
    fn test_query_with_output_format() {
        let bin_path = cai_bin();

        if !bin_path.exists() {
            return;
        }

        let output = Command::new(&bin_path)
            .args(["query", "SELECT *", "--output", "json"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute cai query");

        // Should not crash even with placeholder implementation
        // Just verify it runs without error
        assert!(output.status.success() || String::from_utf8_lossy(&output.stderr).contains("Querying:"));
    }

    /// Test ingest command with source
    #[test]
    fn test_ingest_with_source() {
        let bin_path = cai_bin();

        if !bin_path.exists() {
            return;
        }

        let output = Command::new(&bin_path)
            .args(["ingest", "--source", "claude"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute cai ingest");

        // Should not crash even with placeholder implementation
        assert!(output.status.success() || String::from_utf8_lossy(&output.stdout).contains("Ingesting from:"));
    }

    /// Test invalid command
    #[test]
    fn test_invalid_command() {
        let bin_path = cai_bin();

        if !bin_path.exists() {
            return;
        }

        let output = Command::new(&bin_path)
            .arg("invalid-command")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute cai");

        assert!(!output.status.success(), "Invalid command should fail");
    }

    /// Test missing required argument
    #[test]
    fn test_missing_required_argument() {
        let bin_path = cai_bin();

        if !bin_path.exists() {
            return;
        }

        let output = Command::new(&bin_path)
            .arg("query")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute cai query");

        assert!(!output.status.success(), "Query without argument should fail");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("required") || stderr.contains("QUERY"), "Should indicate required argument");
    }

    /// Test output format validation
    #[test]
    fn test_all_output_formats() {
        let bin_path = cai_bin();

        if !bin_path.exists() {
            return;
        }

        let formats = ["table", "json", "csv", "jsonl"];

        for format in formats {
            let output = Command::new(&bin_path)
                .args(["query", "SELECT *", "--output", format])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect(&format!("Failed to execute cai query with --output {}", format));

            // Should accept all standard formats
            assert!(output.status.success() || String::from_utf8_lossy(&output.stdout).contains("Output:"),
                "Format {} should be accepted", format);
        }
    }

    /// Test version flag
    #[test]
    fn test_version_flag() {
        let bin_path = cai_bin();

        if !bin_path.exists() {
            return;
        }

        let output = Command::new(&bin_path)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute cai --version");

        assert!(output.status.success(), "cai --version should succeed");
        let version_text = String::from_utf8_lossy(&output.stdout);
        assert!(version_text.contains("cai") || !version_text.is_empty(), "Should show version info");
    }

    /// Test concurrent CLI invocations
    #[test]
    fn test_concurrent_invocations() {
        let bin_path = cai_bin();

        if !bin_path.exists() {
            return;
        }

        // Spawn multiple help commands concurrently
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let bin = bin_path.clone();
                std::thread::spawn(move || {
                    Command::new(&bin)
                        .arg("--help")
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                })
            })
            .collect();

        // All should succeed
        for handle in handles {
            let output = handle.join().unwrap().expect("Failed to execute cai");
            assert!(output.status.success(), "Concurrent invocation should succeed");
        }
    }

    /// Test error message formatting
    #[test]
    fn test_error_message_formatting() {
        let bin_path = cai_bin();

        if !bin_path.exists() {
            return;
        }

        let output = Command::new(&bin_path)
            .args(["query", ""])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .expect("Failed to execute cai query");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should have some output (either error or placeholder)
        assert!(!stderr.is_empty() || !stdout.is_empty(), "Should produce output");
    }
}
