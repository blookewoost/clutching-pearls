use std::process::Command;
use std::error::Error;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🔵 Signal Network Node - Bluetooth Beacon");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Check if Bluetooth service is active
    if !check_bluetooth_service() {
        println!("⚠ Bluetooth service not running, attempting to start...");
        start_bluetooth_service()?;
    }

    // Power on the Bluetooth adapter
    println!("Configuring Bluetooth adapter...");
    run_command("bluetoothctl", &["power", "on"])?;
    thread::sleep(Duration::from_millis(500));

    // Set device as discoverable
    run_command("bluetoothctl", &["discoverable", "on"])?;
    thread::sleep(Duration::from_millis(500));

    // Set alias (device name)
    let device_name = "Signal-Node-01";
    run_command("bluetoothctl", &["system-alias", device_name])?;
    thread::sleep(Duration::from_millis(500));

    // Print adapter info
    println!("\n✓ Bluetooth adapter configured");
    println!("  Device name: {}", device_name);
    
    // Show current adapter info
    println!("\nAdapter info:");
    run_command("bluetoothctl", &["show"])?;

    println!("\n🟢 Beacon active - waiting for connections...");
    println!("   Other devices should see '{}' as a discoverable Bluetooth device", device_name);
    println!("\nPress Ctrl+C to stop");
    
    // Keep running
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn check_bluetooth_service() -> bool {
    let output = Command::new("systemctl")
        .args(["is-active", "bluetooth"])
        .output();
    
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

fn start_bluetooth_service() -> Result<(), Box<dyn Error>> {
    let status = Command::new("sudo")
        .args(&["systemctl", "start", "bluetooth"])
        .status()?;
    
    if status.success() {
        println!("✓ Bluetooth service started");
    }
    
    Ok(())
}

fn run_command(program: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
    let output = Command::new(program)
        .args(args)
        .output()?;
    
    if !output.status.success() {
        eprintln!("Error running {}: {}", program, String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(())
}
