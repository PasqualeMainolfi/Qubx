#![allow(unused)]

use core::f64;
use nalgebra::{ matrix, max, vector, ComplexField, DVector, Matrix2 };
use geo::{ ConvexHull, Coord, LineString };
use portaudio::Sample;
use statistical::median;
use super::qoperations::precision_float;
use crate::{ clamp_angle, degtorad, poltocar, scale_in_range };


static N_DECIMALS: i32 = 15;
static REF_ANGLE_RAD: f64 = degtorad!(45.0);

#[derive(Debug, Clone)]
pub enum SpaceError
{
	SpeakersMustBeAtLeastTwo,
	MatrixIsNotInversible,
	SpeakersMustBeTwoInStereoMode,
	SourceAngleMustBeInSpeakersAngleRange,
	ModeNotAllowedInStereo,
	ModeNotAllowedInVbap,
	ModeNotAllowedInDbap,
	ErrorInVbapPairChoose
}

pub trait GenericPoint 
where
	Self: Sized
{
	fn get_coord(&self) -> Coord;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CartesianPoint
{
	pub x: f64,
	pub y: f64
}

impl CartesianPoint
{
	pub fn new() -> Self {
		Self { ..Default::default() }
	}

	pub fn set_x(&mut self, x: f64) {
		self.x = x
	}

	pub fn set_y(&mut self, y: f64) {
		self.y = y
	}

	pub fn set_coord(&mut self, x: f64, y: f64) {
		self.x = x;
		self.y = y;
	}
}

impl CartesianPoint
{
	pub fn get_distance(&self, other: &Self, blur: f64) -> f64 {
		let blur_distance = if blur != 0.0 { blur.powi(2) } else { 0.0 };
		(((other.x - self.x).powi(2) + (other.y - self.y).powi(2)) + blur_distance).sqrt()
	}

	pub fn get_distance_from_multiple_points(&self, points: &[Self], blur: f64) -> Vec<f64> {
		let distances = points
			.iter()
			.map(|coord| self.get_distance(&coord.clone(), blur))
			.collect::<Vec<f64>>();
		distances
	}
}

impl GenericPoint for CartesianPoint
{
	fn get_coord(&self) -> Coord {
		Coord { x: self.x, y: self.y }
	}

}

impl Default for CartesianPoint
{
	fn default() -> Self {
		Self { x: 0.0, y: 0.0 }
	}
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolarPoint
{
	pub r: f64,
	pub theta: f64
}

impl PolarPoint
{
	pub fn new() -> Self {
		Self { ..Default::default() }
	}

	pub fn set_r(&mut self, r: f64) {
		self.r = r
	}

	pub fn set_theta(&mut self, theta: f64) {
		self.theta = precision_float(degtorad!(clamp_angle!(theta)), N_DECIMALS)
	}

	pub fn set_coord(&mut self, r: f64, theta: f64) {
		self.r = r;
		self.theta = precision_float(degtorad!(clamp_angle!(theta)), N_DECIMALS)
	}
}

impl GenericPoint for PolarPoint
{
	fn get_coord(&self) -> Coord {
		Coord { x: self.r, y: precision_float(degtorad!(clamp_angle!(self.theta)), N_DECIMALS) }
	}
}

impl Default for PolarPoint
{
	fn default() -> Self {
		Self { r: 1.0, theta: 0.0 }
	}
}

#[derive(Debug, PartialEq)]
pub enum SpaceMode
{
	Vbap,
	Dbap(f64, Option<f64>, Option<PolarPoint>), // rolloff, blur, ref_point
	StereoLinear,
	StereoCostantPower,
	StereoCompromise
}

#[derive(Debug)]
pub struct SpeakerPair
{
	pub s1_index: usize,
	pub s2_index: usize,
	pub s1_polar: PolarPoint,
	pub s2_polar: PolarPoint,
	pub s1_cartesian: CartesianPoint,
	pub s2_cartesian: CartesianPoint,
	pub inverse_matrix: Matrix2<f64>
}

/// Space Object
/// 
/// # Args
/// -----
/// 
/// `loudspeakers_pos`: speaker positions in degree
/// 
/// 
#[derive(Debug)]
pub struct SpaceObject
{
	pub n_loudspeakers: usize,
	pub speakers_polar_loc: Vec<PolarPoint>,
	pub speakers_cartesian_loc: Vec<CartesianPoint>,
	pub speaker_pairs: Option<Vec<SpeakerPair>>,
	pub geo_center: Option<CartesianPoint>,
	pub dbap_params: Option<(f64, Option<f64>, Option<PolarPoint>)>,
	pub mode: SpaceMode
}

impl SpaceObject
{
	/// Create space object
	/// 
	/// # Args
	/// -----
	/// 
	/// `loudspeakers_loc`: loudspeaker locations in degree  
	/// `mode`: spatialization mode (see `SpatialMode`)  
	/// 
	/// # Return
	/// -------
	/// 
	/// `Result<Self, SpaceError>`
	/// 
	pub fn new(loudspeakers_loc: &[f64], mode: SpaceMode) -> Result<Self, SpaceError> {
		let n_points = loudspeakers_loc.len();
		if n_points <= 1 { return Err(SpaceError::SpeakersMustBeAtLeastTwo) }

		let mut speakers_polar_loc: Vec<PolarPoint> = Vec::new();
		let mut speakers_cartesian_loc: Vec<CartesianPoint> = Vec::new();
		let mut polar_points: Vec<Coord> = Vec::new();
		let mut cartesian_points: Vec<Coord> = Vec::new();

		for deg in loudspeakers_loc.iter() {
			let mut p = PolarPoint::default();
			p.set_theta(*deg);
			let c = poltocar!(p);
			speakers_polar_loc.push(p);
			speakers_cartesian_loc.push(c);
			polar_points.push(p.get_coord());
			cartesian_points.push(c.get_coord());
		}

		let mut spairs: Option<Vec<SpeakerPair>> = None;
		let mut geometric_center: Option<CartesianPoint> = None;
		let mut dbap_params: Option<(f64, Option<f64>, Option<PolarPoint>)> = None;

		match mode {
			SpaceMode::Vbap => {
				let speaker_pairs: Vec<SpeakerPair> = if n_points == 2 {
					let s1_polar = speakers_polar_loc[0];
					let s2_polar = speakers_polar_loc[1];
					let s1_cartesian = speakers_cartesian_loc[0];
					let s2_cartesian = speakers_cartesian_loc[1];
					let m = matrix![s1_cartesian.x, s2_cartesian.x; s1_cartesian.y, s2_cartesian.y];
					let inverse_matrix = match m.try_inverse() {
						Some(inverse) => inverse,
						None => return Err(SpaceError::MatrixIsNotInversible)
					};
					let pair = SpeakerPair { 
						s1_index: 0,
						s2_index: 1,
						s1_polar,
						s2_polar,
						s1_cartesian,
						s2_cartesian,
						inverse_matrix
					};
					vec![pair]
				} else {
					let lstring = LineString::from(cartesian_points.clone());
					let chull = lstring.convex_hull();
					let hull_points = chull.exterior().0.clone();
					let indices = hull_points
						.iter()
						.filter_map(|point| cartesian_points.iter().position(|&p| p == *point))
						.collect::<Vec<usize>>();
					let mut sp = Vec::new();
					for i in 0..(indices.len()) {
						let p1 = indices[i];
						let p2 = indices[(i + 1) % indices.len()];
						if p1 != p2 {	
							let s1_polar = speakers_polar_loc[p1];
							let s2_polar = speakers_polar_loc[p2];
							let s1_cartesian = speakers_cartesian_loc[p1];
							let s2_cartesian = speakers_cartesian_loc[p2];
							let m = matrix![s1_cartesian.x, s2_cartesian.x; s1_cartesian.y, s2_cartesian.y];
							let inverse_matrix = match m.try_inverse() {
								Some(inverse) => inverse,
								None => return Err(SpaceError::MatrixIsNotInversible)
							};
							let pair = SpeakerPair { 
								s1_index: p1,
								s2_index: p2,
								s1_polar,
								s2_polar,
								s1_cartesian,
								s2_cartesian,
								inverse_matrix
							};
							sp.push(pair);
						}
					}
					sp
				};
				spairs = Some(speaker_pairs)
			},
			SpaceMode::Dbap(rolloff, blur, ref_point) => {
				let mut geo_center = CartesianPoint { x: 0.0, y: 0.0 };
				for coord in speakers_cartesian_loc.iter() {
					geo_center.x += coord.x;
					geo_center.y += coord.y;
				}
				geo_center.x /= n_points as f64;
				geo_center.y /= n_points as f64;
				geometric_center = Some(geo_center);
				dbap_params = Some((rolloff, blur, ref_point));
			},
			_ => { }
		}
	
		Ok(Self { 
			n_loudspeakers: n_points, 
			speakers_polar_loc,
			speakers_cartesian_loc,
			speaker_pairs: spairs,
			geo_center: geometric_center,
			dbap_params,
			mode
		})
	}
}

pub struct QSpace<'a>
{
	pub space_object: &'a mut SpaceObject,
	line_intersecator: LineLineIntersection
}

impl<'a> QSpace<'a>
{
	pub fn new(space_object: &'a mut SpaceObject) -> Self {
		Self { space_object, line_intersecator: LineLineIntersection::default() }
	}

	/// Stereo pan
	/// 
	/// # Args
	/// -----
	/// 
	/// `source_angle`: source angle in degree  
	/// 
	/// # Return
	/// -------
	/// 
	/// `Result<Vec<f32>, SpaceError>`
	/// 
	pub fn stereo_pan(&self, source_angle: &f64) -> Result<Vec<f32>, SpaceError> {
		if self.space_object.n_loudspeakers > 2 { return Err(SpaceError::SpeakersMustBeTwoInStereoMode) }
		let theta1 = self.space_object.speakers_polar_loc[0].theta;
		let theta2 = self.space_object.speakers_polar_loc[1].theta;
		let source_angle = degtorad!(source_angle);
		if !(theta1..=theta2).contains(&source_angle) { return Err(SpaceError::SourceAngleMustBeInSpeakersAngleRange) }
		let source_angle_scaled = scale_in_range!(source_angle, theta1, theta2, 0.0, std::f64::consts::PI / 2.0);
		let fac = 2.0 / std::f64::consts::PI;
		let g = match self.space_object.mode {
			SpaceMode::StereoLinear => {
				let s1 = ((std::f64::consts::PI / 2.0) - source_angle_scaled) * fac;
				let s2 = source_angle_scaled * fac;
				(s1, s2)
			},
			SpaceMode::StereoCostantPower => (source_angle_scaled.cos(), source_angle_scaled.sin()),
			SpaceMode::StereoCompromise => {
				let s1 = ((std::f64::consts::PI / 2.0) - source_angle_scaled) * fac;
				let s2 = source_angle_scaled * fac;
				((s1 * fac * source_angle_scaled.cos()).sqrt(), (s2 * fac * source_angle_scaled.sin()).sqrt())
			},
			SpaceMode::Vbap | SpaceMode::Dbap(_, _, _) => return Err(SpaceError::ModeNotAllowedInStereo),
		};
		Ok(vec![g.0 as f32, g.1 as f32])
	}	

	/// VBAP (using Line-Line Intersection)
	/// 
	/// Implementation from: V. Pulkki, *Virtual Sound Source Positioning Using Vector Base Amplitude Panning*.  
	/// Note: the active arc searching function (2D) has been modified, implementing a line-line intersection algortithm which allows for speeding up the computation  
	/// 
	/// # Args
	/// -----
	/// 
	/// `source`: cartesian coordinate of source position (see `CartesianPoint`)  
	/// `normalize`: if true normalize gains  
	/// 
	/// # Return
	/// -------
	/// 
	/// `Result<Vec<f32>, SpaceError>`
	/// 
	pub fn vbap(&mut self, source: &PolarPoint) -> Result<Vec<f32>, SpaceError> {
		if self.space_object.mode != SpaceMode::Vbap { return Err(SpaceError::ModeNotAllowedInVbap) }
		let source_car = poltocar!(source);
		let mut gains = vec![0.0; self.space_object.n_loudspeakers];
		let mut arc: Option<&SpeakerPair>;
		let mut single_speaker: Option<usize> = None;

		for (i, point) in self.space_object.speakers_polar_loc.iter().enumerate() {
			if source.theta == point.theta {
				single_speaker = Some(i)
			}
		}

		if let Some(one) = single_speaker {
			gains[one] = 1.0
		} else {
			self.line_intersecator.set_source_position(source_car);
			let mut pair: Option<&SpeakerPair> = None;
			if let Some(ref speaker_pairs) = self.space_object.speaker_pairs {
				for p in speaker_pairs {
					let intersection_point = self.line_intersecator.get_intersection(&p.s1_cartesian, &p.s2_cartesian);
					if intersection_point.is_some() { 
						pair = Some(p);
						break
					}
				}
			}
			if let Some(pair_check) = pair {
				let source_vector = vector![source_car.x, source_car.y];
				let mut g = pair_check.inverse_matrix * source_vector;

				if source.theta != precision_float(REF_ANGLE_RAD, N_DECIMALS) {
					let c = (g[0].powi(2) + g[1].powi(2)).sqrt();
					g[0] = c * g[0] / c;
					g[1] = c * g[1] / c;
				}

				gains[pair_check.s1_index] = g[0] as f32;
				gains[pair_check.s2_index] = g[1] as f32;
			}
		}
		Ok(gains)
	}

	/// DBAP (using Line-Line Intersection)
	/// 
	/// Implementation from:  T. Lossius, P. Baltazar, *DBAP Distance-Based Amplitude Panning* and  
	/// J. Sundstrom, *Speaker Placement Agnosticism: Improving the Distance-Based Amplitude Panning Algorithm* 
	/// 
	/// # Args
	/// -----
	/// 
	/// `source`: cartesian coordinate of source position (see `CartesianPoint`)  
	/// 
	/// # Return
	/// -------
	/// 
	/// `Result<Vec<f32>, SpaceError>`
	/// 
	pub fn dbap(&mut self, source: &PolarPoint) -> Result<Vec<f32>, SpaceError> {
		let mut rolloff = 0.0;
		let mut blur: Option<f64> = None;
		let mut ref_point: Option<PolarPoint> = None;

		if let Some(params) = self.space_object.dbap_params {
			rolloff = params.0;
			blur = params.1;
			ref_point = params.2;
		} else {
			return Err(SpaceError::ModeNotAllowedInDbap)
		}

		let source_car = poltocar!(source);
		let a = rolloff / (20.0 * (2.0).log10());
		if let Some(reference) = ref_point { 
			self.space_object.geo_center = Some(poltocar!(reference)) 
		}

		let geo_center: CartesianPoint = self.space_object.geo_center.unwrap_or_default();
		let spatial_blur = if let Some(sblur) = blur {
			sblur
		} else {
			let dis = geo_center.get_distance_from_multiple_points(&self.space_object.speakers_cartesian_loc, 0.0);
			let mut s: f64 = dis.iter().sum();
			s /= dis.len() as f64;
			s + 0.2
		};

		let eta = spatial_blur / self.space_object.n_loudspeakers as f64;
		let d = source_car.get_distance_from_multiple_points(&self.space_object.speakers_cartesian_loc, spatial_blur);

		let p = {
			let dis_from_center = geo_center
				.get_distance_from_multiple_points(&self.space_object.speakers_cartesian_loc, spatial_blur);
			let dmax = dis_from_center.iter().fold(f64::MIN, |m, &value| m.max(value));
			let drs = source_car.get_distance(&geo_center, spatial_blur);
			let mut q = dmax / drs;
			q = if q < 1.0 { q } else { 1.0 };
			q
		};

		let b = {
			let mut u = DVector::from_vec(d.clone());
			u = u.map(|value| value - u.max());
			u = {
				let u_norm = u.norm();
				if u_norm > 0.0 { u.map(|value| value / u_norm); }
				u.map(|value| value.powi(2) + eta)
			};
			let distance_median = median(&d);
			u.map(|value| (value / distance_median * ((1.0 / p) + 1.0)).powi(2) + 1.0)
		};

		let k = {
			let bd: f64 = b
				.iter()
				.zip(d.iter())
				.map(|(&v1, &v2)| v1.powi(2) / v2.powf(2.0 * a))
				.collect::<Vec<f64>>()
				.iter()
				.sum();
			p.powf(2.0 * a) / (bd).sqrt()
		};

		let v = {
			let gains: Vec<f32> = b
			.iter()
			.zip(d.iter())
			.map(|(&v1, &v2)| (k * v1) as f32 / v2.powf(a) as f32)
			.collect();

			gains
		};

		Ok(v)
	}

	/// Apply spacer
	/// 
	/// # Args
	/// -----
	/// 
	/// `target`: audio source  
	/// `source_position`: virtual source position
	/// 
	/// 
	/// # Return 
	/// -------
	/// 
	/// `Result<Vec<f32>, SpaceError>`  
	/// Note: return each sample individually. The length of each sample (or chunk) equals the number of loudspeakers  
	/// 
	pub fn spacer(&mut self, sample: f32, source_position: &PolarPoint) -> Result<Vec<f32>, SpaceError> { 
		let mut chunk: Vec<f32> = Vec::with_capacity(self.space_object.n_loudspeakers);

		let gains: Vec<f32> = match self.space_object.mode {
			SpaceMode::StereoLinear | SpaceMode::StereoCostantPower | SpaceMode::StereoCompromise => {
				if self.space_object.n_loudspeakers > 2 { return Err(SpaceError::SpeakersMustBeTwoInStereoMode) }
				self.stereo_pan(&source_position.theta).unwrap()
			},
			SpaceMode::Vbap => self.vbap(source_position).unwrap(),
			SpaceMode::Dbap(_, _, _ ) => self.dbap(source_position).unwrap()
		};
		
		for (i, value) in chunk.iter_mut().enumerate() { 
			*value = sample * gains[i];
		}

		Ok(chunk)
	
	}

}

struct LineLineIntersection
{
	pub(crate) start_point: CartesianPoint,
	pub(crate) end_point: CartesianPoint
}

impl LineLineIntersection
{
	fn new(start_point: CartesianPoint, end_point: CartesianPoint) -> Self {
		Self { start_point, end_point }
	}

	fn set_source_position(&mut self, source_pos: CartesianPoint) {
		self.end_point = source_pos
	}

	fn get_intersection(&self, p1: &CartesianPoint, p2: &CartesianPoint) -> Option<CartesianPoint> {
		let x3 = self.start_point.x;
		let y3 = self.start_point.y;
		let x4 = self.end_point.x;
		let y4 = self.end_point.y;

		let den = (p1.x - p2.x) * (y3 - y4) - (p1.y - p2.y) * (x3 - x4);
		if den == 0.0 { return None }

		let t = ((p1.x - x3) * (y3 - y4) - (p1.y - y3) * (x3 - x4)) / den;
        let u = ((p1.x - x3) * (p1.y - p2.y) - (p1.y - y3) * (p1.x - p2.x)) / den;
		
		if (0.0..=1.0).contains(&t) && (u > 0.0) {
			let x = p1.x + t * (p2.x - p1.x);
            let y = p1.y + t * (p2.y - p1.y);
			return Some(CartesianPoint { x, y })
		}

		None

	}
}

impl Default for LineLineIntersection
{
	fn default() -> Self {
		Self { start_point: CartesianPoint { x: 0.0, y: 0.0 }, end_point: CartesianPoint { x: 0.0, y: 0.0 } }
	}
}