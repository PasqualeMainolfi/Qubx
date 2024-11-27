# QUBX: A RUST LIBRARY FOR QUEUE-BASED MULTITHREADED REAL-TIME PARALLEL AUDIO STREAMS PROCESSING AND MANAGEMENT

Qubx is a Rust library for managing and processing audio streams in parallel.  
Related paper: P. Mainolfi, Qubx: a Rust Library for Queue-Based Multithreaded
Real-Time Parallel Audio Streams Processing and Managment, Dafx24, Surrey UK, 2024.  

## Usage

First, add Qubx to dependencies (in Cargo.toml)

```code
[dependencies]

qubx = { path="path_to/qubx" }
```

Compile, typing in the shell

```shell
cargo build --release
```

Import Qubx and StreamParameters

```rust
use qubx::{ 
    Qubx, 
    StreamParameters, 
    ProcessArg, 
    MasterPatchType, 
    DuplexPatchType, 
    DspProcessArg, 
    DspPatchType, 
    DspHybridType
};

```

now you can use it.

Below an example (master out - dsp process pair)

```rust
// define the stream parameters
let stream_params = StreamParameters {
    chunk: CHUNK,
    sr: SR,
    outchannels: CHANNELS,
    ..StreamParameters::default()
};

// create qubx
let mut q = Qubx::new(true);

// start monitor active processes
q.start_monitoring_active_processe();

// create and starting master out
let mut master_out = q.create_master_streamout(String::from("M1"), stream_params);
let master_clos: MasterPatchType = Box::new(|frame| {
    frame.iter_mut().for_each(|sample| { *sample *= 0.7 }) 
});

master_out.start(ProcessArg::PatchSpace(master_clos));

// create dsp process and associate it with master out names "M1"
// deactivate parallel-data (false)
let mut dsp_process = q.create_parallel_dsp_process(String::from("M1"), false);

loop {

    // do something...
    // .
    // .
    // ...generates audio_data

    let dsp_clos = Box::new(|_audio_data| {
        let y = _audio_data.iter().map(|sample| sample * 0.7).collect();
        y
    });

    dsp_process1.start(DspProcessArgs::HybridType::<DspPatchType, DspHybridType>(audio_data1, dsp_clos));

    if !run {
        break;
    }

    let delay = rng.gen_range(0.1..2.1);
    thread::sleep(Duration::from_secs_f32(delay));

}

// terminate and close qubx
thread::sleep(Duration::from_secs(1));
q.close_qubx();
```

or use a duplex stream

```rust
// create and starting duplex stream
let mut duplex = q.create_duplex_dsp_process(stream_params);
let clos: DuplexPatchType = Box::new(|frame| frame.to_vec());
duplex.start(ProcessArg::PatchSpace(clos));

// define duration
for i in 0..(10 * SR as usize) {
    std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / SR as f32));
}
```

The complete documentation, typing in the shell

```shell
cargo doc --all
cargo doc --open
```
