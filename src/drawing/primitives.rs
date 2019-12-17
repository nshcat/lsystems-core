use crate::drawing::types::*;

#[derive(Clone, Copy, Debug)]
pub struct LineSegment {
	pub begin: Vector3f,
	pub end: Vector3f,
	pub color: i32,
	pub width: f32
}

#[derive(Clone, Debug)]
pub struct Polygon {
	pub vertices: Vec<Vector3f>,
	pub color: i32
}

/// Directions on how to spawn a specific bezier patch. Note that this library
/// does not know anything about this patch - thats the job of the displaying application.
#[derive(Clone, Debug)]
pub struct Patch {
	/// The identifier of the patch to use
	pub identifier: char,
	/// The patches model transformation matrix
	pub model_transform: Matrix4f
}