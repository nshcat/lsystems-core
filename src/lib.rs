// All features used for drawing, i.e. turtle implementation and primtive defintions
pub mod drawing;
// Iteration engine and support structures
pub mod iteration;
// Rule grammar
pub mod grammar;
// Various utilities
pub mod util;
// Interpretation of interated module strings
pub mod interpretation;


use crate::drawing::*;
use crate::drawing::turtle::*;
use crate::iteration::*;
use crate::grammar::*;
use crate::interpretation::*;


/// Top level structure providing the means of describing, iterating, interpreting and drawing of an L-System.
///
/// The user code has to perform the following steps to receive a list of primitives used to draw the L-System:
/// 	- Construct a new L-System instance
/// 	- Call `parse` with an axiom string and a set of rules
/// 	- Set interpretation associations in `interpretation_engine`
/// 	- Set drawing parameters and iteration depth
/// 	- Call `iterate`
/// 	- Call `interpret`
/// 	- Use the primitives saved in `drawing_result` to display the L-System in a platform dependent way
/// 
/// Note that depending on the application, when redrawing a modified system, not all of the above steps have to be
/// repeated. This can significantly speed up the process. For example, if only associations changed, a call to interpret 
/// is enough, since nothing affecting the iteration has changed.
pub struct LSystem {
	/// The engine used to iterate a module string
	pub iteration_engine: IterationEngine,
	/// The engine used to interpret a module string as drawing commands
	pub interpretation_engine: InterpretationEngine,
	/// Parameters to use to apply drawing commands and retrieve primitives
	pub parameters: DrawingParameters,
	/// Drawing commands as the result of the interpretation stage
	pub commands: Vec<DrawingCommand>,
	/// Struct containing all primitives generated during drawing stage  
	pub drawing_result: DrawingResult
}

impl LSystem {
	/// Set the drawing parameters to use to draw the system.
	pub fn set_drawing_parameters(&mut self, params: &DrawingParameters) {
		self.parameters = DrawingParameters::clone(params);
	}

	/// Perform L-System iteration by applying ruleset to axiom string and creating a derived
	/// module string
	pub fn iterate(&mut self) {
		self.iteration_engine.iterate();
	}

	/// Interpret generated module string as sequence of drawing commands
	pub fn interpret(&mut self) {
		self.commands = self.interpretation_engine.interpret(&self.iteration_engine.module_string);

		let mut turtle = Turtle3D::new(self.parameters, self.iteration_engine.iteration_depth);

		turtle.execute_modules(&self.commands);

		self.drawing_result = turtle.retrieve_result().clone();
	}

	/// Parse given axiom string and rule set
	pub fn parse(&mut self, axiom: &str, rules: &str) {
		self.iteration_engine.axiom = grammar::lsystem_parser::module_string(axiom).unwrap_or(Vec::new());
		self.iteration_engine.rules = grammar::lsystem_parser::rule_list(rules).unwrap_or(Vec::new());
	}

	/// Create new, empty L-System.
	pub fn new() -> LSystem {
		LSystem {
			iteration_engine: IterationEngine::new(),
			interpretation_engine: InterpretationEngine::new(),
			parameters: DrawingParameters::new(),
			commands: Vec::new(),
			drawing_result: DrawingResult::new()
		}
	}

	/// Set the iteration depth.
	pub fn set_iteration_depth(&mut self, depth: u32) {
		self.iteration_engine.set_iteration_depth(depth);	
	}
}


