use qubx::qsignals::{ SignalMode, QSignal };
use qubx::qinterp::SignalInterp;
use qubx::SignalParams;

const SR: f32 = 16.0;

fn main() {
    let mut params = SignalParams::new(SignalMode::Sine, SignalInterp::Linear, 2.5, 1.0, 0.0, 1.0, SR);
    let mut signal = QSignal::new(16);
    for i in 0..(SR as usize) {
        let sample = signal.procedural_oscillator(&mut params);
        println!("SAMPLE {i}: {sample}");
    }

    let svec = signal.signal_to_vec(&mut params);
    println!("{:?}", svec);

    let mut paramsc = SignalParams::new(SignalMode::Sine, SignalInterp::Cubic, 2.5, 1.0, 0.0, 1.0, SR);
    let svec = signal.signal_to_vec(&mut paramsc);
    println!("CUBIC: {:?}", svec);
    
    let mut paramsh = SignalParams::new(SignalMode::Sine, SignalInterp::Hermite, 2.5, 1.0, 0.0, 1.0, SR);
    let svec = signal.signal_to_vec(&mut paramsh);
    println!("HERMITE: {:?}", svec);
}
