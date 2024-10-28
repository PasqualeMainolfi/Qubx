#![allow(unused)]

use std::collections::HashMap;
use crate::qubx_common::{ Channels, ChannelError, SignalOperation };
use super::{ 
    qtable::{ TableError, TableMode, TableArg, TableParams }, 
    shared_tools::{ get_phase_motion, update_and_reset_increment, update_increment, interp_buffer_write, build_signal, build_signal_no_table, get_oscillator_phase },
    qoperations::split_into_nchannels,
    qinterp::Interp,
};

const TWOPI: f32 = 2.0 * std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub enum SignalError
{
    SignalModeNotAllowed,
    SignalModeNotAllowedInLookUpOscillator,
    SignalModeAndTableModeMustBeTheSame,
    TableModeNotAllowedForSignal,
    InterpModeNotAllowed
}

/// Signal Parameters
/// 
/// `mode`: type of signal (see `SignalMode`)  
/// `interp`: interpolation type (see `Interp`)  
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

    fn to_signal_object(&mut self, duration: f32, wave_table: Option<&TableParams>, interp: Option<Interp>) -> SignalObject {
        let signal_length = self.sr * duration;
        let vector_signal = (0..signal_length as usize).map(|_| self.proc_oscillator()).collect::<Vec<f32>>();
        SignalObject { vector_signal, n_channels: 1 }
    }

    fn get_mode(&self) -> SignalMode {
        SignalMode::ComplexSignal
    }

    fn get_sr(&self) -> f32 {
        self.sr
    }

}

/// Signal Parameters
/// 
/// `mode`: type of signal (see `SignalMode`)  
/// `interp`: interpolation type (see `Interp`)  
/// `freq`: frequency value in Hz  
/// `amp`: amplitude value  
/// `phase_offset`: start phase value in range [0, 1]
/// `sr`: sample rate in Hz  
/// 
#[derive(Debug, Clone)]
pub struct SignalParams
{
    pub mode: SignalMode,
    pub freq: f32,
    pub amp: f32,
    pub phase_offset: f32,
    pub sr: f32,
    pub(crate) phase_motion: f32,
    pub(crate) interp_buffer: Vec<f32>
}

impl SignalParams
{
    /// Create new signal params
    /// 
    /// 
    pub fn new(mode: SignalMode, freq: f32, amp: f32, phase_offset: f32, sr: f32) -> Self {
        Self {
            mode,
            freq,
            amp,
            phase_offset,
            sr,
            phase_motion: 0.0,
            interp_buffer: Vec::new(),
        }
    }

    pub(crate) fn update_and_set_pmotion(&mut self, value: f32, table_length: f32) {
        update_and_reset_increment(&mut self.phase_motion, value, table_length);
    }
    
    pub(crate) fn update_pmotion(&mut self, value: f32) {
        update_increment(&mut self.phase_motion, value);
    }

    pub(crate) fn write_interp_buffer(&mut self, interp: Interp, sample: f32) {
        interp_buffer_write(&mut self.interp_buffer, interp, sample);
    }
}

impl SignalOperation for SignalParams
{
    fn proc_oscillator(&mut self) -> f32 {
        let sample = get_phase_motion(self.phase_motion, self.freq, self.amp, self.phase_offset, self.sr, &self.mode);
        self.update_pmotion(1.0);
        sample.unwrap()
    }

    fn to_signal_object(&mut self, duration: f32, wave_table: Option<&TableParams>, interp: Option<Interp>) -> SignalObject {
        let sig = match self.mode {
            SignalMode::Phasor | SignalMode::Pulse(_) => build_signal_no_table(self, duration).unwrap(),
            _ => {
                match wave_table {
                    Some(table) => {
                        if let Some(interp_mode) = interp { build_signal(table, self, interp_mode, duration) } else { Vec::new() }
                    },
                    None => { Vec::new() }
                }
            }
        };
        SignalObject { vector_signal: sig, n_channels: 1 }
    }

    fn get_mode(&self) -> SignalMode {
        self.mode
    }

    fn get_sr(&self) -> f32 {
        self.sr
    }

}

impl Default for SignalParams
{
    fn default() -> Self {
        Self {
            mode: SignalMode::Sine,
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

#[derive(Debug, Clone, Copy, PartialEq)]
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

pub struct QSignal { }

impl QSignal
{

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
    /// `Result<SignalObject, SignalError>`
    /// 
    pub fn into_signal_object<T: SignalOperation>(signal_params: &mut T, duration: f32, table: TableArg) -> Result<SignalObject, SignalError> {
        let sig = match table {
            TableArg::WithTable((table, interp)) => {
                match table.mode {
                    TableMode::Signal(sig_mode) => signal_params.to_signal_object(duration, Some(table), Some(interp)),
                    TableMode::Envelope(_) => { return Err(SignalError::TableModeNotAllowedForSignal) }
                }
            }
            TableArg::NoTable => {
                let n_samples = (duration * signal_params.get_sr()).ceil() as usize;
                let signal = (0..n_samples).map(|_| QSignal::procedural_oscillator(signal_params)).collect::<Vec<f32>>();
                SignalObject { vector_signal: signal, n_channels: 1 }
            }
        };
        Ok(sig)
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
    pub fn procedural_oscillator<T: SignalOperation>(signal_params: &mut T) -> f32 {
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
    pub fn table_lookup_oscillator(signal_params: &mut SignalParams, table: &TableParams, interp: Interp) -> Result<f32, SignalError> {
        let sample = match table.mode {
            TableMode::Signal(sig_mode) => {
                if sig_mode == signal_params.mode {
                    match sig_mode {
                        SignalMode::Phasor | SignalMode::Pulse(_) | SignalMode::ComplexSignal => return Err(SignalError::SignalModeNotAllowedInLookUpOscillator),
                        _ => get_oscillator_phase(table, signal_params, interp)
                    }
                } else {
                    return Err(SignalError::SignalModeAndTableModeMustBeTheSame)
                }
            },
            TableMode::Envelope(_) => return Err(SignalError::TableModeNotAllowedForSignal)
        };
        Ok(sample)
    }

}