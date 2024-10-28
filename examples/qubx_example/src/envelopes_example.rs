use qubx::qenvelopes::{ QEnvelope, EnvParams, EnvMode };

pub fn envelopes_example() {
	let env_lin_points = vec![0.0, 0.1, 1.0, 0.1, 0.5, 1.0, 0.0];
	let sr: f32 = 1000.0;

	let mut linear_env_shape = QEnvelope::new(sr);

	let linear_env_params = EnvParams::new(env_lin_points, EnvMode::Linear);
	let _l_env = linear_env_shape.into_envelope_object(&linear_env_params);
	println!("LINEAR ENVELOPE");
	// println!("{:?}", l_env);

	let env_exp_points = vec![0.001, 0.1, 1.0, 0.1, 0.5, 1.0, 1.0, 0.5, 0.01];
	let exponetial_env_params = EnvParams::new(env_exp_points, EnvMode::Exponential);
	let mut exponential_env_shape = QEnvelope::new(sr);
	let _e_env = exponential_env_shape.into_envelope_object(&exponetial_env_params);
	println!("EXPONENTIAL ENVELOPE");
	// println!("{:?}", e_env);

}
