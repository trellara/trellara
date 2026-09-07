use crate::*;

pub(crate) fn execute_native_command(command: NativeCommand) -> Result<String> {
    match command {
        NativeCommand::WorkerReport(args) => {
            let status = native_status_from_args(args.status);
            let view = native_worker_report_view_from_parts(
                &status,
                args.drained_frames,
                args.source_feedback_lsn,
                args.reason,
            );
            render_native_worker_report_view(&view, args.format)
        }
    }
}
