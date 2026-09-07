use super::*;

use crate::replication::{
    replication_endpoint, replication_tcp_host, start_logical_replication_query,
};
use crate::replication_config::ReplicationEndpoint;

#[test]
fn start_replication_uses_slot_bootstrap_lsn() {
    let slot = LogicalSlotBootstrap {
        created: false,
        consistent_lsn: Some("0/16B6C50".to_string()),
    };
    let start_lsn = replication_start_lsn(&slot).expect("start lsn");
    let options = vec![
        ("proto_version", "2"),
        ("publication_names", "trellara_pub"),
    ];

    assert_eq!(start_lsn, "0/16B6C50");
    assert_eq!(
            start_logical_replication_query("trellara_slot", &start_lsn, &options)
                .expect("query"),
            "START_REPLICATION SLOT \"trellara_slot\" LOGICAL 0/16B6C50 (\"proto_version\" '2', \"publication_names\" 'trellara_pub')"
        );
}

#[test]
fn start_replication_rejects_missing_or_invalid_slot_lsn() {
    assert!(matches!(
        replication_start_lsn(&LogicalSlotBootstrap {
            created: true,
            consistent_lsn: None,
        }),
        Err(CaptureError::ReplicationProtocol(message))
            if message.contains("did not report a consistent or confirmed LSN")
    ));
    assert!(matches!(
        start_logical_replication_query("trellara_slot", "1/100000000", &[]),
        Err(CaptureError::ReplicationProtocol(message))
            if message.contains("LSN halves must fit in 32 bits")
    ));
}

#[test]
fn replication_bootstrap_uses_configured_tcp_host() {
    let config = "host=db.example.com user=trellara dbname=retail"
        .parse()
        .expect("postgres config");

    assert_eq!(
        replication_tcp_host(&config).expect("tcp host"),
        "db.example.com"
    );
}

#[test]
fn replication_bootstrap_defaults_to_localhost() {
    let config = "user=trellara dbname=retail"
        .parse()
        .expect("postgres config");

    assert_eq!(
        replication_tcp_host(&config).expect("default host"),
        "localhost"
    );
}

#[test]
fn replication_bootstrap_rejects_ssl_require_until_supported() {
    let config = "host=db.example.com user=trellara dbname=retail sslmode=require"
        .parse()
        .expect("postgres config");

    assert!(matches!(
        replication_tcp_host(&config),
        Err(CaptureError::InvalidConfig(message))
            if message.contains("sslmode=require")
    ));
}

#[test]
fn replication_bootstrap_accepts_unix_socket_hosts() {
    let config = "host=/var/run/postgresql user=trellara dbname=retail"
        .parse()
        .expect("postgres config");

    assert_eq!(
        replication_endpoint(&config).expect("unix socket endpoint"),
        ReplicationEndpoint::Unix {
            path: "/var/run/postgresql/.s.PGSQL.5432".into(),
        }
    );
}

#[test]
fn replication_bootstrap_uses_configured_unix_socket_port() {
    let config = "host=/var/run/postgresql port=6543 user=trellara dbname=retail"
        .parse()
        .expect("postgres config");

    assert_eq!(
        replication_endpoint(&config).expect("unix socket endpoint"),
        ReplicationEndpoint::Unix {
            path: "/var/run/postgresql/.s.PGSQL.6543".into(),
        }
    );
}
