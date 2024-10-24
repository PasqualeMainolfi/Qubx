#![allow(unused)]

use super::qsignals::SignalMode;
use crate::qinterp::SignalInterp;

const TWOPI: f32 = 2.0 * std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub enum SignalError
{
    SignalModeNotAllowed,
    SignalModeNotAllowedInProceduralOscillator
}

pub struct WaveTable
{
    pub table: Vec<f32>,
    pub table_length: f32
}

/// Signal Parameters
/// 
/// `mode`: type of signal (see `SignalMode`)  
/// `interp`: interpolation type (see `SignalInterp`)  
/// `freq`: frequency value in Hz  
/// `amp`: amplitude value  
/// `phase_offset`: start phase value in range [0, 1]
/// `sr`: sample rate in Hz  
/// 
pub struct SignalParams
{
    pub mode: SignalMode,
    pub interp: SignalInterp,
    pub freq: f32,
    pub amp: f32,
    pub phase_offset: f32,
    pub sr: f32,
    phase_motion: f32,
    interp_buffer: Vec<f32>
}

impl SignalParams 
{
    pub fn new(mode: SignalMode, interp: SignalInterp, freq: f32, amp: f32, phase_offset: f32, sr: f32) -> Self {
        Self {
            mode,
            interp,
            freq,
            amp,
            phase_offset,
            sr,
            phase_motion: 0.0,
            interp_buffer: Vec::new(),
        }
    }

    fn update_and_set_pmotion(&mut self, value: f32, table_length: f32) {
        self.phase_motion += value;
        self.phase_motion %= table_length;
    }
    
    fn update_pmotion(&mut self, value: f32) {
        self.phase_motion += value;
    }

    fn write_interp_buffer(&mut self, sample: f32) {
        match self.interp {
            SignalInterp::NoInterp => {
                self.interp_buffer[0] = sample
            },
            SignalInterp::Linear | SignalInterp::Cosine => {
                if self.interp_buffer.len() >= 2 { self.interp_buffer.remove(0); }
                self.interp_buffer.push(sample)
            },
            SignalInterp::Cubic | SignalInterp::Hermite => {
                if self.interp_buffer.len() >= 4 { self.interp_buffer.remove(0); }
                self.interp_buffer.push(sample)
            }
        }
    }
}

struct PhaseInterpolationIndex
{
    int_part: usize,
    frac_part: f32
}

impl PhaseInterpolationIndex
{
    fn new(index: f32) -> Self {
        let mut ip = index as i32;
        ip = if ip < 0 { 0 } else { ip };
        let frac_part = index.fract();
        Self { int_part: ip as usize, frac_part }
    }
}


impl Default for SignalParams
{
    fn default() -> Self {
        
        Self {
            mode: SignalMode::Sine,
            interp: SignalInterp::NoInterp,
            freq: 440.0,
            amp: 1.0,
            phase_offset: 0.0,
            sr: 44100.0,
            phase_motion: 0.0,
            interp_buffer: Vec::new(),
        }
    }
}

pub fn get_phase_motion(signal_params: &mut SignalParams) -> f32 {
    let phase = signal_params.freq * signal_params.phase_motion / signal_params.sr;
    let sample = match signal_params.mode {
        SignalMode::Sine => (TWOPI * phase).sin(),
        SignalMode::Saw => 1.0 - 2.0 * (phase - (phase).floor()),
        SignalMode::Triangle => (2.0 / std::f32::consts::PI) * ((TWOPI * phase).sin()).asin(),
        SignalMode::Square => ((TWOPI * phase).sin()).signum(),
        SignalMode::Phasor => phase - (phase).floor(),
        SignalMode::Pulse(duty) => if (phase - (phase).floor()) < duty { 1.0 } else { 0.0 }
    };
    // println!("MOTION: {}", signal_params.phase_motion);
    signal_params.update_pmotion(1.0);
    sample * signal_params.amp
}

pub fn get_oscillator_phase(wave_table: &WaveTable, signal_params: &mut SignalParams) -> f32 {
    let si = signal_params.freq / signal_params.sr * wave_table.table_length;
    let phase_offset = signal_params.phase_offset * wave_table.table_length;
    let phase_index = (signal_params.phase_motion + phase_offset) % wave_table.table_length;
    let table_index = PhaseInterpolationIndex::new(phase_index);
    let index_int = table_index.int_part;
    let frac_part = table_index.frac_part;
    let table = &wave_table.table;
    signal_params.write_interp_buffer(table[index_int]);
    let sample = signal_params.interp.get_table_interpolation(frac_part, &signal_params.interp_buffer).unwrap();
    signal_params.update_and_set_pmotion(si, wave_table.table_length);
    signal_params.amp * sample
}

pub fn build_signal(wave_table: &WaveTable, signal_params: &mut SignalParams, duration: f32) -> Vec<f32> {

    let n_samples = (duration * signal_params.sr) as usize;
    (0..n_samples).map(|_| get_oscillator_phase(wave_table, signal_params)).collect::<Vec<f32>>()
}

pub fn build_signal_no_table(signal_params: &mut SignalParams, duration: f32) -> Result<Vec<f32>, SignalError> {
    let n_samples = (duration * signal_params.sr) as usize;
    let mut sig: Vec<f32> = Vec::new();
    for value in sig.iter_mut() {
        *value = match signal_params.mode {
            SignalMode::Phasor => get_phase_motion(signal_params),
            SignalMode::Pulse(_) => get_phase_motion(signal_params),
            _ => {
               return Err(SignalError::SignalModeNotAllowed)
            }
        }
    }
    Ok(sig)
}

pub fn build_table(mode: SignalMode, table_length: f32) -> WaveTable {
    let mut table_signal = SignalParams { mode, freq: 1.0, sr: table_length, ..Default::default() };
    let mut table: Vec<f32> = vec![0.0; table_length as usize];
    for value in table.iter_mut() {
        let sample = get_phase_motion(&mut table_signal);
        *value = sample;
    };
    WaveTable { table, table_length }
}