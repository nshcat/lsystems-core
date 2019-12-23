use nalgebra::*;

use crate::drawing::*;
use crate::drawing::primitives::*;
use crate::drawing::types::*;
use crate::interpretation::DrawingCommand;


/// A structure containing all the primitves that were created from a 
/// sequence of drawing commands during a drawing run.
#[derive(Clone)]
pub struct DrawingResult {
	/// Polygons directly created by commands such as SubmitVertex.
	pub polygons: Vec<Polygon>,
	/// Line segments created by moving the turtle.
	pub line_segments: Vec<LineSegment>,
	/// Patch rendering instructions
	pub patches: Vec<Patch>
}

impl DrawingResult {
	pub fn new() -> DrawingResult {
		DrawingResult {
			polygons: Vec::new(),
			line_segments: Vec::new(),
			patches: Vec::new()
		}
	}
}

struct Turtle3DMatrixCache {
	turn_left: Matrix3d,
	turn_right: Matrix3d,
	pitch_up: Matrix3d,
	pitch_down: Matrix3d,
	roll_left: Matrix3d,
	roll_right: Matrix3d,
	turn_around: Matrix3d
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

	pub fn rotU(angle: f64) -> Matrix3d {
		return Matrix3d::new(
			angle.cos(), angle.sin(), 0.0,
			-angle.sin(), angle.cos(), 0.0,
			0.0, 0.0, 1.0 					
		);
	}

	pub fn rotL(angle: f64) -> Matrix3d {
		return Matrix3d::new(
			angle.cos(), 0.0, -angle.sin(),
			0.0, 1.0, 0.0,
			angle.sin(), 0.0, angle.cos() 					
		);
	}

	pub fn rotH(angle: f64) -> Matrix3d {
		return Matrix3d::new(
			1.0, 0.0, 0.0,
			0.0, angle.cos(), -angle.sin(),
			0.0, angle.sin(), angle.cos() 					
		);
	}
}

#[derive(Clone, Copy, Debug)]
struct Turtle3DState {
	position: Vector3d,
	heading: Vector3dU,
	left: Vector3dU,
	up: Vector3dU,
	color_index: i32,
	line_width: f64
}

impl Turtle3DState {
	fn new(start_position: Vector3d, start_angle: f64, initial_line_width: f64) -> Turtle3DState {
		let up = Vector3d::z_axis();
		let heading = Vector3dU::new_unchecked(Vector3d::new(start_angle.cos(), start_angle.sin(), 0.0));
		let left = Vector3dU::new_normalize(up.cross(&heading));

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
	current_state: Turtle3DState,
	state_stack: Vec<Turtle3DState>,
	current_polygon: Vec<Vector3f>,
	drawing_result: DrawingResult,
	num_iterations: u32
}

impl Turtle3D {
	pub fn new(draw_parameters: DrawingParameters, num_iterations: u32) -> Turtle3D {
		return Turtle3D {
			draw_parameters: draw_parameters,
			contracted_length: draw_parameters.step.powf(num_iterations as f64),
			state_stack: Vec::new(),
			matrix_cache: Turtle3DMatrixCache::new(draw_parameters.angle_delta),
			current_polygon: Vec::new(),
			drawing_result: DrawingResult::new(),
			num_iterations: num_iterations,
			current_state: Turtle3DState::new(
				Vector3d::new(draw_parameters.start_position.x as f64, draw_parameters.start_position.y as f64, 0.0),
				draw_parameters.start_angle,
				draw_parameters.initial_line_width
			)
		}	
	}

	fn is_polygon_active(&self) -> bool {
		return !self.current_polygon.is_empty();
	}

	fn submit_vertex(&mut self) {
		self.current_polygon.push(Self::convert_vector(&self.current_state.position));
	}

	fn convert_vector(vec: &Vector3d) -> Vector3f {
		Vector3f::new(vec.x as _, vec.y as _, vec.z as _)
	}

	fn begin_polygon(&mut self) {
		// Nothing to do
	}

