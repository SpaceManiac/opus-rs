# opus-rs [![](https://meritbadge.herokuapp.com/opus)](https://crates.io/crates/opus) [![](https://img.shields.io/badge/docs-online-2020ff.svg)](https://docs.rs/opus/0.2.1/opus/)

Safe Rust bindings for libopus. The rustdoc (available through `cargo doc`)
includes brief descriptions for methods, and detailed API information can be
found at the [libopus documentation](https://opus-codec.org/docs/opus_api-1.1.2/).

## External dependencies

By default, you need either:

* cmake, make, and a C compiler
* pkg-config and opus headers/libraries

These requirements come from [audiopus_sys](https://crates.io/crates/audiopus_sys), where details about overriding these defaults can be found.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
