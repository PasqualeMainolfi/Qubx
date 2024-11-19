use crate::{ 
    qbuffers::ReadBufferDirection, qinterp::Interp, qmod::shared_tools::check_range_value, qsignals::{ SignalMode, SignalParams }, qtable::{ QTable, TableParams, TableMode },
};
use rand::{ rngs::ThreadRng, thread_rng };
use rand_distr::{Distribution, Uniform};


/// Fm Params
/// 
/// `carrier_freq`: carrier frequency in Hz  
/// `carrier_amp`: carrier amplitude  
/// `frequency_ratio`: carrier-modulating frequency ration expressed as fraction  
/// `modulation_index`: index of modulation  
/// 
#[derive(Debug, Clone)]
pub struct ModulationParams
{
    pub(crate) carrier_freq: f32, 
    pub(crate) carrier_amp: f32, 
    pub(crate) modulating_freq: f32, 
    pub(crate) modulation_index: f32,
    pub(crate) modulation_offset: f32, 
    pub(crate) carrier: SignalParams,
    pub(crate) modulating: SignalParams,
    pub(crate) carrier_table: QTable,
    pub(crate) modulating_table: QTable,
}

impl ModulationParams
{
    pub fn new(carried_signal_mode: SignalMode, modulating_signal_mode: SignalMode) -> Self {
        let sr = 44100;
        let carrier = SignalParams { mode: carried_signal_mode, ..Default::default() };
        let modulating = SignalParams { mode: modulating_signal_mode, ..Default::default() };
        let mut carrier_table = QTable::new();
        let mut modulating_table = QTable::new();
        carrier_table.write_table("carrier_table".to_string(), TableMode::Signal(carried_signal_mode), sr).unwrap();
        modulating_table.write_table("modulating_table".to_string(), TableMode::Signal(modulating_signal_mode), sr).unwrap();

        Self { 
            carrier_freq: 0.0,
            carrier_amp: 0.0,
            modulating_freq: 1.0,
            modulation_index: 0.0,
            modulation_offset: 0.0,
            carrier,
            modulating,
            carrier_table,
            modulating_table,
        }
    }

    pub fn set_carrier_freq(&mut self, freq: f32) {
        self.carrier_freq = freq
    }
    
    pub fn set_carrier_amp(&mut self, amp: f32) {
        self.carrier_amp = amp
    }

    pub fn set_modulating_freq(&mut self, freq: f32) { 
        self.modulating_freq = freq
    }

    pub fn set_modulation_index(&mut self, m: f32) {
        self.modulation_index = m
    }

    pub fn set_modulation_offset(&mut self, offset: f32) {
        self.modulation_offset = offset
    }

    pub fn set_table_length(&mut self, length: usize) {
        self.carrier_table.write_table("carrier_table".to_string(), TableMode::Signal(self.carrier.mode), length).unwrap();
        self.modulating_table.write_table("modulating_table".to_string(), TableMode::Signal(self.modulating.mode), length).unwrap();
    }

    pub fn get_modulation_offset(&self) -> f32 {
        self.modulation_offset
    }

    pub fn get_carrier_freq(&self) -> f32 {
        self.carrier_freq
    }
    
    pub fn get_carrier_amp(&self) -> f32 {
        self.carrier_amp
    }

    pub fn get_modulating_freq(&self) -> f32 { 
        self.modulating_freq
    }

    pub fn get_modulation_index(&self) -> f32 {
        self.modulating_freq
    }

}

pub enum ModulationMode
{
    Procedural,         // without table
    TableLookUp(Interp) // interp mode
}

