use crate::barrier_reconstruct_request::LocalBarrierReconstructionRequest;

pub(crate) fn manifest_topic(request: &LocalBarrierReconstructionRequest) -> String {
    format!(
        "trellara.{}.{}.manifest",
        request.source_id, request.dataset_id
    )
}

pub(crate) fn commit_topic(request: &LocalBarrierReconstructionRequest) -> String {
    format!(
        "trellara.{}.{}.commit",
        request.source_id, request.dataset_id
    )
}

pub(crate) fn partition_topic(
    request: &LocalBarrierReconstructionRequest,
    partition_id: u32,
) -> String {
    format!(
        "trellara.{}.{}.partition.{}",
        request.source_id, request.dataset_id, partition_id
    )
}
