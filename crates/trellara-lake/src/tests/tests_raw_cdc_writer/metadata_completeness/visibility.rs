use crate::raw_cdc::metadata::RAW_CDC_VISIBILITY_RULE;

#[test]
fn visibility_rule_requires_data_files_before_metadata() {
    assert!(RAW_CDC_VISIBILITY_RULE.contains("append raw CDC files first"));
    assert!(RAW_CDC_VISIBILITY_RULE.contains("publish epoch metadata"));
    assert!(RAW_CDC_VISIBILITY_RULE.contains("durable"));
}
