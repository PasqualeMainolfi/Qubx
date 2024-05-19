#![allow(unused_variables, dead_code)]

use crate::qlist::QList;
use crate::qubx_common::{Process, ProcessState, StreamParameters};
use pa::PortAudio;
use portaudio as pa;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle, ThreadId};
use threadpool::ThreadPool;
use std::sync::mpsc;
use num_cpus;

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
    /// `dsp_function`: a closure that processes the summation of audio streams from all processes associated with the stream output. Take one arg
    /// `frame`: `&mut [f32]` (frame to be processed)
    ///
    /// Example:
    ///
    /// ```rust
    /// let mut master_out = q.create_master_streamout(String::from("M1"), stream_params);
    /// master_out.start(move |frame| {
    ///    for sample in frame.iter_mut() {
    ///        *sample *= 0.7;
    ///    }
    /// });
    /// ```
    ///
    /// # Return
    /// --------
    ///
    /// `JoinHandle<()>`
    ///
    pub fn start<F>(&self, mut dsp_function: F) -> JoinHandle<()>
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
        let verb2 = Arc::clone(&self.verbose);

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

                dsp_function(&mut block);

                // .

                for (i, sample) in buffer.iter_mut().enumerate() {
                    *sample = block[i];
                }

                let elapsed_time = start_time.elapsed();
                if verb1.load(Ordering::Acquire) {
                    println!("[PROCESS INFO] Thread:::[Name: MASTER OUTPUT {}]:::[ID: {:?}]:::[READ FROM QUEUES, PROCESS AND OUTPUT LATENCY: {:?}]", name1, thread::current().id(), elapsed_time);
                    let mut lat_amount = latency_amount_clone.lock().unwrap();
                    *lat_amount += elapsed_time;
                    drop(lat_amount);
                    let mut count = count_clone.lock().unwrap();
                    *count += 1.0;
                    drop(count);
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

            if verb2.load(Ordering::Acquire) {
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
            }

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
    /// `dsp_function`: a closure that processes the audio streams. Take one arg frame: `&[f32]` (frame to be processed) and must be return
    /// a `Vec<f32>` (frame to output)
    ///
    /// Example:
    /// ```rust
    /// let mut duplex = q.create_duplex_dsp_process(stream_params);
    /// duplex.start(|frame| { frame.to_vec() });
    /// ```
    ///
    /// # Return
    /// --------
    ///
    /// `JoinHandle<()>`
    ///
    pub fn start<F>(&self, mut dsp_function: F) -> JoinHandle<()>
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
        let verb2 = Arc::clone(&self.verbose);

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

                let dsp_inblock = dsp_function(&inblock);

                assert_eq!(dsp_inblock.len(), out_buffer.len(), "[ERROR] The frame returned by the closure must have the same number of channels as the out frame!");

                for (i, sample) in dsp_inblock.iter().enumerate() {
                    out_buffer[i] = *sample
                }

                let end_time = start_time.elapsed();

                if verb1.load(Ordering::Acquire) {
                    println!("[PROCESS INFO] Thread:::[Name: \"DUPLEX STREAM\"]:::[ID: {:?}]:::[READ, PROCESS AND OUTPUT LATENCY: {:?}]", thread::current().id(), end_time);
                    let mut lat_amount = latency_amount_clone.lock().unwrap();
                    *lat_amount += end_time;
                    drop(lat_amount);
                    let mut count = count_clone.lock().unwrap();
                    *count += 1.0;
                    drop(count);
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

            if verb2.load(Ordering::Acquire) {
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
            }

            println!("[INFO] Closing PortaAudio duplex stream...");
            stream.stop().unwrap();
            stream.close().unwrap();
            p.terminate().unwrap();
        })
    }
}

/// # Dsp Process
///
///
pub struct DspProcess {
    monitor_processes: Arc<Mutex<MonitorProcess>>,
    master_streamout: Arc<Mutex<MasterStreamoutProcess>>,
    verbose: Arc<AtomicBool>,
    dsp_latency_amount: Arc<Mutex<std::time::Duration>>,
    count_dsp_iterations: Arc<Mutex<f32>>,
}

