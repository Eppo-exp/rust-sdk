set -e
export RUST_INSTALL_PATH=./rust
mkdir -p $RUST_INSTALL_PATH
export RUSTUP_HOME=$RUST_INSTALL_PATH/rustup
export CARGO_HOME=$RUST_INSTALL_PATH/cargo
export PATH=$CARGO_HOME/bin:$PATH
export RB_SYS_DOCK_CACHE_DIR=./cache
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
rustup install stable
rustup default stable
rb-sys-dock --platform aarch64-linux-musl --mount-toolchains --ruby-versions 3.3 -V --build -V
