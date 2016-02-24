extern crate opus_sys as ffi;
extern crate libc;

use libc::{c_int};

// Generic CTLs
const OPUS_RESET_STATE: c_int = 4028; // void
const OPUS_GET_FINAL_RANGE: c_int = 4031; // *uint
const OPUS_GET_BANDWIDTH: c_int = 4009; // *int
const OPUS_GET_SAMPLE_RATE: c_int = 4029; // *int

pub enum CodingMode {
	Voip = 2048,
	Audio = 2049,
	LowDelay = 2051,
}

pub enum Channels {
	Mono = 1,
	Stereo = 2,
}

pub struct Error(&'static str, c_int);

fn check(what: &'static str, code: c_int) -> Result<()> {
	if code < 0 {
		Err(Error(what, code))
	} else {
		Ok(())
	}
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Encoder {
	ptr: *mut ffi::OpusEncoder,
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
			Err(Error("opus_encoder_create", error))
		} else {
			Ok(Encoder { ptr: ptr })
		}
	}

	pub fn encode(&mut self, input: &[i16], output: &mut [u8]) -> Result<usize> {
		let len = unsafe { ffi::opus_encode(self.ptr,
			input.as_ptr(), input.len() as c_int,
			output.as_mut_ptr(), output.len() as c_int) };
		try!(check("opus_encode", len));
		Ok(len as usize)
	}

	pub fn encode_float(&mut self, input: &[f32], output: &mut [u8]) -> Result<usize> {
		let len = unsafe { ffi::opus_encode_float(self.ptr,
			input.as_ptr(), input.len() as c_int,
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

	// TODO: Many more CTLs
}

impl Drop for Encoder {
	fn drop(&mut self) {
		unsafe { ffi::opus_encoder_destroy(self.ptr) }
	}
}
