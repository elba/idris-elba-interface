use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs::{self, File};
use std::hash::{Hash as _, Hasher as _};
use std::io::Read as _;
// use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{self, Child, Command, Stdio};
use std::sync::mpsc::channel;
use std::time::Duration;

use notify::{watcher, RecursiveMode, Watcher};

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let mut should_watch = false;

    if args.iter().any(|arg| arg == "--build") {
        args.retain(|arg| arg != "--build");
        args.insert(0, "build".to_owned());
        args.insert(1, "--".to_owned());
    } else if args.iter().any(|arg| arg == "--ide-mode") {
        args.retain(|arg| arg != "--ide-mode");
        args.insert(0, "repl".to_owned());
        args.insert(1, "--ide-mode".to_owned());
        args.insert(2, "--".to_owned());
        should_watch = true;
    } else if args.iter().any(|arg| arg == "--ide-mode-socket") {
        args.retain(|arg| arg != "--ide-mode-socket");
        args.insert(0, "repl".to_owned());
        args.insert(1, "--ide-mode-socket".to_owned());
        args.insert(2, "--".to_owned());
        should_watch = true;
    } else {
        args.insert(0, "repl".to_owned());
        args.insert(1, "--".to_owned());
    }

    let project_root = find_project_root();

    if !should_watch || project_root.is_none() {
        let mut process = start_process(&args);
        process::exit(process.wait().unwrap().code().unwrap_or(0));
    }

    let project_root = project_root.unwrap();

    // restart chlid on file manifest or lockfile changes
    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();

    let mut hash_manifest = read_content_hash(&project_root.join("elba.toml"));
    let mut hash_lockfile = read_content_hash(&project_root.join("elba.lock"));

    let mut process = start_process(&args);

    loop {
        watcher
            .watch(&project_root.join("elba.toml"), RecursiveMode::NonRecursive)
            .ok();
        watcher
            .watch(&project_root.join("elba.lock"), RecursiveMode::NonRecursive)
            .ok();

        if let Ok(_) = rx.recv_timeout(Duration::from_millis(100)) {
            let new_hash_manifest = read_content_hash(&project_root.join("elba.toml"));
            let new_hash_lockfile = read_content_hash(&project_root.join("elba.lock"));

            if new_hash_manifest != hash_manifest || new_hash_lockfile != hash_lockfile {
                hash_manifest = new_hash_manifest;
                hash_lockfile = new_hash_lockfile;
                restart_process(&args, &mut process);
            }
        }

        if process.try_wait().unwrap().is_some() {
            restart_process(&args, &mut process);
        }
    }
}

fn start_process(args: &[String]) -> Child {
    Command::new("elba")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("failed to start elba process")
}

fn restart_process(args: &[String], process: &mut Child) {
    process.kill().ok();
    *process = start_process(args);
}

fn find_project_root() -> Option<PathBuf> {
    let cwd = fs::canonicalize(env::current_dir().unwrap()).unwrap();
    let project_root = cwd.ancestors().find(|p| p.join("elba.toml").exists());
    project_root.map(|p| p.to_owned())
}

fn read_content_hash(path: &Path) -> u64 {
    if let Ok(mut file) = File::open(path) {
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    } else {
        0
    }
}
