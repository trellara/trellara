use std::ffi::CString;

use pgrx::guc::GucSetting;

#[path = "pg_guc/registration.rs"]
mod registration;

static ENABLED: GucSetting<bool> = GucSetting::<bool>::new(false);
static DATABASE_NAME: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(Some(c"postgres"));
static SLOT_NAME: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(Some(c"trellara_native"));
static SOURCE_ID: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(Some(c"postgres"));
static DATASET_ID: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(Some(c"default"));
static RELAY_SOCKET_PATH: GucSetting<Option<CString>> =
    GucSetting::<Option<CString>>::new(Some(c"/tmp/trellara-native-relay.sock"));
static RELAY_SECRET: GucSetting<Option<CString>> = GucSetting::<Option<CString>>::new(None);
static QUEUE_CAPACITY: GucSetting<i32> = GucSetting::<i32>::new(8);
static WORKER_POLL_MILLISECONDS: GucSetting<i32> = GucSetting::<i32>::new(100);
static RELAY_TIMEOUT_MILLISECONDS: GucSetting<i32> = GucSetting::<i32>::new(5_000);

pub(crate) fn register() {
    registration::register();
}

pub(crate) fn enabled() -> bool {
    ENABLED.get()
}
pub(crate) fn database_name() -> String {
    string_value(&DATABASE_NAME)
}
pub(crate) fn slot_name() -> String {
    string_value(&SLOT_NAME)
}
pub(crate) fn source_id() -> String {
    string_value(&SOURCE_ID)
}
pub(crate) fn dataset_id() -> String {
    string_value(&DATASET_ID)
}
pub(crate) fn relay_socket_path() -> String {
    string_value(&RELAY_SOCKET_PATH)
}

pub(crate) fn relay_secret() -> Vec<u8> {
    RELAY_SECRET
        .get()
        .map_or_else(Vec::new, |value| value.as_bytes().to_vec())
}

pub(crate) fn queue_capacity() -> usize {
    usize::try_from(QUEUE_CAPACITY.get()).expect("PostgreSQL validates queue capacity")
}

pub(crate) fn worker_poll_milliseconds() -> u64 {
    u64::try_from(WORKER_POLL_MILLISECONDS.get()).expect("PostgreSQL validates poll interval")
}

pub(crate) fn relay_timeout_milliseconds() -> u64 {
    u64::try_from(RELAY_TIMEOUT_MILLISECONDS.get()).expect("PostgreSQL validates relay timeout")
}

fn string_value(setting: &'static GucSetting<Option<CString>>) -> String {
    setting
        .get()
        .map_or_else(String::new, |value| value.to_string_lossy().into_owned())
}
