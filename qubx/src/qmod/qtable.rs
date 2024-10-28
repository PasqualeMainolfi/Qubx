use std::collections::HashMap;
use super::{ qenvelopes::{ EnvParams, QEnvelope}, qsignals::SignalMode, shared_tools::get_phase_motion, qinterp::Interp };


#[derive(Debug, Clone, Copy)]
pub enum TableArg<'a>
{
    WithTable((&'a TableParams, Interp)),
    NoTable
}

#[derive(Debug)]
pub enum TableError
{
    SignalModeNotAllowed,
    TableLeghtMustEqualToPassedShape
}

/// Table Mode
/// 
/// `Signal(SignalMode)`: for signal table lookup  
/// `Envelope(EnvParams)`: for envelope table. In this case times in envelope shape must be in samples  
/// 
#[derive(Debug, Clone)]
pub enum TableMode
{
    Signal(SignalMode),
    Envelope(EnvParams)
}

#[derive(Debug)]
pub struct TableParams
{
    pub mode: TableMode,
    pub table: Vec<f32>,
    pub table_length: f32,
}

impl TableParams
{
    pub fn new(mode: TableMode, table: Vec<f32>, table_length: f32) -> Self {
        Self { mode, table, table_length }
    }

}


#[derive(Default, Debug)]
pub struct QTable<'a>
{
    table_cache: HashMap<&'a str, TableParams>
}

impl<'a> QTable<'a>
{
    pub fn new() -> Self {
        Self { table_cache: HashMap::new() }
    }
    /// Build table 
    /// 
    /// # Args
    /// -----
    /// `table_id`: table id  
    /// `mode`: table mode (see `TableMode`). In TableMode::Envelope(...) envelope times in shape must be in samples
    /// `table_length`: table length  
    /// 
    /// 
    /// # Return
    /// 
    /// `Result<(), TableError>`
    /// 
    /// 
    pub fn write_table(&mut self, table_id: &'a str, mode: TableMode, table_length: usize) -> Result<(), TableError> {
        match mode {
            TableMode::Signal(sig_mode) => {
                match sig_mode {
                    SignalMode::ComplexSignal | SignalMode::Phasor | SignalMode::Pulse(_) => Err(TableError::SignalModeNotAllowed),
                    _ => {
                        let mut table: Vec<f32> = vec![0.0; table_length];
                        let mut phase_motion = 0.0;
                        for value in table.iter_mut() {
                            let sample = get_phase_motion(
                                phase_motion, 
                                1.0, 
                                1.0, 
                                0.0, 
                                table_length as f32, 
                                &sig_mode
                            );
                            *value = sample.unwrap();
                            phase_motion += 1.0;
                        };
                        self.table_cache.insert(table_id, TableParams::new(mode, table, table_length as f32));
                        Ok(())
                    }
                }
            },
            TableMode::Envelope(ref env_params) => {
                let check: f32 = env_params.shape.iter().enumerate().filter(|(i, _)| i % 2 != 0).map(|(_, &x)| x).sum();
                if (check as usize) != table_length { return Err(TableError::TableLeghtMustEqualToPassedShape) }
                let mut e = QEnvelope::new(1.0);
                let table = e.into_envelope_object(env_params);
                self.table_cache.insert(table_id, TableParams::new(mode, table.vector_envelope, table_length as f32));
                Ok(())
            }
        }
    }

    pub fn get_table(&mut self, table_id: &'a str) -> &TableParams {
        self.table_cache.get(table_id).unwrap()
    }

    pub fn read_table(&self, table_id: &'a str, i: usize) -> f32 {
        let t = self.table_cache.get(table_id).unwrap();
        t.table[i]
    }

}