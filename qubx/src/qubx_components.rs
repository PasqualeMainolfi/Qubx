#![allow(unused_variables, dead_code)]

use crate::qlist::QList;
use crate::qubx_common::{ DspProcessArgs, Process, ProcessArg, ProcessState, StreamParameters };
use pa::PortAudio;
use portaudio as pa;

use std::collections::HashMap;
use std::sync::atomic::{ AtomicBool, Ordering };
use std::sync::{ Arc, Mutex };
use std::thread::{ self, JoinHandle, ThreadId };
use rayon::prelude::*;

fn get_chunks(audio_data: Vec<f32>, chunk_size: usize) -> Vec<Vec<f32>> {
    let frames :Vec<Vec<f32>> = audio_data
    .chunks(chunk_size)
    .map(|chunk| {
        let mut frame_padded = vec![0.0; chunk_size];
        frame_padded[0..chunk.len()].copy_from_slice(chunk);
        frame_padded
    })
    .collect();
    frames
}

/// # Master Stream-out
///
///
#[derive(Debug)]
pub struct MasterStreamoutProcess {
    pub name: String,
    pub params: Arc<StreamParameters>,
    pub qlist: Arc<Mutex<QList>>,
    pub verbose: Arc<AtomicBool>,
    pub run: Arc<AtomicBool>,
}

impl MasterStreamoutProcess {
    pub fn new(
        name: String,
        params: StreamParameters,
        run: Arc<AtomicBool>,
        verbose: bool,
    ) -> Self {
        let qlist = QList::default();

        Self {
            name,
            qlist: Arc::new(Mutex::new(qlist)),
            params: Arc::new(params),
            verbose: Arc::new(AtomicBool::new(verbose)),
            run,
        }
    }

