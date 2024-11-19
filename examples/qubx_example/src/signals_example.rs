use qubx::qsignals::{ SignalMode, QSignal, SignalParams };
use qubx::qtable::{ QTable, TableMode, TableArg };
use qubx::qinterp::Interp;

const SR: f32 = 16.0;

pub fn signals_example() {
    let mut params = SignalParams::new(SignalMode::Sine, 1.0, 1.0, 0.0, SR);
    for i in 0..(SR as usize) {
        let sample = QSignal::procedural_oscillator(&mut params);
        println!("SAMPLE {i}: {sample}");
    }

    let mut sine_table = QTable::new();
    let _ = sine_table.write_table("sine".to_string(), TableMode::Signal(SignalMode::Sine), 16);

    let sinel = QSignal::into_signal_object(&mut params, 1.0, TableArg::WithTable((sine_table.get_table("sine".to_string()), Interp::Linear))).unwrap();
    println!("{:?}", sinel.vector_signal);


}
