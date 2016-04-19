// Copyright 2016 Tad Hardesty
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! High-level bindings for libopus.
//!
//! Only brief descriptions are included here. For detailed information, consult
//! the [libopus documentation](https://opus-codec.org/docs/opus_api-1.1.2/).
#![warn(missing_docs)]

extern crate opus_sys as ffi;
extern crate libc;

use std::ffi::CStr;
use std::marker::PhantomData;

use libc::c_int;

// ============================================================================
// Constants

// Generic CTLs
const OPUS_RESET_STATE: c_int = 4028; // void
const OPUS_GET_FINAL_RANGE: c_int = 4031; // out *u32
const OPUS_GET_BANDWIDTH: c_int = 4009; // out *i32
const OPUS_GET_SAMPLE_RATE: c_int = 4029; // out *i32
// Encoder CTLs
const OPUS_SET_BITRATE: c_int = 4002; // in i32
const OPUS_GET_BITRATE: c_int = 4003; // out *i32

/// The possible coding modes for the codec.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CodingMode {
	/// Best for most VoIP/videoconference applications where listening quality and intelligibility matter most.
	Voip = 2048,
	/// Best for broadcast/high-fidelity application where the decoded audio should be as close as possible to the input.
	Audio = 2049,
	/// Only use when lowest-achievable latency is what matters most.
	LowDelay = 2051,
}

/// The available channel setings.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Channels {
	/// One channel.
	Mono = 1,
	/// Two channels, left and right.
	Stereo = 2,
}

/// The available bandwidth level settings.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Bandwidth {
	/// Auto/default setting.
	Auto = -1000,
	/// 4kHz bandpass.
	Narrowband = 1101,
	/// 6kHz bandpass.
	Mediumband = 1102,
	/// 8kHz bandpass.
	Wideband = 1103,
	/// 12kHz bandpass.
	Superwideband = 1104,
	/// 20kHz bandpass.
	Fullband = 1105,
}

impl Bandwidth {
	fn from_int(value: i32) -> Option<Bandwidth> {
		Some(match value {
			-1000 => Bandwidth::Auto,
			1101 => Bandwidth::Narrowband,
			1102 => Bandwidth::Mediumband,
			1103 => Bandwidth::Wideband,
			1104 => Bandwidth::Superwideband,
			1105 => Bandwidth::Fullband,
			_ => return None,
		})
	}
}

/// Possible error codes.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ErrorCode {
	/// One or more invalid/out of range arguments.
	BadArg = -1,
	/// Not enough bytes allocated in the buffer.
	BufferTooSmall = -2,
	/// An internal error was detected.
	InternalError = -3,
	/// The compressed data passed is corrupted.
	InvalidPacket = -4,
	/// Invalid/unsupported request number.
	Unimplemented = -5,
	/// An encoder or decoder structure is invalid or already freed.
	InvalidState = -6,
	/// Memory allocation has failed.
	AllocFail = -7,
	/// An unknown failure.
	Unknown = -8,
}

/// Get the libopus version string.
pub fn version() -> &'static str {
	// verison string should always be ASCII
	unsafe { CStr::from_ptr(ffi::opus_get_version_string() as *const _) }.to_str().unwrap()
}

// ============================================================================
// Encoder

/// An Opus encoder with associated state.
pub struct Encoder {
	ptr: *mut ffi::OpusEncoder,
	channels: Channels,
}

impl Encoder {
	/// Create and initialize an encoder.
	pub fn new(sample_rate: u32, channels: Channels, mode: CodingMode) -> Result<Encoder> {
		let mut error = 0;
		let ptr = unsafe { ffi::opus_encoder_create(
			sample_rate as i32,
			channels as c_int,
			mode as c_int,
			&mut error) };
		if error != ffi::OPUS_OK || ptr.is_null() {
			Err(Error::from_code("opus_encoder_create", error))
		} else {
			Ok(Encoder { ptr: ptr, channels: channels })
		}
	}

	/// Encode an Opus frame.
	pub fn encode(&mut self, input: &[i16], output: &mut [u8]) -> Result<usize> {
		let len = unsafe { ffi::opus_encode(self.ptr,
			input.as_ptr(), input.len() as c_int / self.channels as c_int,
			output.as_mut_ptr(), output.len() as c_int) };
		try!(check("opus_encode", len));
		Ok(len as usize)
	}

