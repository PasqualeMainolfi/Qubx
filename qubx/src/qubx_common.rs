#![allow(dead_code)]

use std::thread::JoinHandle;
use std::default::Default;

use crate::{ qinterp::Interp, qsignals::{ SignalMode, SignalObject }, qtable::TableParams };


pub enum QubxExceptions {
    ParamsError,
    FuncArgsError
}

impl QubxExceptions {
    pub fn get_error(error: QubxExceptions) {
        match error {
            Self::ParamsError => {
                println!("[ERROR] Streaming parameters not found!")
            },
            Self::FuncArgsError => {
                println!("[ERROR] Missing function argument!")
            }
        }
    }
}

/// Stream Parameters struct
///
/// # Args
/// ------
///
/// `chunk`: frames per buffer
/// `sr`: sample rate
/// `outchannels`: number of channels (output device)
/// `outdevice`: index of output device
/// `inchannels`: number of channels (input device)
/// `indevice`: index of input device
///

#[derive(Debug)]
pub struct StreamParameters {
    pub chunk: u32,
    pub sr: i32,
    pub outchannels: u32,
    pub outdevice: Option<u32>,
    pub inchannels: u32,
    pub indevice: Option<u32>,
}

impl Default for StreamParameters {
    fn default() -> Self {
        Self {

            chunk: 1024,
            sr: 44100,
            outchannels: 1,
            outdevice: None,
            inchannels: 1,
            indevice: None

        }
    }
}

impl Clone for StreamParameters {
    fn clone(&self) -> Self {
        Self {

            chunk: self.chunk,
            sr: self.sr,
            outchannels: self.outchannels,
            outdevice: self.outdevice,
            inchannels: self.inchannels,
            indevice: self.indevice

        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ProcessState {
    On,
    Off
}

#[derive(Debug)]
pub struct Process {
    pub handle: JoinHandle<()>,
    pub name: String,
    pub state: ProcessState
}

impl Process {
    pub fn new(handle: JoinHandle<()>, name: String, state: ProcessState) -> Self {
        Self { handle, name, state }
    }
}

#[derive(Debug)]
pub enum DspProcessArg<F1: , F2>
where
    F1: Fn() -> Vec<f32> + Send + Sync + 'static,
    F2: for<'a> Fn(&'a [f32]) -> Vec<f32> + Send + Sync + 'static,
{
    Source(Vec<f32>),
    PatchSpace(F1),
    HybridSpace(Vec<f32>, F2)
}

#[derive(Debug)]
pub enum ProcessArg<T>
{
    NoArgs,
    PatchSpace(T),
}

#[derive(Debug)]
pub enum ChannelError
{
    VectorIsEmpty,
    ChannelNumbersError
}

pub trait Channels 
{
    fn to_nchannels(&mut self, out_channels: usize) -> Result<(), ChannelError>;
}

pub trait SignalOperation
{
    fn proc_oscillator(&mut self) -> f32;
    fn to_signal_object(&mut self, duration: f32, wave_table: Option<&TableParams>, interp: Option<Interp>) -> SignalObject;
    fn get_mode(&self) -> SignalMode;
    fn get_sr(&self) -> f32;
}

#[derive(Debug)]
pub enum ToFileError
{
    WritingError,
    SignalIsEmpty
}

pub trait WriteToFile<'a>
{
    fn to_file(&self, name: &'a str) -> Result<(), ToFileError>;
}

pub trait FreqDomainToFloat
{
    type FftType;
    fn get_mag(&self) -> Self::FftType;
    fn get_angle(&self) -> Self::FftType;
    fn get_db(&self) -> Self::FftType;
}

pub trait FreqDomainToComplex
{
    type FftType;
    fn get_conj(&self) -> Self::FftType;
}

pub trait TimeDomainFloat
{
    fn get_vector(&self) -> &Vec<f32>;
    fn get_n_channels(&self) -> usize;
}

pub trait FilteredSample<T>
{
    fn filtered_sample(&mut self, sample: f32) -> T;
}