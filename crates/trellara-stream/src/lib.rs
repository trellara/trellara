mod error;
mod header_ddl;
mod header_manifest;
mod header_metadata;
mod headers;
mod keys;
mod message;
mod message_shape;
mod message_validation;
mod topic;
mod traits;

pub use error::StreamError;
pub use header_manifest::manifest_headers;
pub use headers::{
    commit_marker_headers, envelope_headers, partition_chunk_headers, strict_chunk_headers,
};
pub use keys::{partition_chunk_key, strict_transaction_key};
pub use message::{StreamHeader, StreamMessage, StreamPosition};
pub use message_shape::validate_message_shape;
pub use topic::{StreamMode, TopicLayout};
pub use traits::{PublishAck, StreamConsumer, StreamPublisher};

pub type Result<T> = std::result::Result<T, StreamError>;

#[cfg(test)]
mod tests;
