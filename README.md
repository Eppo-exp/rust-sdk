## Eppo Multiplatform: SDKs and Artifacts to support Flagging and Experimentation

Eppo is a modular flagging and experimentation analysis tool. Eppo's SDKs are built to make assignments in multi-user server-side and client-side contexts. Before proceeding you'll need an Eppo account.

**Features**
* Feature gates
* Kill switches
* Progressive rollouts
* A/B/n experiments
* Mutually exclusive experiments (Layers)
* Dynamic configuration
* Global holdouts
* Contextual multi-armed bandits
## Contributing

### Preparing your environment

1. Install [rustup](https://rustup.rs/).
2. Install Ruby using your preferred package manager.

### Release process

To release a new version of SDK:
1. Make sure that version strings have been updated:
   - Eppo core: [eppo_core/Cargo.toml](eppo_core/Cargo.toml)
   - Rust: [rust-sdk/Cargo.toml](rust-sdk/Cargo.toml)
   - Python: [python-sdk/Cargo.toml](python-sdk/Cargo.toml)
   - Ruby: [ruby-sdk/lib/eppo_client/version.rb](ruby-sdk/lib/eppo_client/version.rb) and [ruby-sdk/ext/eppo_client/Cargo.toml](ruby-sdk/ext/eppo_client/Cargo.toml)
2. If SDK depends on a new version of `eppo_core`, the core should be released first.
3. [Create a new release](https://github.com/Eppo-exp/rust-sdk/releases/new) in GitHub interface.
   - For tag, use one of the following formats (choose "create new tag on publish"):
     - `eppo_core@x.y.z`
     - `rust-sdk@x.y.z`
     - `python-sdk@x.y.z`
     - `ruby-sdk@x.y.z`
   - For generating release notes, select previous tag from the same SDK (e.g., when releasing `python-sdk@4.0.3`, the previous tag should be `python-sdk@4.0.2`). Auto-generate release notes, prune entries that are not relevant for the SDK (e.g., Python SDK release should not list PRs for Ruby).
   - Publish release.
   - CI will automatically push a new release out to package registries.
