pub(crate) const DEV_SOURCE_SERVICE: &str = "source-postgres";
pub(crate) const DEV_TARGET_SERVICE: &str = "target-postgres";
pub(crate) const DEV_KAFKA_SERVICE: &str = "redpanda";
pub(crate) const DEV_RELAY_SERVICE: &str = "trellara-relay";
pub(crate) const DEV_APPLIER_SERVICE: &str = "trellara-applier";

pub(crate) fn dev_services(with_kafka: bool, runtime: bool) -> Vec<String> {
    let mut services = vec![
        DEV_SOURCE_SERVICE.to_string(),
        DEV_TARGET_SERVICE.to_string(),
    ];
    if with_kafka || runtime {
        services.push(DEV_KAFKA_SERVICE.to_string());
    }
    if runtime {
        services.push(DEV_RELAY_SERVICE.to_string());
        services.push(DEV_APPLIER_SERVICE.to_string());
    }
    services
}

pub(crate) fn dev_compose_prefix(runtime: bool) -> Vec<String> {
    let mut args = vec!["compose".to_string()];
    if runtime {
        args.extend(["--profile".to_string(), "runtime".to_string()]);
    }
    args
}