    /// Starting master streamout process
    ///
    /// # Args
    /// ------
    ///
    /// `arg`: can be `ProcessArg::NoArgs` (means take no argumets) or `ProcessArg::Closure::<MasterClosureType>(closure)`.
    /// Closure that processes the summation of audio streams from all processes associated with the stream output. Take one arg
    /// `frame`: `&mut [f32]` (frame to be processed)
    ///
    /// Example:
    ///
    /// ```rust
    /// let mut master_out = q.create_master_streamout(String::from("M1"), stream_params);
    /// let master_clos: MasterClosureType = Box::new(|frame| {
    ///    frame.iter_mut().for_each(|sample| { *sample *= 0.7 }) 
    /// });
    /// master_out.start(ProcessArg::Closure::<MasterClosureType>(master_clos));
    /// ```
    ///
    /// # Return
    /// --------
    ///
    /// `JoinHandle<()>`
    ///
    pub fn start<F>(&self, mut arg: ProcessArg<F>) -> JoinHandle<()>
    where
        F: for<'a> FnMut(&'a mut [f32]) + Send + Sync + 'static,
    {
        let qlist_clone = Arc::clone(&self.qlist);
        let params_clone = Arc::clone(&self.params);
        let sr = params_clone.sr as f64;

        let p = PortAudio::new().unwrap();

        let device = match params_clone.outdevice {
            Some(devout) => pa::DeviceIndex(devout),
            None => p.default_output_device().unwrap(),
        };

        let channels = params_clone.outchannels;
        let chunk = params_clone.chunk;
        let run = Arc::clone(&self.run);

        let name1 = self.name.clone();
        let name2 = self.name.clone();
        let verb1 = Arc::clone(&self.verbose);

        let count_latency_thread: Arc<Mutex<f32>> = Arc::new(Mutex::new(0.0));
        let count_clone = Arc::clone(&count_latency_thread);

        let latency_amount = Arc::new(Mutex::new(std::time::Duration::new(0, 0)));
        let latency_amount_clone = Arc::clone(&latency_amount);

        thread::spawn(move || {
            let callback = move |pa::OutputStreamCallbackArgs { buffer, .. }| {
                let mut q = qlist_clone.lock().unwrap();
                let mut block = vec![0.0; buffer.len()];

                let start_time = std::time::Instant::now();

                for i in 0..q.length as usize {
                    if !q.is_empty_at_index(i) {
                        let frame = q.get_frame(i);
                        for (b, f) in block.iter_mut().zip(frame.iter()) {
                            *b += f;
                        }
                    }
                }

                drop(q);

                // APPLY DSP TO MULTICHANNEL AUDIO OUT -> ON BUFFER VECTOR OR PASS DSP FUNCTION
                // .

                match arg {
                    ProcessArg::NoArgs => { },
                    ProcessArg::Closure(ref mut dsp_function) => dsp_function(&mut block),
                };
                
                // .

                for (i, sample) in buffer.iter_mut().enumerate() {
                    *sample = block[i];
                }

                let elapsed_time = start_time.elapsed();
                let mut lat_amount = latency_amount_clone.lock().unwrap();
                *lat_amount += elapsed_time;
                drop(lat_amount);
                let mut count = count_clone.lock().unwrap();
                *count += 1.0;
                drop(count);

                if verb1.load(Ordering::Acquire) {
                    println!("[PROCESS INFO] Thread:::[Name: MASTER OUTPUT {}]:::[ID: {:?}]:::[READ FROM QUEUES, PROCESS AND OUTPUT LATENCY: {:?}]", name1, thread::current().id(), elapsed_time);
                }

                pa::Continue
            };

            let device_info = p.device_info(device).unwrap();
            let latency = device_info.default_low_output_latency;

            let output_params =
                pa::StreamParameters::<f32>::new(device, channels as i32, true, latency);
            let output_settings = pa::OutputStreamSettings::new(output_params, sr, chunk);
            let mut stream = p
                .open_non_blocking_stream(output_settings, callback)
                .unwrap();

            stream.start().unwrap();

            while run.load(Ordering::Acquire) {
                continue;
            }

            let lat_amount_sec = latency_amount.lock().unwrap();
            let count = count_latency_thread.lock().unwrap();
            let fac = if *count > 0.0 { *count } else { 1.0 };
            let lat_amount = lat_amount_sec.as_secs_f32() / fac;
            print!(
                "\n[PROCESSES INFO]\n:::Process Name: Master streamout {}\n:::Process Id: {:?}\n:::Output device latency: {:?}\n:::Number of iterations: {}\n:::Latency average: {:?}\n\n",
                name2,
                thread::current().id(),
                std::time::Duration::from_secs_f32(latency as f32),
                *count as i32,
                std::time::Duration::from_secs_f32(lat_amount)
            );

            println!("[INFO] Closing stream-out PortAudio...");
            stream.stop().unwrap();
            stream.close().unwrap();
            p.terminate().unwrap();
        })
    }
}

/// # Dsp Duplex Stream
///
///
#[derive(Debug)]
pub struct DuplexProcess {
    params: Arc<StreamParameters>,
    verbose: Arc<AtomicBool>,
    run: Arc<AtomicBool>,
}

impl DuplexProcess {
    pub fn new(params: StreamParameters, run: Arc<AtomicBool>, verbose: bool) -> Self {
        Self {
            params: Arc::new(params),
            verbose: Arc::new(AtomicBool::new(verbose)),
            run,
        }
    }

