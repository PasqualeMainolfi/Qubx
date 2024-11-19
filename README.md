# QUBX: A RUST LIBRARY FOR QUEUE-BASED MULTITHREADED REAL-TIME PARALLEL AUDIO STREAMS PROCESSING AND MANAGEMENT

Qubx is a Rust library for managing and processing audio streams in parallel.  
Related paper: P. Mainolfi, Qubx: a Rust Library for Queue-Based Multithreaded
Real-Time Parallel Audio Streams Processing and Managment, Dafx24, Surrey UK, 2024.  

>**Version 0.5.0**

- Bug fixes
- New! Add `qgenesis` mod featuring most relevant synthesis techniques. It currently supports fm-am-pm and granulation synthesis.
- New! Add `qanalysis`mod. This big module enables analysis in time, frequency and querency domain. In time domain it includes amplitude envelope, zero crossing rate and energy analysis; in frequency domain, fft-ifft, stft-istft and mfcc in querency domain (see doc.)
- New! Add `qwindows` mod. This module enables for generation of window function, currently supporting Rectangular, Hamming, Hanning and Blackman mode
- New! Add `qfilters` mod. This module enable for genarating filters: `Biquad`, `Butter`, `Narrow`, `OnePole`, `TwoZeroTwoPole`, `Harmonic`, `Dc` and `Zavalishin`
- Add macros for working with complex number and coordinate: `scale_in_the_range`, `next_power_of_two_length`, `meltof`, `ftomel`
- Add inline function: `ctor`, `rtoc`, `ctomag`, `ctoangle` and `comp_conj` (for complex values)

>**Version 0.3.0**

- Bug fixes
- Add `mtof`, `ftom`, `atodb`, `dbtoa` and `degtorad`, `radtodeg`, `cartopol` and `poltocar` macro in `qoperations` mod
- New! Add `qspaces` module. This module allows you to manage simple stereo pan (linear, costant power and compromise), VBAP (using line-line intersection) and DBAP technique.

>**Version 0.2.3**

- Bug fixes
- Add possibility to export `SignalObject` as audio file. Now, it is possible to pass to `AudioBuffer::write_to_file()` objects that implements `WriteToFile` trait
- New! Add `DelayBuffer` in `qbuffers` module. This object allows you to create delay lines and complex delay blocks

>**Version 0.2.2**

- Bug fixes
- Change Master, Duplex and Dsp Process Type name. See (`ProcessArgs` and `DspProessArg`)
- New! Add `qbuffers` module for audio source reading and writing

>**Version 0.2.1**

- Bug fixes
- Optimization of signals and envelope modules
- New! Add `qtable` module. This module allows you to write and read tables

>**Version 0.2.0**

- Prepare Qubx to receive modules
- New! Add `qsignals` module. This module allows you to generate raw signals (Sine, Saw, Triangle, Square, Phasor, Pulse)
- New! Add `qenvelopes` module. This module allows you to create and generate envelope shapes
- New! Add `qinterp` module. This module allows you to implement Linear, Cubic and Hermite interpolation
- New! Add `qconvolution` module. This methos allows you to use inside, outside and fft convolution
- Add `qubx_types` module
- Changed the way arguments are passed to the `.start()` function on Matser, Duples and Dsp Process. Now you can use
`ProcessArg` for Master and Duplex and `DspProcessArgs` for DspProcess

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
