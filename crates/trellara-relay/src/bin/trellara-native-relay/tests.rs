use std::collections::BTreeMap;

use super::*;

#[derive(Default)]
struct TestEnv {
    values: BTreeMap<String, String>,
}

impl TestEnv {
    fn with(mut self, name: &str, value: &str) -> Self {
        self.values.insert(name.to_string(), value.to_string());
        self
    }
}

impl NativeRelayEnv for TestEnv {
    fn var(&self, name: &str) -> Option<String> {
        self.values.get(name).cloned()
    }

    fn var_os(&self, name: &str) -> Option<OsString> {
        self.values.get(name).map(OsString::from)
    }
}

#[test]
fn config_defaults_to_spool_durability() {
    let env = TestEnv::default().with("TRELLARA_NATIVE_RELAY_SECRET", "secret");
    let config = config_from_environment(&env).expect("spool config");

    match config {
        RelayConfig::Spool(server) => {
            assert_eq!(server.secret, b"secret");
            assert_eq!(
                server.socket_path,
                PathBuf::from("/tmp/trellara-native-relay.sock")
            );
            assert_eq!(
                server.spool_path,
                PathBuf::from("/tmp/trellara-native-relay.spool")
            );
            assert!(!server.exit_after_sync);
        }
        #[cfg(feature = "kafka")]
        RelayConfig::Kafka(_, _) => panic!("expected spool config"),
    }
}

#[test]
#[cfg(feature = "kafka")]
fn config_parses_kafka_durability() {
    let env = TestEnv::default()
        .with("TRELLARA_NATIVE_RELAY_SECRET", "secret")
        .with("TRELLARA_NATIVE_RELAY_DURABILITY", "kafka")
        .with("TRELLARA_NATIVE_RELAY_SOCKET", "/tmp/custom.sock")
        .with("TRELLARA_NATIVE_RELAY_SPOOL", "/tmp/custom.spool")
        .with("TRELLARA_NATIVE_RELAY_EXIT_AFTER_SYNC", "1")
        .with("TRELLARA_KAFKA_BOOTSTRAP_SERVERS", "localhost:9092")
        .with("TRELLARA_KAFKA_CLIENT_ID", "native-relay-test")
        .with("TRELLARA_KAFKA_MESSAGE_TIMEOUT_MS", "2500")
        .with("TRELLARA_KAFKA_PROOF_PATH", "/tmp/custom.kafka-proof");
    let config = config_from_environment(&env).expect("kafka config");

    match config {
        RelayConfig::Kafka(server, kafka) => {
            assert_eq!(server.socket_path, PathBuf::from("/tmp/custom.sock"));
            assert_eq!(server.spool_path, PathBuf::from("/tmp/custom.spool"));
            assert!(server.exit_after_sync);
            assert_eq!(kafka.bootstrap_servers, "localhost:9092");
            assert_eq!(kafka.client_id, "native-relay-test");
            assert_eq!(kafka.message_timeout, Duration::from_millis(2500));
            assert_eq!(kafka.proof_path, PathBuf::from("/tmp/custom.kafka-proof"));
        }
        RelayConfig::Spool(_) => panic!("expected kafka config"),
    }
}

#[test]
#[cfg(not(feature = "kafka"))]
fn config_rejects_kafka_durability_without_feature() {
    let env = TestEnv::default()
        .with("TRELLARA_NATIVE_RELAY_SECRET", "secret")
        .with("TRELLARA_NATIVE_RELAY_DURABILITY", "kafka");
    let error = config_from_environment(&env).expect_err("kafka disabled");

    assert_eq!(
        error,
        "trellara-native-relay was built without Kafka support; rebuild with --features kafka"
    );
}

#[test]
fn config_rejects_invalid_durability_mode() {
    let env = TestEnv::default()
        .with("TRELLARA_NATIVE_RELAY_SECRET", "secret")
        .with("TRELLARA_NATIVE_RELAY_DURABILITY", "memory");
    let error = config_from_environment(&env).expect_err("invalid durability");

    assert_eq!(
        error,
        "TRELLARA_NATIVE_RELAY_DURABILITY must be spool or kafka"
    );
}

#[test]
#[cfg(feature = "kafka")]
fn kafka_config_rejects_zero_timeout() {
    let env = TestEnv::default()
        .with("TRELLARA_KAFKA_BOOTSTRAP_SERVERS", "localhost:9092")
        .with("TRELLARA_KAFKA_MESSAGE_TIMEOUT_MS", "0");
    let error = kafka_config(&env).expect_err("zero timeout");

    assert_eq!(
        error,
        "TRELLARA_KAFKA_MESSAGE_TIMEOUT_MS must be greater than zero"
    );
}
