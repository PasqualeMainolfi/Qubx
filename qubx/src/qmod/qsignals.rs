#![allow(unused)]

use rand::{ rngs::ThreadRng, thread_rng };
use rand_distr::Uniform;
use super::{ qbuffers::ReadBufferDirection, shared_tools::interp_buffer_write_from_table };

use crate::qubx_common::{
    Channels,
    ChannelError,
    SignalOperation,
    WriteToFile,
    ToFileError,
    TimeDomainFloat
};
use super::{
    qtable::{
        TableMode,
        TableArg,
        TableParams
    },
    shared_tools::{
        get_phase_motion,
        update_and_reset_increment,
        update_increment,
        build_signal,
        build_signal_no_table,
        get_oscillator_phase,
        write_to_file
    },
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
    InterpModeNotAllowed,
    SomethingWentWrongInNoiseGeneration
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
        SignalObject { vector_signal, n_channels: 1, sr: self.sr }
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
    pub read_direction_vec: ReadBufferDirection,
    pub(crate) phase_motion: f32,
    pub(crate) interp_buffer: Vec<f32>,
    pub(crate) noise_tools: Option<(ThreadRng, Uniform<f32>)>,
    pub(crate) t: f32
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
            read_direction_vec: ReadBufferDirection::Forward,
            phase_motion: 0.0,
            interp_buffer: Vec::new(),
            noise_tools: None,
            t: 0.0
        }
    }
}

impl SignalParams
{
    /// Create new signal params
    ///
    ///
    pub fn new(mode: SignalMode, freq: f32, amp: f32, phase_offset: f32, sr: f32) -> Self {
        let noise_tools = if mode == SignalMode::WhiteNoise {
            let distr = Uniform::<f32>::new(-1.0, 1.0);
            let noise_gen = thread_rng();
            Some((noise_gen, distr))
        } else {
            None
        };

        Self {
            mode,
            freq,
            amp,
            phase_offset,
            sr,
            ..Default::default()
        }
    }

    pub(crate) fn update_and_set_pmotion(&mut self, value: f32, table_length: f32) {
        update_and_reset_increment(&mut self.phase_motion, value, table_length, self.read_direction_vec);
    }

    pub(crate) fn update_pmotion(&mut self, value: f32) {
        update_increment(&mut self.phase_motion, value);
        self.phase_motion %= 1.0 // Important!
    }

    pub(crate) fn write_interp_buffer_from_table(&mut self, interp: Interp, table: &[f32], index: usize) {
        interp_buffer_write_from_table(&mut self.interp_buffer, interp, table, index);
    }

    pub(crate) fn reset_signal_history(&mut self) {
        self.phase_motion = 0.0;
        self.interp_buffer = Vec::new();
    }

}

impl SignalOperation for SignalParams
{
    fn proc_oscillator(&mut self) -> f32 {
        let sample = get_phase_motion(self.phase_motion, &self.mode, &mut self.noise_tools);
        self.update_pmotion(self.freq / self.sr + self.phase_offset);
        sample.unwrap() * self.amp
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
        SignalObject { vector_signal: sig, n_channels: 1, sr: self.sr}
    }

    fn get_mode(&self) -> SignalMode {
        self.mode
    }

    fn get_sr(&self) -> f32 {
        self.sr
    }

}

#[derive(Debug, Clone)]
pub struct SignalObject
{
   pub vector_signal: Vec<f32>,
   pub n_channels: usize,
   pub sr: f32
}

impl Channels for SignalObject
{
    fn to_nchannels(&mut self, out_channels: usize) -> Result<(), ChannelError> {
        let prev_channels = self.n_channels;
        self.n_channels = out_channels;
        split_into_nchannels(&mut self.vector_signal, prev_channels, out_channels)
    }
}

impl TimeDomainFloat for SignalObject
{
    fn get_vector(&self) -> &Vec<f32> {
        &self.vector_signal
    }

    fn get_n_channels(&self) -> usize {
        self.n_channels
    }
}

impl<'a> WriteToFile<'a> for SignalObject
{
    fn to_file(&self, name: &'a str) -> Result<(), ToFileError> {
        write_to_file(name, &self.vector_signal, self.n_channels, self.sr)
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
    ComplexSignal,
    WhiteNoise,
    DataVec
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
                    TableMode::Signal(_) => signal_params.to_signal_object(duration, Some(table), Some(interp)),
                    TableMode::Envelope(_) | TableMode::EnvelopeData(_) | TableMode::Data(_) => return Err(SignalError::TableModeNotAllowedForSignal)
                }
            }
            TableArg::NoTable => {
                let n_samples = (duration * signal_params.get_sr()).ceil() as usize;
                let signal = (0..n_samples).map(|_| QSignal::procedural_oscillator(signal_params)).collect::<Vec<f32>>();
                SignalObject { vector_signal: signal, n_channels: 1, sr: signal_params.get_sr() }
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
            TableMode::Data(_) => get_oscillator_phase(table, signal_params, interp),
            TableMode::Envelope(_) | TableMode::EnvelopeData(_) => return Err(SignalError::TableModeNotAllowedForSignal),
        };
        Ok(sample)
    }

}
