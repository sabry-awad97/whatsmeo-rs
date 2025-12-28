//! Basic WhatsApp client example with callbacks

use colored::*;
use qrcode::render::unicode;
use qrcode::QrCode;
use whatsmeow::{init_tracing, WhatsApp};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    println!("🚀 Starting WhatsApp client...");
    println!("   Press Ctrl+C to exit gracefully\n");

    let client = WhatsApp::connect("storage/session.db")
        .on_qr(|qr| {
            if let Some(code) = qr.code() {
                display_qr_code(code);
            }
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
        .build()
        .await?;

    // Handle Ctrl+C gracefully
    tokio::select! {
        result = client.run() => {
            result?;
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\n👋 Shutting down gracefully...");
            client.disconnect();
        }
    }

    Ok(())
}

/// Renders a stunning QR code in the terminal with beautiful styling
pub fn render_qr_code(qr_data: &str) {
    match QrCode::new(qr_data.as_bytes()) {
        Ok(code) => {
            // Render the QR code with high-density Unicode characters for better clarity
            let image = code
                .render::<unicode::Dense1x2>()
                .dark_color(unicode::Dense1x2::Light)
                .light_color(unicode::Dense1x2::Dark)
                .build();

            let lines: Vec<&str> = image.lines().collect();
            let width = lines[0].chars().count();

            // Create a beautiful header with gradient-like effect
            println!();
            println!(
                "    {}",
                "✨ WhatsApp Authentication QR Code ✨".bright_cyan().bold()
            );
            println!();

            // Create decorative top border with double lines
            print!("    {}", "╔".bright_magenta().bold());
            for i in 0..width + 4 {
                if i % 4 == 0 {
                    print!("{}", "═".bright_cyan().bold());
                } else {
                    print!("{}", "═".bright_blue());
                }
            }
            println!("{}", "╗".bright_magenta().bold());

            // Add padding line with side decorations
            println!(
                "    {}{}{}",
                "║".bright_magenta().bold(),
                " ".repeat(width + 4),
                "║".bright_magenta().bold()
            );

            // Print QR code with beautiful side borders and padding
            for (i, line) in lines.iter().enumerate() {
                let left_border = if i % 3 == 0 {
                    "║".bright_magenta().bold()
                } else {
                    "║".bright_blue()
                };
                let right_border = if i % 3 == 0 {
                    "║".bright_magenta().bold()
                } else {
                    "║".bright_blue()
                };

                println!("    {}  {}  {}", left_border, line, right_border);
            }

            // Add padding line
            println!(
                "    {}{}{}",
                "║".bright_magenta().bold(),
                " ".repeat(width + 4),
                "║".bright_magenta().bold()
            );

            // Create decorative bottom border
            print!("    {}", "╚".bright_magenta().bold());
            for i in 0..width + 4 {
                if i % 4 == 0 {
                    print!("{}", "═".bright_cyan().bold());
                } else {
                    print!("{}", "═".bright_blue());
                }
            }
            println!("{}", "╝".bright_magenta().bold());

            // Add beautiful footer with instructions
            println!();
            println!(
                "    {}",
                "📱 Open WhatsApp → Settings → Linked Devices → Link a Device".bright_yellow()
            );
            println!(
                "    {}",
                "⚡ Scan the QR code above to connect instantly!"
                    .bright_green()
                    .bold()
            );
            println!();
        }
        Err(e) => {
            println!(
                "    {}",
                "❌ Failed to generate QR code".bright_red().bold()
            );
            println!("    {}", format!("Error: {}", e).bright_red());
            println!("    {}", "📝 Raw QR data:".bright_yellow());
            println!("    {}", qr_data.bright_white());
        }
    }
}

/// Displays a beautifully formatted QR code with enhanced styling
fn display_qr_code(qr_data: &str) {
    // Create a stunning header with animated-like border
    println!();
    println!("{}", "╔".repeat(80).bright_magenta().bold());
    println!(
        "{}",
        "    � WhatsApp FFI Authentication Portal 🚀"
            .bright_cyan()
            .bold()
    );
    println!("{}", "╚".repeat(80).bright_magenta().bold());
    println!();

    // Display step-by-step instructions with beautiful formatting
    println!("{}", "📋 Quick Setup Guide:".bright_yellow().bold());
    println!(
        "   {} Open WhatsApp on your mobile device",
        "1️⃣".bright_green()
    );
    println!(
        "   {} Navigate to Settings → Linked Devices",
        "2️⃣".bright_green()
    );
    println!("   {} Tap 'Link a Device' button", "3️⃣".bright_green());
    println!(
        "   {} Scan the beautiful QR code below",
        "4️⃣".bright_green()
    );
    println!();

    // Render the beautiful QR code
    render_qr_code(qr_data);

    // Add a beautiful waiting message with animation-like effect
    println!(
        "{}",
        "⏳ Waiting for authentication...".bright_yellow().bold()
    );
    println!(
        "{}",
        "✨ Your connection will be established automatically once scanned!".bright_green()
    );
    println!();
    println!("{}", "═".repeat(80).bright_blue());
    println!();
}
