use chargeguard_core::{
    ChargeGuardError, Diagnostics, INTERFACE_NAME, PowerSupply, SysfsPowerSupply,
};
use tracing::{debug, warn};
use zbus::{fdo, interface, message::Header};

use crate::{
    dbus::{PropertyMap, device_to_map, diagnostics_to_map},
    policy,
};

#[derive(Debug, Clone)]
pub struct ChargeGuardService {
    sysfs: SysfsPowerSupply,
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
    async fn list_devices(&self) -> fdo::Result<Vec<PropertyMap>> {
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
            .map_err(to_fdo_error)
    }

    #[zbus(name = "GetDevice")]
    async fn get_device(&self, id: &str) -> fdo::Result<PropertyMap> {
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
            .map_err(to_fdo_error)
    }

    #[zbus(name = "SetNativeMode")]
    async fn set_native_mode(
        &self,
        id: &str,
        mode: &str,
        #[zbus(header)] header: Header<'_>,
    ) -> fdo::Result<PropertyMap> {
        debug!(device_id = id, mode, "setting native charge mode");
        policy::authorize_set_native_mode(&header)
            .await
            .map_err(to_fdo_error)?;

        self.sysfs
            .set_native_mode(id, mode)
            .and_then(|device| {
                device_to_map(&device).map_err(|error| {
                    ChargeGuardError::Internal(format!(
                        "failed to encode D-Bus device payload: {error}"
                    ))
                })
            })
            .map_err(to_fdo_error)
    }

    #[zbus(name = "GetDiagnostics")]
    async fn get_diagnostics(&self) -> fdo::Result<PropertyMap> {
        debug!("getting chargeguard diagnostics");
        diagnostics_to_map(&self.diagnostics().await)
            .map_err(|error| to_fdo_error(ChargeGuardError::Internal(error.to_string())))
    }
}

fn to_fdo_error(error: ChargeGuardError) -> fdo::Error {
    warn!(code = error.code(), error = %error, "D-Bus method failed");
    match error {
        ChargeGuardError::PermissionDenied(message) => fdo::Error::AccessDenied(message),
        ChargeGuardError::InvalidMode { .. } => fdo::Error::InvalidArgs(error.to_string()),
        ChargeGuardError::UnsupportedDevice(_)
        | ChargeGuardError::NoNativeModeInterface { .. }
        | ChargeGuardError::NativeModesNotEnumerable { .. } => {
            fdo::Error::NotSupported(error.to_string())
        }
        ChargeGuardError::ServiceUnavailable(message) => fdo::Error::ServiceUnknown(message),
        _ => fdo::Error::Failed(format!("{INTERFACE_NAME}: {error}")),
    }
}
