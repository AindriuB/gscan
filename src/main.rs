use gscan::{output, scan};
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

const HELP: &str = "Usage: gscan [DIRECTORY]\n       gscan --help | --version\n\nScan immediate child Git repositories (default: current directory).\nShows changed paths and ahead/behind counts from locally cached refs.\nRead-only: does not fetch. Use -- before a path beginning with '-'.\n";

enum Action {
    Scan(PathBuf),
    Help,
    Version,
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
    let mut positional = None;
    let mut options = true;
    for arg in args {
        if options && arg == "--" {
            options = false;
            continue;
        }
        if options && arg.as_encoded_bytes().starts_with(b"-") {
            return Err("unknown option or help/version combined with other arguments");
        }
        if positional.replace(PathBuf::from(arg)).is_some() {
            return Err("expected at most one directory");
        }
    }
    Ok(Action::Scan(
        positional.unwrap_or_else(|| PathBuf::from(".")),
    ))
}

fn run(out: &mut impl Write, err: &mut impl Write) -> io::Result<u8> {
    let action = match arguments(std::env::args_os().skip(1).collect()) {
        Ok(action) => action,
        Err(message) => {
            writeln!(err, "gscan: {message}\n{HELP}")?;
            return Ok(2);
        }
    };
    let root = match action {
        Action::Help => {
            write!(out, "{HELP}")?;
            return Ok(0);
        }
        Action::Version => {
            writeln!(out, "gscan {}", env!("CARGO_PKG_VERSION"))?;
            return Ok(0);
        }
        Action::Scan(root) => root,
    };
    let report = match scan::scan(&root) {
        Ok(report) => report,
        Err(error) => {
            write!(err, "{}", output::diagnostic(&error))?;
            return Ok(1);
        }
    };
    if !report.repositories.is_empty() {
        write!(out, "{}", output::table(&report.repositories))?;
    } else if report.failures.is_empty() {
        writeln!(out, "No repositories found.")?;
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
