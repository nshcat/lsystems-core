use crate::drawing::types::*;

#[derive(Clone, Copy, Debug)]
pub struct LineSegment {
	pub begin: Vector3f,
	pub end: Vector3f,
	pub color: i32,
	pub width: f64
}

#[derive(Clone, Debug)]
pub struct Polygon {
	pub vertices: Vec<Vector3f>,
	pub color: i32
}
