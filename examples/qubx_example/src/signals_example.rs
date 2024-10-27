use qubx::qsignals::{ SignalMode, QSignal, SignalParams };
use qubx::qinterp::SignalInterp;

const SR: f32 = 16.0;

pub fn signals_example() {
    let mut params = SignalParams::new(SignalMode::Sine, SignalInterp::Linear, 1.0, 1.0, 0.0, SR);
    let mut signal = QSignal::new(16);
    for i in 0..(SR as usize) {
        let sample = signal.procedural_oscillator(&mut params);
        println!("SAMPLE {i}: {sample}");
    }

    let sinel = signal.into_signal_object(&mut params, 1.0);
    println!("{:?}", sinel.vector_signal);

    let mut paramsc = SignalParams::new(SignalMode::Sine, SignalInterp::Cubic, 2.5, 1.0, 0.0, SR);
    let sinec = signal.into_signal_object(&mut paramsc, 1.0);
    println!("CUBIC: {:?}", sinec.vector_signal);
    
    let mut paramsh = SignalParams::new(SignalMode::Sine, SignalInterp::Hermite, 2.5, 1.0, 0.0, SR);
    let sineh = signal.into_signal_object(&mut paramsh, 1.0);
    println!("HERMITE: {:?}", sineh.vector_signal);
}
