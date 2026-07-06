use std::collections::HashMap;

use chargeguard_core::{BatteryDevice, Diagnostics};
use zbus::zvariant::{OwnedValue, Value};

pub type PropertyMap = HashMap<String, OwnedValue>;

pub fn device_to_map(device: &BatteryDevice) -> serde_json::Result<PropertyMap> {
    let mut map = PropertyMap::new();
    insert_string(&mut map, "json", serde_json::to_string(device)?);
    insert_string(&mut map, "id", &device.id);
    insert_string(
        &mut map,
        "nativePath",
        device.native_path.display().to_string(),
    );
    insert_string(&mut map, "type", &device.type_name);
    insert_bool(&mut map, "present", device.present);
    insert_string(
        &mut map,
        "status",
        device.status.clone().unwrap_or_default(),
    );
    if let Some(capacity) = device.capacity {
        insert_u32(&mut map, "capacity", capacity.into());
    }
    insert_string(
        &mut map,
        "manufacturer",
        device.manufacturer.clone().unwrap_or_default(),
    );
    insert_string(
        &mut map,
        "modelName",
        device.model_name.clone().unwrap_or_default(),
    );
    insert_bool(
        &mut map,
        "nativeModeSupported",
        device.native_mode.supported,
    );
    insert_string(
        &mut map,
        "nativeModeSource",
        device.native_mode.source.clone().unwrap_or_default(),
    );
    insert_string(
        &mut map,
        "nativeModeCurrent",
        device.native_mode.current.clone().unwrap_or_default(),
    );
    insert_bool(&mut map, "nativeModeWritable", device.native_mode.writable);
    insert_string(
        &mut map,
        "nativeModeSupportedModesJson",
        serde_json::to_string(&device.native_mode.supported_modes)?,
    );
    Ok(map)
}

pub fn diagnostics_to_map(diagnostics: &Diagnostics) -> serde_json::Result<PropertyMap> {
    let mut map = PropertyMap::new();
    insert_string(&mut map, "json", serde_json::to_string(diagnostics)?);
    insert_string(&mut map, "version", &diagnostics.version);
    insert_string(
        &mut map,
        "kernelVersion",
        diagnostics.kernel_version.clone().unwrap_or_default(),
    );
    insert_string(
        &mut map,
        "powerSupplyRoot",
        diagnostics.power_supply_root.display().to_string(),
    );
    insert_u32(&mut map, "deviceCount", diagnostics.devices.len() as u32);
    Ok(map)
}

fn insert_string(map: &mut PropertyMap, key: &str, value: impl Into<String>) {
    let value = value.into();
    let owned = OwnedValue::try_from(Value::from(value.as_str()))
        .expect("string values are valid D-Bus variants");
    map.insert(key.to_string(), owned);
}

fn insert_bool(map: &mut PropertyMap, key: &str, value: bool) {
    map.insert(key.to_string(), OwnedValue::from(value));
}

fn insert_u32(map: &mut PropertyMap, key: &str, value: u32) {
    map.insert(key.to_string(), OwnedValue::from(value));
}
