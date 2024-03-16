#![allow(dead_code)]

use std::thread::JoinHandle;
use std::default::Default;
use portaudio as pa;

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
    pub outdevice: Option<pa::DeviceIndex>,
    pub inchannels: u32,
    pub indevice: Option<pa::DeviceIndex>,
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