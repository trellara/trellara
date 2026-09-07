use std::ffi::{CStr, CString};

use pgrx::guc::{GucContext, GucFlags, GucRegistry, GucSetting};

use super::{
    DATABASE_NAME, DATASET_ID, ENABLED, QUEUE_CAPACITY, RELAY_SECRET, RELAY_SOCKET_PATH,
    RELAY_TIMEOUT_MILLISECONDS, SLOT_NAME, SOURCE_ID, WORKER_POLL_MILLISECONDS,
};
use crate::pg_shared_queue::MAX_RUNTIME_QUEUE_FRAMES;

pub(super) fn register() {
    define_bool(
        c"trellara.enabled",
        c"Enable the native logical-decoding worker.",
        &ENABLED,
        GucContext::Sighup,
    );
    define_string(
        c"trellara.database_name",
        c"Database containing the native logical replication slot.",
        &DATABASE_NAME,
        GucContext::Postmaster,
    );
    define_string(
        c"trellara.slot_name",
        c"Logical replication slot consumed by the native worker.",
        &SLOT_NAME,
        GucContext::Sighup,
    );
    define_string(
        c"trellara.source_id",
        c"Stable Trellara source identity encoded into native frames.",
        &SOURCE_ID,
        GucContext::Sighup,
    );
    define_string(
        c"trellara.dataset_id",
        c"Stable Trellara dataset identity encoded into native frames.",
        &DATASET_ID,
        GucContext::Sighup,
    );
    define_string(
        c"trellara.relay_socket_path",
        c"Local Unix socket used for durable relay handoff.",
        &RELAY_SOCKET_PATH,
        GucContext::Sighup,
    );
    GucRegistry::define_string_guc(
        c"trellara.relay_secret",
        c"Rotatable secret authenticating the local relay handoff.",
        c"Required before native frames can leave PostgreSQL.",
        &RELAY_SECRET,
        GucContext::Sighup,
        GucFlags::SUPERUSER_ONLY | GucFlags::NO_SHOW_ALL,
    );
    GucRegistry::define_int_guc(
        c"trellara.queue_capacity",
        c"Maximum committed transaction frames buffered in shared memory.",
        c"Changing the capacity requires a PostgreSQL restart.",
        &QUEUE_CAPACITY,
        1,
        i32::try_from(MAX_RUNTIME_QUEUE_FRAMES).expect("queue capacity fits i32"),
        GucContext::Postmaster,
        GucFlags::default(),
    );
    define_int(
        c"trellara.worker_poll_milliseconds",
        c"Worker latch timeout between native drain attempts.",
        &WORKER_POLL_MILLISECONDS,
        10,
        60_000,
    );
    define_int(
        c"trellara.relay_timeout_milliseconds",
        c"Bounded timeout for local relay connect, write, and acknowledgement.",
        &RELAY_TIMEOUT_MILLISECONDS,
        100,
        60_000,
    );
}

fn define_bool(
    name: &'static CStr,
    description: &'static CStr,
    setting: &'static GucSetting<bool>,
    context: GucContext,
) {
    GucRegistry::define_bool_guc(
        name,
        description,
        c"",
        setting,
        context,
        GucFlags::default(),
    );
}

fn define_string(
    name: &'static CStr,
    description: &'static CStr,
    setting: &'static GucSetting<Option<CString>>,
    context: GucContext,
) {
    GucRegistry::define_string_guc(
        name,
        description,
        c"",
        setting,
        context,
        GucFlags::default(),
    );
}

fn define_int(
    name: &'static CStr,
    description: &'static CStr,
    setting: &'static GucSetting<i32>,
    minimum: i32,
    maximum: i32,
) {
    GucRegistry::define_int_guc(
        name,
        description,
        c"",
        setting,
        minimum,
        maximum,
        GucContext::Sighup,
        GucFlags::default(),
    );
}
