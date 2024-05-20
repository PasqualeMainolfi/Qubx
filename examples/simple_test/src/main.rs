#![allow(unused_imports, unused_variables, unused_mut, dead_code)]

use qubx::{Qubx, StreamParameters};
use rand::Rng;
use std::{cmp::min, fs::File, thread, time::Duration};

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
const CHUNK: u32 = 4096;

const FILES: [&str; 2] = [
    "./../audio_files_for_test/vox.wav",
    "./../audio_files_for_test/suzanne_mono.wav",
];

enum TestMode {
    Input,
    Output,
}

fn main() {
    let mode = TestMode::Output;

    let mut run = true;

    let stream_params = StreamParameters {
        chunk: CHUNK,
        sr: SR,
        outchannels: CHANNELS,
        ..StreamParameters::default()
    };

    let mut q = Qubx::new(true);

    // start monitor active processes
    q.start_monitoring_active_processes();
    // ---

    match mode {
        TestMode::Input => {
            let mut duplex = q.create_duplex_dsp_process(stream_params);
            duplex.start(|frame| frame.to_vec());

            for i in 0..(10 * SR as usize) {
                std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / SR as f32));
            }
        }

        TestMode::Output => {
            let mut master_out = q.create_master_streamout(String::from("M1"), stream_params);
            master_out.start(move |frame| {
                for sample in frame.iter_mut() {
                    *sample *= 0.7;
                }
            });

            let mut dsp_process1 = q.create_parallel_dsp_process(String::from("M1"));
            let mut dsp_process2 = q.create_parallel_dsp_process(String::from("M1"));

            let audio1 = open_file(FILES[0]);
            let audio2 = open_file(FILES[1]);

            let audio_sigs: [Vec<f32>; 2] = [audio1, audio2];

            let mut rng = rand::thread_rng();

            let mut count = 0;
            loop {
                let random_size = rng.gen_range(44100..(44100 * 3));
                let n: usize = random_size as usize;
                // let index_audio_array = rng.gen_range(0..2);
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

                dsp_process1.start(audio_data1, move |frame| {
                    for sample in frame.iter_mut() {
                        *sample *= 0.7;
                    }
                });

                dsp_process2.start(audio_data2, move |_frame| {});

                if count >= 5 {
                    run = false
                }
                count += 1;

                if !run {
                    break;
                }

                let delay = rng.gen_range(0.1..2.1);
                thread::sleep(Duration::from_secs_f32(delay));
            }
        }
    }

    thread::sleep(Duration::from_secs(1));
    q.close_qubx();
}
