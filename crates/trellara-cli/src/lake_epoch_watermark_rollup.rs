use trellara_protocol::parse_lsn;

use crate::{LakeEpochSourceWatermark, LakeEpochWatermarkRollup};

pub(crate) fn lake_epoch_watermark_rollup(
    sources: &[LakeEpochSourceWatermark],
) -> LakeEpochWatermarkRollup {
    let mut complete_end_lsns = Vec::<(u64, String)>::new();
    let mut observed_end_lsns = Vec::<(u64, String)>::new();
    let mut complete_source_count = 0;
    let mut lagging_source_count = 0;
    let mut missing_source_count = 0;
    let mut quarantined_source_count = 0;
    let mut invalid_lsn_sources = Vec::new();

    for source in sources {
        match source.state.as_str() {
            "complete" => {
                complete_source_count += 1;
                if let Some(lsn) = &source.end_lsn {
                    if let Some(parsed) = parse_source_lsn(source, lsn, &mut invalid_lsn_sources) {
                        complete_end_lsns.push((parsed, lsn.clone()));
                        observed_end_lsns.push((parsed, lsn.clone()));
                    }
                }
            }
            "lagging" => {
                lagging_source_count += 1;
                if let Some(lsn) = &source.end_lsn {
                    if let Some(parsed) = parse_source_lsn(source, lsn, &mut invalid_lsn_sources) {
                        observed_end_lsns.push((parsed, lsn.clone()));
                    }
                }
            }
            "missing" => missing_source_count += 1,
            "quarantined" => quarantined_source_count += 1,
            _ => {}
        }
    }

    LakeEpochWatermarkRollup {
        global_low_watermark_lsn: min_lsn(complete_end_lsns),
        max_source_watermark_lsn: max_lsn(observed_end_lsns),
        complete_source_count,
        lagging_source_count,
        missing_source_count,
        quarantined_source_count,
        invalid_lsn_source_count: invalid_lsn_sources.len(),
        invalid_lsn_sources,
    }
}

fn parse_source_lsn(
    source: &LakeEpochSourceWatermark,
    lsn: &str,
    invalid_lsn_sources: &mut Vec<String>,
) -> Option<u64> {
    let Ok(value) = parse_lsn(lsn) else {
        invalid_lsn_sources.push(format!("{}:{lsn}", source.source_id));
        return None;
    };
    if value == 0 {
        invalid_lsn_sources.push(format!("{}:{lsn}", source.source_id));
        return None;
    }
    Some(value)
}

fn min_lsn(lsns: Vec<(u64, String)>) -> Option<String> {
    lsns.into_iter()
        .min_by_key(|(value, _)| *value)
        .map(|(_, lsn)| lsn)
}

fn max_lsn(lsns: Vec<(u64, String)>) -> Option<String> {
    lsns.into_iter()
        .max_by_key(|(value, _)| *value)
        .map(|(_, lsn)| lsn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollup_uses_numeric_lsn_ordering_for_global_low_watermark() {
        let sources = vec![
            source("store-a", "complete", Some("1/0")),
            source("store-b", "complete", Some("0/FFFFFFFF")),
            source("store-c", "missing", None),
        ];

        let rollup = lake_epoch_watermark_rollup(&sources);

        assert_eq!(
            rollup.global_low_watermark_lsn.as_deref(),
            Some("0/FFFFFFFF")
        );
        assert_eq!(rollup.max_source_watermark_lsn.as_deref(), Some("1/0"));
        assert_eq!(rollup.complete_source_count, 2);
        assert_eq!(rollup.missing_source_count, 1);
        assert_eq!(rollup.invalid_lsn_source_count, 0);
        assert_eq!(rollup.invalid_lsn_sources, Vec::<String>::new());
    }

    #[test]
    fn rollup_surfaces_invalid_source_lsn_evidence() {
        let sources = vec![
            source("store-a", "complete", Some("not-an-lsn")),
            source("store-b", "complete", Some("0/0")),
            source("store-c", "lagging", Some("0/16B9000")),
        ];

        let rollup = lake_epoch_watermark_rollup(&sources);

        assert_eq!(rollup.global_low_watermark_lsn, None);
        assert_eq!(
            rollup.max_source_watermark_lsn.as_deref(),
            Some("0/16B9000")
        );
        assert_eq!(rollup.complete_source_count, 2);
        assert_eq!(rollup.lagging_source_count, 1);
        assert_eq!(rollup.invalid_lsn_source_count, 2);
        assert_eq!(
            rollup.invalid_lsn_sources,
            vec!["store-a:not-an-lsn".to_string(), "store-b:0/0".to_string()]
        );
    }

    fn source(id: &str, state: &str, end_lsn: Option<&str>) -> LakeEpochSourceWatermark {
        LakeEpochSourceWatermark {
            source_id: id.to_string(),
            state: state.to_string(),
            start_lsn: end_lsn.map(str::to_string),
            end_lsn: end_lsn.map(str::to_string),
            transaction_count: usize::from(end_lsn.is_some()),
            change_count: usize::from(end_lsn.is_some()),
            gap_reason: None,
        }
    }
}
