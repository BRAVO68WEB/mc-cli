// Create a new process to run the server and return a handle
use std::process::{Command, Child};

pub async fn run_cmd(cmd_args: &[&str]) -> Result<Child, Box<dyn std::error::Error>> {
    run_cmd_with_io(cmd_args, true).await
}

pub async fn run_cmd_with_io(cmd_args: &[&str], inherit_stdio: bool) -> Result<Child, Box<dyn std::error::Error>> {
    let mut cmd = Command::new(cmd_args[0]);
    cmd.args(&cmd_args[1..]);

    if inherit_stdio {
        cmd.stdout(std::process::Stdio::inherit());
        cmd.stderr(std::process::Stdio::inherit());
    } else {
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
    }

    let child = cmd.spawn()?;
    println!("Command started successfully with PID: {}", child.id());
    
    // return process handle
    Ok(child)
}
