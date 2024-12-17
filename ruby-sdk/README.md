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

## Build locally

Install all dependencies:
```sh
bundle install
```

Build native extension:
```sh
bundle exec rake build
```

Run tests:
```sh
bundle exec rspec
```

## Releasing

* Bump versions in `ruby-sdk/lib/eppo_client/version.rb` and `ruby-sdk/ext/eppo_client/Cargo.toml`
* Run `cargo update --workspace --verbose` to update `Cargo.lock`
* Run `bundle` to update `Gemfile.lock`


## Building Ruby native lib

1. Clone this repository at the desired Ruby SDK tag, eg.: `git clone --depth 1 --branch ruby-sdk@x.y.z https://github.com/Eppo-exp/eppo-multiplatform.git`
2. Open `build.sh` and update `rb-sys-dock --platform <platform>` with the desired platform, eg.: `rb-sys-dock --platform x86_64-linux`
3. Run `docker build --build-arg WORKDIR=$(pwd) -f Dockerfile.ruby.build -t ruby-sdk-builder .` to build the builder docker image
4. Run the following command to build the gem with native lib:
```
mkdir -p rust/cargo/registry && docker run --rm -it \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v /tmp:/tmp -v \
  $(pwd)/rust:$(pwd)/rust ruby-sdk-builder
```
5. The gem will be available at `ruby-sdk/pkg/eppo-server-sdk-x.y.z-arch.gem`
