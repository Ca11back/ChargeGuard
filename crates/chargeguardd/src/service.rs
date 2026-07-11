use chargeguard_core::{ChargeGuardError, Diagnostics, PowerSupply, SysfsPowerSupply};
use tracing::{debug, warn};
use zbus::{DBusError, interface, message::Header};

use crate::{
    dbus::{PropertyMap, device_to_map, diagnostics_to_map},
    policy,
};

#[derive(Debug, Clone)]
pub struct ChargeGuardService {
    sysfs: SysfsPowerSupply,
}

#[derive(Debug, DBusError)]
#[zbus(prefix = "io.github.ChargeGuard1.Error")]
enum ChargeGuardDBusError {
    UnsupportedDevice(String),
    NoNativeModeInterface(String),
    NativeModesNotEnumerable(String),
    InvalidMode(String),
    PermissionDenied(String),
    SysfsReadFailed(String),
    SysfsWriteFailed(String),
    ReadbackMismatch(String),
    ServiceUnavailable(String),
    InternalError(String),
}

impl ChargeGuardService {
    pub fn new(sysfs: SysfsPowerSupply) -> Self {
        Self { sysfs }
    }

    async fn diagnostics(&self) -> Diagnostics {
        let mut diagnostics = self.sysfs.diagnostics(env!("CARGO_PKG_VERSION"));
        diagnostics.service_status = Some("available".to_string());
        diagnostics.polkit_available = Some(policy::polkit_available().await);
        diagnostics
    }
}

#[interface(name = "io.github.ChargeGuard1.Manager")]
impl ChargeGuardService {
    #[zbus(name = "ListDevices")]
    async fn list_devices(&self) -> std::result::Result<Vec<PropertyMap>, ChargeGuardDBusError> {
        debug!("listing chargeguard devices");
        self.sysfs
            .discover_devices()
            .and_then(|devices| {
                devices
                    .iter()
                    .map(|device| {
                        device_to_map(device).map_err(|error| {
                            ChargeGuardError::Internal(format!(
                                "failed to encode D-Bus device payload: {error}"
                            ))
                        })
                    })
                    .collect()
            })
            .map_err(to_dbus_error)
    }

    #[zbus(name = "GetDevice")]
    async fn get_device(&self, id: &str) -> std::result::Result<PropertyMap, ChargeGuardDBusError> {
        debug!(device_id = id, "getting chargeguard device");
        self.sysfs
            .get_device(id)
            .and_then(|device| {
                device_to_map(&device).map_err(|error| {
                    ChargeGuardError::Internal(format!(
                        "failed to encode D-Bus device payload: {error}"
                    ))
                })
            })
            .map_err(to_dbus_error)
    }

    #[zbus(name = "SetNativeMode")]
    async fn set_native_mode(
        &self,
        id: &str,
        mode: &str,
        #[zbus(header)] header: Header<'_>,
    ) -> std::result::Result<PropertyMap, ChargeGuardDBusError> {
        debug!(device_id = id, mode, "setting native charge mode");
        policy::authorize_set_native_mode(&header)
            .await
            .map_err(to_dbus_error)?;

        self.sysfs
            .set_native_mode(id, mode)
            .and_then(|device| {
                device_to_map(&device).map_err(|error| {
                    ChargeGuardError::Internal(format!(
                        "failed to encode D-Bus device payload: {error}"
                    ))
                })
            })
            .map_err(to_dbus_error)
    }

    #[zbus(name = "GetDiagnostics")]
    async fn get_diagnostics(&self) -> std::result::Result<PropertyMap, ChargeGuardDBusError> {
        debug!("getting chargeguard diagnostics");
        diagnostics_to_map(&self.diagnostics().await)
            .map_err(|error| to_dbus_error(ChargeGuardError::Internal(error.to_string())))
    }
}

