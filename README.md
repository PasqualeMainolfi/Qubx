# A RUST LIBRARY FOR REAL-TIME PARALLEL AUDIO STREAMS PROCESSING AND MANAGMENT

Qubx is a Rust library for managing and processing audio streams in parallel.  

>**Version 0.1.0**

- Creation and managment of an indefinite number of indipendent master audio output
- Creation and managment of an indefinite number of indipendent duplex stream ($in \rightarrow dsp \rightarrow out$)
- Possibility to create an indefinite number of dsp processes

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
master_out.start(move |frame| {
    for sample in frame.iter_mut() {
        *sample *= 0.7;
    }
});

// create dsp process and associate it with master out names "M1"
let mut dsp_process = q.create_parallel_dsp_process(String::from("M1"));

loop {

    // do something...
    // .
    // .
    // ...generates audio_data

    dsp_process1.start(audio_data, move |frame| {
        for sample in frame.iter_mut() {
            *sample *= 0.7;
        }
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