	/// Encode an Opus frame from floating point input.
	pub fn encode_float(&mut self, input: &[f32], output: &mut [u8]) -> Result<usize> {
		let len = unsafe { ffi::opus_encode_float(self.ptr,
			input.as_ptr(), input.len() as c_int / self.channels as c_int,
			output.as_mut_ptr(), output.len() as c_int) };
		try!(check("opus_encode_float", len));
		Ok(len as usize)
	}

	/// Reset the codec state to be equivalent to a freshly initialized state.
	pub fn reset_state(&mut self) -> Result<()> {
		check("opus_encoder_ctl(OPUS_RESET_STATE)", unsafe { ffi::opus_encoder_ctl(self.ptr, OPUS_RESET_STATE) })
	}

	/// Get the final range of the codec's entropy coder.
	pub fn get_final_range(&mut self) -> Result<u32> {
		let mut value: u32 = 0;
		let result = unsafe { ffi::opus_encoder_ctl(self.ptr, OPUS_GET_FINAL_RANGE, &mut value) };
		try!(check("opus_encoder_ctl(OPUS_GET_FINAL_RANGE)", result));
		Ok(value)
	}

	/// Get the encoder's configured bandpass.
	pub fn get_bandwidth(&mut self) -> Result<Bandwidth> {
		let mut value: i32 = 0;
		let result = unsafe { ffi::opus_encoder_ctl(self.ptr, OPUS_GET_BANDWIDTH, &mut value) };
		try!(check("opus_encoder_ctl(OPUS_GET_BANDWIDTH)", result));
		Bandwidth::from_int(result).ok_or_else(|| Error::from_code("opus_encoder_ctl(OPUS_GET_BANDWIDTH)", ffi::OPUS_BAD_ARG))
	}

	/// Get the samping rate the encoder was intialized with.
	pub fn get_sample_rate(&mut self) -> Result<u32> {
		let mut value: i32 = 0;
		let result = unsafe { ffi::opus_encoder_ctl(self.ptr, OPUS_GET_SAMPLE_RATE, &mut value) };
		try!(check("opus_encoder_ctl(OPUS_GET_SAMPLE_RATE)", result));
		Ok(value as u32)
	}

	/// Set the encoder's bitrate.
	pub fn set_bitrate(&mut self, value: i32) -> Result<()> {
		let result = unsafe { ffi::opus_encoder_ctl(self.ptr, OPUS_SET_BITRATE, value) };
		check("opus_encoder_ctl(OPUS_SET_BITRATE)", result)
	}

	/// Get the encoder's bitrate.
	pub fn get_bitrate(&mut self) -> Result<i32> {
		let mut value: i32 = 0;
		let result = unsafe { ffi::opus_encoder_ctl(self.ptr, OPUS_GET_BITRATE, &mut value) };
		try!(check("opus_encoder_ctl(OPUS_GET_BITRATE)", result));
		Ok(value)
	}

	// TODO: Encoder-specific CTLs
}

impl Drop for Encoder {
	fn drop(&mut self) {
		unsafe { ffi::opus_encoder_destroy(self.ptr) }
	}
}

// ============================================================================
// Decoder

/// An Opus decoder with associated state.
pub struct Decoder {
	ptr: *mut ffi::OpusDecoder,
	channels: Channels,
}

impl Decoder {
	/// Create and initialize a decoder.
	pub fn new(sample_rate: u32, channels: Channels) -> Result<Decoder> {
		let mut error = 0;
		let ptr = unsafe { ffi::opus_decoder_create(
			sample_rate as i32,
			channels as c_int,
			&mut error) };
		if error != ffi::OPUS_OK || ptr.is_null() {
			Err(Error::from_code("opus_decoder_create", error))
		} else {
			Ok(Decoder { ptr: ptr, channels: channels })
		}
	}

	/// Decode an Opus packet.
	pub fn decode(&mut self, input: &[u8], output: &mut [i16], fec: bool) -> Result<usize> {
		let len = unsafe { ffi::opus_decode(self.ptr,
			input.as_ptr(), input.len() as c_int,
			output.as_mut_ptr(), output.len() as c_int / self.channels as c_int,
			fec as c_int) };
		try!(check("opus_decode", len));
		Ok(len as usize)
	}

	/// Decode an Opus packet with floating point output.
	pub fn decode_float(&mut self, input: &[u8], output: &mut [f32], fec: bool) -> Result<usize> {
		let len = unsafe { ffi::opus_decode_float(self.ptr,
			input.as_ptr(), input.len() as c_int,
			output.as_mut_ptr(), output.len() as c_int / self.channels as c_int,
			fec as c_int) };
		try!(check("opus_decode_float", len));
		Ok(len as usize)
	}

