extern crate opus_sys as ffi;
extern crate libc;

use std::ffi::CStr;

use libc::{c_int};

// TODO: Documentation

// ============================================================================
// Constants

// Generic CTLs
const OPUS_RESET_STATE: c_int = 4028; // void
const OPUS_GET_FINAL_RANGE: c_int = 4031; // *uint
const OPUS_GET_BANDWIDTH: c_int = 4009; // *int
const OPUS_GET_SAMPLE_RATE: c_int = 4029; // *int

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CodingMode {
	Voip = 2048,
	Audio = 2049,
	LowDelay = 2051,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Channels {
	Stereo = 1,
	Mono = 2,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Bandwidth {
	Narrowband = 1101,
	Mediumband = 1102,
	Wideband = 1103,
	Superwideband = 1104,
	Fullband = 1105,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ErrorCode {
	BadArg = -1,
	BufferTooSmall = -2,
	InternalError = -3,
	InvalidPacket = -4,
	Unimplemented = -5,
	InvalidState = -6,
	AllocFail = -7,
	Unknown = -8,
}

pub fn version() -> &'static str {
	// verison string should always be ASCII
	unsafe { CStr::from_ptr(ffi::opus_get_version_string()) }.to_str().unwrap()
}

// ============================================================================
// Encoder

pub struct Encoder {
	ptr: *mut ffi::OpusEncoder,
	channels: Channels,
}

impl Encoder {
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

	pub fn encode(&mut self, input: &[i16], output: &mut [u8]) -> Result<usize> {
		let len = unsafe { ffi::opus_encode(self.ptr,
			input.as_ptr(), input.len() as c_int / self.channels as c_int,
			output.as_mut_ptr(), output.len() as c_int) };
		try!(check("opus_encode", len));
		Ok(len as usize)
	}

	pub fn encode_float(&mut self, input: &[f32], output: &mut [u8]) -> Result<usize> {
		let len = unsafe { ffi::opus_encode_float(self.ptr,
			input.as_ptr(), input.len() as c_int / self.channels as c_int,
			output.as_mut_ptr(), output.len() as c_int) };
		try!(check("opus_encode_float", len));
		Ok(len as usize)
	}

	pub fn reset_state(&mut self) -> Result<()> {
		check("opus_encoder_ctl(OPUS_RESET_STATE)", unsafe { ffi::opus_encoder_ctl(self.ptr, OPUS_RESET_STATE) })
	}

	pub fn get_final_range(&mut self) -> Result<u32> {
		let mut value = 0;
		let result = unsafe { ffi::opus_encoder_ctl(self.ptr, OPUS_GET_FINAL_RANGE, &mut value) };
		try!(check("opus_encoder_ctl(OPUS_GET_FINAL_RANGE)", result));
		Ok(value)
	}

	pub fn get_bandwidth(&mut self) -> Result<i32> {
		let mut value = 0;
		let result = unsafe { ffi::opus_encoder_ctl(self.ptr, OPUS_GET_BANDWIDTH, &mut value) };
		try!(check("opus_encoder_ctl(OPUS_GET_BANDWIDTH)", result));
		Ok(value)
	}

	pub fn get_sample_rate(&mut self) -> Result<i32> {
		let mut value = 0;
		let result = unsafe { ffi::opus_encoder_ctl(self.ptr, OPUS_GET_SAMPLE_RATE, &mut value) };
		try!(check("opus_encoder_ctl(OPUS_GET_SAMPLE_RATE)", result));
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

pub struct Decoder {
	ptr: *mut ffi::OpusDecoder,
	channels: Channels,
}

impl Decoder {
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

	// TODO: support null inputs ("packet loss")
	pub fn decode(&mut self, input: &[u8], output: &mut [i16], fec: bool) -> Result<usize> {
		let len = unsafe { ffi::opus_decode(self.ptr,
			input.as_ptr(), input.len() as c_int,
			output.as_mut_ptr(), output.len() as c_int / self.channels as c_int,
			fec as c_int) };
		try!(check("opus_decode", len));
		Ok(len as usize)
	}

	pub fn decode_float(&mut self, input: &[u8], output: &mut [f32], fec: bool) -> Result<usize> {
		let len = unsafe { ffi::opus_decode_float(self.ptr,
			input.as_ptr(), input.len() as c_int,
			output.as_mut_ptr(), output.len() as c_int / self.channels as c_int,
			fec as c_int) };
		try!(check("opus_decode_float", len));
		Ok(len as usize)
	}

	pub fn get_nb_samples(&self, packet: &[u8]) -> Result<usize> {
		let len = unsafe { ffi::opus_decoder_get_nb_samples(self.ptr,
			packet.as_ptr(), packet.len() as i32) };
		try!(check("opus_decoder_get_nb_samples", len));
		Ok(len as usize)
	}

	// TODO: Generic CTLs
	// TODO: Decoder-specific CTLs
}

impl Drop for Decoder {
	fn drop(&mut self) {
		unsafe { ffi::opus_decoder_destroy(self.ptr) }
	}
}

// ============================================================================
// Packet Analysis

pub mod packet {
	use super::*;
	use super::{ffi, check};
	use std::{ptr, slice};
	use libc::c_int;

	pub fn get_bandwidth(packet: &[u8]) -> Result<Bandwidth> {
		if packet.len() < 1 {
			return Err(Error::from_code("opus_packet_get_bandwidth", ffi::OPUS_BAD_ARG))
		}
		let bandwidth = unsafe { ffi::opus_packet_get_bandwidth(packet.as_ptr()) };
		try!(check("opus_packet_get_bandwidth", bandwidth));
		match bandwidth {
			1101 => Ok(Bandwidth::Narrowband),
			1102 => Ok(Bandwidth::Mediumband),
			1103 => Ok(Bandwidth::Wideband),
			1104 => Ok(Bandwidth::Superwideband),
			1105 => Ok(Bandwidth::Fullband),
			_ => Err(Error::from_code("opus_packet_get_bandwidth", ffi::OPUS_BAD_ARG)),
		}
	}

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

	pub fn get_nb_frames(packet: &[u8]) -> Result<usize> {
		let frames = unsafe { ffi::opus_packet_get_nb_frames(packet.as_ptr(), packet.len() as c_int) };
		try!(check("opus_packet_get_nb_frames", frames));
		Ok(frames as usize)
	}

	pub fn get_nb_samples(packet: &[u8], sample_rate: u32) -> Result<usize> {
		let frames = unsafe { ffi::opus_packet_get_nb_samples(
			packet.as_ptr(), packet.len() as c_int,
			sample_rate as c_int) };
		try!(check("opus_packet_get_nb_samples", frames));
		Ok(frames as usize)
	}

	pub fn get_samples_per_frame(packet: &[u8], sample_rate: u32) -> Result<usize> {
		if packet.len() < 1 {
			return Err(Error::from_code("opus_packet_get_samples_per_frame", ffi::OPUS_BAD_ARG))
		}
		let samples = unsafe { ffi::opus_packet_get_samples_per_frame(packet.as_ptr(), sample_rate as c_int) };
		try!(check("opus_packet_get_samples_per_frame", samples));
		Ok(samples as usize)
	}

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

	pub struct Packet<'a> {
		pub toc: u8,
		pub frames: Vec<&'a [u8]>,
		pub payload_offset: usize,
	}

	pub fn pad(packet: &mut [u8], prev_len: usize) -> Result<usize> {
		let result = unsafe { ffi::opus_packet_pad(packet.as_mut_ptr(), prev_len as c_int, packet.len() as c_int) };
		try!(check("opus_packet_pad", result));
		Ok(result as usize)
	}

	pub fn unpad(packet: &mut [u8]) -> Result<usize> {
		let result = unsafe { ffi::opus_packet_unpad(packet.as_mut_ptr(), packet.len() as c_int) };
		try!(check("opus_packet_unpad", result));
		Ok(result as usize)
	}
}

// ============================================================================
// Float Soft Clipping

pub struct SoftClip {
	channels: Channels,
	memory: [f32; 2],
}

impl SoftClip {
	pub fn new(channels: Channels) -> SoftClip {
		SoftClip { channels: channels, memory: [0.0; 2] }
	}

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

pub struct Repacketizer {
	ptr: *mut ffi::OpusRepacketizer,
}

impl Repacketizer {
	pub fn new() -> Result<Repacketizer> {
		let ptr = unsafe { ffi::opus_repacketizer_create() };
		if ptr.is_null() {
			Err(Error::from_code("opus_repacketizer_create", ffi::OPUS_ALLOC_FAIL))
		} else {
			Ok(Repacketizer { ptr: ptr })
		}
	}

	pub fn reset(&mut self) {
		unsafe { ffi::opus_repacketizer_init(self.ptr); }
	}

	pub fn get_nb_frames(&mut self) -> usize {
		unsafe { ffi::opus_repacketizer_get_nb_frames(self.ptr) as usize }
	}

	pub fn cat(&mut self, packet: &[u8]) -> Result<()> {
		let result = unsafe { ffi::opus_repacketizer_cat(self.ptr,
			packet.as_ptr(), packet.len() as c_int) };
		check("opus_repacketizer_cat", result)
	}

	pub fn out(&mut self, buffer: &mut [u8]) -> Result<usize> {
		let result = unsafe { ffi::opus_repacketizer_out(self.ptr,
			buffer.as_mut_ptr(), buffer.len() as c_int) };
		try!(check("opus_repacketizer_out", result));
		Ok(result as usize)
	}

	pub fn out_range(&mut self, begin: usize, end: usize, buffer: &mut [u8]) -> Result<usize> {
		let result = unsafe { ffi::opus_repacketizer_out_range(self.ptr,
			begin as c_int, end as c_int,
			buffer.as_mut_ptr(), buffer.len() as c_int) };
		try!(check("opus_repacketizer_out_range", result));
		Ok(result as usize)
	}
}

impl Drop for Repacketizer {
	fn drop(&mut self) {
		unsafe { ffi::opus_repacketizer_destroy(self.ptr) }
	}
}

// ============================================================================
// TODO: Multistream API

// ============================================================================
// Error Handling

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
	function: &'static str,
	description: &'static str,
	code: ErrorCode,
}

impl Error {
	fn from_code(what: &'static str, code: c_int) -> Error {
		// description should always be ASCII
		let description = unsafe { CStr::from_ptr(ffi::opus_strerror(code)) }.to_str().unwrap();
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

	pub fn function(&self) -> &'static str { self.function }
	pub fn description(&self) -> &'static str { self.description }
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