    /// Starting duplex dsp stream
    ///
    /// # Args
    /// ------
    ///
    /// `arg`: can be `ProcessArg::NoArgs` (means take no argumets) or `ProcessArg::Closure::<DuplexClosureType>(closure)`.  
    /// Closure that processes the audio streams. Take one arg frame: `&[f32]` (frame to be processed) and must be return
    /// a `Vec<f32>` (frame to output)
    ///
    /// Example:
    /// ```rust
    /// let mut duplex = q.create_duplex_dsp_process(stream_params);
    /// let clos: DuplexClosureType = Box::new(|frame| frame.to_vec());
    /// duplex.start(ProcessArg::Closure::<DuplexClosureType>(clos));
    /// ```
    ///
    /// # Return
    /// --------
    ///
    /// `JoinHandle<()>`
    ///
    pub fn start<F>(&self, mut arg: ProcessArg<F>) -> JoinHandle<()>
    where
        F: for<'a> FnMut(&'a [f32]) -> Vec<f32> + Send + Sync + 'static,
    {
        let params_clone = Arc::clone(&self.params);
        let sr = params_clone.sr as f64;

        let p = pa::PortAudio::new().unwrap();

        let indevice = match params_clone.indevice {
            Some(dev) => pa::DeviceIndex(dev),
            None => p.default_input_device().unwrap(),
        };

        let inchannels = params_clone.inchannels;

        let outdevice = match params_clone.outdevice {
            Some(dev) => pa::DeviceIndex(dev),
            None => p.default_output_device().unwrap(),
        };

        let outchannels = params_clone.outchannels;

        let chunk = params_clone.chunk;
        let run = Arc::clone(&self.run);
        let verb1 = Arc::clone(&self.verbose);

        let count_latency_thread: Arc<Mutex<f32>> = Arc::new(Mutex::new(0.0));
        let count_clone = Arc::clone(&count_latency_thread);

        let latency_amount = Arc::new(Mutex::new(std::time::Duration::new(0, 0)));
        let latency_amount_clone = Arc::clone(&latency_amount);

        thread::spawn(move || {
            let callback = move |pa::DuplexStreamCallbackArgs {
                                     in_buffer,
                                     out_buffer,
                                     ..
                                 }| {
                let mut inblock = vec![0.0; (chunk * inchannels) as usize];

                let start_time = std::time::Instant::now();

                for (insample, outsample) in in_buffer.iter().zip(inblock.iter_mut()) {
                    *outsample = *insample;
                }

                // ATTENTION: is interleaved format! length of inblock is chunk * chnls

                let dsp_inblock = match arg {
                    ProcessArg::NoArgs => inblock,
                    ProcessArg::Closure(ref mut dsp_function) => dsp_function(&inblock)
                };
                
                assert_eq!(dsp_inblock.len(), out_buffer.len(), "[ERROR] The frame returned by the closure must have the same number of channels as the out frame!");

                for (i, sample) in dsp_inblock.iter().enumerate() {
                    out_buffer[i] = *sample
                }

                let end_time = start_time.elapsed();

                let mut lat_amount = latency_amount_clone.lock().unwrap();
                *lat_amount += end_time;
                drop(lat_amount);
                let mut count = count_clone.lock().unwrap();
                *count += 1.0;
                drop(count);

                if verb1.load(Ordering::Acquire) {
                    println!("[PROCESS INFO] Thread:::[Name: \"DUPLEX STREAM\"]:::[ID: {:?}]:::[READ, PROCESS AND OUTPUT LATENCY: {:?}]", thread::current().id(), end_time);
                }


                pa::Continue
            };

            let devin_info = p.device_info(indevice).unwrap();
            let inlatency = devin_info.default_low_input_latency;

            let inparams =
                pa::StreamParameters::<f32>::new(indevice, inchannels as i32, true, inlatency);

            let devout_info = p.device_info(outdevice).unwrap();
            let outlatency = devout_info.default_low_output_latency;

            let outparams =
                pa::StreamParameters::<f32>::new(outdevice, outchannels as i32, true, outlatency);

            let stream_settings = pa::DuplexStreamSettings::new(inparams, outparams, sr, chunk);

            let mut stream = p
                .open_non_blocking_stream(stream_settings, callback)
                .unwrap();

            stream.start().unwrap();

            while run.load(Ordering::Acquire) {
                continue;
            }

            let lat_amount_sec = latency_amount.lock().unwrap();
            let count = count_latency_thread.lock().unwrap();
            let fac = if *count > 0.0 { *count } else { 1.0 };
            let lat_amount = lat_amount_sec.as_secs_f32() / fac;
            print!(
                "\n[PROCESSES INFO]\n:::Process Name: Duplex Stream\n:::Process Id: {:?}\n:::Input device latency: {:?}\n:::Output device latency: {:?}\n:::Number of iterations: {}\n:::Latency average: {:?}\n\n",
                thread::current().id(),
                std::time::Duration::from_secs_f32(inlatency as f32),
                std::time::Duration::from_secs_f32(outlatency as f32),
                *count as i32,
                std::time::Duration::from_secs_f32(lat_amount)
            );

            println!("[INFO] Closing PortaAudio duplex stream...");
            stream.stop().unwrap();
            stream.close().unwrap();
            p.terminate().unwrap();
        })
    }
}

