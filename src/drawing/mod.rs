pub mod turtle;
pub mod primitives;
pub mod types;

#[cfg(feature = "serde")]
use serde::*;
#[cfg(feature = "serde")]
#[macro_use]
use serde_derive::*;

use crate::drawing::types::*;

#[repr(u8)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawOperation {
	Forward = 0,
	ForwardNoDraw = 1,
	TurnRight = 2,
	TurnLeft = 3,
	SaveState = 4,
	LoadState = 5,
	Ignore = 6,
	/// Uses a step size of s^n, where n is the iteration depth
	ForwardContracting = 7,	

	// 3D operations
	PitchDown = 8,
	PitchUp = 9,
	RollLeft = 10,
	RollRight = 11,
	TurnAround = 12,

	BeginPolygon = 13,
	EndPolygon = 14,
	SubmitVertex = 15,

	IncrementColor = 16,
	DecrementColor = 17,
	IncrementLineWidth = 18,
	DecrementLineWidth = 19
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DrawingParameters {
	pub start_position: Vector2f,
	pub start_angle: f64,
	pub angle_delta: f64,
	pub step: f64,
	pub color_palette_size: u32,
	pub initial_line_width: f64,
	pub line_width_delta: f64
}

impl DrawingParameters {
	pub fn new() -> DrawingParameters {
		return DrawingParameters{
			start_position: Vector2f::new(0.0, 0.0),
			start_angle: 0.0,
			angle_delta: 45.0,
			step: 1.0,
			color_palette_size: 1,
			initial_line_width: 1.0,
			line_width_delta: 0.1
		}	
	}
}
