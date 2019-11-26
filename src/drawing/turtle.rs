use std::num::*;
use nalgebra::*;

use crate::drawing::*;
use crate::drawing::primitives::*;
use crate::drawing::types::*;
use crate::iteration::{DrawingModule};


struct Turtle3DMatrixCache {
	turn_left: Matrix3f,
	turn_right: Matrix3f,
	pitch_up: Matrix3f,
	pitch_down: Matrix3f,
	roll_left: Matrix3f,
	roll_right: Matrix3f,
	turn_around: Matrix3f
}

impl Turtle3DMatrixCache {
	fn new(angle: f64) -> Turtle3DMatrixCache {
		return Turtle3DMatrixCache{
			turn_left: Self::rotU(angle),
			turn_right: Self::rotU(-angle),
			pitch_down: Self::rotL(angle),
			pitch_up: Self::rotL(-angle),
			roll_left: Self::rotH(angle),
			roll_right: Self::rotH(-angle),
			turn_around: Self::rotU(std::f64::consts::PI)
		};
	}

	pub fn rotU(angle: f64) -> Matrix3f {
		return Matrix3f::new(
			angle.cos(), angle.sin(), 0.0,
			-angle.sin(), angle.cos(), 0.0,
			0.0, 0.0, 1.0 					
		);
	}

	pub fn rotL(angle: f64) -> Matrix3f {
		return Matrix3f::new(
			angle.cos(), 0.0, -angle.sin(),
			0.0, 1.0, 0.0,
			angle.sin(), 0.0, angle.cos() 					
		);
	}

	pub fn rotH(angle: f64) -> Matrix3f {
		return Matrix3f::new(
			1.0, 0.0, 0.0,
			0.0, angle.cos(), -angle.sin(),
			0.0, angle.sin(), angle.cos() 					
		);
	}
}

#[derive(Clone, Copy, Debug)]
struct Turtle3DState {
	position: Vector3f,
	heading: Vector3fU,
	left: Vector3fU,
	up: Vector3fU,
	color_index: i32,
	line_width: f64
}

impl Turtle3DState {
	fn new(start_position: Vector3f, start_angle: f64, initial_line_width: f64) -> Turtle3DState {
		let up = Vector3f::z_axis();
		let heading = Vector3fU::new_unchecked(Vector3f::new(start_angle.cos(), start_angle.sin(), 0.0));
		let left = Vector3fU::new_normalize(up.cross(&heading));

		return Turtle3DState {
			up: up,
			heading: heading,
			left: left,
			position: start_position,
			color_index: 0,
			line_width: initial_line_width
		};
	}
}

pub struct Turtle3D {
	draw_parameters: DrawingParameters,
	contracted_length: f64,
	matrix_cache: Turtle3DMatrixCache,
	line_segments: Vec<LineSegment>,
	current_state: Turtle3DState,
	state_stack: Vec<Turtle3DState>,
	current_polygon: Vec<Vector3f>,
	polygons: Vec<Polygon>,
	num_iterations: u32
}

impl Turtle3D {
	pub fn new(draw_parameters: DrawingParameters, num_iterations: u32) -> Turtle3D {
		return Turtle3D {
			draw_parameters: draw_parameters,
			line_segments: Vec::new(),
			contracted_length: draw_parameters.step.powf(num_iterations as f64),
			state_stack: Vec::new(),
			matrix_cache: Turtle3DMatrixCache::new(draw_parameters.angle_delta),
			current_polygon: Vec::new(),
			polygons: Vec::new(),
			num_iterations: num_iterations,
			current_state: Turtle3DState::new(
				Vector3f::new(draw_parameters.start_position.x as f64, draw_parameters.start_position.y as f64, 0.0),
				draw_parameters.start_angle,
				draw_parameters.initial_line_width
			)
		}	
	}

	fn is_polygon_active(&self) -> bool {
		return !self.current_polygon.is_empty();
	}

	fn submit_vertex(&mut self) {
		self.current_polygon.push(self.current_state.position.clone());
	}

	fn begin_polygon(&mut self) {
		// Nothing to do
	}

	fn end_polygon(&mut self) {
		self.polygons.push(
			Polygon {
				vertices: self.current_polygon.clone(),
				color: self.current_state.color_index		
			}
		);

		self.current_polygon.clear();
	}

