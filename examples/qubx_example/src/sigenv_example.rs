use qubx::{ 
    Qubx, 
    StreamParameters, 
    ProcessArg, 
    DspProcessArgs, 
    DspClosureNoArgsType, 
    DspClosureWithArgsType, 
    MasterClosureType,
    qinterp::SignalInterp,
    qsignals::{ QSignal, SignalMode, SignalParams },
    qenvelopes::{ QEnvelope, EnvParams, EnvMode }
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
    let master_clos: MasterClosureType = Box::new(|frame| {
        frame.iter_mut().for_each(|sample| { *sample *= 0.7 }) 
    });
    master_out.start(ProcessArg::Closure::<MasterClosureType>(master_clos));

    let mut dsp_process = q.create_parallel_dsp_process(String::from("M1"), true);

    let duration = 1.1;

    let mut sine_params = SignalParams::new(SignalMode::Sine, SignalInterp::Linear, 440.0, 0.7, 0.0, SR as f32);
    let mut sine = QSignal::new(SR as usize);
    let sine_vec = sine.signal_to_vec(&mut sine_params, duration).unwrap();

    let env_points = vec![0.001, 0.1, 1.0, duration - 0.1, 0.001];
    let exponential_env_params = EnvParams { shape: env_points, mode: EnvMode::Exponential };
	let mut exp_env = QEnvelope::new(SR as f32);
    let env_shape = exp_env.envelope_to_vec(&exponential_env_params);

    let dsp_clos: DspClosureNoArgsType = Box::new(move || {
        let y = sine_vec
            .iter()
            .zip(env_shape.iter())
            .map(|(&x, &e)| x * e)
            .collect::<Vec<f32>>();
        y
    });

    dsp_process.start(DspProcessArgs::Closure::<DspClosureNoArgsType, DspClosureWithArgsType>(dsp_clos));

    std::thread::sleep(std::time::Duration::from_secs_f32(1.5));

    q.close_qubx();
}
