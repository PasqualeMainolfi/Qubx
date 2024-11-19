#![allow(unused)]

use std::borrow::BorrowMut;
use super::genesis_params::GranularParams;
use crate::{ 
    qbuffers::{ AudioBuffer, AudioObject }, 
    qenvelopes::{EnvMode, EnvParams, QEnvelope}, 
    qinterp::Interp, 
    qoperations::split_into_nchannels, 
    qsignals::{ QSignal, SignalMode, SignalParams }, 
    qtable::{ QTable, TableError, TableMode, TableParams } 
};

#[derive(Debug)]
pub enum GranulationError
{
    TableModeNotAllowed,
    EmptySignalAndTable,
    EmptyData,
    GrainEventTermintated
}

pub struct GrainEvent
{
    params: SignalParams,
    duration: usize,
    time_elapsed: usize,
    is_active: bool,
    envelope_reader: QEnvelope,
    env_params: EnvParams,
    sr: f32
}

impl GrainEvent
{
    pub fn new(duration: f32, params: SignalParams, env_params: EnvParams, sr: f32) -> Self {
        Self { 
            params, 
            duration: (duration * sr) as usize, 
            time_elapsed: 0, 
            is_active: true,
            envelope_reader: QEnvelope::new(sr),
            env_params,
            sr
        }
    }

    fn get_envelope_value(&mut self, env_table: &mut TableParams, interp_mode: Interp) -> f32 {
        let d = self.duration as f32 / self.sr; 
        self.envelope_reader.advance_envelope_from_table_private(&mut self.env_params, env_table, interp_mode, d).unwrap()
    }

    pub fn get_sample(&mut self, signal_table: &mut QTable, env_table: &mut TableParams, interp_mode: Interp) -> f32 {
        let sample = QSignal::table_lookup_oscillator(&mut self.params, signal_table.get_table("waveform".to_string()), interp_mode).unwrap();
        self.time_elapsed += 1;
        if self.time_elapsed >= self.duration { 
            self.is_active = false;
        }
        sample * self.get_envelope_value(env_table, interp_mode)
    }

    pub fn is_event_active(&self) -> bool {
        self.is_active
    }
}

pub struct GranularSynthesis
{
    signal: QTable,
    mode: TableMode,
    sr: f32,
    curr_time: usize,
    curr_delay: usize,
    events: Vec<GrainEvent>,
    signal_mode: SignalMode
}

impl GranularSynthesis
{
    /// Create granular synthesis object
    /// 
    /// # Args
    /// -----
    /// 
    /// `source table`: signal as table (see `TableMode`). Only `Signal` or `Data`  
    /// 
    /// 
    /// # Return
    /// -------
    /// 
    /// `Result<Self, GranulationError>`  
    /// 
    pub fn new(source_table: TableMode, sr: f32) -> Result<Self, GranulationError> {
        match source_table {
            TableMode::Envelope(_) | TableMode::EnvelopeData(_) => return Err(GranulationError::TableModeNotAllowed),
            _ => { }
        }
        
        let mut signal_mode = SignalMode::Sine;
        let mut t = QTable::new();
        let signal = match source_table {
            TableMode::Signal(sig_mode) => {
                t.write_table("waveform".to_string(), TableMode::Signal(sig_mode), sr as usize).unwrap();
                signal_mode = sig_mode;
                t
            },
            TableMode::Data((ref data, n_channels)) => {
                t.write_table("waveform".to_string(), TableMode::Data((data.to_vec(), n_channels)), data.len());
                signal_mode = SignalMode::DataVec;
                t
            },
            _ => return Err(GranulationError::TableModeNotAllowed)
        };

        Ok(Self { 
            signal, 
            mode: source_table,
            sr,
            curr_time: 0,
            curr_delay: 0,
            events: Vec::new(),
            signal_mode
        })
    }

    /// Granulate sample by sample  
    /// 
    /// # Args
    /// _____
    /// 
    /// `params`: grain params (see `Granular Params`)  
    /// 
    /// # Return  
    /// 
    /// `f32`  
    /// 
    pub fn granulate(&mut self, params: &mut GranularParams) -> f32 {
        
        if self.curr_time >= self.curr_delay {
            let freq = params.get_frequency_value();
            let amp = params.get_amplitude_value();
            let phase_offset = match params.get_phase_value() { 
                p if p < 0.0 => 0.0,
                p if p > 1.0 => 1.0,
                p if (0.0..=1.0).contains(&p) => p,
                _ => 0.0
            };
            
            let mut sparams: SignalParams = SignalParams { 
                freq, 
                amp, 
                phase_offset, 
                ..Default::default() 
            };
            
            match self.signal_mode {
                SignalMode::DataVec => {
                    sparams.sr = 1.0;
                    sparams.mode = SignalMode::DataVec
                },
                _ => {
                    sparams.sr = self.sr;
                    sparams.mode = self.signal_mode;
                    sparams.read_direction_vec = params.get_grain_read_direction()
                }
            }
            
            let d = params.get_duration_value();
            let eparams = EnvParams::new(vec![], EnvMode::NoMode);
            self.events.push(GrainEvent::new(d, sparams, eparams, self.sr));
            self.curr_delay = (params.get_delay_value() * self.sr) as usize;
            self.curr_time = 0;

        }
        
        let mut sample = 0.0;
        if self.events.is_empty() {
            sample += 0.0
        } else {
            let mut inactive_events = Vec::new();
            for (i, event) in self.events.iter_mut().enumerate() {
                if event.is_event_active() { 
                    sample += event.get_sample(&mut self.signal, params.envelope_table, params.interp_mode);
                } else {
                    inactive_events.push(i);
                }
            }
            for inactive_event in inactive_events.iter() {
                self.events.remove(*inactive_event);
            }
        };
        self.curr_time += 1;
        sample
    }
    
}