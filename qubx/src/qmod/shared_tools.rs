use std::path::Path;
use std::process::{ Command, Stdio };
use std::fs;
use std::io::Write;

use super::{ 
    qsignals::{ SignalMode, SignalError, SignalParams }, 
    qinterp::{ Interp, PhaseInterpolationIndex },
    qtable::TableParams
};

use crate::qubx_common::ToFileError;

const TWOPI: f32 = 2.0 * std::f32::consts::PI;

// --- SIGNAL TOOLS ---

pub(crate) fn get_phase_motion(t: f32, freq: f32, amp: f32, phase_offset: f32, sr: f32, mode: &SignalMode) -> Result<f32, SignalError> {
    let phase = freq * t / sr;
    let sample = match mode {
        SignalMode::Sine => (TWOPI * (phase + phase_offset)).sin(),
        SignalMode::Saw => 1.0 - 2.0 * (phase - (phase).floor()),
        SignalMode::Triangle => (2.0 / std::f32::consts::PI) * ((TWOPI * phase).sin()).asin(),
        SignalMode::Square => ((TWOPI * phase).sin()).signum(),
        SignalMode::Phasor => phase - (phase).floor(),
        SignalMode::Pulse(duty) => if (phase - (phase).floor()) < *duty { 1.0 } else { 0.0 },
        SignalMode::ComplexSignal => { return Err(SignalError::SignalModeNotAllowed) }
    };
    Ok(sample * amp)
}

// --- TOOLS --- 

pub(crate) fn get_oscillator_phase(wave_table: &TableParams, signal_params: &mut SignalParams, interp: Interp) -> f32 {
    let si = signal_params.freq / signal_params.sr * wave_table.table_length;
    let phase_offset = signal_params.phase_offset * wave_table.table_length;
    let phase_index = (signal_params.phase_motion + phase_offset) % wave_table.table_length;
    let table_index = PhaseInterpolationIndex::new(phase_index);
    let index_int = table_index.int_part;
    let frac_part = table_index.frac_part;
    let table = &wave_table.table;
    signal_params.write_interp_buffer(interp, table[index_int]);
    let sample = interp.get_table_interpolation(frac_part, &signal_params.interp_buffer).unwrap();
    signal_params.update_and_set_pmotion(si, wave_table.table_length);
    signal_params.amp * sample
}

pub(crate) fn build_signal(wave_table: &TableParams, signal_params: &mut SignalParams, interp: Interp, duration: f32) -> Vec<f32> {
    let n_samples = (duration * signal_params.sr) as usize;
    (0..n_samples).map(|_| get_oscillator_phase(wave_table, signal_params, interp)).collect::<Vec<f32>>()
}

// fn build_table(mode: SignalMode, table_length: f32) -> (Vec<f32>, f32) {
//     let mut table_signal = SignalParams { mode, freq: 1.0, sr: table_length, ..Default::default() };
//     let mut table: Vec<f32> = vec![0.0; table_length as usize];
//     for value in table.iter_mut() {
//         let sample = get_phase_motion(
//             table_signal.phase_motion, 
//             table_signal.freq, 
//             table_signal.amp, 
//             table_signal.phase_offset, 
//             table_signal.sr, 
//             &table_signal.mode
//         );
//         *value = sample.unwrap();
//         table_signal.update_pmotion(1.0);
//     };
//     (table, table_length)
// }

pub(crate) fn build_signal_no_table(signal_params: &mut SignalParams, duration: f32) -> Result<Vec<f32>, SignalError> {
    let n_samples = (duration * signal_params.sr) as usize;
    let mut sig: Vec<f32> = vec![0.0; n_samples];
    for value in sig.iter_mut() {
        *value = match signal_params.mode {
            SignalMode::Phasor | SignalMode::Pulse(_) => { 
                let sample = get_phase_motion(
                    signal_params.phase_motion, 
                    signal_params.freq, 
                    signal_params.amp, 
                    signal_params.phase_offset, 
                    signal_params.sr, 
                    &signal_params.mode
                );
                signal_params.update_pmotion(1.0);
                sample.unwrap()
            },
            _ => {
               return Err(SignalError::SignalModeNotAllowed)
            }
        }
    }
    Ok(sig)
}


// ------

pub(crate) fn update_and_reset_increment(pmotion: &mut f32, value: f32, table_length: f32) {
    *pmotion += value;
    *pmotion %= table_length;
}

pub(crate) fn update_increment(pmotion: &mut f32, value: f32) {
    *pmotion += value;
}

pub(crate) fn interp_buffer_write(interp_buffer: &mut Vec<f32>, interp: Interp, sample: f32) {
    match interp {
        Interp::NoInterp => {
            if interp_buffer.is_empty() { 
                interp_buffer.push(sample) 
            } else { 
                interp_buffer[0] = sample 
            }
        },
        Interp::Linear | Interp::Cosine => {
            if interp_buffer.len() >= 2 { interp_buffer.remove(0); }
            interp_buffer.push(sample)
        },
        Interp::Cubic | Interp::Hermite => {
            if interp_buffer.len() >= 4 { interp_buffer.remove(0); }
            interp_buffer.push(sample)
        }
    }
}

// ----

/// Write audio file to file
    /// 
    /// # Args
    /// -----
    /// 
    /// `file_name`: output file name  
    /// `audio_object`: audio file as `AudioObject`    
    /// 
    /// # Result
    /// -------
    /// 
    /// ` Result<(), BufferError>`
    /// 
    pub(crate) fn write_to_file(file_name: &str, vector_signal: &[f32], n_channels: usize, sr: f32) -> Result<(), ToFileError> {
        if vector_signal.is_empty() { return Err(ToFileError::SignalIsEmpty) }
        let mut name: String = file_name.split(".").collect::<Vec<&str>>().join("").to_string();
        name.push_str(".wav");

        if Path::new(&name).exists() {
            println!("[INFO] File {} exists, removing and rewriting...", &name); 
            fs::remove_file(&name).unwrap() 
        }

        let mut com = Command::new("ffmpeg")
            .arg("-f")
            .arg("f32le")
            .arg("-c:a")
            .arg("pcm_f32le")
            .arg("-ac")
            .arg(n_channels.to_string())
            .arg("-ar")
            .arg(sr.to_string())
            .arg("-i")
            .arg("pipe:0")
            .arg(&name)
            .stdin(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        
        if let Some(stdin) = com.stdin.as_mut() {
            for sample in vector_signal.iter() {
                stdin.write_all(&sample.to_le_bytes()).unwrap();
            }
        }

        let status = com.wait().unwrap();
        if !status.success() { return Err(ToFileError::WritingError) }
        println!("[INFO] File {} saved!", &name);
        Ok(())
    }