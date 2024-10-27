#![allow(unused)]

use std::collections::HashMap;
use crate::qinterp::SignalInterp;
use crate::qubx_common::{ Channels, ChannelError };
use crate::qoperations::split_into_nchannels;
use crate::qubx_common::SignalOperation;

const TWOPI: f32 = 2.0 * std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub enum SignalError
{
    SignalModeNotAllowed,
    SignalModeNotAllowedInProceduralOscillator
}

#[derive(Debug, Clone)]
pub struct WaveTable
{
    table: Vec<f32>,
    table_length: f32
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

/// Signal Component
/// 
/// `freqs`: frequency values in Hz for each component  
/// `amps`: amplitude values for each component  
/// `phase_offsets`: start phases value in range [0, 1] for each components.
/// 
#[derive(Debug, Clone)]
pub struct ComplexSignalParams 
{
    pub freqs: Vec<f32>,
    pub amps: Vec<f32>,
    pub phase_offsets: Vec<f32>,
    pub sr: f32,
    phase_motion: f32,
    mode: SignalMode
}

impl ComplexSignalParams 
{
    /// Oscillator Bank
    /// 
    /// `freqs`: frequency values in Hz for each component  
    /// `amps`: amplitude values for each component  
    /// `phase_offsets`: start phases value in range [0, 1] for each components. if None sets all to 0.0  
    /// 
    pub fn new(freqs: Vec<f32>, amps: Vec<f32>, phase_offsets: Option<Vec<f32>>, sr: f32) -> Self {
        let n = freqs.len();
        let start_phases = match phase_offsets {
            Some(phases) => phases,
            None => vec![0.0; n],
        };
        Self { freqs, amps, phase_offsets: start_phases, phase_motion: 0.0, sr, mode: SignalMode::ComplexSignal }
    }
}

impl SignalOperation for ComplexSignalParams
{
    fn proc_oscillator(&mut self) -> f32 {
        let n = self.freqs.len();
        let sample = self.freqs
            .iter()
            .zip(self.amps.iter()
                .zip(self.phase_offsets.iter()
                )
            )
            .map(|(f, (a, p))| a * ((TWOPI * f * self.phase_motion / self.sr) + p).sin())
            .sum();
        self.phase_motion += 1.0;
        sample
    }

    fn to_signal_object(&mut self, wave_table: Option<WaveTable>, duration: f32) -> SignalObject {
        let signal_length = self.sr * duration;
        let vector_signal = (0..signal_length as usize).map(|_| self.proc_oscillator()).collect::<Vec<f32>>();
        SignalObject { vector_signal, n_channels: 1 }
    }

    fn get_mode(&self) -> SignalMode {
        SignalMode::ComplexSignal
    }
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
    /// Create new signal params
    /// 
    /// 
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

impl SignalOperation for SignalParams
{
    fn proc_oscillator(&mut self) -> f32 {
        let sample = get_phase_motion(self.phase_motion, self.freq, self.amp, self.phase_offset, self.sr, &self.mode);
        self.update_pmotion(1.0);
        sample.unwrap()
    }

    fn to_signal_object(&mut self, wave_table: Option<WaveTable>, duration: f32) -> SignalObject {
        let sig = match self.mode {
            SignalMode::Phasor | SignalMode::Pulse(_) => build_signal_no_table(self, duration).unwrap(),
            _ => {
                match wave_table {
                    Some(table) => build_signal(&table, self, duration),
                    None => { Vec::new() }
                }
            }
        };
        SignalObject { vector_signal: sig, n_channels: 1 }
    }

