//! Real shell execution using PTY

use anyhow::Result;
use portable_pty::{native_pty_system, CommandBuilder, PtySize, PtySystem};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use vte::{Params, Parser, Perform};

/// Shell executor using PTY
pub struct ShellExecutor {
    pty_system: Box<dyn PtySystem>,
    output_buffer: Arc<Mutex<Vec<u8>>>,
}

impl ShellExecutor {
    pub fn new() -> Result<Self> {
        let pty_system = native_pty_system();

        Ok(Self {
            pty_system,
            output_buffer: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Execute a command and return the output (blocking version for internal use)
    fn execute_blocking(&mut self, command: &str) -> Result<String> {
        // Create a new PTY
        let pair = self.pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Spawn the shell command
        let mut cmd = CommandBuilder::new("sh");
        cmd.arg("-c");
        cmd.arg(command);

        let mut child = pair.slave.spawn_command(cmd)?;

        // Read output from master
        let mut reader = pair.master.try_clone_reader()?;
        let mut output = Vec::new();
        let mut buffer = [0u8; 8192];

        // Read with timeout
        let timeout = std::time::Duration::from_secs(5);
        let start = std::time::Instant::now();

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    output.extend_from_slice(&buffer[..n]);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if start.elapsed() > timeout {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }
                Err(_) => break,
            }

            // Check if command finished
            if let Ok(Some(_)) = child.try_wait() {
                // Read any remaining output
                std::thread::sleep(std::time::Duration::from_millis(50));
                if let Ok(n) = reader.read(&mut buffer) {
                    if n > 0 {
                        output.extend_from_slice(&buffer[..n]);
                    }
                }
                break;
            }

            if start.elapsed() > timeout {
                break;
            }
        }

        // Convert output to string
        let output_str = String::from_utf8_lossy(&output).to_string();

        // Strip ANSI codes for now (we can keep them later for colors)
        let stripped = strip_ansi_codes(&output_str);

        Ok(stripped)
    }

    /// Execute a command asynchronously without blocking the UI
    pub async fn execute(&mut self, command: String, env_vars: std::collections::HashMap<String, String>) -> Result<String> {
        // Use simple tokio Command instead of PTY for now
        // This is more reliable and won't block
        tokio::task::spawn_blocking(move || {
            use std::process::Command;

            // Build command with environment variables
            let mut cmd = Command::new("sh");
            cmd.arg("-c")
               .arg(&command)
               .env("TERM", "xterm-256color");

            // Add custom environment variables
            for (key, value) in env_vars {
                cmd.env(key, value);
            }

            let output = cmd.output()
                .map_err(|e| anyhow::anyhow!("Failed to execute command: {}", e))?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            let mut result = String::new();
            if !stdout.is_empty() {
                // Keep ANSI codes for colored output!
                result.push_str(&stdout);
            }
            if !stderr.is_empty() {
                if !result.is_empty() {
                    result.push_str("\n");
                }
                result.push_str("stderr:\n");
                result.push_str(&stderr);
            }

            if result.is_empty() {
                result = format!("✓ Command completed (exit code: {})", output.status.code().unwrap_or(-1));
            }

            Ok(result)
        })
        .await
        .map_err(|e| anyhow::anyhow!("Task join error: {}", e))?
    }
}

impl Default for ShellExecutor {
    fn default() -> Self {
        Self::new().expect("Failed to create shell executor")
    }
}

/// Strip ANSI escape codes from text
fn strip_ansi_codes(text: &str) -> String {
    let mut result = String::new();
    let mut parser = Parser::new();
    let mut performer = StripAnsiPerformer {
        output: &mut result,
    };

    for byte in text.bytes() {
        parser.advance(&mut performer, byte);
    }

    result
}

struct StripAnsiPerformer<'a> {
    output: &'a mut String,
}

impl<'a> Perform for StripAnsiPerformer<'a> {
    fn print(&mut self, c: char) {
        self.output.push(c);
    }

    fn execute(&mut self, byte: u8) {
        if byte == b'\n' || byte == b'\r' || byte == b'\t' {
            self.output.push(byte as char);
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
    fn csi_dispatch(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {}
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_simple_command() {
        let mut executor = ShellExecutor::new().unwrap();
        let output = executor.execute_blocking("echo 'hello world'").unwrap();
        assert!(output.contains("hello world"));
    }

    #[test]
    fn test_execute_ls() {
        let mut executor = ShellExecutor::new().unwrap();
        let output = executor.execute_blocking("ls").unwrap();
        assert!(!output.is_empty());
    }
}
