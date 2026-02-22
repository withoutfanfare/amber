use std::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::time::{timeout, Duration};

pub struct SshTunnel {
    child: Child,
    local_port: u16,
}

impl SshTunnel {
    /// Spawn an SSH tunnel: `ssh -L local_port:db_host:db_port user@ssh_host -p ssh_port -N`
    pub async fn open(
        ssh_host: &str,
        ssh_port: u16,
        ssh_user: &str,
        ssh_key_path: Option<&str>,
        db_host: &str,
        db_port: u16,
    ) -> Result<Self, crate::error::DsmError> {
        let local_port = find_available_port()?;

        let mut cmd = Command::new("ssh");
        cmd.arg("-L")
            .arg(format!("{local_port}:{db_host}:{db_port}"));
        cmd.arg(format!("{ssh_user}@{ssh_host}"));
        cmd.arg("-p").arg(ssh_port.to_string());
        cmd.arg("-N"); // No remote command -- tunnel only
        cmd.arg("-o").arg("StrictHostKeyChecking=accept-new");
        cmd.arg("-o").arg("ConnectTimeout=10");
        cmd.arg("-o").arg("ServerAliveInterval=15");
        cmd.arg("-o").arg("ServerAliveCountMax=3");
        cmd.arg("-o").arg("ExitOnForwardFailure=yes");

        if let Some(key_path) = ssh_key_path {
            cmd.arg("-i").arg(key_path);
        }

        // Detach from stdin to prevent blocking
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::piped());

        let child = cmd
            .spawn()
            .map_err(|e| crate::error::DsmError::SshError(format!("Failed to spawn ssh: {e}")))?;

        let tunnel = Self { child, local_port };

        // Wait for tunnel to become ready (poll TCP connection to local_port)
        tunnel.wait_for_ready().await?;

        Ok(tunnel)
    }

    /// Poll the local port until it accepts connections (tunnel is ready).
    async fn wait_for_ready(&self) -> Result<(), crate::error::DsmError> {
        let addr = format!("127.0.0.1:{}", self.local_port);
        let result = timeout(Duration::from_secs(10), async {
            loop {
                if TcpStream::connect(&addr).is_ok() {
                    return Ok::<(), crate::error::DsmError>(());
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            _ => Err(crate::error::DsmError::SshError(
                "SSH tunnel failed to become ready within 10 seconds".to_string(),
            )),
        }
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    /// Kill the SSH tunnel process.
    pub async fn close(mut self) -> Result<(), crate::error::DsmError> {
        self.child.kill().await.map_err(|e| {
            crate::error::DsmError::SshError(format!("Failed to kill SSH tunnel: {e}"))
        })
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        // Best-effort kill on drop -- fire and forget
        let _ = self.child.start_kill();
    }
}

/// Find an available TCP port by binding to port 0.
fn find_available_port() -> Result<u16, crate::error::DsmError> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| {
        crate::error::DsmError::SshError(format!("Failed to find available port: {e}"))
    })?;
    Ok(listener.local_addr().unwrap().port())
}
