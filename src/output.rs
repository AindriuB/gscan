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
    table_with_root(repositories, None)
}

pub fn table_with_root(repositories: &[RepoStatus], root: Option<&std::path::Path>) -> String {
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
                root.and_then(|root| repo.path.strip_prefix(root).ok())
                    .map(|p| p.as_os_str())
                    .unwrap_or_else(|| repo.path.file_name().unwrap_or(repo.path.as_os_str()))
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

fn quote(value: &str) -> String {
    let mut result = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\u{0}'..='\u{1f}' => {
                let _ = write!(result, "\\u{:04x}", ch as u32);
            }
            _ => result.push(ch),
        }
    }
    result.push('"');
    result
}

/// Text fields carry the same reversible ASCII representation as the table.
pub fn json(report: &crate::scan::ScanReport, root: &std::path::Path) -> String {
    let mut rows = Vec::new();
    for repo in &report.repositories {
        let (kind, name, oid) = match &repo.branch {
            Branch::Named(name) => ("named", quote(&escape(name)), "null".to_owned()),
            Branch::Unborn(name) => ("unborn", quote(&escape(name)), "null".to_owned()),
            Branch::Detached(oid) => ("detached", "null".to_owned(), quote(&escape(oid))),
        };
        let (ahead, behind) = repo
            .comparison
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .unwrap_or_else(|| ("null".into(), "null".into()));
        let upstream = repo
            .upstream
            .as_ref()
            .map(|s| quote(&escape(s)))
            .unwrap_or_else(|| "null".into());
        let path = quote(&escape(repo.path.as_os_str().as_encoded_bytes()));
        let relative = repo.path.strip_prefix(root).unwrap_or(&repo.path);
        let name_path = quote(&escape(relative.as_os_str().as_encoded_bytes()));
        rows.push(format!("{{\"path\":{path},\"name\":{name_path},\"branch\":{{\"kind\":\"{kind}\",\"name\":{name},\"oid\":{oid}}},\"changes\":{},\"upstream\":{upstream},\"ahead\":{ahead},\"behind\":{behind}}}", repo.changed));
    }
    let errors: Vec<_> = report
        .failures
        .iter()
        .map(|e| {
            format!(
                "{{\"path\":{},\"message\":{}}}",
                quote(&escape(e.path.as_os_str().as_encoded_bytes())),
                quote(&escape(&e.message))
            )
        })
        .collect();
    format!(
        "{{\"schema_version\":1,\"text_encoding\":\"escaped-os-bytes\",\"repositories\":[{}],\"errors\":[{}]}}\n",
        rows.join(","),
        errors.join(",")
    )
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
