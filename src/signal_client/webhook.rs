// signal_client/webhook.rs
//
// HTTP webhook server to receive Signal messages from signal-cli-rest-api

use actix_web::{web, App, HttpResponse, HttpServer};
use serde::Deserialize;
use super::bot::SignalBot;

/// Signal message envelope from signal-cli-rest-api
#[derive(Debug, Deserialize)]
pub struct SignalWebhook {
    envelope: MessageEnvelope,
}

#[derive(Debug, Deserialize)]
pub struct MessageEnvelope {
    source: String,           // Sender's phone number
    #[serde(rename = "sourceName")]
    source_name: Option<String>,
    #[serde(rename = "sourceUuid")]
    source_uuid: Option<String>,
    #[serde(rename = "dataMessage")]
    data_message: Option<DataMessage>,
    #[serde(rename = "syncMessage")]
    sync_message: Option<SyncMessage>,
}

#[derive(Debug, Deserialize)]
pub struct DataMessage {
    timestamp: u64,
    message: Option<String>,
    #[serde(rename = "groupInfo")]
    group_info: Option<GroupInfo>,
}

#[derive(Debug, Deserialize)]
pub struct SyncMessage {
    // Sync messages (sent from another device)
    // We'll ignore these for now
}

#[derive(Debug, Deserialize)]
pub struct GroupInfo {
    #[serde(rename = "groupId")]
    group_id: String,
}

/// Webhook endpoint handler
async fn handle_webhook(
    payload: web::Json<SignalWebhook>,
    bot: web::Data<SignalBot>,
) -> HttpResponse {
    log::debug!("Received webhook: {:?}", payload);
    
    let envelope = &payload.envelope;
    
    // Ignore sync messages (messages from our own devices)
    if envelope.sync_message.is_some() {
        return HttpResponse::Ok().finish();
    }
    
    // Extract data message
    let data_msg = match &envelope.data_message {
        Some(msg) => msg,
        None => {
            log::debug!("No data message in envelope");
            return HttpResponse::Ok().finish();
        }
    };
    
    // Ignore group messages for now
    if data_msg.group_info.is_some() {
        log::debug!("Ignoring group message");
        return HttpResponse::Ok().finish();
    }
    
    // Extract text
    let text = match &data_msg.message {
        Some(txt) if !txt.trim().is_empty() => txt.trim(),
        _ => {
            log::debug!("Empty or missing message text");
            return HttpResponse::Ok().finish();
        }
    };
    
    let sender = &envelope.source;
    
    // Handle message directly (no spawn for now to avoid Send issues)
    bot.handle_message(sender, text).await;
    
    HttpResponse::Ok().finish()
}

/// Health check endpoint
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().body("Signal bot is running")
}

/// Start the webhook server
pub async fn start_webhook_server(bot: SignalBot) -> Result<(), std::io::Error> {
    log::info!("Starting webhook server on 0.0.0.0:3001");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(bot.clone()))
            .route("/signal/webhook", web::post().to(handle_webhook))
            .route("/health", web::get().to(health_check))
    })
    .bind("0.0.0.0:3001")?
    .run()
    .await
}
