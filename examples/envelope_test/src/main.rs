use qubx::qenvelopes::{ QEnvMode, QEnvelope };

fn main() {
	let env_lin_points = vec![0.0, 0.1, 1.0, 0.1, 0.5, 1.0, 0.0];
	let sr: f32 = 1000.0;

	let mut linear_env_shape = QEnvelope::new(sr);
	let l_env = linear_env_shape.envelope_to_vec(&env_lin_points, &QEnvMode::Linear);
	println!("LINEAR ENVELOPE");
	// println!("{:?}", l_env);

	let env_exp_points = vec![0.001, 0.1, 1.0, 0.1, 0.5, 1.0, 1.0, 0.5, 0.01];
	let mut exponential_env_shape = QEnvelope::new(sr);
	let e_env = exponential_env_shape.envelope_to_vec(&env_exp_points, &QEnvMode::Exponential);
	println!("EXPONENTIAL ENVELOPE");
	// println!("{:?}", e_env);

}
