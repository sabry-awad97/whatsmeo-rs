//! Stream-based event handling example

use futures::StreamExt;
use whatsmeow::{Event, WhatsApp, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    println!("📡 Starting WhatsApp client (stream mode)...");

    let client = WhatsApp::connect("session.db").build().await?;

    let mut events = client.events();

    println!("🔄 Listening for events...");

    while let Some(event) = events.next().await {
        match event {
            Event::Qr(qr) => {
                println!("\n📱 Scan this QR code:");
                if let Some(code) = qr.code() {
                    println!("{}", code);
                }
            }
            Event::PairSuccess(info) => {
                println!("🔗 Paired with: {} ({})", info.business_name, info.platform);
            }
            Event::Connected => {
                println!("✅ Connected to WhatsApp!");
            }
            Event::Message(msg) => {
                let text = msg.text();
                if !text.is_empty() {
                    println!("📩 {}: {}", msg.sender_name(), text);

                    // Echo messages that start with "!echo "
                    if text.starts_with("!echo ") {
                        let reply = text.strip_prefix("!echo ").unwrap();
                        if let Err(e) = client.send(&msg.info.chat, reply) {
                            eprintln!("Failed to send reply: {}", e);
                        }
                    }
                }
            }
            Event::Receipt(receipt) => {
                println!(
                    "📬 Receipt: {:?} -> {}",
                    receipt.message_ids, receipt.receipt_type
                );
            }
            Event::Presence(presence) => {
                let status = if presence.is_online() {
                    "online"
                } else {
                    "offline"
                };
                println!("👤 {}: {}", presence.from, status);
            }
            Event::Disconnected => {
                println!("❌ Disconnected, exiting...");
                break;
            }
            Event::LoggedOut(info) => {
                println!("🚪 Logged out (reason: {})", info.reason);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
