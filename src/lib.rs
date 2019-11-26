// All features used for drawing, i.e. turtle implementations,
// line and polygon structures.
pub mod drawing;
// Iteration engine and support structures
pub mod iteration;
// Rule grammar
pub mod grammar;
// Various utilities
pub mod util;

use std::collections::*;
use std::convert::*;

use crate::drawing::*;
use crate::drawing::turtle::*;
use crate::drawing::primitives::*;
use crate::drawing::types::*;
use crate::iteration::*;
use crate::grammar::*;


pub struct InterpretationMap {
	internal_map: HashMap<char, DrawOperation>
}

impl InterpretationMap {
    pub fn associate(&mut self, character:char, operation: DrawOperation) {
		self.internal_map.remove(&character);
		self.internal_map.insert(character, operation);
	}

	fn retrieve(&self, character: char) -> DrawOperation {
		match self.internal_map.get(&character) {
			Some(operation) => return operation.clone(),
			None => panic!("Interpretation map does not contain definition for character {}", character)
		}
	}

	fn has_interpretation(&self, character: char) -> bool {
		return self.internal_map.contains_key(&character);
	}

	fn new() -> InterpretationMap {
		return InterpretationMap{ internal_map: HashMap::new() };
	}
}

// TODO maybe dont store polygons and line_segments here? Maybe own class? InterpretationResult?
// three functions: iterate, interpret, draw
pub struct LSystem {
	pub engine: IterationEngine,
	pub interpretations: InterpretationMap,
	pub parameters: DrawingParameters,	
	pub commands: Vec<DrawingModule>,  
	pub line_segments: Vec<LineSegment>,
	pub polygons: Vec<Polygon>
}

impl LSystem {
	pub fn set_drawing_parameters(&mut self, params: &DrawingParameters) {
		self.parameters = DrawingParameters::clone(params);
	}

	// Perform L-System iteration by applying ruleset to axiom string
	pub fn iterate(&mut self) {
		self.engine.iterate();

		let mut commands: Vec<DrawingModule> = Vec::new();
		
		for module in &self.engine.module_string {
			if(self.interpretations.has_interpretation(module.identifier)) {
				let operation = self.interpretations.retrieve(module.identifier);

				let command = match module.parameter_count() {
					0 => DrawingModule::new(operation),
					1 => DrawingModule::new_with_parameter(operation, module.parameter_values[0]),
					_ => panic!("Drawing operation can't have more than one parameters, but has {}", module.parameter_count())	
				};

				commands.push(command);
			}
		}

		self.commands = commands.clone();
	}

	pub fn interpret(&mut self) {
		let mut turtle = Turtle3D::new(self.parameters, self.engine.iteration_depth);

		turtle.execute_modules(&self.commands);

		self.line_segments = turtle.line_segments().clone();
		self.polygons = turtle.polygons().clone();
	}

	pub fn parse(&mut self, axiom: &str, rules: &str) {
		self.engine.module_string = grammar::lsystem_parser::module_string(axiom).unwrap_or(Vec::new());
		self.engine.rules = grammar::lsystem_parser::rule_list(rules).unwrap_or(Vec::new());
	}

	pub fn new() -> LSystem {
		LSystem {
			engine: IterationEngine::new(),
			interpretations: InterpretationMap::new(),
			parameters: DrawingParameters::new(),
			commands: Vec::new(),
			line_segments: Vec::new(),
			polygons: Vec::new()
		}
	}

	pub fn set_iteration_depth(&mut self, depth: u32) {
		self.engine.set_iteration_depth(depth);	
	}
}


