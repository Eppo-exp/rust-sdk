# Eppo SDK for Ruby

## Getting Started

Refer to our [SDK documentation](https://docs.geteppo.com/feature-flags/sdks/ruby) for how to install and use the SDK.

## Supported Ruby Versions
This version of the SDK is compatible with Ruby 3.0.6 and above.

## Logging

Ruby SDK uses [`env_logger`](https://docs.rs/env_logger/) for logging.

Starting from version 3.3.0, the log level can be configured via `EPPO_LOG` environment variable using one of the following values:
- `off`
- `error`
- `warn`
- `info` (default)
- `debug`
- `trace`

Alternatively, it can be configured using `log_level` parameter for `EppoClient::Config` constructor:
```ruby
config = EppoClient::Config.new("sdk-key", log_level: "debug")
```

# Contributing

## Testing with local version of `eppo_core`

To run build and tests against a local version of `eppo_core`, you should instruct Cargo to look for it at the local path.

Add the following to `.cargo/config.toml` file (relative to `ruby-sdk`):
```toml
[patch.crates-io]
eppo_core = { path = '../eppo_core' }
```

Make sure you remove the override before updating `Cargo.lock`. Otherwise, the lock file will be missing `eppo_core` checksum and will be unsuitable for release. (CI will warn you if you do this accidentally.)

## Releasing

* Bump versions in `ruby-sdk/lib/eppo_client/version.rb` and `ruby-sdk/ext/eppo_client/Cargo.toml`
* Run `cargo update --workspace --verbose` to update `Cargo.lock`
* Run `bundle` to update `Gemfile.lock`


## Building native libs

```
docker build --build-arg WORKDIR=$(pwd) -t ruby-sdk-builder .
mkdir -p rust/cargo/registry
docker run --rm -it \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v /tmp:/tmp -v \
  $(pwd)/rust:$(pwd)/rust ruby-sdk-builder
```