pub enum ModulationType<'a>
{
    Fm(&'a mut ModulationParams),
    Am(&'a mut ModulationParams),
    Pm(&'a mut ModulationParams)
}

/// Granular synthesis params
/// 
/// `frequency_range`: frequencies range in Hz (min, max)  
/// `amplitude_range`: amplitudes range (min, max)  
/// `phase"_range`: phases range (in signal mode)  
/// `duration_range`: durations range in sec. (min, max)  
/// `delay_range`: delay range (between grains) in sec. (min, max) 
/// `overlap_range`: overlap size range (in sampled granulation, not in signal mode)  
/// `grain_read_direction`: samples reading direction (see `ReadBufferDirection`)  
/// `interp_mode`: interpolation mode (see `Interp`)  
/// 
pub struct GranularParams<'a>
{
    pub frequency_range: (f32, f32),
    pub amplitude_range: (f32, f32),
    pub phase_range: (f32, f32),
    pub duration_range: (f32, f32),
    pub delay_range: (f32, f32),
    pub overlap_range: (f32, f32),
    pub grain_read_direction: ReadBufferDirection,
    pub interp_mode: Interp,
    pub(crate) audio_amp: f32,
    pub(crate) envelope_table: &'a mut TableParams,
    rnd_generator: ThreadRng,
}

impl<'a> GranularParams<'a>
{
    /// Create new Granular params object  
    /// 
    /// # Args  
    /// -----  
    /// `sr`: sample rate  
    /// `interp_mode`: table interpolation mode (see `Interp`)  
    /// `grain_envelope`: grain envelope shape as `QTable`  
    /// `table_id`: id of table to read  
    /// 
    pub fn new(interp_mode: Interp, grain_envelope: &'a mut TableParams) -> Self {
        let rnd_generator = thread_rng();
        Self { 
            frequency_range: (90.0, 500.0), 
            amplitude_range: (0.1, 0.7), 
            phase_range: (0.0, 0.0), 
            duration_range: (0.001, 0.01), 
            delay_range: (0.001, 0.1),
            overlap_range: (0.002, 0.02),
            grain_read_direction: ReadBufferDirection::Forward,
            interp_mode,
            audio_amp: 0.0, 
            envelope_table: grain_envelope,
            rnd_generator,
        }
    }

    pub fn set_frequency_range(&mut self, values_range: (f32, f32)) {
       self.frequency_range = values_range
    }
    
    pub fn set_amplitude_range(&mut self, values_range: (f32, f32)) {
        self.amplitude_range = values_range;
        self.audio_amp = self.get_amplitude_value()
    }
    
    pub fn set_phase_range(&mut self, values_range: (f32, f32)) {
        self.phase_range = values_range
    }
    
    pub fn set_duration_range(&mut self, values_range: (f32, f32)) {
        self.duration_range = values_range;
    }
   
    pub fn set_delay_range(&mut self, values_range: (f32, f32)) {
        self.delay_range = values_range
    }
    
    pub fn set_overlap_range(&mut self, values_range: (f32, f32)) {
        self.overlap_range = values_range
    }
    
    pub fn set_grain_reading_direction(&mut self, direction: ReadBufferDirection) {
        self.grain_read_direction = direction
    }
    
    pub(crate) fn get_frequency_value(&mut self) -> f32 {
        if check_range_value(self.frequency_range) { 
            self.frequency_range.0 
        } else { 
            let freq = Uniform::<f32>::new(self.frequency_range.0, self.frequency_range.1);
            freq.sample(&mut self.rnd_generator)
        }
    }
    
    pub(crate) fn get_amplitude_value(&mut self) -> f32 {
        if check_range_value(self.amplitude_range) { 
            self.amplitude_range.0 
        } else { 
            let amp = Uniform::<f32>::new(self.amplitude_range.0, self.amplitude_range.1);
            amp.sample(&mut self.rnd_generator)
        }
    }
    
    pub(crate) fn get_phase_value(&mut self) -> f32 {
        if check_range_value(self.phase_range) { 
            self.phase_range.0 
        } else { 
        let phase = Uniform::<f32>::new(self.phase_range.0, self.phase_range.1);
        phase.sample(&mut self.rnd_generator)
        }
    }
    
    pub(crate) fn get_duration_value(&mut self) -> f32 {
        if check_range_value(self.duration_range) { 
            self.duration_range.0 
        } else { 
            let duration = Uniform::<f32>::new(self.duration_range.0, self.duration_range.1);
            duration.sample(&mut self.rnd_generator)
        }
    }
   
    pub(crate) fn get_delay_value(&mut self) -> f32 {
        if check_range_value(self.delay_range) { 
            self.delay_range.0 
        } else { 
            let delay = Uniform::<f32>::new(self.delay_range.0, self.delay_range.1);
            delay.sample(&mut self.rnd_generator)
        }
    }
    
    pub(crate) fn get_grain_read_direction(&self) -> ReadBufferDirection {
        self.grain_read_direction
    }

}




