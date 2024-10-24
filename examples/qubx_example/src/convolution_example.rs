use qubx::qconvolution::{ QConvolution, ConvolutionMode };
use rand::{thread_rng, Rng};
use std::time::Instant;

pub fn convolution_example() {

    let mut x: Vec<f32> = vec![0.0; 44100];
    let mut h: Vec<f32> = vec![0.0; 3000];

    let mut rng = thread_rng();
    let x = x.iter_mut().map(|_| rng.gen_range(-1.0..1.0)).collect::<Vec<f32>>();
    let h = h.iter_mut().map(|_| rng.gen_range(-1.0..1.0)).collect::<Vec<f32>>();

    get_process_time(&x, &h, ConvolutionMode::InputSide, "INPSIDE");
    get_process_time(&x, &h, ConvolutionMode::OutputSide, "OUTSIDE");
    get_process_time(&x, &h, ConvolutionMode::Fft, "FFT");
    get_process_time(&x, &h, ConvolutionMode::OlaFft(4096), "OLA FFT");

}

fn get_process_time(x: &[f32], h: &[f32], conv_method: ConvolutionMode, label: &str) {
    let now = Instant::now();
    let _ = QConvolution::convolve(x, h, conv_method);
    let elapsed_time = now.elapsed().as_secs_f64();
    println!("{} elapsed time: {}", label, elapsed_time);
}