/// # Dsp Process (TODO: implement frame size for elab != chunk)
///
///
pub struct DspProcess {
    monitor_processes: Arc<Mutex<MonitorProcess>>,
    master_streamout: Arc<Mutex<MasterStreamoutProcess>>,
    verbose: Arc<AtomicBool>,
    dsp_latency_amount: Arc<Mutex<std::time::Duration>>,
    count_dsp_iterations: Arc<Mutex<f32>>,
    use_parallel_computation: bool
}

impl DspProcess {
    pub fn new(
        monitor_processes: Arc<Mutex<MonitorProcess>>,
        master_streamout: Arc<Mutex<MasterStreamoutProcess>>,
        verbose: Arc<AtomicBool>,
        dsp_latency_amount: Arc<Mutex<std::time::Duration>>,
        count_dsp_iterations: Arc<Mutex<f32>>,
        use_parallel: bool
    ) -> Self {
        Self {
            monitor_processes,
            master_streamout,
            verbose,
            dsp_latency_amount,
            count_dsp_iterations,
            use_parallel_computation: use_parallel
        }
    }

    /// Starting dsp process
    ///
    /// # Args
    /// ------
    ///
    /// `args`: can be DspProcessArgs::AudioData (require audio vector only `Vec<f32>`), 
    /// DspProcessArgs::Closure:::<DspClosureNoArgsType, DspClosureWithArgsType> (pass a closure thare take no arguments and return `Vec<f32>` as audio data) or 
    /// DspProcessArgs::AudioAndClosure::<DspClosureNoArgsType, DspClosureWithArgsType> (pass audio data as `Vec<f32>` and closure. 
    /// Closure take one argument `&[f32]` and return a `Vec<f32>`).
    ///
    /// Example:
    /// ```rust
    /// let dsp_clos: DspClosureWithArgsType = Box::new(|_audio_data| {
    /// let y = _audio_data.iter().map(|sample| sample * 0.7).collect();
    /// y
    /// });
    ///
    /// dsp_process1.start(DspProcessArgs::AudioDataAndClosure::<DspClosureNoArgsType, DspClosureWithArgsType>(audio_data1, dsp_clos));
    /// dsp_process2.start(DspProcessArgs::AudioData::<DspClosureNoArgsType, DspClosureWithArgsType>(audio_data2));
    ///
    /// ```
    ///
    /// # Return
    /// --------
    ///
    /// `JoinHandle<()>`
    ///
    pub fn start<F1, F2>(&self, args: DspProcessArgs<F1, F2>) -> JoinHandle<()>
    where
        F1: Fn() -> Vec<f32> + Send + Sync + 'static,
        F2: for<'a> Fn(&'a [f32]) -> Vec<f32> + Send + Sync + 'static,
    {
        let pclone = Arc::clone(&self.monitor_processes);
        let mclone = Arc::clone(&self.master_streamout);
        let verbose = Arc::clone(&self.verbose);
        let dsp_lat_amount_clone = Arc::clone(&self.dsp_latency_amount);
        let count_iter = Arc::clone(&self.count_dsp_iterations);

        // let dsp_ptr = Arc::new(Mutex::new(dsp_function));
        // let dsp_ptr_clone = Arc::clone(&dsp_ptr);

        let use_par_ptr = Arc::new(self.use_parallel_computation);

        let params = self.master_streamout.lock().unwrap();
        let chunk_size = (params.params.chunk * params.params.outchannels) as usize; // Frame length must be chunk size * nchnls out -> streamout
        let ms_name = params.name.to_string();
        drop(params);

        thread::spawn(move || {
            let start = std::time::Instant::now();

            let frames: Vec<Vec<f32>> = match args {

                DspProcessArgs::AudioData(audio_data) => {
                    get_chunks(audio_data, chunk_size)
                },

                DspProcessArgs::Closure(dsp_function) => {
                    let audio_data = dsp_function(); 
                    get_chunks(audio_data, chunk_size)
                },

                DspProcessArgs::AudioDataAndClosure(audio_data, dsp_function) => {
                    let mut f: Vec<Vec<f32>> = get_chunks(audio_data, chunk_size);
                    if *use_par_ptr {
                        f = f.par_iter().map(|frame| dsp_function(frame)).collect();
                    } else {
                        f = f.iter().map(|frame| dsp_function(frame)).collect();
                    }

                    f
                }
            };

            let m = mclone.lock().unwrap();
            let qclone = Arc::clone(&m.qlist);
            let mut q = qclone.lock().unwrap();
            for f in frames.iter() {
            	q.put_frame(f.clone());
            }

            q.get_next_empty_queue();

            drop(q);
            drop(m);

            let end = start.elapsed();
            let id = thread::current().id();

            let mut lat_amount = dsp_lat_amount_clone.lock().unwrap();
            *lat_amount += end;
            drop(lat_amount);

            let mut count = count_iter.lock().unwrap();
            *count += 1.0;
            drop(count);

            if verbose.load(Ordering::Acquire) {
                println!(
                    "[PROCESS INFO] Thread:::[Name: \"DSP\" >>> Master streamout {}]:::[ID: {:?}]:::[PROCESS AND WRITE TO QUEUE LATENCY: {:?}]",
                    ms_name,
                    thread::current().id(),
                    end
                )
            }

            let mut pm = pclone.lock().unwrap();
            if let Some(p) = pm.processes.get_mut(&id) {
                p.state = ProcessState::Off
            };
            drop(pm);
        })
    }
}

