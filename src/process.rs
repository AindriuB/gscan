//! Bounded Git execution. Files avoid pipe-reader threads blocked by grandchildren.
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

const OUTPUT_LIMIT: u64 = 32 * 1024 * 1024;

struct Capture(PathBuf);
impl Capture {
    fn new() -> io::Result<Self> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        for _ in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "gscan-output-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let builder = fs::DirBuilder::new();
            #[cfg(unix)]
            let builder = {
                use std::os::unix::fs::DirBuilderExt;
                let mut builder = builder;
                builder.mode(0o700);
                builder
            };
            match builder.create(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "cannot create capture directory",
        ))
    }
    fn read(&self, name: &str) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        File::open(self.0.join(name))?
            .take(OUTPUT_LIMIT + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > OUTPUT_LIMIT {
            return Err(io::Error::other("Git output exceeded 32 MiB"));
        }
        Ok(bytes)
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Running(Child);
impl Running {
    fn terminate(&mut self) {
        #[cfg(unix)]
        {
            // Each child is a new process group, including normal Git helpers.
            // The separator is essential: older procps can truncate a bare negative PID.
            let _ = Command::new("/bin/kill")
                .args(["-KILL", "--", &format!("-{}", self.0.id())])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/PID", &self.0.id().to_string()])
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
impl Drop for Running {
    fn drop(&mut self) {
        if !matches!(self.0.try_wait(), Ok(Some(_))) {
            self.terminate();
        }
    }
}

pub struct Output {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub fn run(command: &mut Command, timeout: Duration) -> io::Result<Output> {
    let capture = Capture::new()?;
    command
        .stdin(Stdio::null())
        .stdout(File::create(capture.0.join("stdout"))?)
        .stderr(File::create(capture.0.join("stderr"))?);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let start = Instant::now();
    let spawned = command.spawn();
    command.stdout(Stdio::null()).stderr(Stdio::null());
    let mut child = Running(spawned?);
    let status = loop {
        if let Some(status) = child.0.try_wait()? {
            break status;
        }
        if start.elapsed() >= timeout {
            child.terminate();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("Git command timed out after {} ms", timeout.as_millis()),
            ));
        }
        for name in ["stdout", "stderr"] {
            if fs::metadata(capture.0.join(name))?.len() > OUTPUT_LIMIT {
                child.terminate();
                return Err(io::Error::other("Git output exceeded 32 MiB"));
            }
        }
        std::thread::sleep(Duration::from_millis(10).min(timeout.saturating_sub(start.elapsed())));
    };
    Ok(Output {
        status,
        stdout: capture.read("stdout")?,
        stderr: capture.read("stderr")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    #[test]
    #[ignore = "subprocess fixture"]
    fn blocked_child() {
        if std::env::var_os("GSCAN_TEST_CHILD").is_none() {
            return;
        }
        // More than a pipe buffer: deadline must not wait for output readers.
        io::stdout().write_all(&vec![b'x'; 1024 * 1024]).unwrap();
        loop {
            std::thread::park();
        }
    }
    #[test]
    fn deadline_terminates_a_child_with_large_output() {
        let start = Instant::now();
        let error = run(
            Command::new(std::env::current_exe().unwrap())
                .env("GSCAN_TEST_CHILD", "1")
                .args([
                    "--ignored",
                    "--exact",
                    "process::tests::blocked_child",
                    "--nocapture",
                ]),
            Duration::from_millis(200),
        )
        .err()
        .unwrap();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(start.elapsed() < Duration::from_secs(10));
    }
    #[test]
    fn captures_successful_output() {
        let out = run(
            Command::new("git").arg("--version"),
            Duration::from_secs(10),
        )
        .unwrap();
        assert!(out.status.success());
        assert!(out.stdout.starts_with(b"git version "));
    }
}
