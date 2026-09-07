use crate::{lake_completeness_state_label, push_html_escaped, push_status, ChaosRunSummary};

pub(crate) fn push_replay_simulation_section(html: &mut String, summary: &ChaosRunSummary) {
    html.push_str("<section><h2>Replayable Simulation Suite</h2><table><thead><tr><th>Failure point</th><th>Status</th><th>Seed</th><th>Transactions</th><th>Skipped duplicates</th><th>Invariant evidence</th></tr></thead><tbody>");
    for simulation in &summary.simulations {
        html.push_str("<tr><td><code>");
        push_html_escaped(html, simulation.failure_point.as_str());
        html.push_str("</code></td><td>");
        push_status(html, simulation.passed);
        html.push_str("</td><td>");
        html.push_str(&simulation.seed.to_string());
        html.push_str("</td><td>");
        html.push_str(&format!(
            "{}/{} applied",
            simulation.applied_transactions, simulation.transaction_count
        ));
        html.push_str("</td><td>");
        html.push_str(&simulation.skipped_duplicates.to_string());
        html.push_str("</td><td><code>");
        push_html_escaped(html, &simulation.repro_command);
        html.push_str("</code></td></tr>");
    }
    html.push_str("</tbody></table></section>");
}

pub(crate) fn push_snapshot_simulation_section(html: &mut String, summary: &ChaosRunSummary) {
    html.push_str("<section><h2>Snapshot Handoff Simulation Suite</h2><table><thead><tr><th>Failure point</th><th>Status</th><th>Seed</th><th>Tables</th><th>Post-snapshot replay</th><th>Invariant evidence</th></tr></thead><tbody>");
    for simulation in &summary.snapshot_simulations {
        html.push_str("<tr><td><code>");
        push_html_escaped(html, simulation.failure_point.as_str());
        html.push_str("</code></td><td>");
        push_status(html, simulation.passed && simulation.verification_matched);
        html.push_str("</td><td>");
        html.push_str(&simulation.seed.to_string());
        html.push_str("</td><td>");
        html.push_str(&format!(
            "{}/{} copied",
            simulation.copied_tables, simulation.table_count
        ));
        html.push_str("</td><td>");
        html.push_str(&format!(
            "{}/{} replayed",
            simulation.stream_replayed_transactions, simulation.writes_after_snapshot
        ));
        html.push_str("</td><td><code>");
        push_html_escaped(html, &simulation.repro_command);
        html.push_str("</code></td></tr>");
    }
    html.push_str("</tbody></table></section>");
}

pub(crate) fn push_strict_chunk_simulation_section(html: &mut String, summary: &ChaosRunSummary) {
    html.push_str("<section><h2>Strict Chunk Simulation Suite</h2><table><thead><tr><th>Failure point</th><th>Status</th><th>Seed</th><th>Chunks</th><th>Duplicates</th><th>Apply boundary</th><th>Invariant evidence</th></tr></thead><tbody>");
    for simulation in &summary.strict_chunk_simulations {
        html.push_str("<tr><td><code>");
        push_html_escaped(html, simulation.failure_point.as_str());
        html.push_str("</code></td><td>");
        push_status(
            html,
            simulation.passed
                && simulation.manifest_published
                && simulation.applied_transactions == 1,
        );
        html.push_str("</td><td>");
        html.push_str(&simulation.seed.to_string());
        html.push_str("</td><td>");
        html.push_str(&format!(
            "{}/{} published",
            simulation.chunks_published, simulation.chunk_count
        ));
        html.push_str("</td><td>");
        html.push_str(&simulation.duplicate_chunks.to_string());
        html.push_str("</td><td>");
        html.push_str(&format!(
            "{} applied after manifest",
            simulation.applied_transactions
        ));
        html.push_str("</td><td><code>");
        push_html_escaped(html, &simulation.repro_command);
        html.push_str("</code></td></tr>");
    }
    html.push_str("</tbody></table></section>");
}

