//! Test that supplying empty packets does forward error correction.

extern crate opus;
use opus::*;

#[test]
fn blah() {
    let mut opus = Decoder::new(48000, Channels::Mono).unwrap();

    let mut output = vec![0i16; 5760];
    let size = opus.decode(&[], &mut output[..], true).unwrap();
    assert_eq!(size, 5760);
}