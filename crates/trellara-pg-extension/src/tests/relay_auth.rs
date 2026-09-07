use super::*;

#[test]
fn relay_auth_plan_requires_local_identity_and_rotated_secret() {
    let plan = native_relay_auth_plan(
        "trellara-relay@cluster-a",
        "pg_parameter:trellara.relay_secret",
        "/var/run/postgresql/trellara-relay.sock",
    )
    .expect("relay auth plan");

    assert_eq!(plan.contract, RELAY_AUTH_CONTRACT);
    assert_eq!(plan.relay_identity, "trellara-relay@cluster-a");
    assert_eq!(plan.credential_source, "pg_parameter:trellara.relay_secret");
    assert_eq!(plan.socket_path, "/var/run/postgresql/trellara-relay.sock");
    assert_eq!(plan.network_boundary, RELAY_NETWORK_BOUNDARY);
    assert!(plan.rotation_required);
    assert!(plan.handoff_allowed);
}

#[test]
fn relay_auth_plan_rejects_ambiguous_or_remote_handoff_boundary() {
    assert_eq!(
        native_relay_auth_plan("", "pg_parameter:trellara.relay_secret", "/tmp/relay.sock"),
        Err(NativeRelayAuthError::MissingRelayIdentity)
    );
    assert_eq!(
        native_relay_auth_plan("trellara-relay", " ", "/tmp/relay.sock"),
        Err(NativeRelayAuthError::MissingCredentialSource)
    );
    assert_eq!(
        native_relay_auth_plan("trellara-relay", "pg_parameter:secret", ""),
        Err(NativeRelayAuthError::MissingSocketPath)
    );
    assert_eq!(
        native_relay_auth_plan(
            "trellara-relay",
            "pg_parameter:secret",
            "tcp://relay.internal:8432"
        ),
        Err(NativeRelayAuthError::NonLocalSocketPath)
    );
}
