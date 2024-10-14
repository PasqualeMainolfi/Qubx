use qubx::qenvelopes::{ QEnvMode, QEnvelope };


fn main() {
	let env_points = vec![0.0001, 0.1, 1.0, 1.0, 0.0];
	let sr: f64 = 100.0;

	let mut linear_env_shape = QEnvelope::new(QEnvMode::Linear, sr);
	let l_env = linear_env_shape.generate(&env_points);
	println!("LINEAR ENVELOPE");
	println!("{:?}", l_env);

	let mut exponential_env_shape = QEnvelope::new(QEnvMode::Exponential(0.01), sr);
	let e_env = exponential_env_shape.generate(&env_points);
	println!("EXPONENTIAL ENVELOPE");
	println!("{:?}", e_env);

}
