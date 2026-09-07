use std::fmt;

pub const MAX_LOGICAL_FRAME_BYTES: usize = 64 * 1024;
const FRAME_MAGIC: &[u8; 4] = b"TRLD";
const FRAME_VERSION: u16 = 1;
const COMMIT_LSN_OFFSET: usize = 4 + 2 + 4 + 8;
const EVENT_COUNT_OFFSET: usize = COMMIT_LSN_OFFSET + 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum NativeLogicalOperation {
    Insert = 1,
    Update = 2,
    Delete = 3,
    Truncate = 4,
    Message = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum NativeLogicalColumnStatus {
    Null = 0,
    Value = 1,
    Dropped = 2,
    UnchangedToast = 3,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeLogicalFrameError {
    AlreadyActive,
    NotActive,
    EmptyIdentity {
        field: &'static str,
    },
    IdentityTooLong {
        field: &'static str,
        length: usize,
    },
    EmptyTransaction,
    FrameTooLarge {
        attempted_bytes: usize,
        max_bytes: usize,
    },
    CountOverflow {
        field: &'static str,
    },
    ValueTooLong {
        length: usize,
    },
}

impl fmt::Display for NativeLogicalFrameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyActive => formatter.write_str("a logical transaction is already active"),
            Self::NotActive => formatter.write_str("no logical transaction is active"),
            Self::EmptyIdentity { field } => write!(formatter, "{field} cannot be empty"),
            Self::IdentityTooLong { field, length } => {
                write!(formatter, "{field} is {length} bytes; maximum is 255")
            }
            Self::EmptyTransaction => {
                formatter.write_str("a committed logical frame must contain at least one event")
            }
            Self::FrameTooLarge {
                attempted_bytes,
                max_bytes,
            } => write!(
                formatter,
                "logical transaction frame requires {attempted_bytes} bytes; maximum is {max_bytes}"
            ),
            Self::CountOverflow { field } => write!(formatter, "{field} overflowed"),
            Self::ValueTooLong { length } => {
                write!(
                    formatter,
                    "logical column value is {length} bytes; maximum is u32::MAX"
                )
            }
        }
    }
}

impl std::error::Error for NativeLogicalFrameError {}

pub struct NativeLogicalFrameBuilder {
    bytes: Box<[u8; MAX_LOGICAL_FRAME_BYTES]>,
    len: usize,
    event_count: u32,
    active: bool,
}

impl Default for NativeLogicalFrameBuilder {
    fn default() -> Self {
        Self {
            bytes: Box::new([0; MAX_LOGICAL_FRAME_BYTES]),
            len: 0,
            event_count: 0,
            active: false,
        }
    }
}

mod builder;
mod encoding;