	fn apply_rotation(&mut self, matrix: Matrix3f) {
		let composite = Matrix3f::from_columns(&[
			self.current_state.heading.into_inner(),
			self.current_state.left.into_inner(),
			self.current_state.up.into_inner()
		]);

		let new_composite = composite * matrix;

		self.current_state.heading = Vector3fU::new_normalize(new_composite.column(0).into());
		self.current_state.left = Vector3fU::new_normalize(new_composite.column(1).into());
		self.current_state.up = Vector3fU::new_normalize(new_composite.column(2).into());
	}

	pub fn modify_line_width(&mut self, delta: f64) {
		self.current_state.line_width = (self.current_state.line_width + delta).max(0.0);	
	}

	pub fn execute_modules(&mut self, commands: &[DrawingModule]) {
		for command in commands {
			match command {
				// Moving
				DrawingModule{operation: DrawOperation::Forward, parameter: p} => self.move_forward(p.unwrap_or(self.draw_parameters.step), true),
				DrawingModule{operation: DrawOperation::ForwardNoDraw, parameter: p} => self.move_forward(p.unwrap_or(self.draw_parameters.step), false),
				DrawingModule{operation: DrawOperation::ForwardContracting, parameter: p} => {
					let distance = if(p.is_some()) {
						p.unwrap().powf(self.num_iterations as f64)
					} else {
						self.contracted_length
					};

					self.move_forward(distance, true);
				},

				// State handling
				DrawingModule{operation: DrawOperation::SaveState, ..} => self.push_state(),
				DrawingModule{operation: DrawOperation::LoadState, ..} => self.pop_state(),

				// Direction changes
				DrawingModule{operation: DrawOperation::TurnLeft, parameter: p} =>  self.apply_rotation(self.rotU(p, self.matrix_cache.turn_left)),
				DrawingModule{operation: DrawOperation::TurnRight, parameter: p} =>  self.apply_rotation(self.rotUInv(p, self.matrix_cache.turn_right)),
				DrawingModule{operation: DrawOperation::PitchDown, parameter: p} =>  self.apply_rotation(self.rotL(p, self.matrix_cache.pitch_down)),
				DrawingModule{operation: DrawOperation::PitchUp, parameter: p} =>  self.apply_rotation(self.rotLInv(p, self.matrix_cache.pitch_up)),
				DrawingModule{operation: DrawOperation::RollLeft, parameter: p} =>  self.apply_rotation(self.rotH(p, self.matrix_cache.roll_left)),
				DrawingModule{operation: DrawOperation::RollRight, parameter: p} =>  self.apply_rotation(self.rotHInv(p, self.matrix_cache.roll_right)),
				DrawingModule{operation: DrawOperation::TurnAround, parameter: p} =>  self.apply_rotation(self.matrix_cache.turn_around),
				
				// Polygon handling
				DrawingModule{operation: DrawOperation::BeginPolygon, ..} => self.begin_polygon(),
				DrawingModule{operation: DrawOperation::EndPolygon, ..} => self.end_polygon(),
				DrawingModule{operation: DrawOperation::SubmitVertex, ..} => self.submit_vertex(),

				// Color handling
				DrawingModule{operation: DrawOperation::IncrementColor, ..} => self.modify_color_index(1),
				DrawingModule{operation: DrawOperation::DecrementColor, ..} => self.modify_color_index(-1),

				// Line width handling
				DrawingModule{operation: DrawOperation::IncrementLineWidth, parameter: p} => self.modify_line_width(p.unwrap_or(self.draw_parameters.line_width_delta)),
				DrawingModule{operation: DrawOperation::DecrementLineWidth, parameter: p} => self.modify_line_width(-p.unwrap_or(self.draw_parameters.line_width_delta)),

				_ => ()
			}
		}
	}

