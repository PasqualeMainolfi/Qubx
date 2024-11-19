use crate::{
    qinterp::Interp, qmod::shared_tools::get_phase_motion, qsignals::QSignal
};

use super::genesis_params::ModulationParams;

#[derive(Debug)]
pub struct Fap 
{ 
    sr: f32
}

impl Fap
{
    pub fn new(sr: f32) -> Self {
        Self { sr }
    }

    pub fn procedural_fm_modulated_sample(&self, fm_params: &mut ModulationParams) -> f32 {
        let m = get_phase_motion(fm_params.modulating.phase_motion, &fm_params.modulating.mode, &mut None).unwrap();
        let c = get_phase_motion(fm_params.carrier.phase_motion, &fm_params.carrier.mode, &mut None).unwrap();
        fm_params.modulating.phase_motion = (fm_params.modulating.phase_motion + (fm_params.modulating_freq / self.sr)) % 1.0;
        fm_params.carrier.phase_motion = (fm_params.carrier.phase_motion + ((fm_params.carrier_freq + (m * fm_params.modulation_index)) / self.sr)) % 1.0;
        c * fm_params.carrier_amp
    }

    pub fn table_lookup_fm_sample(&mut self, fm_params: &mut ModulationParams, interp: Interp) -> f32 {
        let carrier_table = fm_params.carrier_table.get_table("carrier_table".to_string());
        let modulating_table = fm_params.modulating_table.get_table("modulating_table".to_string());
        fm_params.modulating.freq = fm_params.modulating_freq;
        fm_params.modulating.amp = fm_params.modulation_index;
        let modulating_sample = QSignal::table_lookup_oscillator(&mut fm_params.modulating, modulating_table, interp);
        fm_params.carrier.freq = fm_params.carrier_freq + modulating_sample.unwrap();
        fm_params.carrier.amp = fm_params.carrier_amp; 
        let modulated_sample = QSignal::table_lookup_oscillator(&mut fm_params.carrier, carrier_table, interp);
        modulated_sample.unwrap_or(0.0)
    }

    pub fn procedural_am_modulated_sample(&mut self, am_params: &mut ModulationParams) -> f32 {
        let m = get_phase_motion(am_params.modulating.phase_motion, &am_params.modulating.mode, &mut None).unwrap();
        let c = get_phase_motion(am_params.carrier.phase_motion, &am_params.carrier.mode, &mut None).unwrap();
        am_params.modulating.phase_motion = (am_params.modulating.phase_motion + am_params.modulating_freq / self.sr) % 1.0;
        am_params.carrier.phase_motion = (am_params.carrier.phase_motion + am_params.carrier_freq / self.sr) % 1.0;
        am_params.carrier_amp * (am_params.modulation_offset + (m * am_params.modulation_index)) * c
    }

    pub fn table_lookup_am_sample(&mut self, am_params: &mut ModulationParams, interp: Interp) -> f32 {
        let carrier_table = am_params.carrier_table.get_table("carrier_table".to_string());
        let modulating_table = am_params.modulating_table.get_table("modulating_table".to_string());
        am_params.modulating.freq = am_params.modulating_freq;
        am_params.modulating.amp = am_params.modulation_index;
        am_params.carrier.freq = am_params.carrier_freq;
        am_params.carrier.amp = 1.0;
        let modulating_sample = QSignal::table_lookup_oscillator(&mut am_params.modulating, modulating_table, interp).unwrap_or(0.0);
        let carrier_sample = QSignal::table_lookup_oscillator(&mut am_params.carrier, carrier_table, interp).unwrap_or(0.0);
        am_params.carrier_amp * (am_params.modulation_offset + modulating_sample) * carrier_sample
    }
    
    pub fn procedural_pm_modulated_sample(&mut self, pm_params: &mut ModulationParams) -> f32 {
        let m = get_phase_motion(pm_params.modulating.phase_motion, &pm_params.modulating.mode, &mut None).unwrap();
        let c = get_phase_motion(pm_params.carrier.phase_motion, &pm_params.carrier.mode, &mut None).unwrap();
        let mod_index = pm_params.modulation_index / (2.0 * std::f32::consts::PI);
        pm_params.carrier.phase_offset = m * mod_index;
        pm_params.modulating.phase_motion = (pm_params.modulating.phase_motion + pm_params.modulating_freq / self.sr) % 1.0;
        pm_params.carrier.phase_motion = (pm_params.carrier.phase_motion + pm_params.carrier_freq / self.sr + pm_params.carrier.phase_offset) % 1.0;
        c * pm_params.carrier_amp
    }

    pub fn table_lookup_pm_sample(&mut self, pm_params: &mut ModulationParams, interp: Interp) -> f32 {
        let carrier_table = pm_params.carrier_table.get_table("carrier_table".to_string());
        let modulating_table = pm_params.modulating_table.get_table("modulating_table".to_string());
        pm_params.modulating.freq = pm_params.modulating_freq;
        pm_params.modulating.amp = pm_params.modulation_index / 2.0 * std::f32::consts::PI;
        pm_params.carrier.freq = pm_params.carrier_freq;
        pm_params.carrier.amp = pm_params.carrier_amp;
        let modulating_sample = QSignal::table_lookup_oscillator(&mut pm_params.modulating, modulating_table, interp).unwrap_or(0.0);
        pm_params.carrier.phase_offset = modulating_sample;
        QSignal::table_lookup_oscillator(&mut pm_params.carrier, carrier_table, interp).unwrap_or(0.0)
    }

}