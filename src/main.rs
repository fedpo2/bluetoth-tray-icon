use tray_item::TrayItem;
use std::process::Command;
use std::env;


#[derive(Debug, Clone)]
struct BluetoothDevice {
    mac: String,
    name: String,
    connected: bool,
    paired: bool,
}

fn main() {
    let _ = gtk::init();

    let initial_state = get_bluetooth_status();
    let icon_name = if initial_state { "bluetooth" } else { "bluetooth-disabled" };

    let mut tray = TrayItem::new("Bluetooth Manager", tray_item::IconSource::Resource(icon_name))
        .expect("Failed to create tray item");

    build_tray_menu(&mut tray, initial_state);

    println!("Aplicación de Bluetooth iniciada. Estado inicial: {}", 
             if initial_state { "Encendido" } else { "Apagado" });
}

fn restart(){
    let current_exe = env::current_exe().unwrap();
    Command::new(current_exe)
        .spawn()
        .expect("Error al reiniciar la aplicación");
    std::process::exit(0);
}

fn build_tray_menu(tray: &mut TrayItem, bluetooth_enabled: bool) {
    let status_text = if bluetooth_enabled { "Bluetooth: Encendido" } else { "Bluetooth: Apagado" };
    tray.add_label(status_text).unwrap();

    tray.add_menu_item("Alternar Bluetooth", move || {
        let current_state = get_bluetooth_status();
        let new_state = !current_state;

        let success = if new_state {
            enable_bluetooth()
        } else {
            disable_bluetooth()
        };

        if success {
            let status = if new_state { "Encendido" } else { "Apagado" };
            println!("Bluetooth: {}", status);
            show_notification(&format!("Bluetooth {} - Reinicia la app para ver cambios", status));
            restart();
        } else {
            println!("Error: No se pudo cambiar el estado del Bluetooth");
            show_notification("Error al cambiar estado del Bluetooth");
        }
    }).unwrap();

    if bluetooth_enabled {
        add_devices_submenu(tray);
    } else {
        tray.add_label("(Bluetooth deshabilitado)").unwrap();
    }

    tray.add_menu_item("Reiniciar App", || {
        println!("Reiniciando aplicación...");
        show_notification("Reiniciando aplicación...");
        restart();
    }).unwrap();

    tray.add_menu_item("Salir", || {
        println!("Cerrando aplicación...");
        std::process::exit(0);
    }).unwrap();

    println!("Aplicación de Bluetooth iniciada. Estado inicial: {}", 
             if get_bluetooth_status() { "Encendido" } else { "Apagado" });

    loop {
        gtk::main_iteration_do(false);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn get_bluetooth_status() -> bool {
    if let Ok(output) = Command::new("bluetoothctl")
        .args(&["show"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        return output_str.contains("Powered: yes");
    }
    false
}

fn show_notification(message: &str) {
    // Try to show a desktop notification
    let _ = Command::new("notify-send")
        .args(&["Bluetooth Manager", message])
        .status();
}

fn get_bluetooth_devices() -> Vec<BluetoothDevice> {
    let mut devices = Vec::new();

    // Get paired devices
    if let Ok(output) = Command::new("bluetoothctl")
        .args(&["devices", "Paired"])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            if let Some(device) = parse_device_line(line, true) {
                devices.push(device);
            }
        }
    }

    for device in &mut devices {
        device.connected = is_device_connected(&device.mac);
    }

    devices
}

fn parse_device_line(line: &str, is_paired: bool) -> Option<BluetoothDevice> {
    // Format: "Device XX:XX:XX:XX:XX:XX Device Name"
    if line.starts_with("Device ") {
        let parts: Vec<&str> = line.splitn(3, ' ').collect();
        if parts.len() >= 3 {
            let mac = parts[1].to_string();
            let name = parts[2].to_string();

            return Some(BluetoothDevice {
                mac,
                name,
                connected: false,
                paired: is_paired,
            });
        }
    }
    None
}

fn is_device_connected(mac: &str) -> bool {
    if let Ok(output) = Command::new("bluetoothctl")
        .args(&["info", mac])
        .output()
    {
        let output_str = String::from_utf8_lossy(&output.stdout);
        return output_str.contains("Connected: yes");
    }
    false
}

fn enable_bluetooth() -> bool {
    let _ = Command::new("bluetoothctl")
        .args(&["power", "on"])
        .status();
    return true;
}

fn disable_bluetooth() -> bool {
    // Try to power off the adapter first
    let _ = Command::new("bluetoothctl")
        .args(&["power", "off"])
        .status();
    return true;
}

fn add_devices_submenu(tray: &mut TrayItem) {
    let devices = get_bluetooth_devices();

    if devices.is_empty() {
        tray.add_label("No hay dispositivos").unwrap();
        return;
    }

    tray.add_label("=== DISPOSITIVOS ===").unwrap();

    for device in devices {
        let status_icons = format!("{}{}",
                                   if device.paired { "🔗" } else { "❌" },
                                   if device.connected { "📱" } else { "💤" }
        );

        let menu_text = format!("{} {}", status_icons, device.name);
        let device_mac = device.mac.clone();
        let device_connected = device.connected;

        tray.add_menu_item(&menu_text, move || {
            if device_connected {
                println!("Desconectando dispositivo: {}", device_mac);
                if disconnect_device(&device_mac) {
                    show_notification(&format!("Dispositivo {} desconectado", device_mac));
                    restart();
                } else {
                    show_notification("Error al desconectar dispositivo");
                }
            } else {
                println!("Conectando dispositivo: {}", device_mac);
                if connect_device(&device_mac) {
                    show_notification(&format!("Dispositivo {} conectado", device_mac));
                    restart();
                } else {
                    show_notification("Error al conectar dispositivo");
                }
            }
        }).unwrap();
    }
}

fn connect_device(mac: &str) -> bool {
    Command::new("bluetoothctl")
        .args(&["connect", mac])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn disconnect_device(mac: &str) -> bool {
    Command::new("bluetoothctl")
        .args(&["disconnect", mac])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