	pub fn execute_operations(&mut self, commands: &[DrawOperation]) {
		for command in commands {
			match command {
				DrawOperation::Forward => self.move_forward(self.draw_parameters.step, true),
				DrawOperation::ForwardNoDraw => self.move_forward(self.draw_parameters.step, false),
				DrawOperation::ForwardContracting => self.move_forward(self.contracted_length, true),
				DrawOperation::Ignore => (),
				DrawOperation::SaveState => self.push_state(),
				DrawOperation::LoadState => self.pop_state(),
			
				DrawOperation::TurnLeft => self.apply_rotation(self.matrix_cache.turn_left),
				DrawOperation::TurnRight => self.apply_rotation(self.matrix_cache.turn_right),
				DrawOperation::PitchDown => self.apply_rotation(self.matrix_cache.pitch_down),
				DrawOperation::PitchUp => self.apply_rotation(self.matrix_cache.pitch_up),
				DrawOperation::RollLeft => self.apply_rotation(self.matrix_cache.roll_left),
				DrawOperation::RollRight => self.apply_rotation(self.matrix_cache.roll_right),
				DrawOperation::TurnAround => self.apply_rotation(self.matrix_cache.turn_around),

				DrawOperation::BeginPolygon => self.begin_polygon(),
				DrawOperation::EndPolygon => self.end_polygon(),
				DrawOperation::SubmitVertex => self.submit_vertex(),
				
				DrawOperation::IncrementColor => self.modify_color_index(1),
				DrawOperation::DecrementColor => self.modify_color_index(-1),

				DrawOperation::IncrementLineWidth => self.modify_line_width(self.draw_parameters.line_width_delta),
				DrawOperation::DecrementLineWidth => self.modify_line_width(-self.draw_parameters.line_width_delta)
			}
		}
	}

	fn rotU(&self, param: &Option<f64>, def: Matrix3f) -> Matrix3f {
		self.matrix_helper(Turtle3DMatrixCache::rotU, param, def, 1.0)
	}

	fn rotH(&self, param: &Option<f64>, def: Matrix3f) -> Matrix3f {
		self.matrix_helper(Turtle3DMatrixCache::rotH, param, def, 1.0)
	}

	fn rotL(&self, param: &Option<f64>, def: Matrix3f) -> Matrix3f {
		self.matrix_helper(Turtle3DMatrixCache::rotL, param, def, 1.0)
	}

	fn rotUInv(&self, param: &Option<f64>, def: Matrix3f) -> Matrix3f {
		self.matrix_helper(Turtle3DMatrixCache::rotU, param, def, -1.0)
	}

	fn rotHInv(&self, param: &Option<f64>, def: Matrix3f) -> Matrix3f {
		self.matrix_helper(Turtle3DMatrixCache::rotH, param, def, -1.0)
	}

	fn rotLInv(&self, param: &Option<f64>, def: Matrix3f) -> Matrix3f {
		self.matrix_helper(Turtle3DMatrixCache::rotL, param, def, -1.0)
	}

	fn matrix_helper(&self, mat: fn(f64) -> Matrix3f, param: &Option<f64>, def: Matrix3f, md: f64) -> Matrix3f {
		if(param.is_some()) {
			panic!("");
			mat(param.unwrap() * md)
		} else {
			def
		}
	}

	fn modify_color_index(&mut self, value: i32) {
		self.current_state.color_index = clamp(self.current_state.color_index + value, 0, (self.draw_parameters.color_palette_size - 1) as i32);
	}

	pub fn line_segments(& self) -> &Vec<LineSegment> {
		return &self.line_segments;	
	}

	pub fn polygons(& self) -> &Vec<Polygon> {
		return &self.polygons;	
	}

	pub fn move_forward(&mut self, distance: f64, draw: bool) {
		let old_position = self.current_state.position;
		
		let mv = self.current_state.heading.into_inner() * distance;

		self.current_state.position = old_position + mv;

		if(draw) {
			let begin = Vector3f::new(old_position.x, old_position.y, old_position.z);
			let end = Vector3f::new(self.current_state.position.x, self.current_state.position.y, self.current_state.position.z);

			self.line_segments.push(LineSegment{
				begin: begin,
				end: end,
				color: self.current_state.color_index,
				width: self.current_state.line_width
			});		
		}
	}

	pub fn push_state(&mut self) {
		self.state_stack.push(self.current_state.clone())
	}

	pub fn pop_state(&mut self) {
		if(self.state_stack.len() > 0) {
			self.current_state = self.state_stack.last().unwrap().clone();
			self.state_stack.pop();
		}
	}
}
