use super::Result;
use std::{
    ffi::OsStr,
    process::{Command, Stdio},
};

pub enum WaitFor<C>
where
    C: std::fmt::Display,
{
    Delete,
    Condition(C),
}

pub fn namespace<CMD, NS, R, COND, EX>(
    kubectl_command: CMD,
    namespace: NS,
    resources: impl IntoIterator<Item = R>,
    wait_for: WaitFor<COND>,
    extra: impl IntoIterator<Item = EX>,
) -> Result<()>
where
    CMD: AsRef<OsStr>,
    NS: AsRef<OsStr>,
    R: AsRef<OsStr>,
    COND: std::fmt::Display,
    EX: AsRef<OsStr>,
{
    let mut command = prepare_base_command(kubectl_command, resources, wait_for, extra);
    command.arg("-n").arg(namespace);
    run_wait_command(command)
}

pub fn all_namespaces<CMD, R, COND, EX>(
    kubectl_command: CMD,
    resources: impl IntoIterator<Item = R>,
    wait_for: WaitFor<COND>,
    extra: impl IntoIterator<Item = EX>,
) -> Result<()>
where
    CMD: AsRef<OsStr>,
    R: AsRef<OsStr>,
    COND: std::fmt::Display,
    EX: AsRef<OsStr>,
{
    let mut command = prepare_base_command(kubectl_command, resources, wait_for, extra);
    command.arg("--all-namespaces=true");
    run_wait_command(command)
}

pub fn prepare_base_command<CMD, R, COND, EX>(
    kubectl_command: CMD,
    resources: impl IntoIterator<Item = R>,
    wait_for: WaitFor<COND>,
    extra: impl IntoIterator<Item = EX>,
) -> Command
where
    CMD: AsRef<OsStr>,
    R: AsRef<OsStr>,
    COND: std::fmt::Display,
    EX: AsRef<OsStr>,
{
    let mut command = Command::new(kubectl_command);

    command
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    command.arg("wait");
    command.args(resources);

    command.arg("--for");
    match wait_for {
        WaitFor::Delete => command.arg("delete"),
        WaitFor::Condition(cond) => command.arg(format!("condition={}", cond)),
    };

    command.args(extra);
    command
}

fn run_wait_command(mut command: Command) -> Result<()> {
    let mut child = command.spawn()?;
    let exit_status = child.wait()?;
    if !exit_status.success() {
        Err(format!("waiting for resources failed: {:?}", command))?;
    }
    Ok(())
}