fn to_dbus_error(error: ChargeGuardError) -> ChargeGuardDBusError {
    warn!(code = error.code(), error = %error, "D-Bus method failed");
    match error {
        ChargeGuardError::UnsupportedDevice(_) => {
            ChargeGuardDBusError::UnsupportedDevice(error.to_string())
        }
        ChargeGuardError::NoNativeModeInterface { .. } => {
            ChargeGuardDBusError::NoNativeModeInterface(error.to_string())
        }
        ChargeGuardError::NativeModesNotEnumerable { .. } => {
            ChargeGuardDBusError::NativeModesNotEnumerable(error.to_string())
        }
        ChargeGuardError::InvalidMode { .. } => {
            ChargeGuardDBusError::InvalidMode(error.to_string())
        }
        ChargeGuardError::PermissionDenied(_) => {
            ChargeGuardDBusError::PermissionDenied(error.to_string())
        }
        ChargeGuardError::SysfsReadFailed { .. } => {
            ChargeGuardDBusError::SysfsReadFailed(error.to_string())
        }
        ChargeGuardError::SysfsWriteFailed { .. } => {
            ChargeGuardDBusError::SysfsWriteFailed(error.to_string())
        }
        ChargeGuardError::ReadbackMismatch { .. } => {
            ChargeGuardDBusError::ReadbackMismatch(error.to_string())
        }
        ChargeGuardError::ServiceUnavailable(_) => {
            ChargeGuardDBusError::ServiceUnavailable(error.to_string())
        }
        ChargeGuardError::ParseNativeModes { .. } | ChargeGuardError::Internal(_) => {
            ChargeGuardDBusError::InternalError(error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{io, path::PathBuf};

    use super::*;

    fn dbus_name(error: ChargeGuardError) -> String {
        to_dbus_error(error).name().to_string()
    }

    #[test]
    fn maps_every_core_error_to_an_exact_wire_name() {
        let path = PathBuf::from("/sys/class/power_supply/BAT0/charge_type");
        let cases = [
            (
                ChargeGuardError::UnsupportedDevice("BAT0".to_string()),
                "io.github.ChargeGuard1.Error.UnsupportedDevice",
            ),
            (
                ChargeGuardError::NoNativeModeInterface {
                    device_id: "BAT0".to_string(),
                },
                "io.github.ChargeGuard1.Error.NoNativeModeInterface",
            ),
            (
                ChargeGuardError::NativeModesNotEnumerable {
                    device_id: "BAT0".to_string(),
                },
                "io.github.ChargeGuard1.Error.NativeModesNotEnumerable",
            ),
            (
                ChargeGuardError::InvalidMode {
                    device_id: "BAT0".to_string(),
                    requested: "Turbo".to_string(),
                },
                "io.github.ChargeGuard1.Error.InvalidMode",
            ),
            (
                ChargeGuardError::PermissionDenied("denied".to_string()),
                "io.github.ChargeGuard1.Error.PermissionDenied",
            ),
            (
                ChargeGuardError::SysfsReadFailed {
                    path: path.clone(),
                    source: io::Error::other("read failed"),
                },
                "io.github.ChargeGuard1.Error.SysfsReadFailed",
            ),
            (
                ChargeGuardError::SysfsWriteFailed {
                    path: path.clone(),
                    source: io::Error::other("write failed"),
                },
                "io.github.ChargeGuard1.Error.SysfsWriteFailed",
            ),
            (
                ChargeGuardError::ReadbackMismatch {
                    device_id: "BAT0".to_string(),
                    requested: "Fast".to_string(),
                    actual: Some("Standard".to_string()),
                },
                "io.github.ChargeGuard1.Error.ReadbackMismatch",
            ),
            (
                ChargeGuardError::ServiceUnavailable("missing".to_string()),
                "io.github.ChargeGuard1.Error.ServiceUnavailable",
            ),
            (
                ChargeGuardError::ParseNativeModes {
                    path: path.clone(),
                    message: "bad modes".to_string(),
                },
                "io.github.ChargeGuard1.Error.InternalError",
            ),
            (
                ChargeGuardError::Internal("unexpected".to_string()),
                "io.github.ChargeGuard1.Error.InternalError",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(dbus_name(error), expected);
        }
    }
}
