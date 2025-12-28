//! Basic WhatsApp client example with callbacks

use whatsmeow::{init_tracing, WhatsApp};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    println!("🚀 Starting WhatsApp client...");

    WhatsApp::connect("whatsapp.dll", "session.db")
        .on_qr(|qr| {
            println!("\n📱 Scan this QR code:\n{}", qr.code);
        })
        .on_connected(|_| {
            println!("✅ Connected to WhatsApp!");
        })
        .on_message(|msg| {
            println!("📩 {}: {}", msg.sender_name(), msg.text);
        })
        .on_disconnected(|_| {
            println!("❌ Disconnected");
        })
        .run()
        .await?;

    Ok(())
}
