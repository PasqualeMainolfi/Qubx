#![allow(unused)]

use core::fmt;
use std::time::Duration;
use std::collections::HashMap;
use std::fmt::Display;
use crate::qubx_common::{ Channels, ChannelError };
use super::{ 
	qinterp::{ Interp, PhaseInterpolationIndex }, qoperations::split_into_nchannels, qtable::{ TableArg, TableError, TableMode, TableParams }, shared_tools::{ interp_buffer_write, update_and_reset_increment, update_increment }
};

#[derive(Debug, Clone, Copy)]
pub enum EnvelopeError
{
	EnvPointsError,
	EnvExponetialZeroValue,
	EnvLengthExceeded,
	EnvToSignalErrorDifferentChannelNumbers,
	TableNotAllowed
}

impl fmt::Display for EnvelopeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::EnvExponetialZeroValue => write!(f, "<EnvExponentialZeroValue>"),
			Self::EnvPointsError => write!(f, "<EnvPointsError>"),
			Self::EnvLengthExceeded => write!(f, "<EnvLengthExceeded>"),
			Self::EnvToSignalErrorDifferentChannelNumbers => write!(f, "<EnvToSignalErrorDifferentChannelNumbers>"),
			Self::TableNotAllowed => write!(f, "<TableNotAllowed>"),
		}
	}
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum EnvMode
{
	Linear,
	Exponential
}

impl fmt::Display for EnvMode
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Linear => write!(f, "Linear"),
			Self::Exponential => write!(f, "Exponential")
		}
	}
}

pub struct EnvelopeObject
{
	pub vector_envelope: Vec<f32>,
	pub n_channels: usize
}

impl Channels for EnvelopeObject
{
	fn to_nchannels(&mut self, out_channels: usize) -> Result<(), ChannelError> {
		let prev_channels = self.n_channels;
        self.n_channels = out_channels;
        split_into_nchannels(&mut self.vector_envelope, prev_channels, out_channels)
	}
}

#[derive(Debug)]
struct EnvRT
{
	values: Vec<f32>,
	times: Vec<f32>,
	current_segment: usize,
	current_segment_length: usize,
	current_index: usize,
	current_linear_step: f32,
	current_exponential_components: (f32, f32),
	current_value: f32
}

impl EnvRT
{
	fn new() -> Self {
		Self {
			values: Vec::new(),
			times: Vec::new(),
			current_segment: 0,
			current_segment_length: 0,
			current_index: 0,
			current_linear_step: 0.0,
			current_exponential_components: (0.0, 0.0),
			current_value: 0.0,
		}
	}
}

#[derive(Debug, Clone)]
pub struct EnvParams
{
	pub shape: Vec<f32>,
	pub mode: EnvMode,
	phase_motion: f32,
	interp_buffer: Vec<f32>
}

impl EnvParams
{
	pub fn new(shape: Vec<f32>, mode: EnvMode) -> Self {
		Self { shape, mode, phase_motion: 0.0, interp_buffer: Vec::new() }
	}

	fn update_and_set_pmotion(&mut self, value: f32, table_length: f32) {
        update_and_reset_increment(&mut self.phase_motion, value, table_length);
    }
    
    fn update_pmotion(&mut self, value: f32) {
        update_increment(&mut self.phase_motion, value);
    }

    fn write_interp_buffer(&mut self, interp: Interp, sample: f32) {
        interp_buffer_write(&mut self.interp_buffer, interp, sample);
    }
}

/// Adsr Parameters
/// 
/// `attack_dur`: duration of attack in sec  
/// `decay_dur`: duration of decay in sec  
/// `sustain_dur`: duration of sustain in sec  
/// `sustain_level`: level of sustain [0, 1]  
/// `release_dur`: duration of release in sec  
/// `mode`: envelope mode (see `EnvMode`)  
/// 
#[derive(Debug, Clone, Copy)]
pub struct AdsrParams
{
	pub attack_dur: f32,
	pub decay_dur: f32,
	pub sustain_dur: f32,
	pub sustain_level: f32,
	pub release_dur: f32,
	pub mode: EnvMode
}

