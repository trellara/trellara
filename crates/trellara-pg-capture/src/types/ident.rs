use trellara_protocol::ReplicaIdentity;

pub(crate) fn quote_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

pub(crate) fn decode_replica_identity(code: String) -> ReplicaIdentity {
    match code.as_str() {
        "d" => ReplicaIdentity::Default,
        "i" => ReplicaIdentity::Index,
        "f" => ReplicaIdentity::Full,
        "n" => ReplicaIdentity::Nothing,
        _ => ReplicaIdentity::Unspecified,
    }
}
