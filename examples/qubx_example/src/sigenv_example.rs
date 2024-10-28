use qubx::{ 
    Qubx, 
    StreamParameters, 
    ProcessArg, 
    DspProcessArgs, 
    DspCAType,
    DspCNAType, 
    MasterCType,
    qinterp::Interp,
    qsignals::{ QSignal, SignalMode, SignalParams, ComplexSignalParams },
    qenvelopes::{ QEnvelope, EnvParams, EnvMode },
    qoperations::envelope_to_signal,
    qtable::{ QTable, TableMode, TableArg }
};

const SR: i32 = 44100;
const CHANNELS: u32 = 1;
const CHUNK: u32 = 1024;

pub fn sigenv_example() {

    let stream_params = StreamParameters {
        chunk: CHUNK,
        sr: SR,
        outchannels: CHANNELS,
        ..StreamParameters::default()
    };
    
    let mut q = Qubx::new(false);
    q.start_monitoring_active_processes();
    
    let mut master_out = q.create_master_streamout(String::from("M1"), stream_params);
    let master_clos: MasterCType = Box::new(|frame| {
        frame.iter_mut().for_each(|sample| { *sample *= 0.7 }) 
    });
    master_out.start(ProcessArg::Closure(master_clos));

    let mut dsp_process = q.create_parallel_dsp_process(String::from("M1"), true);

    let duration = 1.1;

	let mut exp_env = QEnvelope::new(SR as f32);
    
    let mut sine_params = SignalParams::new(SignalMode::Sine, 440.0, 0.7, 0.0, SR as f32);
    let mut sine_table = QTable::new();
    let _ = sine_table.write_table("sine", TableMode::Signal(SignalMode::Sine), SR as usize);
    let signal_sine = QSignal::into_signal_object(&mut sine_params, duration, TableArg::WithTable((sine_table.get_table("sine"), Interp::Cubic))).unwrap();
    
    let mut comp_params = ComplexSignalParams::new([440.0, 567.0, 768.0].to_vec(), [0.7, 0.5, 0.3].to_vec(), None, SR as f32);
    let signal_comp = QSignal::into_signal_object(&mut comp_params, duration, TableArg::NoTable).unwrap();

    let env_points = vec![0.001, 0.1, 1.0, duration - 0.1, 0.001];
    let exponential_env_params = EnvParams::new(env_points, EnvMode::Exponential);
    let env_shape = exp_env.into_envelope_object(&exponential_env_params);

    // let dsp_clos: DspClosureNoArgsType = Box::new(move || {
    //     let y = sine_vec
    //         .iter()
    //         .zip(env_shape.iter())
    //         .map(|(&x, &e)| x * e)
    //         .collect::<Vec<f32>>();
    //     y
    // });
    // dsp_process.start(DspProcessArgs::Closure::<DspCNAType, DspCAType>(dsp_clos));

    let enveloped_sine_signal = envelope_to_signal(&signal_sine, &env_shape).unwrap();
    dsp_process.start(DspProcessArgs::AudioData::<DspCNAType, DspCAType>(enveloped_sine_signal.vector_signal));
    std::thread::sleep(std::time::Duration::from_secs_f32(1.5));

    let enveloped_comp_signal = envelope_to_signal(&signal_comp, &env_shape).unwrap();
    dsp_process.start(DspProcessArgs::AudioData::<DspCNAType, DspCAType>(enveloped_comp_signal.vector_signal));
    std::thread::sleep(std::time::Duration::from_secs_f32(1.5));
    

    q.close_qubx();
}