	/// Determine bezier patch orientation and save to result
	fn create_patch(&mut self, identifier: char, scale: f64) {
		// The model transformation matrix contains both a base change matrix as well as
		// a translation component.
		// The goal is to achieve the following transformations: 
		// 	- translate the local origin to the turtle position
		//	- rotate local x-axis onto the turtles direction
		//  - rotate local y-axis onto the turtles right direction (-left)
		//  - rotate local z-axis onto the turtles up direction
		//
		let H = Self::convert_vector(&self.current_state.heading.into_inner());
		let U = Self::convert_vector(&self.current_state.up.into_inner());
		let L = Self::convert_vector(&self.current_state.left.into_inner());
		let p = Self::convert_vector(&self.current_state.position);

		let model_matrix = Matrix4f::new(
			H.x, -L.x, U.x, p.x,
			H.y, -L.y, U.y, p.y,
			H.z, -L.z, U.z, p.z,
			0.0,  0.0, 0.0, 1.0
		);

		let scaling = Matrix4f::new_scaling(scale as _);

		self.drawing_result.patches.push(
			Patch {
				model_transform: model_matrix * scaling,
				identifier: identifier
			}
		)
	}

	pub fn retrieve_result(&self) -> &DrawingResult {
		&self.drawing_result
	}

	fn end_polygon(&mut self) {
		self.drawing_result.polygons.push(
			Polygon {
				vertices: self.current_polygon.clone(),
				color: self.current_state.color_index		
			}
		);

		self.current_polygon.clear();
	}

	fn apply_rotation(&mut self, matrix: Matrix3d) {
		let composite = Matrix3d::from_columns(&[
			self.current_state.heading.into_inner(),
			self.current_state.left.into_inner(),
			self.current_state.up.into_inner()
		]);

		let new_composite = composite * matrix;

		self.current_state.heading = Vector3dU::new_normalize(new_composite.column(0).into());
		self.current_state.left = Vector3dU::new_normalize(new_composite.column(1).into());
		self.current_state.up = Vector3dU::new_normalize(new_composite.column(2).into());
	}

	pub fn modify_line_width(&mut self, delta: f64) {
		self.current_state.line_width = (self.current_state.line_width + delta).max(0.0);	
	}

