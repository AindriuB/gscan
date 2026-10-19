use crate::git::{self, RepoStatus};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

#[derive(Debug)]
pub struct Failure {
    pub path: PathBuf,
    pub message: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct ScanReport {
    pub repositories: Vec<RepoStatus>,
    pub failures: Vec<Failure>,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub recursive: bool,
    pub jobs: usize,
    pub timeout: Duration,
    pub fetch: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            recursive: false,
            jobs: 1,
            timeout: Duration::from_secs(30),
            fetch: false,
        }
    }
}

fn failure(path: &Path, error: impl std::fmt::Display) -> Failure {
    Failure {
        path: path.to_owned(),
        message: error.to_string().into_bytes(),
    }
}

fn bare_store(path: &Path) -> bool {
    path.join("HEAD").is_file() && path.join("objects").is_dir() && path.join("refs").is_dir()
}

fn discover(
    root: &Path,
    recursive: bool,
    report: &mut ScanReport,
) -> Result<Vec<PathBuf>, Failure> {
    let mut pending = vec![root.to_owned()];
    let mut candidates = Vec::new();
    while let Some(directory) = pending.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(e) if directory == root => {
                return Err(failure(root, format!("cannot read directory: {e}")));
            }
            Err(e) => {
                report.failures.push(failure(&directory, e));
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    report.failures.push(failure(&directory, e));
                    continue;
                }
            };
            let path = entry.path();
            if entry.file_name() == ".git" {
                continue;
            }
            match entry.file_type() {
                Ok(kind) if kind.is_dir() => {}
                Ok(_) => continue,
                Err(e) => {
                    report.failures.push(failure(&path, e));
                    continue;
                }
            }
            match fs::symlink_metadata(path.join(".git")) {
                Ok(_) => candidates.push(path),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    if recursive && !bare_store(&path) {
                        pending.push(path);
                    }
                }
                Err(e) => report.failures.push(failure(&path, e)),
            }
        }
    }
    candidates.sort();
    Ok(candidates)
}

pub fn scan(root: &Path) -> Result<ScanReport, Failure> {
    scan_with(root, &Options::default())
}

pub fn scan_with(root: &Path, options: &Options) -> Result<ScanReport, Failure> {
    if !(1..=64).contains(&options.jobs) || options.timeout.is_zero() {
        return Err(failure(root, "invalid worker count or timeout"));
    }
    let mut report = ScanReport::default();
    let candidates = discover(root, options.recursive, &mut report)?;
    // Linked worktrees may share refs. Never run concurrent fetch writers.
    if options.fetch {
        for path in &candidates {
            if let Err(message) = git::fetch(path, options.timeout) {
                report.failures.push(Failure {
                    path: path.clone(),
                    message,
                });
            }
        }
    }
    let next = AtomicUsize::new(0);
    let worker = || {
        let mut results = Vec::new();
        while let Some(path) = candidates.get(next.fetch_add(1, Ordering::Relaxed)) {
            results.push((
                path.clone(),
                git::inspect_with_timeout(path, options.timeout),
            ));
        }
        results
    };
    let batches = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for _ in 1..options.jobs.min(candidates.len()) {
            match std::thread::Builder::new().spawn_scoped(scope, worker) {
                Ok(handle) => handles.push(handle),
                Err(_) => break, // Available workers (including this thread) finish the queue.
            }
        }
        let mut batches = vec![worker()];
        for handle in handles {
            match handle.join() {
                Ok(batch) => batches.push(batch),
                Err(_) => report
                    .failures
                    .push(failure(root, "inspection worker panicked")),
            }
        }
        batches
    });
    for (path, result) in batches.into_iter().flatten() {
        match result {
            Ok(status) => report.repositories.push(status),
            Err(message) => report.failures.push(Failure { path, message }),
        }
    }
    report.repositories.sort_by(|a, b| a.path.cmp(&b.path));
    report
        .failures
        .sort_by(|a, b| (&a.path, &a.message).cmp(&(&b.path, &b.message)));
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::process::Command;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "gscan-scan-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[cfg(unix)]
    #[test]
    fn discovery_orders_rows_and_preserves_partial_results() {
        let temp = Temp::new();
        for name in ["z", "a"] {
            let out = Command::new("git")
                .args(["init", "-b", "main"])
                .arg(temp.0.join(name))
                .env("GIT_CONFIG_GLOBAL", "/dev/null")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .output()
                .unwrap();
            assert!(out.status.success());
        }
        fs::create_dir(temp.0.join("ordinary")).unwrap();
        fs::create_dir(temp.0.join("broken")).unwrap();
        fs::write(temp.0.join("broken/.git"), "bad metadata").unwrap();
        std::os::unix::fs::symlink(temp.0.join("a"), temp.0.join("linked-directory")).unwrap();
        let report = scan(&temp.0).unwrap();
        assert_eq!(
            report
                .repositories
                .iter()
                .map(|r| r.path.file_name().unwrap().to_str().unwrap())
                .collect::<Vec<_>>(),
            ["a", "z"]
        );
        assert_eq!(report.failures.len(), 1);
        assert_eq!(report.failures[0].path, temp.0.join("broken"));
    }
    #[test]
    fn empty_and_missing_roots() {
        let temp = Temp::new();
        let report = scan(&temp.0).unwrap();
        assert!(report.repositories.is_empty() && report.failures.is_empty());
        assert!(scan(&temp.0.join("missing")).is_err());
    }
}
