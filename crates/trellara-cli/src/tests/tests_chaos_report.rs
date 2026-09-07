use super::tests_chaos_report_boundary_contract::assert_chaos_report_boundary_contract;
use super::tests_chaos_report_pgoutput_contract::assert_chaos_report_pgoutput_contract;
use super::tests_chaos_report_source_local_identity_contract::assert_chaos_report_source_local_identity_contract;
use super::tests_chaos_report_summary_contract::assert_chaos_report_summary_contract;
use super::*;

#[test]
fn chaos_run_summary_lists_failure_matrix() {
    let summary = ChaosRunSummary::default();

    assert_chaos_report_summary_contract(&summary);
    assert_chaos_report_pgoutput_contract(&summary);
    assert_chaos_report_source_local_identity_contract(&summary);
    assert_chaos_report_boundary_contract(&summary);
}
