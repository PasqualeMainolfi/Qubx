# QUBX: A RUST LIBRARY FOR QUEUE-BASED MULTITHREADED REAL-TIME PARALLEL AUDIO STREAMS PROCESSING AND MANAGEMENT

Qubx is a Rust library for managing and processing audio streams in parallel.  
Related paper: P. Mainolfi, Qubx: a Rust Library for Queue-Based Multithreaded 
Real-Time Parallel Audio Streams Processing and Managment, Dafx24, Surrey UK, 2024.  


>**Version 0.1.0**

- Possibility to activate data parallelization under conditions of excessive computation load
- Creation and managment of an indefinite number of indipendent master audio output
- Creation and managment of an indefinite number of indipendent duplex stream ($in \rightarrow dsp \rightarrow out$)
- Possibility to create an indefinite number of dsp processes
- Possibility to use parallel-data in each dsp process

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
use qubx::{ Qubx, StreamParameters };
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
master_out.start(|frame| {
    for sample in frame.iter_mut() {
        *sample *= 0.7;
    }
});

// create dsp process and associate it with master out names "M1"
// deactivate parallel-data (false)
let mut dsp_process = q.create_parallel_dsp_process(String::from("M1"), false);

loop {

    // do something...
    // .
    // .
    // ...generates audio_data

    dsp_process1.start(audio_data1, |_audio_data| {
    	let y = _audio_data.iter().map(|sample| sample * 0.7).collect();
     	y
    });

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
duplex.start(|frame| { frame.to_vec() });

// define the duration
for i in 0..(10 * SR as usize) {
    std::thread::sleep(std::time::Duration::from_secs_f32(1.0 / SR as f32));
}
```

The complete documentation, typing in the shell

```shell
cargo doc --all
cargo doc --open
```