pub(crate) fn push_fleet_fanin_simulation_section(html: &mut String, summary: &ChaosRunSummary) {
    html.push_str("<section><h2>Fleet Fan-In Lake Simulation Suite</h2><table><thead><tr><th>Failure point</th><th>Status</th><th>Epoch</th><th>Sources</th><th>Rows</th><th>State</th><th>Invariant evidence</th></tr></thead><tbody>");
    for simulation in &summary.fleet_fanin_simulations {
        html.push_str("<tr><td><code>");
        push_html_escaped(html, simulation.failure_point.as_str());
        html.push_str("</code></td><td>");
        push_status(html, simulation.passed);
        html.push_str("</td><td><code>");
        push_html_escaped(html, &simulation.epoch_id);
        html.push_str("</code></td><td>");
        html.push_str(&format!(
            "{}/{} complete, {} missing, {} quarantined",
            simulation.complete_source_count,
            simulation.required_source_count,
            simulation.missing_source_count,
            simulation.quarantined_source_count
        ));
        html.push_str("</td><td>");
        html.push_str(&format!(
            "{} transactions / {} changes, {} replays",
            simulation.transaction_count,
            simulation.change_count,
            simulation.duplicate_replay_count
        ));
        html.push_str("</td><td><code>");
        push_html_escaped(
            html,
            lake_completeness_state_label(simulation.initial_state),
        );
        if let Some(recovered_state) = simulation.recovered_state {
            html.push_str(" -> ");
            push_html_escaped(html, lake_completeness_state_label(recovered_state));
        }
        html.push_str("</code></td><td><code>");
        push_html_escaped(html, &simulation.repro_command);
        html.push_str("</code></td></tr>");
    }
    html.push_str("</tbody></table></section>");
}

pub(crate) fn push_qualification_simulation_section(html: &mut String, summary: &ChaosRunSummary) {
    html.push_str("<section><h2>Lane D Qualification Suite</h2><table><thead><tr><th>Failure point</th><th>Status</th><th>Durable boundary</th><th>Transactions</th><th>Soak/load</th><th>Observability assertions</th><th>Invariant evidence</th></tr></thead><tbody>");
    for simulation in &summary.qualification_simulations {
        html.push_str("<tr><td><code>");
        push_html_escaped(html, simulation.failure_point.as_str());
        html.push_str("</code></td><td>");
        push_status(html, simulation.passed);
        html.push_str("</td><td><code>");
        push_html_escaped(html, &simulation.durable_boundary);
        html.push_str("</code><br>");
        push_html_escaped(html, &simulation.invariant);
        html.push_str("</td><td>");
        html.push_str(&format!(
            "{}/{} applied, {} replayed",
            simulation.applied_transactions,
            simulation.transaction_count,
            simulation.duplicate_replays
        ));
        html.push_str("</td><td>");
        if simulation.failure_point
            == trellara_sim::QualificationFailurePoint::TwentyFourHourSoakLargeTransactionMemoryCeiling
        {
            html.push_str(&format!(
                "{}h soak, {} changes, {} / {} MiB",
                simulation.soak_hours,
                simulation.large_transaction_change_count,
                simulation.peak_memory_mib,
                simulation.memory_ceiling_mib
            ));
        } else {
            html.push_str("failure harness");
        }
        html.push_str("</td><td>");
        for (index, assertion) in simulation.observability_assertions.iter().enumerate() {
            if index > 0 {
                html.push_str("<br>");
            }
            html.push_str("<code>");
            push_html_escaped(html, &assertion.code);
            html.push_str("</code>: ");
            push_html_escaped(html, &assertion.signal);
        }
        html.push_str("</td><td><code>");
        push_html_escaped(html, &simulation.repro_command);
        html.push_str("</code><br><code>");
        push_html_escaped(html, &simulation.recovery_command);
        html.push_str("</code></td></tr>");
    }
    html.push_str("</tbody></table></section>");
}
