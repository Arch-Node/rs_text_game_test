// discord_client/formatter.rs
//
// Discord message formatting

/// Format welcome message for new players
pub fn format_welcome(player_name: &str) -> String {
    format!(
        "🎮 **Welcome, {}!**\n\n\
        You've been authenticated and can now play the game.\n\
        Type `help` to see available commands.\n\n\
        ⚔️ Your adventure begins now!",
        player_name
    )
}

/// Format help message
pub fn format_help() -> String {
    "📖 **Available Commands**\n\n\
    **Movement:**\n\
    • `north` (or `n`) - Move north\n\
    • `south` (or `s`) - Move south\n\
    • `east` (or `e`) - Move east\n\
    • `west` (or `w`) - Move west\n\
    • `up` (or `u`) - Move up\n\
    • `down` (or `d`) - Move down\n\n\
    **Info Commands:**\n\
    • `look` - Look around current room\n\
    • `status` - Show your current status\n\
    • `rooms` - List rooms in your dimension\n\n\
    **System:**\n\
    • `help` - Show this help message\n\n\
    💡 **Tip:** Commands are processed every few seconds (tick-based gameplay)".to_string()
}

/// Format player status
pub fn format_status(player_name: &str) -> String {
    format!(
        "👤 **Player Status**\n\n\
        Name: {}\n\
        Status: ✅ Online\n\
        Interface: Discord",
        player_name
    )
}

/// Format command result
pub fn format_command_result(result: &str) -> String {
    if result.contains("error") || result.contains("failed") || result.contains("Error") {
        format_error(result)
    } else {
        format!("✅ {}", result)
    }
}

/// Format error message
pub fn format_error(error: &str) -> String {
    format!("❌ **Error:** {}", error)
}

/// Format room description
pub fn format_room(name: &str, description: &str, exits: &[String]) -> String {
    let mut result = format!("📍 **{}**\n\n{}\n\n", name, description);
    
    if !exits.is_empty() {
        result.push_str("🚪 **Exits:** ");
        result.push_str(&exits.join(", "));
    } else {
        result.push_str("🚪 **No obvious exits**");
    }
    
    result
}

/// Truncate message to Discord's limit (2000 characters)
pub fn truncate_message(text: &str) -> String {
    const MAX_LENGTH: usize = 2000;
    
    if text.len() <= MAX_LENGTH {
        text.to_string()
    } else {
        let truncated = &text[..MAX_LENGTH - 20];
        format!("{}...\n\n*(truncated)*", truncated)
    }
}
