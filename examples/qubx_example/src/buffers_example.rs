use qubx::qbuffers::AudioBuffer;

pub fn buffers_example() {

    let path: &str = "/Users/pm/AcaHub/AudioSamples/cane.wav";
    let buffer = AudioBuffer::new(44100);

    let audio = buffer.to_audio_object(path).unwrap();
    println!("AUDIO: {:?}", audio.vector_signal);
}