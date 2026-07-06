use std::{io, path::PathBuf};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ChargeGuardError>;

#[derive(Debug, Error)]
pub enum ChargeGuardError {
    #[error("unsupported device: {0}")]
    UnsupportedDevice(String),

    #[error("device '{device_id}' has no native mode interface")]
    NoNativeModeInterface { device_id: String },

    #[error(
        "device '{device_id}' exposes current mode but writable native modes are not enumerable"
    )]
    NativeModesNotEnumerable { device_id: String },

    #[error("invalid mode '{requested}' for device '{device_id}'")]
    InvalidMode {
        device_id: String,
        requested: String,
    },

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("failed to read {path}: {source}")]
    SysfsReadFailed { path: PathBuf, source: io::Error },

    #[error("failed to write {path}: {source}")]
    SysfsWriteFailed { path: PathBuf, source: io::Error },

    #[error(
        "readback mismatch on device '{device_id}': requested '{requested}', actual '{actual:?}'"
    )]
    ReadbackMismatch {
        device_id: String,
        requested: String,
        actual: Option<String>,
    },

    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("failed to parse native modes from {path}: {message}")]
    ParseNativeModes { path: PathBuf, message: String },

    #[error("internal error: {0}")]
    Internal(String),
}

impl ChargeGuardError {
    pub fn code(&self) -> &'static str {
        match self {
            ChargeGuardError::UnsupportedDevice(_) => "unsupported_device",
            ChargeGuardError::NoNativeModeInterface { .. } => "no_native_mode_interface",
            ChargeGuardError::NativeModesNotEnumerable { .. } => "native_modes_not_enumerable",
            ChargeGuardError::InvalidMode { .. } => "invalid_mode",
            ChargeGuardError::PermissionDenied(_) => "permission_denied",
            ChargeGuardError::SysfsReadFailed { .. } => "sysfs_read_failed",
            ChargeGuardError::SysfsWriteFailed { .. } => "sysfs_write_failed",
            ChargeGuardError::ReadbackMismatch { .. } => "readback_mismatch",
            ChargeGuardError::ServiceUnavailable(_) => "service_unavailable",
            ChargeGuardError::ParseNativeModes { .. } | ChargeGuardError::Internal(_) => {
                "internal_error"
            }
        }
    }
}
