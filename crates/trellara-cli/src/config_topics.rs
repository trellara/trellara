use trellara_stream::{StreamMode, TopicLayout};

use crate::{DatasetMode, Result, StreamConfig, TrellaraConfig};

pub(crate) fn kafka_consumer_topics(config: &TrellaraConfig) -> Result<Vec<String>> {
    let StreamConfig::Kafka { topic, .. } = &config.stream else {
        return Ok(Vec::new());
    };

    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder => strict_topics(
            &config.source.id,
            &config.dataset.id,
            topic,
            config.dataset.strict_chunking.is_some(),
        ),
        DatasetMode::PartitionedScaleMode => partitioned_topics(config),
    }
}

pub(crate) fn replay_redelivery_topics(config: &TrellaraConfig) -> Result<Vec<String>> {
    match &config.stream {
        StreamConfig::Kafka { .. } => kafka_consumer_topics(config),
        StreamConfig::Local { .. } => local_stream_topics(config),
    }
}

pub(crate) fn local_stream_topics(config: &TrellaraConfig) -> Result<Vec<String>> {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder => {
            let layout = TopicLayout::new(&config.source.id, &config.dataset.id)?;
            let mut topics = vec![layout.topic_for(StreamMode::Strict)];
            if config.dataset.strict_chunking.is_some() {
                topics.push(layout.topic_for(StreamMode::Manifest));
                topics.push(layout.topic_for(StreamMode::Commit));
            }
            Ok(topics)
        }
        DatasetMode::PartitionedScaleMode => partitioned_topics(config),
    }
}

fn strict_topics(
    source_id: &str,
    dataset_id: &str,
    strict_topic: &str,
    strict_chunking: bool,
) -> Result<Vec<String>> {
    if !strict_chunking {
        return Ok(vec![strict_topic.to_string()]);
    }

    let layout = TopicLayout::new(source_id, dataset_id)?;
    Ok(vec![
        strict_topic.to_string(),
        layout.topic_for(StreamMode::Manifest),
        layout.topic_for(StreamMode::Commit),
    ])
}

fn partitioned_topics(config: &TrellaraConfig) -> Result<Vec<String>> {
    let partition = config
        .dataset
        .partition
        .as_ref()
        .expect("validated partition config");
    let layout = TopicLayout::new(&config.source.id, &config.dataset.id)?;
    let mut topics = vec![
        layout.topic_for(StreamMode::Manifest),
        layout.topic_for(StreamMode::Commit),
    ];
    topics.extend(
        (0..partition.partition_count)
            .map(|partition| layout.topic_for(StreamMode::Partition { partition })),
    );
    Ok(topics)
}
