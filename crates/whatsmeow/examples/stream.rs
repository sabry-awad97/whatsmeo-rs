//! Stream-based event handling example with tokio::spawn

use futures::StreamExt;
use whatsmeow::{Event, WhatsApp, init_tracing};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    println!("📡 Starting WhatsApp client (stream mode)...");
    println!("   Press Ctrl+C to exit gracefully\n");

    let client = WhatsApp::connect("storage/session.db").build().await?;

    // Clone client for the event loop task
    let client_clone = client.clone();

    // Spawn the internal event pump in a background task
    let run_handle = tokio::spawn(async move {
        if let Err(e) = client_clone.run().await {
            eprintln!("Event loop error: {}", e);
        }
    });

    // Process events from the stream
    let mut events = client.events();

    println!("🔄 Listening for events...");

    loop {
        tokio::select! {
            Some(event) = events.next() => {
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
                            if msg.info.is_from_me && text.starts_with("!echo ") {
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
            _ = tokio::signal::ctrl_c() => {
                println!("\n👋 Shutting down gracefully...");
                client.disconnect();
                break;
            }
        }
    }

    // Wait for the run task to finish
    let _ = run_handle.await;

    Ok(())
}
