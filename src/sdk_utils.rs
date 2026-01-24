// Utility functions for SpacetimeDB SDK client setup

use crate::spacetimedb_client::DbConnection;
use spacetimedb_sdk::DbContext;

/// Create a properly configured DbConnection with all necessary subscriptions
/// 
/// This ensures that all SDK clients automatically subscribe to the tables they need,
/// making tests and real clients work consistently without duplicated subscription code.
pub fn create_connection(
    uri: &str,
    module_name: &str,
) -> Result<DbConnection, Box<dyn std::error::Error>> {
    let conn = DbConnection::builder()
        .with_uri(uri)
        .with_module_name(module_name)
        .build()?;
    
    // Subscribe to all essential tables
    let queries = vec![
        "SELECT * FROM command_log".to_string(),
        "SELECT * FROM player".to_string(),
        "SELECT * FROM session".to_string()
    ];
    conn.subscription_builder()
        .subscribe(queries);
    
    Ok(conn)
}

/// Create a connection and start the background message processor
/// 
/// Returns the connection wrapped in Arc for sharing across async tasks
pub async fn create_connection_with_processor(
    uri: &str,
    module_name: &str,
) -> Result<std::sync::Arc<DbConnection>, Box<dyn std::error::Error>> {
    let conn = create_connection(uri, module_name)?;
    let conn = std::sync::Arc::new(conn);
    
    // Spawn background task to process WebSocket messages
    let conn_clone = conn.clone();
    tokio::spawn(async move {
        let _ = conn_clone.run_async().await;
    });
    
    Ok(conn)
}
