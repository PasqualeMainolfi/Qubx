#![allow(unused)]

use super::qsignal_tools::{ build_signal, build_signal_no_table, build_table, 
    get_oscillator_phase, get_phase_motion, SignalParams, WaveTable, SignalError };
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum SignalMode
{
    Sine,
    Saw,
    Triangle,
    Square,
    Phasor,
    Pulse(f32)
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
    /// `Result<Vec<f32>, SignalError>`
    /// 
    pub fn signal_to_vec(&mut self, signal_params: &mut SignalParams, duration: f32) -> Result<Vec<f32>, SignalError> {
        match signal_params.mode {
            SignalMode::Sine => Ok(build_signal(&self.sine, signal_params, duration)),
            SignalMode::Saw => Ok(build_signal(&self.saw, signal_params, duration)),
            SignalMode::Triangle => Ok(build_signal(&self.triangle, signal_params, duration)),
            SignalMode::Square => Ok(build_signal(&self.square, signal_params, duration)),
            SignalMode::Phasor => build_signal_no_table(signal_params, duration),
            SignalMode::Pulse(_) => build_signal_no_table(signal_params, duration)
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
    pub fn procedural_oscillator(&self, signal_params: &mut SignalParams) -> f32 {
        get_phase_motion(signal_params)
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
