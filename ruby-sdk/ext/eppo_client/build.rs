fn main() {
    // Without this flag, building via `cargo build` fails with undefined references to ruby
    // library. This is fine as `eppo_client` is going to be loaded as an extension by the host Ruby.
    println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
}
