use whatsmeow::WhatsApp;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 Testing whatsmeow from crates.io...");

    // Initialize the client
    let _client = WhatsApp::connect("external.db")
        .on_qr(|qr| async move {
            if let Some(code) = qr.code() {
                println!("🔗 New QR Code: {}", code);
            }
        })
        .on_message(|msg| async move {
            println!("📩 Message from {}: {}", msg.sender_name(), msg.text());
        })
        .build()
        .await?;

    println!("📡 Connecting...");

    // In a real scenario, you'd call client.run().await
    // For this demonstration, we'll just check if the handle is valid
    println!("✅ Handled created successfully using crates.io version!");

    Ok(())
}
