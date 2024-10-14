#![allow(unused)]

use std::time::Duration;
use std::collections::HashMap;

pub enum QEnvMode
{
	Linear,
	Exponential(f64)
}

pub struct QEnvelope
{
	pub mode: QEnvMode,
	pub sample_rate: f64,
	cache: HashMap<String, Vec<f64>>
}


impl QEnvelope
{
	/// Define new envelope shape
	///
	/// # Args
	/// ------
	///
	/// `env_mode`: type of envelope.
	/// 			Linear = QEnvMode::Linear
	/// 			Exponential = QEnvMode::Exponential(f64) where the argument represent the type of the curve
	/// `sample_rate`: sample rate
	///

	pub fn new(env_mode: QEnvMode, sample_rate: f64) -> Self {
		Self { mode: env_mode, sample_rate, cache: HashMap::new() }
	}

	/// Generate envelope shape
	///
	/// # Args
	/// ------
	///
	/// `env_points`: envelope shape encoded as a, t0, b, t1, c, tn, ...
	/// 			from a to b in t0 seconds and from b to c in t1 sec, and so on

	pub fn generate(&mut self, env_points: &Vec<f64>) -> &Vec<f64> {
		let env_points_lenght = env_points.len();

		if env_points_lenght % 2 == 0 {
			eprintln!("[ERROR] env_points len must be odd. Even indexes for value; odd index for time values");
			std::process::exit(1)
		}

		let key: String = env_points
			.iter()
			.map(|v| v.to_string())
			.collect::<Vec<String>>()
			.join(", ");

		if self.cache.contains_key(&key) {
			return &self.cache[&key];
		};

		let mut env: Vec<f64> = vec![];
		for i in (1..=env_points_lenght - 1).step_by(2) {
			let segment_length = (env_points[i] * self.sample_rate).ceil();
			let start_value = env_points[i - 1];
			let end_value = env_points[i + 1];
			match self.mode {
				QEnvMode::Linear => {
					let step = (end_value - start_value) / (segment_length - 1.0);
					let mut value = start_value;
					for _ in (0..segment_length as usize) {
						env.push(value);
						value += step;
					}
				},
				QEnvMode::Exponential(r) => {
					if r <= 0.0 { 0.001; }
					if start_value <= 0.0 || end_value <= 0.0 { println!("[WARNING] In exponential values less or equal to zero are not allowed! Values will be changed to 0.0001"); }
					let exp_start_value = if start_value <= 0.0 { 0.0001 } else { start_value };
					let exp_end_value = if end_value <= 0.0 { 0.0001 } else { end_value };
					env.push(exp_start_value);
					let start_value_curve = exp_start_value + r;
					let b = (exp_end_value / start_value_curve).powf(1.0 / (segment_length - 1.0) as f64);
					let a = start_value_curve / b;
					for i in (1..segment_length as usize) {
						let y = a * b.powi(i as i32 + 1);
						env.push(y);
					}
				}
			}
		}

		self.cache.insert(key.to_string(), env);
		&self.cache[&key]
	}
}
