use chargeguard_core::{
    BatteryDevice, BatteryHealth, ChargeGuardError, Diagnostics, RawPowerSupplyEntry,
};

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
        native_mode_source_display(device).unwrap_or_else(|| "none".to_string())
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
    if let Some(power) = &device.power {
        println!("Power: {}", format_watts(power.microwatts));
    }
    println!(
        "Native mode source: {}",
        native_mode_source_display(device).unwrap_or_else(|| "none".to_string())
    );
    if let Some(health) = &device.health {
        print_battery_health(health);
    }
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

fn native_mode_source_display(device: &BatteryDevice) -> Option<String> {
    device
        .native_mode
        .source_path
        .as_ref()
        .map(|path| path.display().to_string())
        .or_else(|| device.native_mode.source.clone())
}

fn print_battery_health(health: &BatteryHealth) {
    if let Some(cycle_count) = health.cycle_count {
        println!("Cycle count: {cycle_count}");
    }
    if let Some(capacity_level) = &health.capacity_level {
        println!("Capacity level: {capacity_level}");
    }

    let has_energy = health.full_energy_microwatt_hours.is_some()
        || health.design_energy_microwatt_hours.is_some();
    if has_energy {
        if let Some(full_energy) = health.full_energy_microwatt_hours {
            println!("Full energy: {}", format_watt_hours(full_energy));
        }
        if let Some(design_energy) = health.design_energy_microwatt_hours {
            println!("Design energy: {}", format_watt_hours(design_energy));
        }
        if let Some(percent) = health_percentage(
            health.full_energy_microwatt_hours,
            health.design_energy_microwatt_hours,
        ) {
            println!("Battery health: {percent:.1}%");
        }
        return;
    }

    if let Some(full_charge) = health.full_charge_microamp_hours {
        println!("Full charge: {}", format_amp_hours(full_charge));
    }
    if let Some(design_charge) = health.design_charge_microamp_hours {
        println!("Design charge: {}", format_amp_hours(design_charge));
    }
    if let Some(percent) = health_percentage(
        health.full_charge_microamp_hours,
        health.design_charge_microamp_hours,
    ) {
        println!("Battery health: {percent:.1}%");
    }
}

fn health_percentage(full: Option<u64>, design: Option<u64>) -> Option<f64> {
    match (full, design) {
        (Some(full), Some(design)) if design > 0 => Some(full as f64 * 100.0 / design as f64),
        _ => None,
    }
}

fn format_watts(microwatts: u64) -> String {
    let rounded_tenths = (microwatts + 50_000) / 100_000;
    if rounded_tenths == 0 {
        return "0 W".to_string();
    }

    if rounded_tenths < 100 && !rounded_tenths.is_multiple_of(10) {
        return format!("{}.{} W", rounded_tenths / 10, rounded_tenths % 10);
    }

    format!("{} W", (rounded_tenths + 5) / 10)
}

fn format_watt_hours(microwatt_hours: u64) -> String {
    format_micro_units(microwatt_hours, "Wh")
}

fn format_amp_hours(microamp_hours: u64) -> String {
    format_micro_units(microamp_hours, "Ah")
}

fn format_micro_units(value: u64, unit: &str) -> String {
    let rounded_hundredths = (value + 5_000) / 10_000;
    let whole = rounded_hundredths / 100;
    let fractional = rounded_hundredths % 100;

    if fractional == 0 {
        return format!("{whole} {unit}");
    }

    if fractional.is_multiple_of(10) {
        return format!("{}.{} {unit}", whole, fractional / 10);
    }

    format!("{whole}.{fractional:02} {unit}")
}
