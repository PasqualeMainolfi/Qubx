use super::genesis::{
    genesis_params::{ ModulationMode, ModulationType, GranularParams },
    fap_modulation::Fap,
    granulation::GranularSynthesis
};
use super::qtable::TableMode;

pub struct QModulation
{
    model: Fap
}

impl QModulation
{
    pub fn new(sr: f32) -> Self {
        let model = Fap::new(sr);
        Self { model }
    }

    /// Generate modulated sample (FM, AM-RM or PM)
    /// 
    /// # Args
    /// -----
    /// 
    /// `modulation_type`: type of modulation (see `ModulationType`)  
    /// `mode`: generation sample mode (see `ModulationMode`)  
    /// 
    /// # Return
    /// -------
    /// 
    /// f32  
    /// 
    pub fn process(&mut self, simple_modulation_type: &mut ModulationType, mode: ModulationMode) -> f32 {
        match mode {
            ModulationMode::Procedural => {
                match simple_modulation_type {
                    ModulationType::Fm(fm_params) => self.model.procedural_fm_modulated_sample(fm_params),
                    ModulationType::Am(am_params) => self.model.procedural_am_modulated_sample(am_params),
                    ModulationType::Pm(pm_params) => self.model.procedural_pm_modulated_sample(pm_params)
                }
            },
            ModulationMode::TableLookUp(interp_mode) => {
                match simple_modulation_type {
                    ModulationType::Fm(fm_params) => self.model.table_lookup_fm_sample(fm_params, interp_mode),
                    ModulationType::Am(am_params) => self.model.table_lookup_am_sample(am_params, interp_mode),
                    ModulationType::Pm(pm_params) => self.model.table_lookup_pm_sample(pm_params, interp_mode)
                }
            }
        }
    }
}

pub struct QGranulator
{
    granulator: GranularSynthesis
}

impl QGranulator {
    pub fn new(source_table: TableMode, sr: f32) -> Self {
        Self { granulator: GranularSynthesis::new(source_table, sr).unwrap() }
    }

    pub fn process(&mut self, params: &mut GranularParams) -> f32 {
        self.granulator.granulate(params)
    }
}