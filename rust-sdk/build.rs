use gh_workflow::*;
use indexmap::IndexMap;
use serde_json::json;
use toolchain::Toolchain;

fn main() {
    let mut submodules_map = IndexMap::new();
    submodules_map.insert("submodules".to_string(), json!("true"));

    let build = Job::new("Build and Test Rust SDK")
        .add_step(Step::checkout().add_with(submodules_map))
        .add_step(Step::run("npm ci"))
        .add_step(Toolchain::default().add_stable().add_clippy().add_fmt())
        .add_step(
            Cargo::new("build")
                .args("--all-features --all-targets --verbose")
                .name("Cargo Build"),
        )
        .add_step(
            Cargo::new("test")
                .args("--all-features --verbose")
                .name("Cargo Test"),
        )
        .add_step(Cargo::new("fmt").args("--check").name("Cargo Fmt"))
        .add_step(
            Cargo::new("clippy")
                .args("--all-features --workspace -- -D warnings")
                .name("Cargo Clippy"),
        )
        .add_step(Cargo::new("doc").args("--verbose").name("Cargo Doc"));

    let event = Event::default()
        .push(Push::default().add_branch("main"))
        .pull_request(PullRequest::default());

    Workflow::new("Build and Test")
        .on(event)
        .add_job("build", build)
        .generate()
        .unwrap();
}