/// # Monitoring active processes
#[derive(Debug)]
pub struct MonitorProcess {
    pub processes: HashMap<ThreadId, Process>,
    verbose: Arc<AtomicBool>,
}

impl MonitorProcess {
    pub fn new(verbose: bool) -> Self {
        let processes: HashMap<ThreadId, Process> = HashMap::new();
        Self {
            processes,
            verbose: Arc::new(AtomicBool::new(verbose)),
        }
    }

    pub fn add_process(&mut self, process: Process) {
        if Arc::clone(&self.verbose).load(Ordering::Acquire) {
            println!(
                "[PROCESS INFO] Thread:::[Name: {:?}]:::[ID: {:?}]:::[State: {:?}]",
                process.name,
                process.handle.thread().id(),
                process.state
            )
        }

        self.processes.insert(process.handle.thread().id(), process);
    }

    pub fn remove_inactive_processes(&mut self) {
        let mut inactive_processes: Vec<ThreadId> = Vec::new();
        for (id, process) in self.processes.iter() {
            if process.state == ProcessState::Off {
                inactive_processes.push(*id);
            }
        }

        for id in inactive_processes {
            if let Some(p) = self.processes.remove(&id) {
                if Arc::clone(&self.verbose).load(Ordering::Acquire) {
                    println!(
                        "[PROCESS INFO] Thread:::[Name: {:?}]:::[ID: {:?}]:::[State: {:?}]",
                        p.name,
                        p.handle.thread().id(),
                        p.state
                    )
                }

                p.handle.join().unwrap()
            }
        }
    }

    pub fn join_and_remove_all(&mut self) {
        let mut to_remove: Vec<ThreadId> = Vec::new();
        for (id, process) in self.processes.iter_mut() {
            process.state = ProcessState::Off;
            to_remove.push(*id);
        }

        for key in to_remove {
            if let Some(process) = self.processes.remove(&key) {
                if Arc::clone(&self.verbose).load(Ordering::Acquire) {
                    println!(
                        "[PROCESS INFO] Thread:::[Name: {:?}]:::[ID: {:?}]:::[State: {:?}]",
                        process.name,
                        process.handle.thread().id(),
                        process.state
                    )
                }

                process.handle.join().unwrap();
            }
        }
        self.processes.clear();
    }
}
