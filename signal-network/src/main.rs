use std::process::Command;
use std::thread;
use std::time::Duration;

const POEM: &str = "Do you see it too? A signal in the darkness, waiting to be found.";

fn main() {
    println!("🔵 Signal Network Node - Bluetooth Beacon");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Power on Bluetooth
    Command::new("bluetoothctl").args(&["power", "on"]).output().ok();
    thread::sleep(Duration::from_millis(500));

    // Make discoverable
    Command::new("bluetoothctl").args(&["discoverable", "on"]).output().ok();
    thread::sleep(Duration::from_millis(500));

    // Broadcast the poem as the device name
    Command::new("bluetoothctl").args(&["system-alias", POEM]).output().ok();
    thread::sleep(Duration::from_millis(500));

    println!("📡 Broadcasting signal:");
    println!("  {}\n", POEM);
    println!("🟢 Beacon active - Press Ctrl+C to stop");

    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}