impl DspProcess {
    pub fn new(
        monitor_processes: Arc<Mutex<MonitorProcess>>,
        master_streamout: Arc<Mutex<MasterStreamoutProcess>>,
        verbose: Arc<AtomicBool>,
        dsp_latency_amount: Arc<Mutex<std::time::Duration>>,
        count_dsp_iterations: Arc<Mutex<f32>>,
    ) -> Self {
        Self {
            monitor_processes,
            master_streamout,
            verbose,
            dsp_latency_amount,
            count_dsp_iterations,
        }
    }

    /// Starting dsp process
    ///
    /// # Args
    /// ------
    ///
    /// `audio_data`: audio data to be processed
    /// `dsp_function`: a closure that processes the audio streams. Take one arg frame `&mut [f32]`
    ///
    /// Example:
    /// ```rust
    /// let mut dsp_process = q.create_parallel_dsp_process(String::from("M1"));
    ///
    /// dsp_process.start(audio_data, move |frame| {
    /// for sample in frame.iter_mut() {
    ///     *sample *= 0.7;
    /// }
    /// });
    ///
    /// ```
    ///
    /// # Return
    /// --------
    ///
    /// `JoinHandle<()>`
    ///
    pub fn start<F>(&self, audio_data: Vec<f32>, dsp_function: F) -> JoinHandle<()>
    where
        F: for<'a> FnMut(&'a mut [f32]) + Send + Sync + 'static,
    {
        let pclone = Arc::clone(&self.monitor_processes);
        let mclone = Arc::clone(&self.master_streamout);
        let verbose = Arc::clone(&self.verbose);
        let dsp_lat_amount_clone = Arc::clone(&self.dsp_latency_amount);
        let count_iter = Arc::clone(&self.count_dsp_iterations);

        thread::spawn(move || {
            let start = std::time::Instant::now();

            let m = mclone.lock().unwrap();
            let ms_name = m.name.to_string();
            let chunk_size = (m.params.chunk * m.params.outchannels) as usize; // Frame length must be chunk size * nchnls out -> streamout

            drop(m);

            let mut frames: Vec<Vec<f32>> = Vec::new();

            for i in (0..audio_data.len()).step_by(chunk_size) {
                let start = i;
                let end = std::cmp::min(i + chunk_size, audio_data.len());
                let mut frame_padded = vec![0.0; chunk_size];
                let size = end - start;
                frame_padded[0..size].copy_from_slice(&audio_data[start..end]);
                frames.push(frame_padded);
            }

            let frames_size = frames.len();

            let num_core = num_cpus::get() / 2;
            let pool = ThreadPool::new(num_core);

            let frames_ptr = Arc::new(Mutex::new(frames));
            let dsp_ptr = Arc::new(Mutex::new(dsp_function));

            let (sender, receiver) = mpsc::channel();

            for i in 0..frames_size {
            	let frames_ptr_clone = Arc::clone(&frames_ptr);
             	let dsp_ptr_clone = Arc::clone(&dsp_ptr);
              	let sender_clone = sender.clone();

	            pool.execute(move || {
	            	let mut fptr = frames_ptr_clone.lock().unwrap();
					if let Some(fp) = fptr.get_mut(i) {
						let mut dsp = dsp_ptr_clone.lock().unwrap();
						dsp(fp);
					}
					sender_clone.send(()).unwrap();
	            });
            }

			drop(sender);

            for _ in 0..frames_size {
            	receiver.recv().unwrap();
            }

            let frames_ptr_to_queue = Arc::clone(&frames_ptr);
            let fq = frames_ptr_to_queue.lock().unwrap();

            let m = mclone.lock().unwrap();
            let qclone = Arc::clone(&m.qlist);
            let mut q = qclone.lock().unwrap();
            for frame in fq.iter() {
                q.put_frame(frame.clone());
            }

            drop(fq);

            q.get_next_empty_queue();

            drop(q);
            drop(m);

            let end = start.elapsed();
            let id = thread::current().id();

            if verbose.load(Ordering::Acquire) {
                let mut lat_amount = dsp_lat_amount_clone.lock().unwrap();
                *lat_amount += end;
                drop(lat_amount);

                let mut count = count_iter.lock().unwrap();
                *count += 1.0;
                drop(count);

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
