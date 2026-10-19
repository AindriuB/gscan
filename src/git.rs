use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub enum Branch {
    Named(Vec<u8>),
    Unborn(Vec<u8>),
    Detached(Vec<u8>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct RepoStatus {
    pub path: PathBuf,
    pub branch: Branch,
    pub changed: usize,
    pub upstream: Option<Vec<u8>>,
    pub comparison: Option<(usize, usize)>,
}

fn command(path: &Path) -> Command {
    let mut command = Command::new("git");
    for (name, _) in std::env::vars_os() {
        if name.as_encoded_bytes().starts_with(b"GIT_") {
            command.env_remove(name);
        }
    }
    command
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GCM_INTERACTIVE", "Never")
        .env("GIT_ASKPASS", "")
        .env("SSH_ASKPASS", "")
        .env("LC_ALL", "C")
        .args(["--no-optional-locks", "-C"])
        .arg(path);
    command
}

fn execute(path: &Path, args: &[&str], timeout: Duration) -> Result<Vec<u8>, Vec<u8>> {
    let output = crate::process::run(command(path).args(args), timeout)
        .map_err(|e| format!("cannot run git: {e}").into_bytes())?;
    if !output.status.success() {
        let mut message = format!("git {} failed ({})", args[0], output.status).into_bytes();
        if !output.stderr.is_empty() {
            message.extend_from_slice(b": ");
            message.extend_from_slice(output.stderr.trim_ascii_end());
        }
        return Err(message);
    }
    Ok(output.stdout)
}

pub fn inspect(path: &Path) -> Result<RepoStatus, Vec<u8>> {
    inspect_with_timeout(path, Duration::from_secs(30))
}

pub fn inspect_with_timeout(path: &Path, timeout: Duration) -> Result<RepoStatus, Vec<u8>> {
    let output = execute(
        path,
        &[
            "status",
            "--porcelain=v2",
            "--branch",
            "-z",
            "--untracked-files=all",
        ],
        timeout,
    )?;
    parse(path, &output).map_err(|e| format!("invalid git status output: {e}").into_bytes())
}

pub fn fetch(path: &Path, timeout: Duration) -> Result<(), Vec<u8>> {
    execute(
        path,
        &[
            "fetch",
            "--all",
            "--no-tags",
            "--no-prune",
            "--no-prune-tags",
            "--no-recurse-submodules",
            "--no-auto-maintenance",
            "--no-write-fetch-head",
        ],
        timeout,
    )
    .map(|_| ())
}

fn number(bytes: &[u8]) -> Result<usize, &'static str> {
    if bytes.is_empty() || !bytes.iter().all(u8::is_ascii_digit) {
        return Err("invalid comparison count");
    }
    std::str::from_utf8(bytes)
        .map_err(|_| "invalid count encoding")?
        .parse()
        .map_err(|_| "comparison count overflow")
}

fn parse(path: &Path, data: &[u8]) -> Result<RepoStatus, &'static str> {
    if !data.ends_with(&[0]) {
        return Err("missing NUL terminator");
    }
    let mut records = data[..data.len() - 1].split(|b| *b == 0);
    let (mut head, mut oid, mut upstream, mut comparison) = (None, None, None, None);
    let mut changed = 0;
    while let Some(record) = records.next() {
        if let Some(value) = record.strip_prefix(b"# branch.head ") {
            if value.is_empty() || head.replace(value).is_some() {
                return Err("invalid branch header");
            }
        } else if let Some(value) = record.strip_prefix(b"# branch.oid ") {
            if oid.replace(value).is_some() {
                return Err("duplicate object header");
            }
        } else if let Some(value) = record.strip_prefix(b"# branch.upstream ") {
            if value.is_empty() || upstream.replace(value.to_vec()).is_some() {
                return Err("invalid upstream header");
            }
        } else if let Some(value) = record.strip_prefix(b"# branch.ab +") {
            let mut parts = value.split(|b| *b == b' ');
            let ahead = number(parts.next().ok_or("missing ahead count")?)?;
            let behind = number(
                parts
                    .next()
                    .and_then(|p| p.strip_prefix(b"-"))
                    .ok_or("missing behind count")?,
            )?;
            if parts.next().is_some() || comparison.replace((ahead, behind)).is_some() {
                return Err("invalid comparison header");
            }
        } else if record.starts_with(b"# ") {
            // Git permits new headers; ignore ones not used by this version.
        } else if record.starts_with(b"? ") || record.starts_with(b"! ") {
            if record.len() == 2 {
                return Err("missing path");
            }
            if record[0] == b'?' {
                changed += 1;
            }
        } else {
            let fields = match record.first() {
                Some(b'1') => 9,
                Some(b'2') => 10,
                Some(b'u') => 11,
                _ => return Err("unknown status record"),
            };
            let parts: Vec<_> = record.splitn(fields, |b| *b == b' ').collect();
            if parts.len() != fields || parts.iter().any(|p| p.is_empty()) || parts[0].len() != 1 {
                return Err("malformed status record");
            }
            if parts[1].len() != 2 || parts[2].len() != 4 {
                return Err("malformed status fields");
            }
            if record[0] == b'2' && records.next().is_none_or(|p| p.is_empty()) {
                return Err("missing rename source");
            }
            changed += 1;
        }
    }
    let head = head.ok_or("missing branch header")?;
    let oid = oid.ok_or("missing object header")?;
    let branch = if oid == b"(initial)" {
        if head == b"(detached)" {
            return Err("detached unborn branch");
        }
        comparison = None;
        Branch::Unborn(head.to_vec())
    } else {
        if !matches!(oid.len(), 40 | 64) || !oid.iter().all(u8::is_ascii_hexdigit) {
            return Err("invalid object ID");
        }
        if head == b"(detached)" {
            comparison = None;
            Branch::Detached(oid.to_vec())
        } else {
            Branch::Named(head.to_vec())
        }
    };
    if upstream.is_none() {
        comparison = None;
    }
    Ok(RepoStatus {
        path: path.to_owned(),
        branch,
        changed,
        upstream,
        comparison,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(records: &[u8]) -> Result<RepoStatus, &'static str> {
        let mut data =
            b"# branch.oid 0123456789012345678901234567890123456789\0# branch.head main\0".to_vec();
        data.extend_from_slice(records);
        parse(Path::new("repo"), &data)
    }

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let path = std::env::temp_dir().join(format!(
                "gscan-git-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn git(&self, args: &[&str]) {
            let mut cmd = Command::new("git");
            for (key, _) in std::env::vars_os() {
                if key.as_encoded_bytes().starts_with(b"GIT_") {
                    cmd.env_remove(key);
                }
            }
            let out = cmd
                .current_dir(&self.0)
                .env(
                    "GIT_CONFIG_GLOBAL",
                    if cfg!(windows) { "NUL" } else { "/dev/null" },
                )
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_AUTHOR_NAME", "Test")
                .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
                .env("GIT_COMMITTER_NAME", "Test")
                .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
                .args([
                    "-c",
                    "commit.gpgsign=false",
                    "-c",
                    if cfg!(windows) {
                        "core.hooksPath=NUL"
                    } else {
                        "core.hooksPath=/dev/null"
                    },
                ])
                .args(args)
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "git {args:?}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn real_repository_states_and_rename() {
        let f = Fixture::new();
        f.git(&["init", "-b", "main"]);
        assert!(matches!(inspect(&f.0).unwrap().branch, Branch::Unborn(_)));
        std::fs::write(f.0.join("old"), "content").unwrap();
        f.git(&["add", "old"]);
        f.git(&["commit", "-m", "initial"]);
        assert_eq!(inspect(&f.0).unwrap().changed, 0);
        f.git(&["mv", "old", "new name"]);
        assert_eq!(inspect(&f.0).unwrap().changed, 1);
        f.git(&["commit", "-m", "rename"]);
        f.git(&["checkout", "--detach"]);
        assert!(matches!(inspect(&f.0).unwrap().branch, Branch::Detached(_)));
    }

    #[test]
    fn real_local_upstream_ahead_and_behind() {
        let f = Fixture::new();
        f.git(&["init", "-b", "main"]);
        f.git(&["commit", "--allow-empty", "-m", "initial"]);
        f.git(&["init", "--bare", "remote.git"]);
        f.git(&["remote", "add", "origin", "remote.git"]);
        f.git(&["push", "-u", "origin", "main"]);
        assert_eq!(inspect(&f.0).unwrap().comparison, Some((0, 0)));
        f.git(&["commit", "--allow-empty", "-m", "ahead"]);
        assert_eq!(inspect(&f.0).unwrap().comparison, Some((1, 0)));
        f.git(&["push", "origin", "main"]);
        f.git(&["reset", "--hard", "HEAD~1"]);
        assert_eq!(inspect(&f.0).unwrap().comparison, Some((0, 1)));
    }

    #[test]
    fn counts_paths_and_consumes_rename_source() {
        let result = status(
            b"1 MM N... 100644 100644 100644 abc def staged and dirty\0\
            2 R. N... 100644 100644 100644 abc def R100 new\nname\0? original\0\
            u UU N... 100644 100644 100644 100644 abc def ghi conflict\0\
            ? directory/file\0! ignored\0",
        )
        .unwrap();
        assert_eq!(result.changed, 4);
    }

    #[test]
    fn preserves_upstream_and_missing_comparisons() {
        assert_eq!(status(b"").unwrap().comparison, None);
        let result = status(b"# branch.upstream origin/main\0# branch.ab +2 -3\0").unwrap();
        assert_eq!(result.comparison, Some((2, 3)));
        assert_eq!(result.upstream, Some(b"origin/main".to_vec()));
        let result = status(b"# branch.upstream origin/missing\0").unwrap();
        assert!(result.upstream.is_some());
        assert_eq!(result.comparison, None);
    }

    #[test]
    fn unborn_and_detached() {
        let result = parse(
            Path::new("repo"),
            b"# branch.oid (initial)\0# branch.head main\0? file\0",
        )
        .unwrap();
        assert_eq!(result.branch, Branch::Unborn(b"main".to_vec()));
        let result = parse(
            Path::new("repo"),
            b"# branch.oid 0123456789012345678901234567890123456789\0# branch.head (detached)\0",
        )
        .unwrap();
        assert!(matches!(result.branch, Branch::Detached(_)));
    }

    #[test]
    fn malformed_output_is_an_error() {
        for records in [
            b"? \0".as_slice(),
            b"1 M N...\0",
            b"2 R. N... 1 1 1 a b R100 new\0",
            b"# branch.ab +x -1\0",
            b"? unterminated",
            b"\0",
        ] {
            assert!(status(records).is_err(), "{records:?}");
        }
        assert!(parse(Path::new("repo"), b"").is_err());
    }
}
