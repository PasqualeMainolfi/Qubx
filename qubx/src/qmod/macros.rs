use num_traits::Float;
use rustfft::num_complex::Complex;


/// Midi to Frequency
/// 
/// # Args
/// -----
/// 
/// `value`: midi value
/// 
/// # Return
/// --------
/// 
/// `f32`
/// 
#[macro_export]
macro_rules! mtof {
    (&midi:expr) => {{
        let m = $midi.abs();
        440.0 * ((m - 69.0) / 12.0).powi(2)
    }};
}

/// Frequency to Midi
/// 
/// # Args
/// -----
/// 
/// `value`: frequency value in Hz
/// 
/// # Return
/// --------
/// 
/// `f32`
/// 
#[macro_export]
macro_rules! ftom {
    (&freq:expr) => {{
        let f = $freq.abs();
        (12.0 * (f / 440.0).log2() + 69.0).floor()
    }};
}

/// Amp to dB
/// 
/// # Args
/// -----
/// 
/// `value`: amp value
/// 
/// # Return
/// --------
/// 
/// `f32`
/// 
#[macro_export]
macro_rules! atodb {
    ($amp:expr) => {{ ($amp / 20.0).powi(10) }};
}

/// dB to amp
/// 
/// # Args
/// -----
/// 
/// `value`: dB value
/// 
/// # Return
/// --------
/// 
/// `f32`
///  
#[macro_export]
macro_rules! dbtoa {
    ($db:expr) => {{ ($db / 20.0).powi(10) }};
}

/// From degree to rad
/// 
/// # Args
/// -----
/// 
/// `value`: degree value
/// 
/// # Return
/// --------
/// 
/// `f32`
///  
#[macro_export]
macro_rules! degtorad {
    ($degree:expr) => {{ $degree * std::f64::consts::PI / 180.0 }};
}

/// From rad to degree
/// 
/// # Args
/// -----
/// 
/// `value`: rad value
/// 
/// # Return
/// --------
/// 
/// `f32`
///  
#[macro_export]
macro_rules! radtodeg {
    ($rad:expr) => {{ $rad * 180.0 / std::f64::consts::PI }};
}

/// Make degree in a range from [0, pi]
/// 
/// # Args
/// -----
/// 
/// `value`: degree value
/// 
/// # Return
/// --------
/// 
/// `f32`
///  
#[macro_export]
macro_rules! clamp_angle {
    ($deg:expr) => {{ if $deg > 180.0 { $deg - 360.0 } else { $deg } }};
}

#[macro_export]
macro_rules! cartopol {
    ($car:expr) => {{
        let r = ($car.x * $car.x + $car.y * $car.y).sqrt();
        let theta = $car.y.atan2($car.x);
        PolarPoint { r, theta }
    }};
}

#[macro_export]
macro_rules! poltocar {
    ($pol:expr) => {{
        let x = $pol.r * $pol.theta.cos();
        let y = $pol.r * $pol.theta.sin();
        CartesianPoint { x, y }
    }};
}

#[macro_export]
macro_rules! scale_in_range {
    ($angle:expr, $in_min:expr, $in_max:expr, $out_min:expr, $out_max:expr) => {{
        $out_min + (($angle - $in_min) / ($in_max - $in_min)) * ($out_max - $out_min)
    }};
}

#[macro_export]
macro_rules! next_power_ot_two_length {
    ($length:expr) => {{ 1 << (($length as f32).log2() + 1.0) as usize }};
}

#[macro_export]
macro_rules! meltof {
    ($mel:expr) => {{ 700.0 * (10_f32.powf($mel / 2595.0) - 1.0) }};
}

#[macro_export]
macro_rules! ftomel {
    ($freq:expr) => {{ 2595.0 * (1.0 + $freq / 700.0).log10() }};
}


// --- inline

#[inline]
pub fn rtoc<T: Float>(rvalue: T) -> Complex<T> {
    Complex { re: T::from(rvalue).unwrap(), im: T::zero() }
}

#[inline]
pub fn ctor<T: Float>(cvalue: Complex<T>) -> T {
    cvalue.re
}

#[inline]
pub fn ctomag<T: Float>(cvalue: Complex<T>) -> T {
    cvalue.norm()
}

#[inline]
pub fn ctoangle<T: Float>(cvalue: Complex<T>) -> T {
    cvalue.arg()
}

#[inline]
pub fn comp_conj<T: Float>(cvalue: Complex<T>) -> Complex<T> {
    cvalue.conj()
}