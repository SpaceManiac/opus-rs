# opus-rs

Safe Rust bindings for [libopus](https://opus-codec.org/). The rustdoc
includes brief descriptions for methods, and detailed API information can be
found at the [libopus documentation][upstream docs].

[crates.io] - [docs.rs] - [upstream docs]

[crates.io]: https://crates.io/crates/opus
[docs.rs]: https://docs.rs/opus/
[upstream docs]: https://opus-codec.org/docs/opus_api-1.5/

## External dependencies

By default, you need either:

* pkg-config and opus headers/libraries
* cmake, make, and a C compiler

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
