// output.rs
//
// Flexible output system that works in terminal or GUI contexts

/// Accumulates game output messages that can be displayed in any context
#[derive(Debug, Clone, Default)]
pub struct MessageBuffer {
    messages: Vec<String>,
    pub mode: OutputMode,
}

/// Controls how messages are handled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputMode {
    /// Accumulate messages only (for GUI/web/testing)
    BufferOnly,
    /// Print to stdout immediately AND accumulate (for terminal play)
    #[default]
    TerminalAndBuffer,
    /// Print to stdout immediately without buffering
    TerminalOnly,
}

impl MessageBuffer {
    /// Create a new message buffer with the specified output mode
    pub fn new(mode: OutputMode) -> Self {
        Self {
            messages: Vec::new(),
            mode,
        }
    }

    /// Add a line of output
    pub fn add_line(&mut self, message: impl Into<String>) {
        let msg = message.into();
        
        match self.mode {
            OutputMode::BufferOnly => {
                self.messages.push(msg);
            }
            OutputMode::TerminalAndBuffer => {
                println!("{}", msg);
                self.messages.push(msg);
            }
            OutputMode::TerminalOnly => {
                println!("{}", msg);
            }
        }
    }

    /// Add an empty line
    pub fn add_blank_line(&mut self) {
        self.add_line("");
    }

    /// Add multiple lines at once
    pub fn add_lines(&mut self, lines: impl IntoIterator<Item = impl Into<String>>) {
        for line in lines {
            self.add_line(line);
        }
    }

    /// Get all accumulated messages
    pub fn get_messages(&self) -> &[String] {
        &self.messages
    }

    /// Get all messages joined with newlines
    pub fn get_text(&self) -> String {
        self.messages.join("\n")
    }

    /// Clear the message buffer
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get and clear the buffer (useful for processing output in chunks)
    pub fn take_messages(&mut self) -> Vec<String> {
        std::mem::take(&mut self.messages)
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get the number of messages
    pub fn len(&self) -> usize {
        self.messages.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_only_mode() {
        let mut output = MessageBuffer::new(OutputMode::BufferOnly);
        output.add_line("Hello");
        output.add_line("World");
        
        assert_eq!(output.get_messages(), &["Hello", "World"]);
        assert_eq!(output.get_text(), "Hello\nWorld");
    }

    #[test]
    fn test_clear() {
        let mut output = MessageBuffer::new(OutputMode::BufferOnly);
        output.add_line("Test");
        assert_eq!(output.len(), 1);
        
        output.clear();
        assert!(output.is_empty());
    }
}