impl AdsrParams
{
	fn get_env_params(&self) -> EnvParams {
		let t0 = self.attack_dur;
		let t1 = self.decay_dur;
		let t2 = self.sustain_dur;
		let t3 = self.release_dur;
		let attack_value = if self.mode == EnvMode::Linear { 0.0 } else { 0.001 };
		let sustain_value = self.sustain_level;
		let end_value = attack_value;

		let shape = [attack_value, t0, 1.0, t1, sustain_value, t2, sustain_value, t3, end_value];
		EnvParams::new(shape.to_vec(), self.mode)
	}
}

pub struct QEnvelope
{
	pub sr: f32,
	cache_vec: HashMap<String, Vec<f32>>,
	cache_rt: HashMap<String, EnvRT>
}

impl QEnvelope
{
	/// Envelope Obj
	///
	/// # Args
	/// ------
	///
	/// `sr`: sample rate
	///

	pub fn new(sr: f32) -> Self {
		Self { sr, cache_vec: HashMap::new(), cache_rt: HashMap::new() }
	}

	fn get_times_and_values(&self, env_points: &[f32]) -> (Vec<f32>, Vec<f32>) {
		let mut ts = Vec::new();
		let mut vs = Vec::new();
		for (i, value) in env_points.iter().enumerate() {
			if i % 2 == 0 { vs.push(*value) } else { ts.push(*value) }
		}
		(vs, ts)
	}

	fn get_envelope_key(&self, env_params: &EnvParams) -> String {

		let mut key: String = env_params.shape
			.iter()
			.map(|v| v.to_string())
			.collect::<Vec<String>>()
			.join(",");
		key.push_str(&env_params.mode.to_string());
		key
	}

	/// Generate envelope shape to vector
	///
	/// # Args
	/// ------
	///
	/// `env_params`: `EnvParams` struct in which you can specify:
	/// envelope shape encoded as a, t0, b, t1, c, tn, ...
	/// from a to b in t0 seconds and from b to c in t1 sec, and so on, and 
	/// envelope mode, linear or exponential (see `EnvMode`)  
	/// 
	/// # Return
	/// --------
	/// 
	/// `EnvelopeObject`
	/// 
	/// 
	pub fn into_envelope_object(&mut self, env_params: &EnvParams) -> EnvelopeObject {
		let mut env = Vec::new();
		loop {
			match self.advance_envelope(env_params, true) {
				Ok(sample) => env.push(sample),
				Err(e) => {
					match e {
						EnvelopeError::EnvLengthExceeded => break,
						_ => {
							eprintln!("[ERROR]: {}", &e.to_string());
							std::process::exit(1);
						}
					}
				}
			}
		}
		EnvelopeObject { vector_envelope: env, n_channels: 1 }
	}

	/// Generate ADSR shape to vec
	/// The envelope generates is in the range [0.0, 1.0]
	/// 
	/// # Args
	/// ------
	/// 
	/// `adsr_params`: ADSR params (see `AdsrParams`). Sustain level must be in range [0, 1]
	/// 
	/// # Return
	/// --------
	/// 
	/// `EnvelopeObject`
	/// 
	pub fn adsr_to_envelope_object(&mut self, adsr_params: &AdsrParams) -> EnvelopeObject { 
		let env_params = adsr_params.get_env_params();
		self.into_envelope_object(&env_params)
	}

	/// Generate ADSR sample by sample
	/// The envelope generates is in the range [0, 1]
	/// 
	/// # Args
	/// ------
	/// 
	/// `adsr_params`: ADSR params (see `AdsrParams`). Sustain level must be in the range [0, 1]  
	/// `length exceeded`: if `true` return `Err(EnvelopeError::EnvLengthExceeded)`  
	/// at the end of envelope. If `false`, at the end of envelope it remain  
	/// on the last value
	/// 
	/// # Return
	/// --------
	/// 
	/// `Result<f32, EnvelopeError>`
	///
	pub fn advance_adsr(&mut self, adsr_params: &AdsrParams, length_exceeded: bool) -> Result<f32, EnvelopeError> {
		let env_params = adsr_params.get_env_params();
		self.advance_envelope(&env_params, length_exceeded)
	}

