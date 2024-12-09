# Eppo Assignments on Fastly Compute@Edge

TODO: Add a description

## Development

Install Rust toolchain: 

`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

Install Fastly CLI: 

`brew install fastly/tap/fastly`

https://www.fastly.com/documentation/reference/tools/cli/

Install Viceroy: 

`cargo install viceroy`

Build with Fastly:

`make build`

## Testing

Install nextest:

`cargo binstall cargo-nextest --secure`

Run tests:

`make test`
