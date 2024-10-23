#![allow(unused)]

use super::qsignal_tools::{ build_signal, build_signal_no_table, build_table, get_oscillator_phase, get_phase_motion, SignalParams, WaveTable };
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
    /// CREATE NEW QSIGNAL
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

    /// GENERATE SIGNAL TO VEC
    /// 
    /// Generate signal as a vector
    /// 
    /// # Args
    /// -----
    /// 
    /// `signal_params`: signal parameters (see `SignalParams`)
    /// 
    /// # Return
    /// --------
    /// 
    /// Vec<f32>
    /// 
    pub fn signal_to_vec(&mut self, signal_params: &mut SignalParams) -> Vec<f32> {
        match signal_params.mode {
            SignalMode::Sine => build_signal(&self.sine, signal_params),
            SignalMode::Saw => build_signal(&self.saw, signal_params),
            SignalMode::Triangle => build_signal(&self.triangle, signal_params),
            SignalMode::Square => build_signal(&self.square, signal_params),
            SignalMode::Phasor => build_signal_no_table(signal_params),
            SignalMode::Pulse(_) => build_signal_no_table(signal_params)
        }
    }

    /// GENERATE PHASE VALUE
    /// Generate phase value with no table look-up
    /// 
    /// # Args
    /// -----
    /// 
    /// `phase`: 
    /// `signal_params`: signal parameters (see `SignalParams`)
    /// 
    /// # Return
    /// --------
    /// 
    /// f32 
    /// 
    pub fn procedural_oscillator(&self, signal_params: &mut SignalParams) -> f32 {
        get_phase_motion(signal_params)
    }

    /// TABLE LOOK-UP OSCILLATOR
    /// Generate phase motion using table look-up oscillator
    /// 
    /// # Args
    /// -----
    /// 
    /// `signal_params`: signal parameters (`SignalParams`)
    /// 
    pub fn table_lookup_oscillator(&self, signal_params: &mut SignalParams) -> f32 {
        let table: &WaveTable = match signal_params.mode {
            SignalMode::Sine => &self.sine,
            SignalMode::Saw => &self.saw,
            SignalMode::Triangle => &self.triangle,
            SignalMode::Square => &self.square,
            _ => {
                eprintln!("[ERROR] For Phasor and Pulse use procedural_oscillator()");
                std::process::exit(1)
            } 
        };
        get_oscillator_phase(table, signal_params)
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