	/// Get the number of samples of an Opus packet.
	pub fn get_nb_samples(&self, packet: &[u8]) -> Result<usize> {
		let len = unsafe { ffi::opus_decoder_get_nb_samples(self.ptr,
			packet.as_ptr(), packet.len() as i32) };
		try!(check("opus_decoder_get_nb_samples", len));
		Ok(len as usize)
	}

	/// Reset the codec state to be equivalent to a freshly initialized state.
	pub fn reset_state(&mut self) -> Result<()> {
		check("opus_decoder_ctl(OPUS_RESET_STATE)", unsafe { ffi::opus_decoder_ctl(self.ptr, OPUS_RESET_STATE) })
	}

	/// Get the final range of the codec's entropy coder.
	pub fn get_final_range(&mut self) -> Result<u32> {
		let mut value: u32 = 0;
		let result = unsafe { ffi::opus_decoder_ctl(self.ptr, OPUS_GET_FINAL_RANGE, &mut value) };
		try!(check("opus_decoder_ctl(OPUS_GET_FINAL_RANGE)", result));
		Ok(value)
	}

	/// Get the decoder's last bandpass.
	pub fn get_bandwidth(&mut self) -> Result<Bandwidth> {
		let mut value: i32 = 0;
		let result = unsafe { ffi::opus_decoder_ctl(self.ptr, OPUS_GET_BANDWIDTH, &mut value) };
		try!(check("opus_decoder_ctl(OPUS_GET_BANDWIDTH)", result));
		Bandwidth::from_int(result).ok_or_else(|| Error::from_code("opus_decoder_ctl(OPUS_GET_BANDWIDTH)", ffi::OPUS_BAD_ARG))
	}

	/// Get the samping rate the decoder was intialized with.
	pub fn get_sample_rate(&mut self) -> Result<u32> {
		let mut value: i32 = 0;
		let result = unsafe { ffi::opus_decoder_ctl(self.ptr, OPUS_GET_SAMPLE_RATE, &mut value) };
		try!(check("opus_decoder_ctl(OPUS_GET_SAMPLE_RATE)", result));
		Ok(value as u32)
	}

	// TODO: Decoder-specific CTLs
}

impl Drop for Decoder {
	fn drop(&mut self) {
		unsafe { ffi::opus_decoder_destroy(self.ptr) }
	}
}

// ============================================================================
// Packet Analysis

/// Analyze raw Opus packets.
pub mod packet {
	use super::*;
	use super::{ffi, check};
	use std::{ptr, slice};
	use libc::c_int;

	/// Get the bandwidth of an Opus packet.
	pub fn get_bandwidth(packet: &[u8]) -> Result<Bandwidth> {
		if packet.len() < 1 {
			return Err(Error::from_code("opus_packet_get_bandwidth", ffi::OPUS_BAD_ARG))
		}
		let bandwidth = unsafe { ffi::opus_packet_get_bandwidth(packet.as_ptr()) };
		try!(check("opus_packet_get_bandwidth", bandwidth));
		Bandwidth::from_int(bandwidth).ok_or_else(|| Error::from_code("opus_packet_get_bandwidth", ffi::OPUS_BAD_ARG))
	}

	/// Get the number of channels from an Opus packet.
	pub fn get_nb_channels(packet: &[u8]) -> Result<Channels> {
		if packet.len() < 1 {
			return Err(Error::from_code("opus_packet_get_nb_channels", ffi::OPUS_BAD_ARG))
		}
		let channels = unsafe { ffi::opus_packet_get_nb_channels(packet.as_ptr()) };
		try!(check("opus_packet_get_nb_channels", channels));
		match channels {
			1 => Ok(Channels::Mono),
			2 => Ok(Channels::Stereo),
			_ => Err(Error::from_code("opus_packet_get_nb_channels", ffi::OPUS_BAD_ARG)),
		}
	}

	/// Get the number of frames in an Opus packet.
	pub fn get_nb_frames(packet: &[u8]) -> Result<usize> {
		let frames = unsafe { ffi::opus_packet_get_nb_frames(packet.as_ptr(), packet.len() as c_int) };
		try!(check("opus_packet_get_nb_frames", frames));
		Ok(frames as usize)
	}

	/// Get the number of samples of an Opus packet.
	pub fn get_nb_samples(packet: &[u8], sample_rate: u32) -> Result<usize> {
		let frames = unsafe { ffi::opus_packet_get_nb_samples(
			packet.as_ptr(), packet.len() as c_int,
			sample_rate as c_int) };
		try!(check("opus_packet_get_nb_samples", frames));
		Ok(frames as usize)
	}

