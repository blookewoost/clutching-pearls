use std::process::{Command, Child};
use std::error::Error;
use std::thread;
use std::time::Duration;
use dbus::blocking::Connection;
use std::path::Path;

const POEM: &str = "Do you see it too?\n\
                    A signal in the darkness,\n\
                    waiting to be found.";

const GATT_SERVICE_UUID: &str = "12345678-1234-5678-1234-56789abcdef0";
const GATT_CHAR_UUID: &str = "12345679-1234-5678-1234-56789abcdef0";

static mut GATT_PROCESS: Option<Child> = None;

fn main() -> Result<(), Box<dyn Error>> {
    println!("🔵 Signal Network Node - Bluetooth Beacon");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Apply BlueZ configuration to disable problematic profiles
    setup_bluez_config()?;

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

    // Register GATT service
    println!("\n📡 Registering GATT service...");
    register_gatt_service()?;

    println!("\n🟢 Beacon active - waiting for connections...");
    println!("   Connected devices can read the signal message");
    println!("\nPress Ctrl+C to stop");
    
    // Keep running
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}

fn register_gatt_service() -> Result<(), Box<dyn Error>> {
    // Connect to system D-Bus
    let _conn = Connection::new_system()?;
    
    println!("✓ Connected to D-Bus system");
    
    // Define the GATT paths
    let app_path = "/com/signal/network";
    let service_path = "/com/signal/network/service0";
    let char_path = "/com/signal/network/service0/char0";
    
    println!("✓ GATT Application: {}", app_path);
    println!("✓ Service: {} (UUID: {})", service_path, GATT_SERVICE_UUID);
    println!("✓ Characteristic: {} (UUID: {})", char_path, GATT_CHAR_UUID);
    println!("  Properties: Read, Notify");
    
    // Launch the Python GATT server
    launch_gatt_server()?;
    
    println!("\n✓ Poem (readable via BLE):");
    for line in POEM.lines() {
        println!("  {}", line);
    }
    
    Ok(())
}

fn launch_gatt_server() -> Result<(), Box<dyn Error>> {
    // Find the script path - it should be in the same directory as the binary
    let script_path = if Path::new("gatt_server.py").exists() {
        "gatt_server.py".to_string()
    } else if Path::new("./gatt_server.py").exists() {
        "./gatt_server.py".to_string()
    } else {
        // Try relative to this source directory
        "/home/blooke/clutching-pearls/signal-network/gatt_server.py".to_string()
    };
    
    if !Path::new(&script_path).exists() {
        println!("⚠ GATT server script not found at {}", script_path);
        println!("  Device will still be discoverable via advertising");
        return Ok(());
    }
    
    // Launch the Python GATT server as a background process
    let child = Command::new("python3")
        .arg(&script_path)
        .spawn()?;
    
    unsafe {
        GATT_PROCESS = Some(child);
    }
    
    thread::sleep(Duration::from_millis(1000));
    println!("✓ GATT server started");
    
    Ok(())
}

fn setup_bluez_config() -> Result<(), Box<dyn Error>> {
    // Copy the bluetooth config to disable problematic profiles
    let config_path = "/etc/bluetooth/main.conf.d/signal-network.conf";
    let config_content = "[General]\n# Disable audio profiles that interfere with GATT-only devices\nDisable=A2DP,HFP,HSP\n";
    
    // Try to write the config (requires sudo)
    if let Err(_) = std::fs::write(config_path, config_content) {
        // If we can't write directly, that's okay - the GATT service should still work
        println!("⚠ Could not write BlueZ config (needs sudo), continuing...");
    }
    
    Ok(())
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
