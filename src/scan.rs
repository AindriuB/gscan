use crate::git::{self, RepoStatus};
use std::fs;
use std::path::{Path, PathBuf};

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

fn failure(path: &Path, error: impl std::fmt::Display) -> Failure {
    Failure {
        path: path.to_owned(),
        message: error.to_string().into_bytes(),
    }
}

pub fn scan(root: &Path) -> Result<ScanReport, Failure> {
    let entries =
        fs::read_dir(root).map_err(|e| failure(root, format!("cannot read directory: {e}")))?;
    let mut report = ScanReport::default();
    let mut candidates = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                report.failures.push(failure(root, e));
                continue;
            }
        };
        let path = entry.path();
        match entry.file_type() {
            Ok(kind) if kind.is_dir() => {}
            Ok(_) => continue,
            Err(e) => {
                report.failures.push(failure(&path, e));
                continue;
            }
        }
        match fs::symlink_metadata(path.join(".git")) {
            // Let Git diagnose malformed or dangling metadata too.
            Ok(_) => candidates.push(path),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => report.failures.push(failure(&path, e)),
        }
    }
    candidates.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    for path in candidates {
        match git::inspect(&path) {
            Ok(status) => report.repositories.push(status),
            Err(message) => report.failures.push(Failure { path, message }),
        }
    }
    report.failures.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
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