	/// Get the number of samples per frame from an Opus packet.
	pub fn get_samples_per_frame(packet: &[u8], sample_rate: u32) -> Result<usize> {
		if packet.len() < 1 {
			return Err(Error::from_code("opus_packet_get_samples_per_frame", ffi::OPUS_BAD_ARG))
		}
		let samples = unsafe { ffi::opus_packet_get_samples_per_frame(packet.as_ptr(), sample_rate as c_int) };
		try!(check("opus_packet_get_samples_per_frame", samples));
		Ok(samples as usize)
	}

	/// Parse an Opus packet into one or more frames.
	pub fn parse(packet: &[u8]) -> Result<Packet> {
		let mut toc: u8 = 0;
		let mut frames = [ptr::null(); 48];
		let mut sizes = [0i16; 48];
		let mut payload_offset: i32 = 0;
		let num_frames = unsafe { ffi::opus_packet_parse(
			packet.as_ptr(), packet.len() as c_int,
			&mut toc, frames.as_mut_ptr(),
			sizes.as_mut_ptr(), &mut payload_offset) };
		try!(check("opus_packet_parse", num_frames));

		let mut frames_vec = Vec::with_capacity(num_frames as usize);
		for i in 0..num_frames as usize {
			frames_vec.push(unsafe { slice::from_raw_parts(frames[i], sizes[i] as usize) });
		}

		Ok(Packet {
			toc: toc,
			frames: frames_vec,
			payload_offset: payload_offset as usize,
		})
	}

	/// A parsed Opus packet, retuned from `parse`.
	pub struct Packet<'a> {
		/// The TOC byte of the packet.
		pub toc: u8,
		/// The frames contained in the packet.
		pub frames: Vec<&'a [u8]>,
		/// The offset into the packet at which the payload is located.
		pub payload_offset: usize,
	}

	/// Pad a given Opus packet to a larger size.
	///
	/// The packet will be extended from the first `prev_len` bytes of the
	/// buffer into the rest of the available space.
	pub fn pad(packet: &mut [u8], prev_len: usize) -> Result<usize> {
		let result = unsafe { ffi::opus_packet_pad(packet.as_mut_ptr(), prev_len as c_int, packet.len() as c_int) };
		try!(check("opus_packet_pad", result));
		Ok(result as usize)
	}

	/// Remove all padding from a given Opus packet and rewrite the TOC sequence
	/// to minimize space usage.
	pub fn unpad(packet: &mut [u8]) -> Result<usize> {
		let result = unsafe { ffi::opus_packet_unpad(packet.as_mut_ptr(), packet.len() as c_int) };
		try!(check("opus_packet_unpad", result));
		Ok(result as usize)
	}
}

// ============================================================================
// Float Soft Clipping

/// Soft-clipping to bring a float signal within the [-1,1] range.
pub struct SoftClip {
	channels: Channels,
	memory: [f32; 2],
}

impl SoftClip {
	/// Initialize a new soft-clipping state.
	pub fn new(channels: Channels) -> SoftClip {
		SoftClip { channels: channels, memory: [0.0; 2] }
	}

	/// Apply soft-clipping to a float signal.
	pub fn apply(&mut self, signal: &mut [f32]) {
		unsafe { ffi::opus_pcm_soft_clip(
			signal.as_mut_ptr(),
			signal.len() as c_int / self.channels as c_int,
			self.channels as c_int,
			self.memory.as_mut_ptr()) };
	}
}

// ============================================================================
// Repacketizer

/// A repacketizer used to merge together or split apart multiple Opus packets.
pub struct Repacketizer {
	ptr: *mut ffi::OpusRepacketizer,
}

impl Repacketizer {
	/// Create and initialize a repacketizer.
	pub fn new() -> Result<Repacketizer> {
		let ptr = unsafe { ffi::opus_repacketizer_create() };
		if ptr.is_null() {
			Err(Error::from_code("opus_repacketizer_create", ffi::OPUS_ALLOC_FAIL))
		} else {
			Ok(Repacketizer { ptr: ptr })
		}
	}

	/// Shortcut to combine several smaller packets into one larger one.
	pub fn combine(&mut self, input: &[&[u8]], output: &mut [u8]) -> Result<usize> {
		let mut state = self.begin();
		for &packet in input {
			try!(state.cat(packet));
		}
		state.out(output)
	}

