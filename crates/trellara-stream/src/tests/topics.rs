use super::*;

#[test]
fn topic_layouts_are_stable() {
    let layout = TopicLayout::new("source_a", "sales").expect("layout");

    assert_eq!(layout.strict_topic(), "trellara.source_a.sales.strict");
    assert_eq!(layout.manifest_topic(), "trellara.source_a.sales.manifest");
    assert_eq!(layout.commit_topic(), "trellara.source_a.sales.commit");
    assert_eq!(
        layout.partition_topic(7),
        "trellara.source_a.sales.partition.7"
    );
}

#[test]
fn invalid_topic_components_are_rejected() {
    assert!(TopicLayout::new("source.a", "sales").is_err());
    assert!(TopicLayout::new("source_a", "").is_err());
}
