# Changelog

## [0.5.0] - 19-11-2024

- New! Add `qgenesis` mod featuring most relevant synthesis techniques. It currently supports fm-am-pm and granulation synthesis.
- New! Add `qanalysis`mod. This big module enables analysis in time, frequency and querency domain. In time domain it includes amplitude envelope, zero crossing rate and energy analysis; in frequency domain, fft-ifft, stft-istft and mfcc in querency domain (see doc.)
- New! Add `qwindows` mod. This module enables for generation of window function, currently supporting Rectangular, Hamming, Hanning and Blackman mode
- New! Add `qfilters` mod. This module enable for genarating filters: `Biquad`, `Butter`, `Narrow`, `OnePole`, `TwoZeroTwoPole`, `Harmonic`, `Dc` and `Zavalishin`
- Add macros for working with complex number and coordinate: `scale_in_the_range`, `next_power_of_two_length`, `meltof`, `ftomel`
- Add inline function: `ctor`, `rtoc`, `ctomag`, `ctoangle` and `comp_conj` (for complex values)

- Bug fixed

## [0.3.0] - 05-11-2024

- Add `mtof`, `ftom`, `atodb`, `dbtoa` and `degtorad`, `radtodeg`, `cartopol` and `poltocar` macro in `qoperations` mod
- New! Add `qspaces` module. This module allows you to manage simple stereo pan (linear, costant power and compromise), VBAP (using line-line intersection) and DBAP technique.

- Bug fixed

## [0.2.3] - 30-10-2024

- Add possibility to export `SignalObject` as audio file. Now, it is possible to pass to `AudioBuffer::write_to_file()` objects that implements `WriteToFile` trait
- New! Add `DelayBuffer` in `qbuffers` module. This object allows you to create delay lines and complex delay blocks

- Bug fixes

## [0.2.2] - 29-10-2024

- Change Master, Duplex and Dsp Process Type name. See (`ProcessArgs` and `DspProessArg`)
- New! Add `qbuffers` module for audio source reading and writing

- Bug fixes

## [0.2.1] - 28-10-2024

- Optimization of signals and envelope modules
- New! Add `qtable` module. This module allows you to write and read tables

- Bug fixes

## [0.2.0] - 23-10-2024

- Prepare Qubx to receive modules
- New! Add `qsignals` module. This module allows you to generate raw signals (Sine, Saw, Triangle, Square, Phasor, Pulse)
- New! Add `qenvelopes` module. This module allows you to create and generate envelope shapes
- New! Add `qinterp` module. This module allows you to implement Linear, Cubic and Hermite interpolation
- New! Add `qconvolution` module. This methos allows you to use inside, outside and fft convolution
- Add `qubx_types` module
- Changed the way arguments are passed to the `.start()` function on Matser, Duples and Dsp Process. Now you can use
`ProcessArg` for Master and Duplex and `DspProcessArgs` for DspProcess

## [0.1.0] - 23-05-2025

- Possibility to activate data parallelization under conditions of excessive computation load
- Creation and managment of an indefinite number of indipendent master audio output
- Creation and managment of an indefinite number of indipendent duplex stream ($in \rightarrow dsp \rightarrow out$)
- Possibility to create an indefinite number of dsp processes
- Possibility to use parallel-data in each dsp process