	/// Begin using the repacketizer.
	pub fn begin<'rp, 'buf>(&'rp mut self) -> RepacketizerState<'rp, 'buf> {
		unsafe { ffi::opus_repacketizer_init(self.ptr); }
		RepacketizerState { rp: self, phantom: PhantomData }
	}
}

impl Drop for Repacketizer {
	fn drop(&mut self) {
		unsafe { ffi::opus_repacketizer_destroy(self.ptr) }
	}
}

// To understand why these lifetime bounds are needed, imagine that the
// repacketizer keeps an internal Vec<&'buf [u8]>, which is added to by cat()
// and accessed by get_nb_frames(), out(), and out_range(). To prove that these
// lifetime bounds are correct, a dummy implementation with the same signatures
// but a real Vec<&'buf [u8]> rather than unsafe blocks may be substituted.

/// An in-progress repacketization.
pub struct RepacketizerState<'rp, 'buf> {
	rp: &'rp mut Repacketizer,
	phantom: PhantomData<&'buf [u8]>,
}

impl<'rp, 'buf> RepacketizerState<'rp, 'buf> {
	/// Add a packet to the current repacketizer state.
	pub fn cat(&mut self, packet: &'buf [u8]) -> Result<()> {
		let result = unsafe { ffi::opus_repacketizer_cat(self.rp.ptr,
			packet.as_ptr(), packet.len() as c_int) };
		check("opus_repacketizer_cat", result)
	}

	/// Add a packet to the current repacketizer state, moving it.
	#[inline]
	pub fn cat_move<'b2>(self, packet: &'b2 [u8]) -> Result<RepacketizerState<'rp, 'b2>> where 'buf: 'b2 {
		let mut shorter = self;
		try!(shorter.cat(packet));
		Ok(shorter)
	}

	/// Get the total number of frames contained in packet data submitted so
	/// far via `cat`.
	pub fn get_nb_frames(&mut self) -> usize {
		unsafe { ffi::opus_repacketizer_get_nb_frames(self.rp.ptr) as usize }
	}

	/// Construct a new packet from data previously submitted via `cat`.
	///
	/// All previously submitted frames are used.
	pub fn out(&mut self, buffer: &mut [u8]) -> Result<usize> {
		let result = unsafe { ffi::opus_repacketizer_out(self.rp.ptr,
			buffer.as_mut_ptr(), buffer.len() as c_int) };
		try!(check("opus_repacketizer_out", result));
		Ok(result as usize)
	}

	/// Construct a new packet from data previously submitted via `cat`, with
	/// a manually specified subrange.
	///
	/// The `end` index should not exceed the value of `get_nb_frames()`.
	pub fn out_range(&mut self, begin: usize, end: usize, buffer: &mut [u8]) -> Result<usize> {
		let result = unsafe { ffi::opus_repacketizer_out_range(self.rp.ptr,
			begin as c_int, end as c_int,
			buffer.as_mut_ptr(), buffer.len() as c_int) };
		try!(check("opus_repacketizer_out_range", result));
		Ok(result as usize)
	}
}

// ============================================================================
// TODO: Multistream API

// ============================================================================
// Error Handling

/// Opus error Result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// An error generated by the Opus library.
#[derive(Debug)]
pub struct Error {
	function: &'static str,
	description: &'static str,
	code: ErrorCode,
}

impl Error {
	fn from_code(what: &'static str, code: c_int) -> Error {
		// description should always be ASCII
		let description = unsafe { CStr::from_ptr(ffi::opus_strerror(code) as *const _) }.to_str().unwrap();
		let code = match code {
			-1 => ErrorCode::BadArg,
			-2 => ErrorCode::BufferTooSmall,
			-3 => ErrorCode::InternalError,
			-4 => ErrorCode::InvalidPacket,
			-5 => ErrorCode::Unimplemented,
			-6 => ErrorCode::InvalidState,
			-7 => ErrorCode::AllocFail,
			_ => ErrorCode::Unknown,
		};
		Error { function: what, description: description, code: code }
	}

	/// Get the name of the Opus function from which the error originated.
	pub fn function(&self) -> &'static str { self.function }
	/// Get a textual description of the error provided by Opus.
	pub fn description(&self) -> &'static str { self.description }
	/// Get the Opus error code of the error.
	pub fn code(&self) -> ErrorCode { self.code }
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}: {}", self.function, self.description)
	}
}

impl std::error::Error for Error {
	fn description(&self) -> &str {
		self.description
	}
}

fn check(what: &'static str, code: c_int) -> Result<()> {
	if code < 0 {
		Err(Error::from_code(what, code))
	} else {
		Ok(())
	}
}