	pub fn execute_modules(&mut self, commands: &[DrawingCommand]) {
		for command in commands {
			match command {
				// Patch creation
				DrawingCommand::SpawnPatch{patch_id, scaling} => self.create_patch(*patch_id, *scaling),

				// Moving
				DrawingCommand::BasicCommand{operation: TurtleCommand::Forward, parameter: p} => self.move_forward(p.unwrap_or(self.draw_parameters.step), true),
				DrawingCommand::BasicCommand{operation: TurtleCommand::ForwardNoDraw, parameter: p} => self.move_forward(p.unwrap_or(self.draw_parameters.step), false),
				DrawingCommand::BasicCommand{operation: TurtleCommand::ForwardContracting, parameter: p} => {
					let distance = if(p.is_some()) {
						p.unwrap().powf(self.num_iterations as f64)
					} else {
						self.contracted_length
					};

					self.move_forward(distance, true);
				},

				// State handling
				DrawingCommand::BasicCommand{operation: TurtleCommand::SaveState, ..} => self.push_state(),
				DrawingCommand::BasicCommand{operation: TurtleCommand::LoadState, ..} => self.pop_state(),

				// Direction changes
				DrawingCommand::BasicCommand{operation: TurtleCommand::TurnLeft, parameter: p} =>  self.apply_rotation(self.rotU(p, self.matrix_cache.turn_left)),
				DrawingCommand::BasicCommand{operation: TurtleCommand::TurnRight, parameter: p} =>  self.apply_rotation(self.rotUInv(p, self.matrix_cache.turn_right)),
				DrawingCommand::BasicCommand{operation: TurtleCommand::PitchDown, parameter: p} =>  self.apply_rotation(self.rotL(p, self.matrix_cache.pitch_down)),
				DrawingCommand::BasicCommand{operation: TurtleCommand::PitchUp, parameter: p} =>  self.apply_rotation(self.rotLInv(p, self.matrix_cache.pitch_up)),
				DrawingCommand::BasicCommand{operation: TurtleCommand::RollLeft, parameter: p} =>  self.apply_rotation(self.rotH(p, self.matrix_cache.roll_left)),
				DrawingCommand::BasicCommand{operation: TurtleCommand::RollRight, parameter: p} =>  self.apply_rotation(self.rotHInv(p, self.matrix_cache.roll_right)),
				DrawingCommand::BasicCommand{operation: TurtleCommand::TurnAround, parameter: p} =>  self.apply_rotation(self.matrix_cache.turn_around),
				
				// Polygon handling
				DrawingCommand::BasicCommand{operation: TurtleCommand::BeginPolygon, ..} => self.begin_polygon(),
				DrawingCommand::BasicCommand{operation: TurtleCommand::EndPolygon, ..} => self.end_polygon(),
				DrawingCommand::BasicCommand{operation: TurtleCommand::SubmitVertex, ..} => self.submit_vertex(),

				// Color handling
				DrawingCommand::BasicCommand{operation: TurtleCommand::IncrementColor, ..} => self.modify_color_index(1),
				DrawingCommand::BasicCommand{operation: TurtleCommand::DecrementColor, ..} => self.modify_color_index(-1),

				// Line width handling
				// If no parameter is given, the line width commands increment or decrement the line width by the line width delta value.
				DrawingCommand::BasicCommand{operation: TurtleCommand::IncrementLineWidth, parameter: None} => self.modify_line_width(self.draw_parameters.line_width_delta),
				DrawingCommand::BasicCommand{operation: TurtleCommand::DecrementLineWidth, parameter: None} => self.modify_line_width(-self.draw_parameters.line_width_delta),

				// If a parameter is given, they set the line width to that parameters value.
				DrawingCommand::BasicCommand{operation: TurtleCommand::IncrementLineWidth, parameter: Some(p)} => self.current_state.line_width = p.max(0.0),
				DrawingCommand::BasicCommand{operation: TurtleCommand::DecrementLineWidth, parameter: Some(p)} => self.current_state.line_width = p.max(0.0),

				_ => ()
			}
		}
	}

	fn rotU(&self, param: &Option<f64>, def: Matrix3d) -> Matrix3d {
		self.matrix_helper(Turtle3DMatrixCache::rotU, param, def, 1.0)
	}

	fn rotH(&self, param: &Option<f64>, def: Matrix3d) -> Matrix3d {
		self.matrix_helper(Turtle3DMatrixCache::rotH, param, def, 1.0)
	}

	fn rotL(&self, param: &Option<f64>, def: Matrix3d) -> Matrix3d {
		self.matrix_helper(Turtle3DMatrixCache::rotL, param, def, 1.0)
	}

	fn rotUInv(&self, param: &Option<f64>, def: Matrix3d) -> Matrix3d {
		self.matrix_helper(Turtle3DMatrixCache::rotU, param, def, -1.0)
	}

	fn rotHInv(&self, param: &Option<f64>, def: Matrix3d) -> Matrix3d {
		self.matrix_helper(Turtle3DMatrixCache::rotH, param, def, -1.0)
	}

	fn rotLInv(&self, param: &Option<f64>, def: Matrix3d) -> Matrix3d {
		self.matrix_helper(Turtle3DMatrixCache::rotL, param, def, -1.0)
	}

	fn matrix_helper(&self, mat: fn(f64) -> Matrix3d, param: &Option<f64>, def: Matrix3d, md: f64) -> Matrix3d {
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

	pub fn move_forward(&mut self, distance: f64, draw: bool) {
		let old_position = self.current_state.position;
		
		let mv = self.current_state.heading.into_inner() * distance;

		self.current_state.position = old_position + mv;

		if draw {
			let begin = Vector3f::new(
				old_position.x as _,
				old_position.y as _,
				old_position.z as _
			);

			let end = Vector3f::new(
				self.current_state.position.x as _,
				self.current_state.position.y as _,
				self.current_state.position.z as _
			);

			self.drawing_result.line_segments.push(LineSegment{
				begin: begin,
				end: end,
				color: self.current_state.color_index,
				width: self.current_state.line_width as _
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
