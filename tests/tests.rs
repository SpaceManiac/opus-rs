extern crate opus;

fn check_ascii(s: &str) -> &str {
	for &b in s.as_bytes() {
		assert!(b < 0x80, "Non-ASCII character in string");
		assert!(b > 0x00, "NUL in string")
	}
	std::str::from_utf8(s.as_bytes()).unwrap()
}

#[test]
fn strings_ascii() {
	use opus::ErrorCode::*;

	println!("\nVersion: {}", check_ascii(opus::version()));

	let codes = [BadArg, BufferTooSmall, InternalError, InvalidPacket,
		Unimplemented, InvalidState, AllocFail, Unknown];
	for &code in codes.iter() {
		println!("{:?}: {}", code, check_ascii(code.description()));
	}
}

// 48000Hz * 1 channel * 20 ms / 1000 = 960
const MONO_20MS: usize = 48000 * 20 / 1000;

#[test]
fn encode_mono() {
	let mut encoder = opus::Encoder::new(48000, opus::Channels::Mono, opus::Application::Audio).unwrap();

	let mut output = [0; 256];
	let len = encoder.encode(&[0_i16; MONO_20MS], &mut output).unwrap();
	assert_eq!(&output[..len], &[248, 255, 254]);

	let len = encoder.encode(&[0_i16; MONO_20MS], &mut output).unwrap();
	assert_eq!(&output[..len], &[248, 255, 254]);

	let len = encoder.encode(&[1_i16; MONO_20MS], &mut output).unwrap();
	assert!(len > 190 && len < 220);

	let len = encoder.encode(&[0_i16; MONO_20MS], &mut output).unwrap();
	assert!(len > 170 && len < 190);

	let myvec = encoder.encode_vec(&[1_i16; MONO_20MS], output.len()).unwrap();
	assert!(myvec.len() > 120 && myvec.len() < 140);
}

#[test]
fn encode_stereo() {
	let mut encoder = opus::Encoder::new(48000, opus::Channels::Stereo, opus::Application::Audio).unwrap();

	let mut output = [0; 512];
	let len = encoder.encode(&[0_i16; 2 * MONO_20MS], &mut output).unwrap();
	assert_eq!(&output[..len], &[252, 255, 254]);

	let len = encoder.encode(&[0_i16; 4 * MONO_20MS], &mut output).unwrap();
	assert_eq!(&output[..len], &[253, 255, 254, 255, 254]);

	let len = encoder.encode(&[17_i16; 2 * MONO_20MS], &mut output).unwrap();
	assert!(len > 240);

	let len = encoder.encode(&[0_i16; 2 * MONO_20MS], &mut output).unwrap();
	assert!(len > 240);

	// Very small buffer should still succeed
	let len = encoder.encode(&[95_i16; 2 * MONO_20MS], &mut [0; 20]).unwrap();
	assert!(len <= 20);

	let myvec = encoder.encode_vec(&[95_i16; 2 * MONO_20MS], 20).unwrap();
	assert!(myvec.len() <= 20);
}

#[test]
fn encode_decode_stereo() {
	let mut opus_encoder =
		opus::Encoder::new(48000, opus::Channels::Stereo, opus::Application::Voip).unwrap();
	let mut opus_decoder = opus::Decoder::new(48000, opus::Channels::Stereo).unwrap();
	let mut pcm_raw_data = vec![17_i16; MONO_20MS * 2];
	pcm_raw_data[1] = 1;

	let mut encoded_opus = vec![0; 1500];
	let size = opus_encoder.encode(&pcm_raw_data, &mut encoded_opus).unwrap();
	let packet = &encoded_opus[..size];
	dbg!(size);

	// get_nb_samples() returns the number of samples per channel. To get the
	// total sample count, multiply by the number of channels.
	assert_eq!(MONO_20MS, opus_decoder.get_nb_samples(packet).unwrap());
	assert_eq!(MONO_20MS, opus::packet::get_nb_samples(packet, 48000).unwrap());
	assert_eq!(1, opus::packet::get_nb_frames(packet).unwrap());
	assert_eq!(opus::Channels::Stereo, opus::packet::get_nb_channels(packet).unwrap());
	assert_eq!(MONO_20MS, opus::packet::get_samples_per_frame(packet, 48000).unwrap());

	let mut output = vec![0i16; MONO_20MS * 2];
	// decode() returns the number of samples per channel.
	assert_eq!(MONO_20MS, opus_decoder.decode(packet, &mut output, false).unwrap());
}

#[test]
fn encode_bad_rate() {
	match opus::Encoder::new(48001, opus::Channels::Mono, opus::Application::Audio) {
		Ok(_) => panic!("Encoder::new did not return BadArg"),
		Err(err) => assert_eq!(err.code(), opus::ErrorCode::BadArg),
	}
}

#[test]
fn encode_bad_buffer() {
	let mut encoder = opus::Encoder::new(48000, opus::Channels::Stereo, opus::Application::Audio).unwrap();
	match encoder.encode(&[1_i16; 2 * MONO_20MS], &mut [0; 0]) {
		Ok(_) => panic!("encode with 0-length buffer did not return BadArg"),
		Err(err) => assert_eq!(err.code(), opus::ErrorCode::BadArg),
	}
}

#[test]
fn repacketizer() {
	let mut rp = opus::Repacketizer::new().unwrap();
	let mut out = [0; 256];

	for _ in 0..2 {
		let packet1 = [249, 255, 254, 255, 254];
		let packet2 = [248, 255, 254];

		let mut state = rp.begin();
		state.cat(&packet1).unwrap();
		state.cat(&packet2).unwrap();
		let len = state.out(&mut out).unwrap();
		assert_eq!(&out[..len], &[251, 3, 255, 254, 255, 254, 255, 254]);
	}
	for _ in 0..2 {
		let packet = [248, 255, 254];
		let state = rp.begin().cat_move(&packet).unwrap();
		let packet = [249, 255, 254, 255, 254];
		let state = state.cat_move(&packet).unwrap();
		let len = {state}.out(&mut out).unwrap();
		assert_eq!(&out[..len], &[251, 3, 255, 254, 255, 254, 255, 254]);
	}
	for _ in 0..2 {
		let len = rp.combine(&[
			&[249, 255, 254, 255, 254],
			&[248, 255, 254],
		], &mut out).unwrap();
		assert_eq!(&out[..len], &[251, 3, 255, 254, 255, 254, 255, 254]);
	}
	for _ in 0..2 {
		let len = rp.begin()
			.cat_move(&[248, 255, 254]).unwrap()
			.cat_move(&[248, 71, 71]).unwrap()
			.out(&mut out).unwrap();
		assert_eq!(&out[..len], &[249, 255, 254, 71, 71]);
	}
}
