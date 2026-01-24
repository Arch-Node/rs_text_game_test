// signal_client/formatter.rs
//
// Format game output for Signal Messenger

/// Format game text for Signal
/// - Keep messages under 2000 characters
/// - Use Unicode emojis for visual appeal
/// - Format lists nicely
pub fn format_for_signal(text: &str) -> String {
    let mut formatted = text.to_string();
    
    // Add some visual polish
    formatted = formatted.replace("You moved", "🚶 You moved");
    formatted = formatted.replace("Welcome", "👋 Welcome");
    formatted = formatted.replace("Error:", "❌ Error:");
    formatted = formatted.replace("Success:", "✅ Success:");
    
    // Truncate if too long
    if formatted.len() > 2000 {
        formatted.truncate(1997);
        formatted.push_str("...");
    }
    
    formatted
}

/// Format room description for Signal
pub fn format_room(name: &str, description: &str, exits: &[String]) -> String {
    let mut output = format!("📍 **{}**\n\n{}\n\n", name, description);
    
    if !exits.is_empty() {
        output.push_str("🚪 Exits: ");
        output.push_str(&exits.join(", "));
    } else {
        output.push_str("🚪 No obvious exits");
    }
    
    output
}

/// Format player status for Signal
pub fn format_player_status(
    name: &str,
    position: (i32, i32, i32),
    dimension: &str,
) -> String {
    format!(
        "👤 **{}**\n\
        📍 Position: ({}, {}, {})\n\
        🌍 Dimension: {}",
        name, position.0, position.1, position.2, dimension
    )
}

/// Format command result for Signal
pub fn format_command_result(command: &str, result: &str, success: bool) -> String {
    let icon = if success { "✅" } else { "❌" };
    format!("{} Command: {}\n{}", icon, command, result)
}

/// Format error message for Signal
pub fn format_error(message: &str) -> String {
    format!("❌ {}", message)
}

/// Format help text for Signal
pub fn format_help() -> String {
    "📖 **Layered Realms - Commands**\n\n\
    🚶 **Movement**\n\
    • north, n - Move north\n\
    • south, s - Move south\n\
    • east, e - Move east\n\
    • west, w - Move west\n\
    • up, u - Move up\n\
    • down, d - Move down\n\n\
    ℹ️ **Information**\n\
    • look, l - Look around\n\
    • status - Check your status\n\
    • inventory, i - Check inventory\n\
    • rooms - List rooms\n\n\
    ⚙️ **System**\n\
    • help, ? - Show this help\n\
    • quit - Exit game".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_truncate_long_message() {
        let long_text = "a".repeat(2100);
        let formatted = format_for_signal(&long_text);
        assert!(formatted.len() <= 2000);
        assert!(formatted.ends_with("..."));
    }
    
    #[test]
    fn test_format_room() {
        let formatted = format_room(
            "Town Square",
            "A bustling plaza",
            &["north".to_string(), "east".to_string()]
        );
        assert!(formatted.contains("📍"));
        assert!(formatted.contains("Town Square"));
        assert!(formatted.contains("north, east"));
    }
}