    fn get_mode(&self) -> SignalMode {
        self.mode
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
            interp_buffer: Vec::new()
        }
    }
}

pub struct SignalObject
{
   pub vector_signal: Vec<f32>,
   pub n_channels: usize,
}

impl Channels for SignalObject
{
    fn to_nchannels(&mut self, out_channels: usize) -> Result<(), ChannelError> {
        let prev_channels = self.n_channels;
        self.n_channels = out_channels;
        split_into_nchannels(&mut self.vector_signal, prev_channels, out_channels)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SignalMode
{
    Sine,
    Saw,
    Triangle,
    Square,
    Phasor,
    Pulse(f32),
    ComplexSignal
}

pub struct QSignal
{
    n_points: usize,
    sine: WaveTable,
    saw: WaveTable,
    triangle: WaveTable,
    square: WaveTable
}

impl QSignal
{
    /// Qsignal obj
    /// 
    /// # Args
    /// -----
    /// 
    /// `table_length`: wave table oscillator length
    /// 
    pub fn new(table_length: usize) -> Self {
        let sine = build_table(SignalMode::Sine, table_length as f32);
        let saw = build_table(SignalMode::Saw, table_length as f32);        
        let triangle = build_table(SignalMode::Triangle, table_length as f32);
        let square = build_table(SignalMode::Square, table_length as f32);
        Self { n_points: table_length, sine, saw, triangle, square }
    }

    /// Generate simple signal as a vector
    /// 
    /// # Args
    /// -----
    /// 
    /// `signal_params`: signal parameters (see `SignalParams`)  
    /// `duration`: signal duration in seconds
    /// 
    /// # Return
    /// --------
    /// 
    /// `SignalObject`
    /// 
    pub fn into_signal_object<T: SignalOperation>(&self, signal_params: &mut T, duration: f32) -> SignalObject {
        match signal_params.get_mode() {
            SignalMode::ComplexSignal | SignalMode::Phasor | SignalMode::Pulse(_) => signal_params.to_signal_object(None, duration),
            SignalMode::Sine => signal_params.to_signal_object(Some(self.sine.clone()), duration),
            SignalMode::Saw => signal_params.to_signal_object(Some(self.saw.clone()), duration),
            SignalMode::Square => signal_params.to_signal_object(Some(self.square.clone()), duration),
            SignalMode::Triangle => signal_params.to_signal_object(Some(self.triangle.clone()), duration)
        }
    }

    /// Generate procedural phase value (no table-lookup)
    /// 
    /// # Args
    /// -----
    /// 
    /// `signal_params`: signal parameters (see `SignalParams`)
    /// 
    /// # Return
    /// --------
    /// 
    /// `f32` 
    /// 
    pub fn procedural_oscillator<T: SignalOperation>(&self, signal_params: &mut T) -> f32 {
        signal_params.proc_oscillator()
    }

    /// Table-lookup oscillator
    /// 
    /// # Args
    /// -----
    /// 
    /// `signal_params`: signal parameters (`SignalParams`)
    /// 
    /// # Return 
    /// --------
    /// 
    /// Result<f32, SignalError>
    /// 
    pub fn table_lookup_oscillator(&self, signal_params: &mut SignalParams) -> Result<f32, SignalError> {
        let table: &WaveTable = match signal_params.mode {
            SignalMode::Sine => &self.sine,
            SignalMode::Saw => &self.saw,
            SignalMode::Triangle => &self.triangle,
            SignalMode::Square => &self.square,
            _ => {
                return Err(SignalError::SignalModeNotAllowedInProceduralOscillator)
            } 
        };
        let sample = get_oscillator_phase(table, signal_params);
        Ok(sample)
    }

}

impl Default for QSignal
{
    fn default() -> Self {
        let table_length: usize = 4096;
        let sine = build_table(SignalMode::Sine, table_length as f32);
        let saw = build_table(SignalMode::Saw, table_length as f32);        
        let triangle = build_table(SignalMode::Triangle, table_length as f32);
        let square = build_table(SignalMode::Square, table_length as f32);

        Self { n_points: table_length, sine, saw, triangle, square }
    }
}

// --- TOOLS ---

fn get_phase_motion(t: f32, freq: f32, amp: f32, phase_offset: f32, sr: f32, mode: &SignalMode) -> Result<f32, SignalError> {
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

fn get_oscillator_phase(wave_table: &WaveTable, signal_params: &mut SignalParams) -> f32 {
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

fn build_signal(wave_table: &WaveTable, signal_params: &mut SignalParams, duration: f32) -> Vec<f32> {
    let n_samples = (duration * signal_params.sr) as usize;
    (0..n_samples).map(|_| get_oscillator_phase(wave_table, signal_params)).collect::<Vec<f32>>()
}

fn build_signal_no_table(signal_params: &mut SignalParams, duration: f32) -> Result<Vec<f32>, SignalError> {
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

fn build_table(mode: SignalMode, table_length: f32) -> WaveTable {
    let mut table_signal = SignalParams { mode, freq: 1.0, sr: table_length, ..Default::default() };
    let mut table: Vec<f32> = vec![0.0; table_length as usize];
    for value in table.iter_mut() {
        let sample = get_phase_motion(
            table_signal.phase_motion, 
            table_signal.freq, 
            table_signal.amp, 
            table_signal.phase_offset, 
            table_signal.sr, 
            &table_signal.mode
        );
        *value = sample.unwrap();
        table_signal.update_pmotion(1.0);
    };
    WaveTable { table, table_length }
}