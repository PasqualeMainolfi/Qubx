
use qubx::qenvelopes::{ EnvMode, EnvParams };
use qubx::qgenesis::{ QGranulator, QModulation };
use qubx::genesis_params::{ GranularParams, ModulationMode, ModulationParams, ModulationType };
use qubx::qtable::{ QTable, TableArg, TableMode };
use qubx::qsignals::{ QSignal, SignalMode, SignalObject, SignalParams };
use qubx::qinterp::Interp;
use qubx::qwindow::QWindow;
use qubx::qbuffers::AudioBuffer;

pub fn genesis_example() {
	let table_env_length = 16384.0;
	let sr = 44100.0;
	let table_id: String = "grain_envelope".to_string();
	let mut grain_envelope = QTable::new();

	let atk = table_env_length * 0.01;
	let rel = table_env_length - atk;
	let win_env = EnvParams::new(vec![0.001, atk, 1.0, rel, 0.0001], EnvMode::Exponential);
	grain_envelope.write_table(table_id.clone(), TableMode::Envelope(win_env), table_env_length as usize).unwrap();

	// let win = QWindow::Hamming.get_window(table_env_length as usize);
	// grain_envelope.write_table(table_id.clone(), TableMode::EnvelopeData(win), table_env_length as usize).unwrap();


	let path: &str = "/Users/pm/AcaHub/AudioSamples/cane.wav";
	let audio = AudioBuffer::new(sr as i32);
	let audio_object = audio.to_audio_object(path).unwrap();

	let mut table_envelope = grain_envelope.get_table(table_id);
	let mut grain_params = GranularParams::new(Interp::Linear, &mut table_envelope);
	grain_params.set_frequency_range((500.0, 3000.0));
	grain_params.set_duration_range((0.001, 0.5));
	grain_params.set_delay_range((0.1, 0.3));
	grain_params.set_phase_range((0.0, 1.0));

	// let table_mode = TableMode::Data((audio_object.vector_signal, audio_object.n_channels));
	let table_mode = TableMode::Signal(SignalMode::Sine);

	let mut granulator = QGranulator::new(table_mode, sr);
	let mut samples = Vec::new();
	for _ in 0..(5 * sr as usize) {
		let sample = granulator.process(&mut grain_params);
		samples.push(sample);
	}

	// let mut table = QTable::new();
	// table.write_table(String::from("sine"), TableMode::Signal(SignalMode::Sine), sr as usize).unwrap();

	// let mut sig_test = SignalParams::new(SignalMode::Sine, 2000.0, 0.7, 0.0, sr);
	// let s = QSignal::into_signal_object(
	// 	&mut sig_test, 3.0, TableArg::WithTable((table.get_table("sine".to_string()), Interp::Cubic))
	// ).unwrap();

	let sig = SignalObject { vector_signal: samples, n_channels: 1, sr };
	AudioBuffer::write_to_file("grain_test", &sig).unwrap();


	let mut modulation = QModulation::new(sr);
	let mut fm_params = ModulationParams::new(SignalMode::Sine, SignalMode::Sine);
	fm_params.set_carrier_freq(220.0);
	fm_params.set_carrier_amp(0.7);
	fm_params.set_modulating_freq(440.0);
	fm_params.set_modulation_index(1.2 * 440.0);

	let mut samples = Vec::new();
	for _ in 0..(30 * sr as usize) {
		let sample = modulation.process(&mut ModulationType::Fm(&mut fm_params), ModulationMode::Procedural);
		samples.push(sample);
	}

	let sig = SignalObject { vector_signal: samples, n_channels: 1, sr };
	AudioBuffer::write_to_file("mod_test", &sig).unwrap();

}
