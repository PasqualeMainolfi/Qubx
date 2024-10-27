#![allow(unused_imports, unused_variables, unused_mut, dead_code)]

use qubx::{ 
    Qubx, 
    StreamParameters, 
    ProcessArg, 
    DspProcessArgs, 
    DspCNAType, 
    DspCAType, 
    DuplexCType, 
    MasterCType 
};
use rand::Rng;
use std::{ cmp::min, fs::File, thread, time::Duration };

fn open_file(path: &str) -> Vec<f32> {
    let file = File::open(path).unwrap_or_else(|err| {
        println!("[ERROR] File {path} not found!, {err}");
        std::process::exit(1)
    });

    let (_, samples) = wav_io::read_from_file(file).unwrap();
    samples
}

fn hanning_window(size: usize) -> Vec<f32> {
    let mut w = vec![0.0; size];

    for (i, value) in w.iter_mut().enumerate() {
        *value = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / size as f32).cos());
    }
    w
}

const SR: i32 = 44100;
const CHANNELS: u32 = 1;
const CHUNK: u32 = 1024;


const FILES: [&str; 2] = [
    "./../audio_files_for_test/maderna_continuo.wav",
    "./../audio_files_for_test/maderna_continuo.wav",
];

enum TestMode {
    Input,
    Output,
}

fn simple_example() {
	println!("[INFO] FRAME LENGTH: {}", CHUNK);
    let mode = TestMode::Output;

    let mut run = true;

    let stream_params = StreamParameters {
        chunk: CHUNK,
        sr: SR,
        outchannels: CHANNELS,
        ..StreamParameters::default()
    };

    let mut q = Qubx::new(false);

    // start monitor active processes
    q.start_monitoring_active_processes();
    // ---

    match mode {
        TestMode::Input => {
            let mut duplex = q.create_duplex_dsp_process(stream_params);
            let clos: DuplexCType = Box::new(|frame| frame.to_vec());
            duplex.start(ProcessArg::Closure(clos));

            for i in 0..(10 * SR as usize) {
                std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / SR as f32));
            }
        }

        TestMode::Output => {
            let mut master_out = q.create_master_streamout(String::from("M1"), stream_params);
            let master_clos: MasterCType = Box::new(|frame| {
                frame.iter_mut().for_each(|sample| { *sample *= 0.7 }) 
            });

            master_out.start(ProcessArg::Closure(master_clos));

            let mut dsp_process1 = q.create_parallel_dsp_process(String::from("M1"), true);
            let mut dsp_process2 = q.create_parallel_dsp_process(String::from("M1"), true);

            let audio1 = open_file(FILES[0]);
            let audio2 = open_file(FILES[1]);

            let audio_sigs: [Vec<f32>; 2] = [audio1, audio2];

            let mut rng = rand::thread_rng();

            let mut count = 0;
            loop {
                let random_size = rng.gen_range((44100 * (5 * 60))..(44100 * (7 * 60)));
                let n: usize = random_size as usize;
                let sig_size1 = audio_sigs[0].len();
                let start_index1 = rng.gen_range(0..sig_size1 - n);
                let end_index1 = (start_index1 + random_size).min(sig_size1 - 1);
                let audio_ref1: &[f32] = &audio_sigs[0][start_index1..end_index1];

                let sig_size2 = audio_sigs[1].len();
                let start_index2 = rng.gen_range(0..sig_size2 - n);
                let end_index2 = (start_index2 + random_size).min(sig_size2 - 1);
                let audio_ref2: &[f32] = &audio_sigs[1][start_index2..end_index2];

                let mut audio_data1 = vec![0.0; n];
                audio_data1[0..audio_ref1.len()].copy_from_slice(audio_ref1);

                let mut audio_data2 = vec![0.0; n];
                audio_data2[0..audio_ref2.len()].copy_from_slice(audio_ref2);

                let envelope = hanning_window(n);
                for (i, (sample1, sample2)) in audio_data1
                    .iter_mut()
                    .zip(audio_data2.iter_mut())
                    .enumerate()
                {
                    *sample1 *= envelope[i];
                    *sample2 *= envelope[i];
                }

                let dsp_clos: DspCAType = Box::new(|_audio_data| {
                	let y = _audio_data.iter().map(|sample| sample * 0.7).collect();
                 	y
                });

                dsp_process1.start(DspProcessArgs::AudioDataAndClosure::<DspCNAType, DspCAType>(audio_data1, dsp_clos));
                dsp_process2.start(DspProcessArgs::AudioData::<DspCNAType, DspCAType>(audio_data2));
                
                if count >= 30 {
                    run = false
                }
                count += 1;

                if !run {
                    break;
                }

                let delay = rng.gen_range(0.5..1.0);
                thread::sleep(Duration::from_secs_f32(delay));
            }
        }
    }

    thread::sleep(Duration::from_secs(1));
    q.close_qubx();
}
