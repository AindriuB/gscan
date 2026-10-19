use std::ffi::OsString;
use std::fs;
use std::os::unix::{ffi::OsStringExt, fs::symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Fixture {
    base: PathBuf,
    root: PathBuf,
    home: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let base = std::env::temp_dir().join(format!(
            "gscan-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&base).unwrap();
        let root = base.join("projects");
        let home = base.join("home");
        fs::create_dir(&root).unwrap();
        fs::create_dir(&home).unwrap();
        Self { base, root, home }
    }
    fn command(&self, executable: &str) -> Command {
        let mut cmd = Command::new(executable);
        for (key, _) in std::env::vars_os() {
            if key.as_encoded_bytes().starts_with(b"GIT_") {
                cmd.env_remove(key);
            }
        }
        cmd.current_dir(&self.root)
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", &self.home)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_AUTHOR_NAME", "Test")
            .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
            .env("GIT_COMMITTER_NAME", "Test")
            .env("GIT_COMMITTER_EMAIL", "test@example.invalid");
        cmd
    }
    fn git(&self, repo: &Path, args: &[&str]) -> Output {
        self.command("git")
            .arg("-C")
            .arg(repo)
            .args([
                "-c",
                "commit.gpgsign=false",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .args(args)
            .output()
            .unwrap()
    }
    fn ok(&self, repo: &Path, args: &[&str]) {
        let out = self.git(repo, args);
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    fn repo(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        fs::create_dir(&path).unwrap();
        self.ok(&path, &["init", "-b", "main"]);
        self.ok(&path, &["config", "user.name", "Test"]);
        self.ok(&path, &["config", "user.email", "test@example.invalid"]);
        path
    }
    fn commit(&self, repo: &Path) {
        self.ok(repo, &["add", "--all"]);
        self.ok(repo, &["commit", "--allow-empty", "-m", "test"]);
    }
    fn cli(&self) -> Command {
        self.command(env!("CARGO_BIN_EXE_gscan"))
    }
    fn run(&self) -> Output {
        self.cli().output().unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}
fn text(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).unwrap()
}
fn fields(out: &Output, name: &str) -> Vec<String> {
    text(out)
        .lines()
        .find(|l| l.split_whitespace().next() == Some(name))
        .unwrap()
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

#[test]
fn arguments_empty_and_invalid_roots() {
    let f = Fixture::new();
    assert_eq!(text(&f.run()), "No repositories found.\n");
    for args in [vec!["--help"], vec!["-h"], vec!["--version"]] {
        let out = f.cli().args(args).output().unwrap();
        assert!(out.status.success());
        assert!(out.stderr.is_empty());
    }
    for args in [
        vec!["--wat"],
        vec!["a", "b"],
        vec!["--help", "a"],
        vec!["--version", "a"],
    ] {
        let out = f.cli().args(args).output().unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert!(!out.stderr.is_empty());
    }
    fs::create_dir(f.root.join("-dir")).unwrap();
    assert!(
        f.cli()
            .args(["--", "-dir"])
            .output()
            .unwrap()
            .status
            .success()
    );
    let out = f.cli().arg("absent").output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    fs::write(f.root.join("file"), "").unwrap();
    assert_eq!(f.cli().arg("file").output().unwrap().status.code(), Some(1));
}

#[test]
fn counts_changed_paths_and_preserves_repository_state() {
    let f = Fixture::new();
    let repo = f.repo("dirty");
    fs::write(repo.join("file"), "initial").unwrap();
    fs::write(repo.join(".gitignore"), "ignored\n").unwrap();
    f.commit(&repo);
    fs::write(repo.join("file"), "staged").unwrap();
    f.ok(&repo, &["add", "file"]);
    fs::write(repo.join("file"), "unstaged").unwrap();
    fs::create_dir(repo.join("directory")).unwrap();
    fs::write(repo.join("directory/a"), "a").unwrap();
    fs::write(repo.join("directory/b"), "b").unwrap();
    fs::write(repo.join("ignored"), "ignored").unwrap();
    let paths = [
        ".git/index",
        ".git/config",
        ".git/HEAD",
        ".git/refs/heads/main",
    ];
    let before: Vec<_> = paths
        .iter()
        .map(|p| fs::read(repo.join(p)).unwrap())
        .collect();
    let out = f.run();
    assert!(out.status.success());
    assert_eq!(fields(&out, "dirty"), ["dirty", "main", "3", "-", "-"]);
    let after: Vec<_> = paths
        .iter()
        .map(|p| fs::read(repo.join(p)).unwrap())
        .collect();
    assert_eq!(before, after);
}

#[test]
fn states_worktrees_symlinks_bare_and_partial_failure() {
    let f = Fixture::new();
    let a = f.repo("a");
    f.commit(&a);
    f.ok(
        &a,
        &[
            "worktree",
            "add",
            "--detach",
            f.root.join("worktree").to_str().unwrap(),
        ],
    );
    let z = f.repo("z");
    fs::write(z.join("untracked"), "x").unwrap();
    symlink(&a, f.root.join("symlink")).unwrap();
    f.ok(&f.root, &["init", "--bare", "bare.git"]);
    fs::create_dir(f.root.join("ordinary")).unwrap();
    fs::create_dir(f.root.join("broken")).unwrap();
    fs::write(f.root.join("broken/.git"), "garbage").unwrap();
    let out = f.run();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(fields(&out, "a"), ["a", "main", "0", "-", "-"]);
    assert!(fields(&out, "worktree")[1].starts_with("detached@"));
    assert_eq!(fields(&out, "z"), ["z", "main", "(unborn)", "1", "-", "-"]);
    let names: Vec<_> = text(&out)
        .lines()
        .skip(1)
        .map(|l| l.split_whitespace().next().unwrap().to_owned())
        .collect();
    assert_eq!(names, ["a", "worktree", "z"]);
    assert!(String::from_utf8(out.stderr).unwrap().contains("broken"));
}

#[test]
fn all_failed_is_not_empty_and_missing_git_fails() {
    let f = Fixture::new();
    let repo = f.repo("repo");
    let out = f.cli().env("PATH", &f.home).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    fs::remove_dir_all(repo.join(".git")).unwrap();
    fs::write(repo.join(".git"), "broken").unwrap();
    let out = f.run();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
}

#[test]
fn hostile_environment_does_not_redirect_status() {
    let f = Fixture::new();
    let repo = f.repo("repo");
    f.commit(&repo);
    let out = f
        .cli()
        .env("GIT_DIR", "/does-not-exist")
        .env("GIT_WORK_TREE", "/does-not-exist")
        .env("GIT_INDEX_FILE", "/does-not-exist")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.bare")
        .env("GIT_CONFIG_VALUE_0", "true")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(fields(&out, "repo"), ["repo", "main", "0", "-", "-"]);
}

#[test]
fn odd_paths_and_rename_names_are_safe() {
    let f = Fixture::new();
    let repo = f.repo("original");
    fs::write(repo.join("old\nname"), "content").unwrap();
    f.commit(&repo);
    f.ok(&repo, &["mv", "old\nname", "new\nname"]);
    let name = OsString::from_vec(b"odd\n\x1b\\\xff".to_vec());
    fs::rename(&repo, f.root.join(name)).unwrap();
    let out = f.cli().arg(&f.root).output().unwrap();
    assert!(out.status.success());
    assert_eq!(
        fields(&out, "odd\\x0a\\x1b\\\\\\xff"),
        ["odd\\x0a\\x1b\\\\\\xff", "main", "1", "-", "-"]
    );
    assert!(
        out.stdout
            .iter()
            .all(|b| b.is_ascii() && (*b >= b' ' || *b == b'\n'))
    );
    let odd_root = f.base.join(OsString::from_vec(b"root\xff".to_vec()));
    fs::rename(&f.root, &odd_root).unwrap();
    assert!(
        f.cli()
            .current_dir(&f.base)
            .arg(odd_root)
            .output()
            .unwrap()
            .status
            .success()
    );
}

#[test]
fn upstream_divergence_and_deleted_tracking_ref() {
    let f = Fixture::new();
    let repo = f.repo("repo");
    f.commit(&repo);
    let remote = f.base.join("remote.git");
    f.ok(&f.base, &["init", "--bare", remote.to_str().unwrap()]);
    f.ok(
        &repo,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    f.ok(&repo, &["push", "-u", "origin", "main"]);
    f.commit(&repo);
    f.ok(&repo, &["push", "origin", "main"]);
    f.ok(&repo, &["reset", "--hard", "HEAD~1"]);
    fs::write(repo.join("different"), "different").unwrap();
    f.commit(&repo);
    assert_eq!(fields(&f.run(), "repo"), ["repo", "main", "0", "1", "1"]);
    f.ok(&repo, &["update-ref", "-d", "refs/remotes/origin/main"]);
    assert_eq!(fields(&f.run(), "repo"), ["repo", "main", "0", "-", "-"]);
}

#[test]
fn merge_conflicts_count_as_paths_not_errors() {
    let f = Fixture::new();
    let repo = f.repo("repo");
    fs::write(repo.join("file"), "base\n").unwrap();
    f.commit(&repo);
    f.ok(&repo, &["checkout", "-b", "other"]);
    fs::write(repo.join("file"), "other\n").unwrap();
    f.commit(&repo);
    f.ok(&repo, &["checkout", "main"]);
    fs::write(repo.join("file"), "main\n").unwrap();
    f.commit(&repo);
    assert!(!f.git(&repo, &["merge", "other"]).status.success());
    let out = f.run();
    assert!(out.status.success());
    assert_eq!(fields(&out, "repo"), ["repo", "main", "1", "-", "-"]);
}

#[test]
fn output_errors_return_failure() {
    let f = Fixture::new();
    let full = fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .unwrap();
    let out = f.cli().stdout(full).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(
        String::from_utf8(out.stderr)
            .unwrap()
            .contains("output failed")
    );
}

#[test]
fn submodules_count_once_and_honour_ignore_config() {
    let f = Fixture::new();
    let source = f.repo("source");
    fs::write(source.join("file"), "initial").unwrap();
    f.commit(&source);
    let repo = f.repo("repo");
    f.ok(
        &repo,
        &[
            "-c",
            "protocol.file.allow=always",
            "submodule",
            "add",
            source.to_str().unwrap(),
            "sub",
        ],
    );
    f.commit(&repo);
    fs::write(repo.join("sub/file"), "modified").unwrap();
    fs::write(repo.join("sub/untracked"), "untracked").unwrap();
    assert_eq!(fields(&f.run(), "repo"), ["repo", "main", "1", "-", "-"]);
    f.ok(&repo, &["config", "submodule.sub.ignore", "all"]);
    assert_eq!(fields(&f.run(), "repo"), ["repo", "main", "0", "-", "-"]);
}
