#![allow(unused)]

use core::fmt;
use std::time::Duration;
use std::collections::HashMap;
use std::fmt::Display;

pub enum EnvelopeError
{
	EnvPointsError,
	EnvExponetialZeroValue,
	EnvLengthExceeded
}

impl fmt::Display for EnvelopeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::EnvExponetialZeroValue => write!(f, "<EnvExponentialZeroValue>"),
			Self::EnvPointsError => write!(f, "<EnvPointsError>"),
			Self::EnvLengthExceeded => write!(f, "<EnvLengthExceeded>"),
		}
	}
}

#[derive(PartialEq, Debug)]
pub enum QEnvMode
{
	Linear,
	Exponential
}

impl fmt::Display for QEnvMode
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Linear => write!(f, "Linear"),
			Self::Exponential => write!(f, "Exponential")
		}
	}
}

#[derive(Debug)]
struct QEnvRT
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

impl QEnvRT
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

pub struct QEnvelope
{
	pub sr: f32,
	cache_vec: HashMap<String, Vec<f32>>,
	cache_rt: HashMap<String, QEnvRT>
}

impl QEnvelope
{
	/// ENVELOPE OBJECT
	///
	/// # Args
	/// ------
	///
	/// `env_mode`: type of envelope.
	/// \tLinear = QEnvMode::Linear
	/// \tExponential = QEnvMode::Exponential(f32) where the argument represent the type of the curve
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

	fn get_envelope_key(&self, env_points: &[f32], env_mode: &QEnvMode) -> String {
		let mut key: String = env_points
			.iter()
			.map(|v| v.to_string())
			.collect::<Vec<String>>()
			.join(",");
		key.push_str(&env_mode.to_string());
		key
	}

	/// GENERATE ENVELOPE SHAPE TO VEC
	///
	/// # Args
	/// ------
	///
	/// `env_points`: envelope shape encoded as a, t0, b, t1, c, tn, ...
	/// \tfrom a to b in t0 seconds and from b to c in t1 sec, and so on
	/// 
	/// 
	/// # Return
	/// --------
	/// 
	/// Vec<f32>
	/// 
	/// 
	pub fn envelope_to_vec(&mut self, env_points: &[f32], env_mode: &QEnvMode) -> Vec<f32> {
		let mut env = Vec::new();
		loop {
			match self.advance_envelope(env_points, env_mode) {
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
		env
	}

	/// GENERATE ENVELOPE SHAPE SAMPLE BY SAMPLE
	///
	/// # Args
	/// ------
	///
	/// `env_points`: envelope shape encoded as a, t0, b, t1, c, tn, ...
	/// \tfrom a to b in t0 seconds and from b to c in t1 sec, and so on
	/// 
	/// 
	/// # Return
	/// --------
	/// 
	/// Result<f32, EnvelopeError>
	/// 
	/// 
	pub fn advance_envelope(&mut self, env_points: &[f32], env_mode: &QEnvMode) -> Result<f32, EnvelopeError>{
		let key: String = self.get_envelope_key(env_points, env_mode);
		let (values, times) = self.get_times_and_values(env_points);
		if values.len() != times.len() + 1 { return Err(EnvelopeError::EnvPointsError) }

		if *env_mode == QEnvMode::Exponential && (values[0] == 0.0 || values[values.len() - 1] == 0.0) {
			return Err(EnvelopeError::EnvExponetialZeroValue)
		}
		
		if !self.cache_rt.contains_key(&key) { 
			self.cache_rt.insert(key.to_string(), QEnvRT::new()); 
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
						QEnvMode::Linear => {
							e.current_linear_step = (p2 - p1) / seg_length as f32;
							e.current_value = p1;	
						},
						QEnvMode::Exponential => {
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
							QEnvMode::Linear => {
								e.current_value += e.current_linear_step;
							},
							QEnvMode::Exponential => {
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

			println!("[INFO] value: {}, segment: {}, index: {}", e.current_value, e.current_segment, e.current_index);
			Ok(e.current_value)					
		} else {
			Err(EnvelopeError::EnvLengthExceeded)
		}
	}
}
