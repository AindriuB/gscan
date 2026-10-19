use gscan::{output, scan};
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

const HELP: &str = "Usage: gscan [OPTIONS] [DIRECTORY]
       gscan --help | --version

Scan child Git repositories (default directory: current directory).
  --dirty          Show only repositories with changed paths
  --json           Emit schema-versioned JSON, including errors
  --recursive      Descend ordinary directories; stop at repositories
  --jobs N         Parallel inspections, 1-64 (default: 1)
  --timeout N      Seconds per Git command, 1-3600 (default: 30)
  --fetch          Refresh configured remotes before inspecting (writes refs)

Default scans are read-only and use locally cached upstream refs.
Use -- before a path beginning with '-'. Help/version must stand alone.
";

struct ScanArgs {
    root: PathBuf,
    dirty_only: bool,
    json: bool,
    options: scan::Options,
}

enum Action {
    Scan(ScanArgs),
    Help,
    Version,
}

fn numeric(value: Option<OsString>, max: u64) -> Result<u64, &'static str> {
    let value = value.ok_or("missing numeric option value")?;
    let text = value.to_str().ok_or("numeric option value must be ASCII")?;
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err("numeric option value must be a positive integer");
    }
    let number = text
        .parse::<u64>()
        .map_err(|_| "numeric option value is too large")?;
    if number == 0 || number > max {
        return Err("numeric option value is out of range");
    }
    Ok(number)
}

fn arguments(args: Vec<OsString>) -> Result<Action, &'static str> {
    if args.len() == 1 {
        if args[0] == "--help" || args[0] == "-h" {
            return Ok(Action::Help);
        }
        if args[0] == "--version" {
            return Ok(Action::Version);
        }
    }
    let mut scan = ScanArgs {
        root: PathBuf::from("."),
        dirty_only: false,
        json: false,
        options: scan::Options::default(),
    };
    let mut positional = None;
    let mut options = true;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        if options && arg == "--" {
            options = false;
            continue;
        }
        if options {
            match arg.to_str() {
                Some("--dirty") => {
                    scan.dirty_only = true;
                    continue;
                }
                Some("--json") => {
                    scan.json = true;
                    continue;
                }
                Some("--recursive") => {
                    scan.options.recursive = true;
                    continue;
                }
                Some("--fetch") => {
                    scan.options.fetch = true;
                    continue;
                }
                _ => {}
            }
            let pair = arg.to_str().and_then(|s| s.split_once('='));
            let key = pair.map(|p| p.0).or_else(|| arg.to_str());
            if matches!(key, Some("--jobs" | "--timeout")) {
                let value = pair.map(|p| OsString::from(p.1)).or_else(|| args.next());
                if key == Some("--jobs") {
                    scan.options.jobs = numeric(value, 64)? as usize;
                } else {
                    scan.options.timeout = std::time::Duration::from_secs(numeric(value, 3600)?);
                }
                continue;
            }
            if arg.as_encoded_bytes().starts_with(b"-") {
                return Err("unknown option or help/version combined with other arguments");
            }
        }
        if positional.replace(PathBuf::from(arg)).is_some() {
            return Err("expected at most one directory");
        }
    }
    if let Some(root) = positional {
        scan.root = root;
    }
    Ok(Action::Scan(scan))
}

fn run(out: &mut impl Write, err: &mut impl Write) -> io::Result<u8> {
    let action = match arguments(std::env::args_os().skip(1).collect()) {
        Ok(action) => action,
        Err(message) => {
            writeln!(err, "gscan: {message}\n{HELP}")?;
            return Ok(2);
        }
    };
    let args = match action {
        Action::Help => {
            write!(out, "{HELP}")?;
            return Ok(0);
        }
        Action::Version => {
            writeln!(out, "gscan {}", env!("CARGO_PKG_VERSION"))?;
            return Ok(0);
        }
        Action::Scan(args) => args,
    };
    let mut report = match scan::scan_with(&args.root, &args.options) {
        Ok(report) => report,
        Err(error) => {
            if args.json {
                let report = scan::ScanReport {
                    repositories: Vec::new(),
                    failures: vec![error],
                };
                write!(out, "{}", output::json(&report, &args.root))?;
            } else {
                write!(err, "{}", output::diagnostic(&error))?;
            }
            return Ok(1);
        }
    };
    let had_repositories = !report.repositories.is_empty();
    if args.dirty_only {
        report.repositories.retain(|repo| repo.changed > 0);
    }
    if args.json {
        write!(out, "{}", output::json(&report, &args.root))?;
        return Ok(u8::from(!report.failures.is_empty()));
    }
    if !report.repositories.is_empty() {
        write!(
            out,
            "{}",
            output::table_with_root(&report.repositories, Some(&args.root))
        )?;
    } else if report.failures.is_empty() {
        if args.dirty_only && had_repositories {
            writeln!(out, "No dirty repositories found.")?;
        } else {
            writeln!(out, "No repositories found.")?;
        }
    }
    for error in &report.failures {
        write!(err, "{}", output::diagnostic(error))?;
    }
    Ok(u8::from(!report.failures.is_empty()))
}

fn main() -> ExitCode {
    let mut out = io::stdout().lock();
    let mut err = io::stderr().lock();
    let result = run(&mut out, &mut err).and_then(|code| {
        out.flush()?;
        err.flush()?;
        Ok(code)
    });
    match result {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            let _ = writeln!(
                err,
                "gscan: output failed: {}",
                output::escape(error.to_string().as_bytes())
            );
            ExitCode::FAILURE
        }
    }
}
