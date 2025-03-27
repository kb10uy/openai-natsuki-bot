use std::process::Command;

fn main() {
    export_git_commit_hash();
}

fn export_git_commit_hash() {
    let git_output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("error in fetching commit hash");
    let commit_hash = String::from_utf8(git_output.stdout).expect("invalid commit hash");
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", commit_hash.trim());
}
