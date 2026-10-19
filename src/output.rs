use crate::git::{Branch, RepoStatus};
use crate::scan::Failure;
use std::fmt::Write;

pub fn escape(bytes: &[u8]) -> String {
    let mut text = String::new();
    for &byte in bytes {
        match byte {
            b'\\' => text.push_str("\\\\"),
            0x20..=0x7e => text.push(char::from(byte)),
            _ => {
                let _ = write!(text, "\\x{byte:02x}");
            }
        }
    }
    text
}

pub fn diagnostic(error: &Failure) -> String {
    format!(
        "gscan: {}: {}\n",
        escape(error.path.as_os_str().as_encoded_bytes()),
        escape(&error.message)
    )
}

pub fn table(repositories: &[RepoStatus]) -> String {
    let mut rows = vec![["REPOSITORY", "BRANCH", "CHANGES", "AHEAD", "BEHIND"].map(str::to_owned)];
    for repo in repositories {
        let branch = match &repo.branch {
            Branch::Named(name) => escape(name),
            Branch::Unborn(name) => format!("{} (unborn)", escape(name)),
            Branch::Detached(oid) => format!("detached@{}", escape(&oid[..oid.len().min(12)])),
        };
        let (ahead, behind) = repo
            .comparison
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .unwrap_or_else(|| ("-".into(), "-".into()));
        rows.push([
            escape(
                repo.path
                    .file_name()
                    .unwrap_or(repo.path.as_os_str())
                    .as_encoded_bytes(),
            ),
            branch,
            repo.changed.to_string(),
            ahead,
            behind,
        ]);
    }
    let mut widths = [0; 5];
    for row in &rows {
        for (i, field) in row.iter().enumerate() {
            widths[i] = widths[i].max(field.len());
        }
    }
    let mut output = String::new();
    for row in rows {
        let _ = writeln!(
            output,
            "{:<w0$}  {:<w1$}  {:>w2$}  {:>w3$}  {:>w4$}",
            row[0],
            row[1],
            row[2],
            row[3],
            row[4],
            w0 = widths[0],
            w1 = widths[1],
            w2 = widths[2],
            w3 = widths[3],
            w4 = widths[4]
        );
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escapes_bytes_without_terminal_controls() {
        assert_eq!(escape(b"a\\\n\x1b\xff"), "a\\\\\\x0a\\x1b\\xff");
    }
    #[test]
    fn renders_states_and_unavailable_counts() {
        let mut repo = RepoStatus {
            path: "a".into(),
            branch: Branch::Unborn(b"main".to_vec()),
            changed: 12,
            upstream: None,
            comparison: None,
        };
        assert_eq!(
            table(&[repo]),
            "REPOSITORY  BRANCH         CHANGES  AHEAD  BEHIND\na           main (unborn)       12      -       -\n"
        );
        repo = RepoStatus {
            path: "a".into(),
            branch: Branch::Detached(b"0123456789abcdef".to_vec()),
            changed: 0,
            upstream: None,
            comparison: None,
        };
        assert!(table(&[repo]).contains("detached@0123456789ab"));
    }
}
