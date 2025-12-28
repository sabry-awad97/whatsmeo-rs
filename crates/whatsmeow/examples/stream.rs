//! Stream-based event handling example

use futures::StreamExt;
use whatsmeow::{init_tracing, Event, WhatsApp};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    println!("📡 Starting WhatsApp client (stream mode)...");

    let client = WhatsApp::connect("whatsapp.dll", "session.db")
        .build()
        .await?;

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
                println!("📩 {}: {}", msg.sender_name(), msg.text);

                // Echo messages that start with "!echo "
                if msg.text.starts_with("!echo ") {
                    let reply = msg.text.strip_prefix("!echo ").unwrap();
                    if let Err(e) = client.send(&msg.from, reply) {
                        eprintln!("Failed to send reply: {}", e);
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
