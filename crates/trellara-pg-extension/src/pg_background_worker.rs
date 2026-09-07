use std::time::Duration;

use pgrx::bgworkers::{BackgroundWorker, SignalWakeFlags};
use pgrx::{pg_sys, Spi};

use crate::pg_shared_queue::{self, RuntimeQueuedFrame};
use crate::NativeRelayWireFrame;

#[path = "pg_background_worker/relay_client.rs"]
mod relay_client;

#[pgrx::pg_guard]
#[unsafe(no_mangle)]
pub extern "C-unwind" fn trellara_native_worker_main(_argument: pg_sys::Datum) {
    BackgroundWorker::attach_signal_handlers(SignalWakeFlags::SIGHUP | SignalWakeFlags::SIGTERM);
    BackgroundWorker::connect_worker_to_spi(Some(&crate::pg_guc::database_name()), None);
    if let Err(error) = pg_shared_queue::record_worker_start(unsafe { pg_sys::MyProcPid }) {
        pgrx::error!("failed to initialize Trellara native worker: {error}");
    }

    while BackgroundWorker::wait_latch(Some(Duration::from_millis(
        crate::pg_guc::worker_poll_milliseconds(),
    ))) {
        if BackgroundWorker::sighup_received() {
            unsafe { pg_sys::ProcessConfigFile(pg_sys::GucContext::PGC_SIGHUP) };
        }
        if !crate::pg_guc::enabled() {
            continue;
        }
        drain_one();
        decode_one_if_capacity();
    }
}

fn drain_one() {
    let queued = match pg_shared_queue::peek() {
        Ok(Some(frame)) => frame,
        Ok(None) => return,
        Err(error) => pgrx::error!("native queue read failed: {error}"),
    };
    if relay_and_acknowledge(queued).is_err() {
        let _ = pg_shared_queue::record_relay_failure();
    }
}

fn relay_and_acknowledge(queued: RuntimeQueuedFrame) -> Result<(), String> {
    let frame = queued.to_wire();
    let secret = crate::pg_guc::relay_secret();
    if secret.is_empty() {
        return Err("trellara.relay_secret is not configured".to_string());
    }
    let ack = relay_client::exchange_with_relay(&frame, &secret)?;
    if ack.commit_lsn != queued.commit_lsn() || ack.payload_digest != queued.payload_digest() {
        return Err("relay acknowledgement does not cover the queued frame".to_string());
    }
    advance_replication_slot(queued.commit_lsn())?;
    pg_shared_queue::mark_durable(&queued).map_err(|error| error.to_string())
}

fn decode_one_if_capacity() {
    let capacity = crate::pg_guc::queue_capacity();
    match pg_shared_queue::has_capacity(capacity) {
        Ok(true) => {}
        Ok(false) => return,
        Err(error) => pgrx::error!("native queue capacity check failed: {error}"),
    }
    let slot_name = crate::pg_guc::slot_name();
    if slot_name.is_empty() || !logical_slot_exists(&slot_name) {
        return;
    }
    let decoded = BackgroundWorker::transaction(|| {
        Spi::connect_mut(|client| {
            let table = client.update(
                "select lsn::text, xid::text::bigint, data \
                   from pg_catalog.pg_logical_slot_peek_binary_changes($1, null, 1) \
                  limit 1",
                Some(1),
                &[slot_name.as_str().into()],
            )?;
            if table.is_empty() {
                Ok(None)
            } else {
                table.first().get_three::<String, i64, Vec<u8>>().map(Some)
            }
        })
    });
    let Some((Some(commit_lsn), Some(xid), Some(payload))) = decoded
        .unwrap_or_else(|error| pgrx::error!("native logical decoding query failed: {error}"))
    else {
        return;
    };
    let commit_lsn = trellara_protocol::parse_lsn(&commit_lsn)
        .unwrap_or_else(|error| pgrx::error!("decoded commit LSN is invalid: {error}"));
    let xid = u32::try_from(xid)
        .unwrap_or_else(|_| pgrx::error!("decoded transaction ID {xid} does not fit u32"));
    let frame = NativeRelayWireFrame::new(
        xid,
        commit_lsn,
        crate::pg_guc::source_id(),
        crate::pg_guc::dataset_id(),
        payload,
    )
    .unwrap_or_else(|error| pgrx::error!("decoded frame is invalid: {error}"));
    if let Err(error) = pg_shared_queue::enqueue(&frame, capacity) {
        pgrx::error!("native queue admission failed: {error}");
    }
}

fn logical_slot_exists(slot_name: &str) -> bool {
    BackgroundWorker::transaction(|| {
        Spi::get_one_with_args::<bool>(
            "select exists (select 1 from pg_catalog.pg_replication_slots where slot_name = $1 and plugin = 'trellara_pg_extension')",
            &[slot_name.into()],
        )
    })
    .ok()
    .flatten()
    .unwrap_or(false)
}

fn advance_replication_slot(commit_lsn: u64) -> Result<(), String> {
    let slot_name = crate::pg_guc::slot_name();
    let commit_lsn = pg_shared_queue::format_lsn(commit_lsn);
    let advanced = BackgroundWorker::transaction(|| {
        Spi::get_one_with_args::<String>(
            "select end_lsn::text from pg_catalog.pg_replication_slot_advance($1, $2::pg_lsn)",
            &[slot_name.as_str().into(), commit_lsn.as_str().into()],
        )
    })
    .map_err(|error| format!("replication slot acknowledgement failed: {error}"))?;
    if advanced.is_none() {
        return Err("replication slot acknowledgement returned no LSN".to_string());
    }
    Ok(())
}
