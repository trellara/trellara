use std::process::Command as ProcessCommand;

use serde::Serialize;

use crate::{
    dev_compose_prefix, dev_services, shell_quote, CliError, DevDownArgs, DevLogsArgs,
    DevStackArgs, Result,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct DevCommandSummary {
    action: String,
    command: String,
    services: Vec<String>,
    dry_run: bool,
    executed: bool,
    exit_status: Option<String>,
    note: String,
}

pub(crate) fn dev_up_command(args: &DevStackArgs) -> Result<String> {
    render_dev_command_summary(dev_up_summary(args)?)
}

pub(crate) fn dev_down_command(args: &DevDownArgs) -> Result<String> {
    render_dev_command_summary(dev_down_summary(args)?)
}

pub(crate) fn dev_logs_command(args: &DevLogsArgs) -> Result<String> {
    render_dev_command_summary(dev_logs_summary(args)?)
}

fn dev_up_summary(args: &DevStackArgs) -> Result<DevCommandSummary> {
    let services = dev_services(args.with_kafka, args.runtime);
    let mut command_args = dev_compose_prefix(args.runtime);
    command_args.extend(["up".to_string(), "-d".to_string()]);
    command_args.extend(services.iter().cloned());
    let command = shell_command("docker", &command_args);
    let exit_status = run_process_command("docker", &command_args, args.dry_run)?;

    Ok(DevCommandSummary {
        action: "up".to_string(),
        command,
        services,
        dry_run: args.dry_run,
        executed: !args.dry_run,
        exit_status,
        note: if args.runtime {
            "starts source Postgres, target Postgres, Redpanda, relay, and applier"
        } else if args.with_kafka {
            "starts source Postgres, target Postgres, and Redpanda for Kafka-mode tests"
        } else {
            "starts the no-broker local source and target Postgres stack"
        }
        .to_string(),
    })
}

fn dev_down_summary(args: &DevDownArgs) -> Result<DevCommandSummary> {
    let mut command_args = vec!["compose".to_string(), "down".to_string()];
    if args.volumes {
        command_args.push("-v".to_string());
    }
    let command = shell_command("docker", &command_args);
    let exit_status = run_process_command("docker", &command_args, args.dry_run)?;

    Ok(DevCommandSummary {
        action: "down".to_string(),
        command,
        services: Vec::new(),
        dry_run: args.dry_run,
        executed: !args.dry_run,
        exit_status,
        note: if args.volumes {
            "stops the local development stack and removes compose volumes"
        } else {
            "stops the local development stack"
        }
        .to_string(),
    })
}

fn dev_logs_summary(args: &DevLogsArgs) -> Result<DevCommandSummary> {
    let services = dev_services(args.with_kafka, args.runtime);
    let mut command_args = dev_compose_prefix(args.runtime);
    command_args.push("logs".to_string());
    if args.follow {
        command_args.push("-f".to_string());
    }
    command_args.extend(services.iter().cloned());
    let command = shell_command("docker", &command_args);
    let exit_status = run_process_command("docker", &command_args, args.dry_run)?;

    Ok(DevCommandSummary {
        action: "logs".to_string(),
        command,
        services,
        dry_run: args.dry_run,
        executed: !args.dry_run,
        exit_status,
        note: "shows local development stack logs".to_string(),
    })
}

fn render_dev_command_summary(summary: DevCommandSummary) -> Result<String> {
    Ok(serde_json::to_string_pretty(&summary)?)
}

fn run_process_command(program: &str, args: &[String], dry_run: bool) -> Result<Option<String>> {
    if dry_run {
        return Ok(None);
    }

    let status = ProcessCommand::new(program)
        .args(args)
        .status()
        .map_err(|source| CliError::RunCommand {
            command: shell_command(program, args),
            source,
        })?;
    if status.success() {
        Ok(Some(status_display(status)))
    } else {
        Err(CliError::CommandFailed {
            command: shell_command(program, args),
            status: status_display(status),
        })
    }
}

fn status_display(status: std::process::ExitStatus) -> String {
    status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| "signal".to_string())
}

fn shell_command(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().map(|arg| shell_quote(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}
