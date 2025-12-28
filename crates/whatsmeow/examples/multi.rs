//! Multi-client example - managing multiple WhatsApp accounts

use whatsmeow::{init_tracing, WhatsAppManager};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    println!("🧠 Starting multi-client manager...");

    let manager = WhatsAppManager::new();

    // Spawn first bot
    let bot1 = manager
        .spawn("bot-1", "bot1.db")?
        .on_qr(|qr| {
            println!("[Bot1] 📱 QR: {:?}", qr.code());
        })
        .on_message(|msg| {
            println!("[Bot1] 📩 {}: {}", msg.sender_name(), msg.text());
        });

    // Spawn second bot
    let bot2 = manager
        .spawn("bot-2", "bot2.db")?
        .on_qr(|qr| {
            println!("[Bot2] 📱 QR: {:?}", qr.code());
        })
        .on_message(|msg| {
            println!("[Bot2] 📩 {}: {}", msg.sender_name(), msg.text());
        });

    // Run both in parallel
    let (r1, r2) = tokio::join!(bot1.run(), bot2.run());

    r1?;
    r2?;

    // Shutdown all
    manager.shutdown_all();

    println!("👋 All clients shut down");

    Ok(())
}