	/// Generate Envelope shape sample by sample
	///
	/// # Args
	/// ------
	///
	/// `env_params`: `EnvParams` struct in which you can specify:
	/// envelope shape encoded as a, t0, b, t1, c, tn, ...
	/// from a to b in t0 seconds and from b to c in t1 sec, and so on, and 
	/// envelope mode, linear or exponential (see `EnvMode`)  
	/// `length exceeded`: if `true` return `Err(EnvelopeError::EnvLengthExceeded)`  
	/// at the end of envelope. If `false`, at the end of envelope it remain  
	/// on the last value
	/// 
	/// # Return
	/// --------
	/// 
	/// `Result<f32, EnvelopeError>`
	/// 
	/// 
	pub fn advance_envelope(&mut self, env_params: &EnvParams, length_exceeded: bool) -> Result<f32, EnvelopeError>{
		let key: String = self.get_envelope_key(env_params);
		let (values, times) = self.get_times_and_values(&env_params.shape);
		if values.len() != times.len() + 1 { return Err(EnvelopeError::EnvPointsError) }

		let env_mode = &env_params.mode;

		if *env_mode == EnvMode::Exponential && (values[0] == 0.0 || values[values.len() - 1] == 0.0) {
			return Err(EnvelopeError::EnvExponetialZeroValue)
		}
		
		if !self.cache_rt.contains_key(&key) { 
			self.cache_rt.insert(key.to_string(), EnvRT::new()); 
			let e = self.cache_rt.get_mut(&key).unwrap();
			e.values = values;
			e.times = times;
		}
		
		let e = self.cache_rt.get_mut(&key).unwrap();
		if e.current_segment < e.times.len() {
			e.current_segment_length = (e.times[e.current_segment] * self.sr).ceil() as usize;
			let seg_length = e.current_segment_length;
			let seg_cur = e.current_segment;
			let seg_next = e.current_segment + 1;
			let p1 = e.values[seg_cur];
			let p2 = e.values[seg_next];
			match e.current_index {
				0 => {
					match env_mode {
						EnvMode::Linear => {
							e.current_linear_step = (p2 - p1) / seg_length as f32;
							e.current_value = p1;	
						},
						EnvMode::Exponential => {
							e.current_exponential_components.1 = (p2 / p1).powf(1.0 / (seg_length - 1) as f32);
							e.current_exponential_components.0 = p1 / e.current_exponential_components.1;
							e.current_value = p1;
						}
					}
					e.current_index += 1;
				}
				_ => {
					if seg_next == e.values.len() - 1 && e.current_index == seg_length - 1 {
						e.current_value = e.values[seg_next]
					} else {
						match env_mode {
							EnvMode::Linear => {
								e.current_value += e.current_linear_step;
							},
							EnvMode::Exponential => {
								let a = e.current_exponential_components.0;
								let b = e.current_exponential_components.1;
								let x = e.current_index as i32 + 1;
								e.current_value = a * b.powi(x);
							}
						}
					}
					e.current_index += 1;
				}
			}

			if e.current_index >= seg_length {
				e.current_index = 0;
				e.current_segment += 1;
			}

			// println!("[INFO] value: {}, segment: {}, index: {}", e.current_value, e.current_segment, e.current_index);

			Ok(e.current_value)					
		} else {
			if !length_exceeded { return Ok(e.current_value) } 
			Err(EnvelopeError::EnvLengthExceeded)
		}
	}

	pub fn advance_envelope_from_table(&self, envelope_table: &mut TableParams, interp: Interp, duration: f32) -> Result<f32, EnvelopeError> {
		match envelope_table.mode {
			TableMode::Envelope(ref mut params) => {
				let f = 1.0 / duration;
				let si = f * envelope_table.table_length / self.sr;
				let table_index = PhaseInterpolationIndex::new(params.phase_motion);
				let index_int = table_index.int_part;
				let frac_part = table_index.frac_part;
				params.write_interp_buffer(interp, envelope_table.table[index_int]);
				let sample = interp.get_table_interpolation(frac_part, &params.interp_buffer).unwrap();
				params.update_and_set_pmotion(si, envelope_table.table_length);
				Ok(sample)
			},
			TableMode::Signal(_) => Err(EnvelopeError::TableNotAllowed)
		}
	}
}
