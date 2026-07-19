use git2::{Repository, RepositoryInitOptions};
use std::process::Command;
use tempfile::TempDir;

fn repo_init() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let mut opts = RepositoryInitOptions::new();
    opts.initial_head("main");
    let repo = Repository::init_opts(temp_dir.path(), &opts).unwrap();
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "name").unwrap();
    config.set_str("user.email", "email").unwrap();
    (temp_dir, repo)
}

fn add_commit(repository: &Repository, message: &str) {
    let id = repository.index().unwrap().write_tree().unwrap();
    let tree = repository.find_tree(id).unwrap();
    let sig = repository.signature().unwrap();

    let parents = repository
        .head()
        .ok()
        .and_then(|head| head.peel_to_commit().ok());
    let parents = match &parents {
        Some(commit) => vec![commit],
        None => vec![],
    };

    repository
        .commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .unwrap();
}

#[test]
fn warns_about_non_conventional_commits_on_stderr() {
    // Given
    let (temp_dir, repository) = repo_init();
    add_commit(&repository, "initial_commit");
    add_commit(&repository, ":sparkles: add a feature");

    // When
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-semantic-release"))
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Then
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("initial_commit"),
        "expected a warning about the non-conventional commit on stdout, got: {}",
        stdout
    );
}

#[test]
fn truncates_commit_messages_to_first_line() {
    // Given
    let (temp_dir, repository) = repo_init();
    let long_commit_message = "this commit message is ".repeat(10);
    let body = "body ".repeat(10);
    let long_message = format!("{long_commit_message}\n{body}");
    add_commit(&repository, &long_message);
    add_commit(&repository, ":sparkles: add a feature");

    // When
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-semantic-release"))
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Then
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected_prefix: String = format!("{long_commit_message}\n");
    assert!(
        stdout.contains(&expected_prefix),
        "expected a truncated warning, got: {}",
        stdout
    );
}

#[test]
fn truncates_commit_messages_at_the_first_newline() {
    // Given
    let (temp_dir, repository) = repo_init();
    add_commit(&repository, "short subject\nlonger body text");
    add_commit(&repository, ":sparkles: add a feature");

    // When
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-semantic-release"))
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Then
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("short subject"),
        "expected the warning to stop at the newline, got: {}",
        stdout
    );
}
