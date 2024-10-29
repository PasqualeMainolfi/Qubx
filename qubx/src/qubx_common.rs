#![allow(dead_code)]

use std::thread::JoinHandle;
use std::default::Default;
use crate::{qinterp::Interp, qsignals::{ SignalMode, SignalObject }, qtable::TableParams};


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