use crate::pgoutput_relation_registry::PgOutputRelationRegistry;
use crate::pgoutput_stream_state::PgOutputStreamState;
use crate::{CaptureError, LogicalEvent, PgOutputReader, Result};

const MAX_PGOUTPUT_TRUNCATE_RELATIONS: usize = 1600;

pub(crate) fn decode_truncate(
    relations: &PgOutputRelationRegistry,
    stream: &PgOutputStreamState,
    reader: &mut PgOutputReader<'_>,
) -> Result<LogicalEvent> {
    let transaction_id = reader.read_stream_xid(stream.active_xid())?;
    let relation_count = reader.read_u32()?;
    let _options = reader.read_u8()?;
    let relation_count = truncate_relation_count(relation_count, reader.remaining_len())?;
    let mut truncate_relations = Vec::with_capacity(relation_count);
    for _ in 0..relation_count {
        let relation_oid = reader.read_u32()?;
        truncate_relations.push(relations.get(relation_oid)?.id.clone());
    }
    Ok(LogicalEvent::Truncate {
        transaction_id,
        relations: truncate_relations,
    })
}

fn truncate_relation_count(relation_count: u32, remaining_bytes: usize) -> Result<usize> {
    let relation_count = usize::try_from(relation_count).map_err(|_| {
        CaptureError::PgOutputParse(format!(
            "truncate relation count {relation_count} exceeds supported usize range"
        ))
    })?;
    if relation_count > MAX_PGOUTPUT_TRUNCATE_RELATIONS {
        return Err(CaptureError::PgOutputParse(format!(
            "truncate relation count {relation_count} exceeds supported maximum {MAX_PGOUTPUT_TRUNCATE_RELATIONS}"
        )));
    }
    let required_bytes = relation_count.checked_mul(4).ok_or_else(|| {
        CaptureError::PgOutputParse(format!(
            "truncate relation count {relation_count} exceeds supported byte range"
        ))
    })?;
    if required_bytes > remaining_bytes {
        return Err(CaptureError::PgOutputParse(format!(
            "truncate relation count {relation_count} requires {required_bytes} bytes of relation OIDs, only {remaining_bytes} remain"
        )));
    }
    Ok(relation_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_relation_count_accepts_configured_maximum() {
        assert_eq!(
            truncate_relation_count(
                MAX_PGOUTPUT_TRUNCATE_RELATIONS as u32,
                MAX_PGOUTPUT_TRUNCATE_RELATIONS * 4,
            )
            .expect("truncate relation count"),
            MAX_PGOUTPUT_TRUNCATE_RELATIONS
        );
    }

    #[test]
    fn truncate_relation_count_rejects_absurd_count_before_allocation() {
        let error = truncate_relation_count(
            MAX_PGOUTPUT_TRUNCATE_RELATIONS as u32 + 1,
            (MAX_PGOUTPUT_TRUNCATE_RELATIONS + 1) * 4,
        )
        .expect_err("truncate relation count");

        assert!(
            matches!(error, CaptureError::PgOutputParse(message) if message.contains("exceeds supported maximum"))
        );
    }
}
