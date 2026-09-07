use super::*;

use crate::replication::replication_options_sql;

#[test]
fn pgoutput_protocol_defaults_to_v2_streaming_options() {
    let pgoutput = PgOutputProtocolConfig::default();
    let protocol_version = pgoutput.protocol_version.to_string();
    let streaming = pgoutput.streaming.to_string();
    let options = pgoutput.start_options("trellara_pub", &protocol_version, &streaming);

    assert_eq!(
        options,
        vec![
            ("proto_version", "2"),
            ("publication_names", "trellara_pub"),
            ("streaming", "true")
        ]
    );
    assert_eq!(
        replication_options_sql(&options),
        "\"proto_version\" '2', \"publication_names\" 'trellara_pub', \"streaming\" 'true'"
    );
}

#[test]
fn pgoutput_protocol_v1_scopes_out_streaming_option() {
    let pgoutput = PgOutputProtocolConfig {
        protocol_version: 1,
        streaming: false,
    };
    let protocol_version = pgoutput.protocol_version.to_string();
    let streaming = pgoutput.streaming.to_string();
    let options = pgoutput.start_options("trellara_pub", &protocol_version, &streaming);

    assert_eq!(
        options,
        vec![
            ("proto_version", "1"),
            ("publication_names", "trellara_pub")
        ]
    );
}

#[test]
fn pgoutput_streaming_requires_protocol_v2() {
    let pgoutput = PgOutputProtocolConfig {
        protocol_version: 1,
        streaming: true,
    };

    assert!(matches!(
        pgoutput.validate(),
        Err(CaptureError::InvalidConfig(message))
            if message.contains("streaming requires protocol_version 2")
    ));
}
