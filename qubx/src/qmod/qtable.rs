use std::collections::HashMap;
use super::{
    qenvelopes::{ EnvMode, EnvParams, QEnvelope },
    qinterp::Interp,
    qoperations::split_into_nchannels,
    qsignals::SignalMode,
    shared_tools::get_phase_motion
};


#[derive(Debug)]
pub enum TableArg<'a>
{
    WithTable((&'a mut TableParams, Interp)),
    NoTable
}

#[derive(Debug)]
pub enum TableError
{
    SignalModeNotAllowed,
    TableLeghtMustEqualToPassedShape,
    TableModeNotAllowed
}

/// Table Mode
///
/// `Signal(SignalMode)`: for signal table lookup
/// `Envelope(EnvParams)`: for envelope table. In this case times in envelope shape must be in samples
/// `EnvelopeData(&'a [f32])`: create table from vec
/// `Data((&'a [f32], usize))`: create table from audio data from vector
///
#[derive(Debug, Clone)]
pub enum TableMode
{
    Signal(SignalMode),       // SignalMode
    Envelope(EnvParams),      // EnvParams
    EnvelopeData(Vec<f32>),  // envelope from data vector
    Data((Vec<f32>, usize))  // data vector, number of channels
}

#[derive(Debug, Clone)]
pub struct TableParams
{
    pub mode: TableMode,
    pub table: Vec<f32>,
    pub table_length: f32,
    pub(crate) env_params: Option<EnvParams>
}

impl TableParams
{
    pub fn new(mode: TableMode, table: Vec<f32>, table_length: f32) -> Self {
        Self { mode, table, table_length, env_params: None }
    }
}

#[derive(Default, Debug, Clone)]
pub struct QTable
{
    table_cache: HashMap<String, TableParams>
}

impl QTable
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
    pub fn write_table(&mut self, table_id: String, mode: TableMode, table_length: usize) -> Result<(), TableError> {
        match mode {
            TableMode::Signal(sig_mode) => {
                match sig_mode {
                    SignalMode::ComplexSignal | SignalMode::Phasor | SignalMode::Pulse(_) | SignalMode::WhiteNoise => Err(TableError::SignalModeNotAllowed),
                    _ => {
                        let mut table: Vec<f32> = vec![0.0; table_length];
                        let mut phase_motion = 0.0;
                        for (i, value) in table.iter_mut().enumerate() {
                            let sample = get_phase_motion(phase_motion, &sig_mode, &mut None);
                            *value = sample.unwrap();
                            phase_motion = i as f32 / table_length as f32;
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
            },
            TableMode::EnvelopeData(ref data) => {
                let d = data.to_vec();
                let length = d.len();
                let mut t = TableParams::new(mode, d, length as f32);
                t.env_params = Some(EnvParams::new(vec![], EnvMode::Linear));
                self.table_cache.insert(table_id, t);
                Ok(())
            },
            TableMode::Data((ref data, n_channels)) => {
                let mut d = data.to_vec();
                if n_channels > 1 { split_into_nchannels(&mut d, n_channels, 1).unwrap() }
                self.table_cache.insert(table_id, TableParams::new(mode, d, table_length as f32));
                Ok(())
            }
        }
    }

    pub fn get_table(&mut self, table_id: String) -> &mut TableParams {
        self.table_cache.get_mut(table_id.as_str()).unwrap()
    }

    pub fn get_table_length(&mut self, table_id: String) -> f32 {
    	self.table_cache.get(table_id.as_str()).unwrap().table_length
    }

    pub fn read_table(&self, table_id: String, i: usize) -> f32 {
        let t = self.table_cache.get(table_id.as_str()).unwrap();
        t.table[i]
    }

}
