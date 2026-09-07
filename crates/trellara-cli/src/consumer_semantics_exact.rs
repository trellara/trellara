use crate::{ConsumerSemanticsAvailability, ConsumerSemanticsMode};

pub(crate) fn exact_transaction_semantics_mode(
    partitioned: bool,
    strict_chunking_configured: bool,
    config_display: &str,
) -> ConsumerSemanticsMode {
    ConsumerSemanticsMode {
        mode: "exact_transaction".to_string(),
        best_for: "Postgres replicas, financial flows, and recovery-sensitive operational sync"
            .to_string(),
        availability: if partitioned {
            ConsumerSemanticsAvailability::SupportedWithBarrier
        } else {
            ConsumerSemanticsAvailability::Native
        },
        transaction_boundary: if partitioned {
            "manifest plus commit marker barrier; consumer waits for all participating partition chunks"
                .to_string()
        } else if strict_chunking_configured {
            "strict chunk manifest plus commit marker barrier before visibility".to_string()
        } else {
            "complete source transaction envelope before visibility".to_string()
        },
        ordering: if partitioned {
            "source transaction identity preserved; global visibility waits for barrier reconstruction"
                .to_string()
        } else {
            "global source transaction order".to_string()
        },
        latency: "medium".to_string(),
        throughput: if partitioned {
            "high".to_string()
        } else {
            "medium".to_string()
        },
        visibility_rule: if partitioned {
            "expose only after manifest, commit marker, and every participating partition chunk are present"
                .to_string()
        } else {
            "expose after one verified source transaction boundary has been durably published"
                .to_string()
        },
        consumer_obligations: vec![
            "verify envelope or manifest checksum before apply".to_string(),
            "advance checkpoint only after durable downstream commit".to_string(),
            "deduplicate by source transaction id and commit LSN on replay".to_string(),
        ],
        not_guaranteed: vec![
            "lower-latency partition-local visibility".to_string(),
            "multi-consumer atomicity outside a barrier-aware consumer".to_string(),
        ],
        proof_commands: vec![
            format!("trellara contract-test --config {config_display}"),
            format!("trellara status --config {config_display} --view report --format text"),
            "trellara inspect-transaction --file <envelope.pb> --format text".to_string(),
        ],
    }
}
