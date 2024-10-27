#![allow(dead_code)]

use std::thread::JoinHandle;
use std::default::Default;
use crate::qsignals::{ SignalMode, SignalObject, WaveTable };


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
pub enum DspProcessArgs<T, U>
{
    AudioData(Vec<f32>),
    Closure(T),
    AudioDataAndClosure(Vec<f32>, U)
}

#[derive(Debug)]
pub enum ProcessArg<T>
{
    NoArgs,
    Closure(T),
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
    fn to_signal_object(&mut self, wave_table: Option<WaveTable>, duration: f32) -> SignalObject;
    fn get_mode(&self) -> SignalMode;
}