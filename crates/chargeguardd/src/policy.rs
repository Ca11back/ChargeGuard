use std::collections::HashMap;

use chargeguard_core::{ChargeGuardError, POLKIT_ACTION_SET_NATIVE_MODE, Result};
use zbus::message::Header;
use zbus_polkit::policykit1::{AuthorityProxy, CheckAuthorizationFlags, Subject};

pub async fn authorize_set_native_mode(header: &Header<'_>) -> Result<()> {
    let subject = Subject::new_for_message_header(header)
        .map_err(|error| ChargeGuardError::PermissionDenied(error.to_string()))?;
    let connection = zbus::Connection::system()
        .await
        .map_err(|error| ChargeGuardError::PermissionDenied(error.to_string()))?;
    let proxy = AuthorityProxy::new(&connection)
        .await
        .map_err(|error| ChargeGuardError::PermissionDenied(error.to_string()))?;
    let details = HashMap::new();
    let result = proxy
        .check_authorization(
            &subject,
            POLKIT_ACTION_SET_NATIVE_MODE,
            &details,
            CheckAuthorizationFlags::AllowUserInteraction.into(),
            "",
        )
        .await
        .map_err(|error| ChargeGuardError::PermissionDenied(error.to_string()))?;

    if result.is_authorized {
        Ok(())
    } else {
        Err(ChargeGuardError::PermissionDenied(
            "polkit authorization was denied".to_string(),
        ))
    }
}

pub async fn polkit_available() -> bool {
    let Ok(connection) = zbus::Connection::system().await else {
        return false;
    };

    AuthorityProxy::new(&connection).await.is_ok()
}
