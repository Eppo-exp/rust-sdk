set -e
export RUST_INSTALL_PATH=$PWD/rust
mkdir -p $RUST_INSTALL_PATH
export RUSTUP_HOME=$RUST_INSTALL_PATH/rustup
export CARGO_HOME=$RUST_INSTALL_PATH/cargo
export PATH=$CARGO_HOME/bin:$PATH
export RB_SYS_DOCK_CACHE_DIR=$PWD/cache
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
rustup install stable
rustup default stable
mkdir -p /.cargo
echo "[patch.crates-io.eppo_core]" > "/.cargo/config.toml"
echo "path = '$PWD/eppo_core'" >> "/.cargo/config.toml"
cargo --config /.cargo/config.toml fetch
cd ruby-sdk
rb-sys-dock --platform aarch64-linux-musl --mount-toolchains --ruby-versions 3.3 --build -V
