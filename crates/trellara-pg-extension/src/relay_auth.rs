use serde::Serialize;

pub const RELAY_AUTH_CONTRACT: &str =
    "local_relay_peer_identity_and_rotated_secret_before_native_handoff";
pub const RELAY_NETWORK_BOUNDARY: &str =
    "postgres_local_unix_socket_only_no_remote_broker_io_inside_extension";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct NativeRelayAuthPlan {
    pub contract: &'static str,
    pub relay_identity: String,
    pub credential_source: String,
    pub socket_path: String,
    pub network_boundary: &'static str,
    pub rotation_required: bool,
    pub handoff_allowed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeRelayAuthError {
    MissingRelayIdentity,
    MissingCredentialSource,
    MissingSocketPath,
    NonLocalSocketPath,
}

pub fn native_relay_auth_plan(
    relay_identity: impl Into<String>,
    credential_source: impl Into<String>,
    socket_path: impl Into<String>,
) -> Result<NativeRelayAuthPlan, NativeRelayAuthError> {
    let relay_identity = clean(
        relay_identity.into(),
        NativeRelayAuthError::MissingRelayIdentity,
    )?;
    let credential_source = clean(
        credential_source.into(),
        NativeRelayAuthError::MissingCredentialSource,
    )?;
    let socket_path = clean(socket_path.into(), NativeRelayAuthError::MissingSocketPath)?;

    if !socket_path.starts_with('/') {
        return Err(NativeRelayAuthError::NonLocalSocketPath);
    }

    Ok(NativeRelayAuthPlan {
        contract: RELAY_AUTH_CONTRACT,
        relay_identity,
        credential_source,
        socket_path,
        network_boundary: RELAY_NETWORK_BOUNDARY,
        rotation_required: true,
        handoff_allowed: true,
    })
}

fn clean(value: String, missing: NativeRelayAuthError) -> Result<String, NativeRelayAuthError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(missing);
    }
    Ok(trimmed.to_owned())
}
