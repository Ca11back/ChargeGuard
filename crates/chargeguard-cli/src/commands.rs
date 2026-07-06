use std::collections::HashMap;

use chargeguard_core::{
    BatteryDevice, ChargeGuardError, Diagnostics, INTERFACE_NAME, OBJECT_PATH, SERVICE_NAME,
};
use clap::{Parser, Subcommand};
use serde::de::DeserializeOwned;
use thiserror::Error;
use zbus::zvariant::OwnedValue;

pub type Result<T> = std::result::Result<T, CliError>;
type PropertyMap = HashMap<String, OwnedValue>;

#[derive(Debug, Parser)]
#[command(name = "chargeguard")]
#[command(about = "Manage native Linux battery charging modes")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    List,
    Current {
        device: Option<String>,
    },
    Modes {
        device: Option<String>,
    },
    Set {
        mode: String,
        #[arg(long)]
        device: Option<String>,
    },
    Doctor {
        #[arg(long)]
        json: bool,
    },
    Raw {
        device: Option<String>,
    },
}

pub struct ChargeGuardClient {
    connection: zbus::Connection,
}

impl ChargeGuardClient {
    pub async fn connect() -> Result<Self> {
        let connection = zbus::Connection::system()
            .await
            .map_err(|error| ChargeGuardError::ServiceUnavailable(error.to_string()))?;

        Ok(Self { connection })
    }

    pub async fn list_devices(&self) -> Result<Vec<BatteryDevice>> {
        let reply = self
            .connection
            .call_method(
                Some(SERVICE_NAME),
                OBJECT_PATH,
                Some(INTERFACE_NAME),
                "ListDevices",
                &(),
            )
            .await?;
        let maps = reply.body().deserialize::<Vec<PropertyMap>>()?;
        maps.iter().map(from_json_field).collect()
    }

    pub async fn get_device(&self, id: &str) -> Result<BatteryDevice> {
        let reply = self
            .connection
            .call_method(
                Some(SERVICE_NAME),
                OBJECT_PATH,
                Some(INTERFACE_NAME),
                "GetDevice",
                &(id),
            )
            .await?;
        let map = reply.body().deserialize::<PropertyMap>()?;
        from_json_field(&map)
    }

    pub async fn set_native_mode(&self, id: &str, mode: &str) -> Result<BatteryDevice> {
        let reply = self
            .connection
            .call_method(
                Some(SERVICE_NAME),
                OBJECT_PATH,
                Some(INTERFACE_NAME),
                "SetNativeMode",
                &(id, mode),
            )
            .await?;
        let map = reply.body().deserialize::<PropertyMap>()?;
        from_json_field(&map)
    }

    pub async fn diagnostics(&self) -> Result<Diagnostics> {
        let reply = self
            .connection
            .call_method(
                Some(SERVICE_NAME),
                OBJECT_PATH,
                Some(INTERFACE_NAME),
                "GetDiagnostics",
                &(),
            )
            .await?;
        let map = reply.body().deserialize::<PropertyMap>()?;
        from_json_field(&map)
    }

    pub async fn resolve_device(&self, device: Option<String>) -> Result<BatteryDevice> {
        let id = self.resolve_device_id(device).await?;
        self.get_device(&id).await
    }

    pub async fn resolve_device_id(&self, device: Option<String>) -> Result<String> {
        if let Some(device) = device {
            return Ok(device);
        }

        let devices = self.list_devices().await?;
        match devices.as_slice() {
            [] => Err(ChargeGuardError::UnsupportedDevice(
                "no present batteries were discovered".to_string(),
            )
            .into()),
            [device] => Ok(device.id.clone()),
            _ => Err(CliError::AmbiguousDevice),
        }
    }
}

fn from_json_field<T: DeserializeOwned>(map: &PropertyMap) -> Result<T> {
    let value = map
        .get("json")
        .ok_or_else(|| CliError::Protocol("missing D-Bus json field".to_string()))?;
    let json = String::try_from(value.clone())
        .map_err(|error| CliError::Protocol(format!("invalid D-Bus json field: {error}")))?;
    Ok(serde_json::from_str(&json)?)
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Core(#[from] ChargeGuardError),

    #[error("D-Bus error: {0}")]
    Zbus(#[from] zbus::Error),

    #[error("D-Bus decode error: {0}")]
    Zvariant(#[from] zbus::zvariant::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("multiple batteries were discovered; pass --device or DEVICE")]
    AmbiguousDevice,

    #[error("{0}")]
    Protocol(String),
}

impl CliError {
    pub fn code(&self) -> &'static str {
        match self {
            CliError::Core(error) => error.code(),
            CliError::Zbus(_) | CliError::Protocol(_) | CliError::Zvariant(_) => {
                "service_unavailable"
            }
            CliError::Json(_) => "internal_error",
            CliError::AmbiguousDevice => "invalid_mode",
        }
    }
}
