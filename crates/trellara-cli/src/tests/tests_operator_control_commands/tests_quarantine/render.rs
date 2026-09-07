use super::*;

#[test]
fn quarantine_summaries_render_control_plane_state() {
    let record = latest_quarantine();
    let list = QuarantineListSummary {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        count: 1,
        records: vec![record],
    };
    let clear = QuarantineClearSummary {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-blocked".to_string(),
        commit_lsn: "0/16B6D28".to_string(),
        cleared: true,
    };
    let replay_ready = QuarantineReplayReadySummary {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-blocked".to_string(),
        commit_lsn: "0/16B6D28".to_string(),
        quarantine_reason: "target_postgres_error".to_string(),
        quarantine_detail: "target table is missing".to_string(),
        dedup_removed: true,
        quarantine_cleared: true,
        redelivery_required: true,
        redelivery_topics: vec!["trellara.source-a.sales.strict".to_string()],
        redelivery_boundary: None,
        redelivery_warnings: vec!["local redelivery boundary is not_found".to_string()],
        exact_seek_commands: Vec::new(),
        redelivery_commands: vec![
            "trellara stream locate-local --config <config> --transaction-id tx-blocked --commit-lsn 0/16B6D28"
                .to_string(),
            "trellara stream seek-local --config <config> --topic trellara.source-a.sales.strict --next-offset <offset>"
                .to_string(),
        ],
        redelivery_hint: "redeliver or seek the durable stream message".to_string(),
        safety_contract: quarantine_replay_safety_contract_with_boundary(true, true, None, 0),
    };

    let list_json = serde_json::to_string(&list).expect("list json");
    let clear_json = serde_json::to_string(&clear).expect("clear json");
    let replay_ready_json = serde_json::to_string(&replay_ready).expect("replay ready json");

    assert!(list_json.contains("\"count\":1"));
    assert!(list_json.contains("\"reason\":\"target_postgres_error\""));
    assert!(clear_json.contains("\"cleared\":true"));
    assert!(replay_ready_json.contains("\"quarantine_reason\":\"target_postgres_error\""));
    assert!(replay_ready_json.contains("\"quarantine_detail\":\"target table is missing\""));
    assert!(replay_ready_json.contains("\"dedup_removed\":true"));
    assert!(replay_ready_json.contains("\"redelivery_required\":true"));
    assert!(replay_ready_json.contains("\"redelivery_topics\""));
    assert!(replay_ready_json.contains("\"redelivery_boundary\":null"));
    assert!(replay_ready_json.contains("\"redelivery_warnings\""));
    assert!(replay_ready_json.contains("\"exact_seek_commands\":[]"));
    assert!(replay_ready_json.contains("\"redelivery_commands\""));
    assert!(replay_ready_json.contains("stream locate-local"));
    assert!(replay_ready_json.contains("stream seek-local"));
    assert!(replay_ready_json.contains("\"safety_contract\""));
    assert!(replay_ready_json.contains("\"exact_boundary_required\":true"));
    assert!(replay_ready_json.contains("\"boundary_evidence\""));
    assert!(replay_ready_json.contains("\"exact_seek_available\":false"));
    assert!(replay_ready_json.contains("\"boundary_status\":\"not_located\""));
    assert!(replay_ready_json.contains("dedup and quarantine rows cleared"));
    assert!(replay_ready_json.contains("locate the durable stream transaction boundary"));
}
