use chargeguard_core::{BatteryDevice, ChargeGuardError, Diagnostics, RawPowerSupplyEntry};

use crate::commands::Result;

pub fn print_device_list(devices: &[BatteryDevice]) {
    if devices.is_empty() {
        println!("No present batteries discovered.");
        return;
    }

    for device in devices {
        print_device(device);
    }
}

pub fn print_current(device: &BatteryDevice) {
    println!("Device: {}", device.id);
    match &device.native_mode.current {
        Some(current) => println!("Current native mode: {current}"),
        None => println!("Current native mode: unavailable"),
    }
    if let Some(label) = &device.native_mode.current_label {
        println!("Display label: {label}");
    }
}

pub fn print_modes(device: &BatteryDevice) {
    println!("Device: {}", device.id);
    println!(
        "Native mode source: {}",
        device.native_mode.source.as_deref().unwrap_or("none")
    );

    if device.native_mode.supported_modes.is_empty() {
        println!(
            "{}",
            device
                .native_mode
                .message
                .as_deref()
                .unwrap_or("Supported native modes cannot be enumerated.")
        );
        return;
    }

    println!("Supported native modes:");
    for mode in &device.native_mode.supported_modes {
        let marker = if Some(mode.raw.as_str()) == device.native_mode.current.as_deref() {
            "*"
        } else {
            "-"
        };
        println!("  {marker} {} ({})", mode.label, mode.raw);
    }
}

pub fn print_set_result(device: &BatteryDevice) {
    println!("Device: {}", device.id);
    println!(
        "Current native mode: {}",
        device
            .native_mode
            .current
            .as_deref()
            .unwrap_or("unavailable")
    );
}

pub fn print_diagnostics(diagnostics: &Diagnostics) {
    println!("ChargeGuard version: {}", diagnostics.version);
    println!(
        "Kernel version: {}",
        diagnostics.kernel_version.as_deref().unwrap_or("unknown")
    );
    println!(
        "Power supply root: {}",
        diagnostics.power_supply_root.display()
    );
    println!(
        "D-Bus service status: {}",
        diagnostics.service_status.as_deref().unwrap_or("unknown")
    );
    println!(
        "polkit available: {}",
        diagnostics
            .polkit_available
            .map(|value| value.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    );
    println!("Devices: {}", diagnostics.devices.len());
    for device in &diagnostics.devices {
        print_device(device);
    }

    if !diagnostics.informational_attributes.is_empty() {
        println!("Informational non-MVP attributes:");
        for attr in &diagnostics.informational_attributes {
            println!(
                "  {} {}={}",
                attr.device_id,
                attr.name,
                attr.value.as_deref().unwrap_or("<unreadable>")
            );
        }
    }
}

pub fn print_raw(diagnostics: &Diagnostics, device: Option<&str>) -> Result<()> {
    let entries: Vec<&RawPowerSupplyEntry> = diagnostics
        .raw_entries
        .iter()
        .filter(|entry| device.is_none_or(|id| entry.id == id))
        .collect();

    if entries.is_empty() {
        return Err(ChargeGuardError::UnsupportedDevice(
            device.unwrap_or("no power_supply entries").to_string(),
        )
        .into());
    }

    for entry in entries {
        println!("Device: {}", entry.id);
        println!("Path: {}", entry.path.display());
        for (name, value) in &entry.attributes {
            println!("  {name}: {}", value.as_deref().unwrap_or("<missing>"));
        }
    }

    Ok(())
}

fn print_device(device: &BatteryDevice) {
    println!("Device: {}", device.id);
    if let Some(status) = &device.status {
        println!("Status: {status}");
    }
    if let Some(capacity) = device.capacity {
        println!("Capacity: {capacity}%");
    }
    println!(
        "Native mode source: {}",
        device.native_mode.source.as_deref().unwrap_or("none")
    );
    println!(
        "Current native mode: {}",
        device
            .native_mode
            .current
            .as_deref()
            .unwrap_or("unavailable")
    );
    if !device.native_mode.supported_modes.is_empty() {
        println!("Supported native modes:");
        for mode in &device.native_mode.supported_modes {
            println!("  * {}", mode.raw);
        }
    } else if let Some(message) = &device.native_mode.message {
        println!("{message}");
    }
    println!();
}
