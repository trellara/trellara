use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
#[cfg(feature = "kafka")]
use std::time::Duration;

#[cfg(feature = "kafka")]
use trellara_relay::{run_native_kafka_relay, NativeKafkaRelayConfig};
use trellara_relay::{run_native_relay, NativeRelayServerConfig, NativeRelayTransportResult};

fn main() {
    let config = match config_from_environment(&ProcessEnv) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("trellara-native-relay configuration error: {error}");
            std::process::exit(2);
        }
    };
    if let Err(error) = run(config) {
        eprintln!("trellara-native-relay failed: {error}");
        std::process::exit(1);
    }
}

#[derive(Debug)]
enum RelayConfig {
    Spool(NativeRelayServerConfig),
    #[cfg(feature = "kafka")]
    Kafka(NativeRelayServerConfig, NativeKafkaRelayConfig),
}

trait NativeRelayEnv {
    fn var(&self, name: &str) -> Option<String>;
    fn var_os(&self, name: &str) -> Option<OsString>;
}

struct ProcessEnv;

impl NativeRelayEnv for ProcessEnv {
    fn var(&self, name: &str) -> Option<String> {
        env::var(name).ok()
    }

    fn var_os(&self, name: &str) -> Option<OsString> {
        env::var_os(name)
    }
}

fn run(config: RelayConfig) -> NativeRelayTransportResult<()> {
    match config {
        RelayConfig::Spool(server) => run_native_relay(server),
        #[cfg(feature = "kafka")]
        RelayConfig::Kafka(server, kafka) => run_native_kafka_relay(server, kafka),
    }
}

fn config_from_environment(env: &impl NativeRelayEnv) -> Result<RelayConfig, String> {
    let secret = env
        .var("TRELLARA_NATIVE_RELAY_SECRET")
        .ok_or_else(|| "TRELLARA_NATIVE_RELAY_SECRET is required".to_string())?
        .into_bytes();
    if secret.is_empty() {
        return Err("TRELLARA_NATIVE_RELAY_SECRET cannot be empty".to_string());
    }
    let server = NativeRelayServerConfig {
        socket_path: environment_path(
            env,
            "TRELLARA_NATIVE_RELAY_SOCKET",
            "/tmp/trellara-native-relay.sock",
        ),
        spool_path: environment_path(
            env,
            "TRELLARA_NATIVE_RELAY_SPOOL",
            "/tmp/trellara-native-relay.spool",
        ),
        secret,
        exit_after_sync: env
            .var_os("TRELLARA_NATIVE_RELAY_EXIT_AFTER_SYNC")
            .is_some(),
    };
    match env
        .var("TRELLARA_NATIVE_RELAY_DURABILITY")
        .unwrap_or_else(|| "spool".to_string())
        .as_str()
    {
        "spool" => Ok(RelayConfig::Spool(server)),
        #[cfg(feature = "kafka")]
        "kafka" => Ok(RelayConfig::Kafka(server, kafka_config(env)?)),
        #[cfg(not(feature = "kafka"))]
        "kafka" => Err(
            "trellara-native-relay was built without Kafka support; rebuild with --features kafka"
                .to_string(),
        ),
        _ => Err("TRELLARA_NATIVE_RELAY_DURABILITY must be spool or kafka".to_string()),
    }
}

fn environment_path(env: &impl NativeRelayEnv, name: &str, default: &str) -> PathBuf {
    env.var_os(name)
        .map_or_else(|| PathBuf::from(default), PathBuf::from)
}

#[cfg(feature = "kafka")]
fn kafka_config(env: &impl NativeRelayEnv) -> Result<NativeKafkaRelayConfig, String> {
    let bootstrap_servers = env
        .var("TRELLARA_KAFKA_BOOTSTRAP_SERVERS")
        .ok_or_else(|| "TRELLARA_KAFKA_BOOTSTRAP_SERVERS is required in kafka mode".to_string())?;
    let timeout_ms = env
        .var("TRELLARA_KAFKA_MESSAGE_TIMEOUT_MS")
        .unwrap_or_else(|| "30000".to_string())
        .parse::<u64>()
        .map_err(|_| "TRELLARA_KAFKA_MESSAGE_TIMEOUT_MS must be an integer".to_string())?;
    if timeout_ms == 0 {
        return Err("TRELLARA_KAFKA_MESSAGE_TIMEOUT_MS must be greater than zero".to_string());
    }
    Ok(NativeKafkaRelayConfig {
        bootstrap_servers,
        client_id: env
            .var("TRELLARA_KAFKA_CLIENT_ID")
            .unwrap_or_else(|| "trellara-native-relay".to_string()),
        message_timeout: Duration::from_millis(timeout_ms),
        proof_path: environment_path(
            env,
            "TRELLARA_KAFKA_PROOF_PATH",
            "/tmp/trellara-native-relay.kafka-proof",
        ),
    })
}

#[cfg(test)]
#[path = "trellara-native-relay/tests.rs"]
mod tests;
