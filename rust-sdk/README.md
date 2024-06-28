# Eppo Rust SDK

![Crates.io Version](https://img.shields.io/crates/v/eppo) ![Crates.io MSRV](https://img.shields.io/crates/msrv/eppo) ![docs.rs](https://img.shields.io/docsrs/eppo)

[Eppo](https://www.geteppo.com/) is a modular flagging and experimentation analysis tool. Eppo's Rust SDK is designed to facilitate assignments in multi-user server-side contexts. You will need an Eppo account before proceeding.

Refer to [SDK documentation](https://docs.geteppo.com/feature-flags/sdks/rust) for how to install and use the SDK.

## Features

- Feature gates
- Kill switches
- Progressive rollouts
- A/B/n experiments
- Mutually exclusive experiments (Layers)
- Dynamic configuration

## Installation

Add it with cargo:
```sh
cargo add eppo
```

Or add it to `Cargo.toml` manually:
```toml
[dependencies]
eppo = "0.1.0"
```

## Quick Start

Initialize an instance of Eppo's client. Once initialized, the client can be used to make assignments in your app.

### Initialize Client

```rust
use eppo::ClientConfig;

let mut client = ClientConfig::from_api_key("api-key").to_client();
client.start_poller_thread();
```

### Assign Anywhere

```rust
let user = get_current_user();

let assignment = client.get_assignment(
    "show-new-feature",
    &user.id,
    &user.attributes,
);
```

## Assignment Logger

Pass a logging callback function to the `assignment_logger` method in `ClientConfig` when initializing the SDK to capture assignment data for analysis.

```rust
struct MyAssignmentLogger;

impl AssignmentLogger for MyAssignmentLogger {
    fn log_assignment(&self, event: AssignmentEvent) {
        // Implement assignment logging logic here
    }
}
```
