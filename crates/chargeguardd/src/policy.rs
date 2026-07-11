use std::collections::HashMap;

use chargeguard_core::{ChargeGuardError, POLKIT_ACTION_SET_NATIVE_MODE, Result};
use zbus::message::Header;
use zbus_polkit::policykit1::{AuthorityProxy, CheckAuthorizationFlags, Subject};

pub async fn authorize_set_native_mode(header: &Header<'_>) -> Result<()> {
    let subject = Subject::new_for_message_header(header).map_err(|error| {
        infrastructure_error("failed to identify D-Bus caller for polkit", error)
    })?;
    let connection = zbus::Connection::system().await.map_err(|error| {
        infrastructure_error("failed to connect to the system bus for polkit", error)
    })?;
    let proxy = AuthorityProxy::new(&connection).await.map_err(|error| {
        infrastructure_error("failed to create the polkit authority proxy", error)
    })?;
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
        .map_err(|error| infrastructure_error("polkit authorization check failed", error))?;

    authorization_decision(result.is_authorized)
}

fn infrastructure_error(context: &str, error: impl std::fmt::Display) -> ChargeGuardError {
    ChargeGuardError::Internal(format!("{context}: {error}"))
}

fn authorization_decision(is_authorized: bool) -> Result<()> {
    if is_authorized {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_decision_is_permission_denied() {
        let error = authorization_decision(false).unwrap_err();
        assert_eq!(error.code(), "permission_denied");
    }

    #[test]
    fn infrastructure_failure_is_internal() {
        let error = infrastructure_error("polkit authorization check failed", "transport");
        assert_eq!(error.code(), "internal_error");
        assert!(error.to_string().contains("transport"));
    }
}
