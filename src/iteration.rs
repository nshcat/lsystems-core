#![allow(dead_code)]

use std::string::*;
use std::collections::*;
use std::fmt::*;
use rand::*;
use rand::rngs::*;
use rand::distributions::*;
use crate::drawing::DrawOperation;
use crate::util::*;

trait Evaluatable {
	type Result;
	fn eval(&self, env: &Environment) -> Self::Result;
}

/// An expression that evaluates to a number. It is used to both create new parameter values
/// when iterating a module string, as well as in boolean expressions in module patterns.
#[derive(Clone, Debug, PartialEq)]
pub enum ArithmeticExpression {
	Add(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
	Sub(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
	Mul(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
	Div(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
	Pow(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
	Neg(Box<ArithmeticExpression>),
	Const(f64),
	Param(char)
}

impl Evaluatable for ArithmeticExpression {
	type Result = f64;

	fn eval(&self, env: &Environment) -> Self::Result {
		match *self {
			ArithmeticExpression::Add(ref left, ref right) => left.eval(env) + right.eval(env),
			ArithmeticExpression::Sub(ref left, ref right) => left.eval(env) - right.eval(env),
			ArithmeticExpression::Mul(ref left, ref right) => left.eval(env) * right.eval(env),
			ArithmeticExpression::Div(ref left, ref right) => left.eval(env) / right.eval(env),
			ArithmeticExpression::Pow(ref left, ref right) => left.eval(env).powf(right.eval(env)),
			ArithmeticExpression::Neg(ref expr) => -expr.eval(env),
			ArithmeticExpression::Const(x) => x,
			ArithmeticExpression::Param(p) => {
				if(!env.has_parameter(p)) {
					panic!("No definition for parameter '{}' found in environment", p);				
				}
				env.get_parameter_value(p)
			}
		}
	}
}

impl Display for ArithmeticExpression {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self {
			ArithmeticExpression::Add(ref left, ref right) => write!(f, "({} + {})", left, right),
			ArithmeticExpression::Sub(ref left, ref right) => write!(f, "({} - {})", left, right),
			ArithmeticExpression::Mul(ref left, ref right) => write!(f, "({} * {})", left, right),
			ArithmeticExpression::Div(ref left, ref right) => write!(f, "({} / {})", left, right),
			ArithmeticExpression::Pow(ref left, ref right) => write!(f, "({}^{})", left, right),
			ArithmeticExpression::Neg(ref expr) => write!(f, "(-{})", expr),
			ArithmeticExpression::Const(x) => write!(f, "{}", x),
			ArithmeticExpression::Param(p) => write!(f, "{}", p)
		}
    }
}

/// A boolean expression used in module pattern as part of the left side of rules.
#[derive(Clone, Debug, PartialEq)]
pub enum BooleanExpression {
	Not(Box<BooleanExpression>),
	And(Box<BooleanExpression>, Box<BooleanExpression>),
	Or(Box<BooleanExpression>, Box<BooleanExpression>),
	Lth(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
	Leq(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
	Gth(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
	Geq(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
	Eq(Box<ArithmeticExpression>, Box<ArithmeticExpression>),
	Const(bool)
}

impl Display for BooleanExpression {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match *self {
			BooleanExpression::Not(ref expr) => write!(f, "!({})", expr),
			BooleanExpression::And(ref left, ref right) => write!(f, "({} && {})", left, right),
			BooleanExpression::Or(ref left, ref right) => write!(f, "({} || {})", left, right),
			BooleanExpression::Lth(ref left, ref right) => write!(f, "({} < {})", left, right),
			BooleanExpression::Leq(ref left, ref right) => write!(f, "({} <= {})", left, right),
			BooleanExpression::Gth(ref left, ref right) => write!(f, "({} > {})", left, right),
			BooleanExpression::Geq(ref left, ref right) => write!(f, "({} >= {})", left, right),
			BooleanExpression::Eq(ref left, ref right) => write!(f, "({} == {})", left, right),
			BooleanExpression::Const(val) => write!(f, "{}", val)
		}
    }
}

impl Evaluatable for BooleanExpression {
	type Result = bool;

	fn eval(&self, env: &Environment) -> Self::Result {
		match *self {
			BooleanExpression::Not(ref expr) => !expr.eval(env),
			BooleanExpression::And(ref left, ref right) => left.eval(env) && right.eval(env),
			BooleanExpression::Or(ref left, ref right) => left.eval(env) || right.eval(env),
			BooleanExpression::Lth(ref left, ref right) => left.eval(env) < right.eval(env),
			BooleanExpression::Leq(ref left, ref right) => left.eval(env) <= right.eval(env),
			BooleanExpression::Gth(ref left, ref right) => left.eval(env) > right.eval(env),
			BooleanExpression::Geq(ref left, ref right) => left.eval(env) >= right.eval(env),
			BooleanExpression::Eq(ref left, ref right) => left.eval(env) == right.eval(env),
			BooleanExpression::Const(val) => val
		}
	}
}

/// A statement used in the execution block of a rule. Statements can be evaluated, which causes
/// 
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
	IfThenElse(Box<BooleanExpression>, Box<Statement>, Option<Box<Statement>>),
	Assignment(char, Box<ArithmeticExpression>)
}


/// Environment used for binding parameter names to actual values. Used when checking if a module
/// satisfies a module pattern with condition, as well as when creating a module from a module 
/// template when applying a rule.
#[derive(Debug, Clone)]
pub struct Environment {
	parameter_map: HashMap<char, f64>
}

impl Environment {
	fn new() -> Environment {
		return Environment { parameter_map: HashMap::new() };	
	}

	fn has_parameter(&self, param: char) -> bool {
		self.parameter_map.contains_key(&param)
	}

	fn get_parameter_value(&self, param: char) -> f64 {
		match self.parameter_map.get(&param) {
			Some(value) => *value,
			None => panic!("Environment does not contain value for parameter {}", param)
		}	
	}
	
	fn define_parameter(&mut self, param: char, value: f64) {
		if(self.has_parameter(param)) {
			panic!("Tried to define parameter {} as {} but already exists as {}", param, value, self.get_parameter_value(param));		
		}

		self.parameter_map.insert(param, value);
	}	
}

impl Display for Environment {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		write!(f, "{:?}", self.parameter_map)
    }
}

/// A description of how a module "looks" like, e.g. "A(x,y,z)".
/// This is used as part of module patterns as part of iteration rules.
#[derive(Debug, Clone)]
pub struct ModuleSignature {
	pub identifier: char,
	pub parameters: Vec<char>
}

impl ModuleSignature {
	fn has_parameters(& self) -> bool {
		self.parameters.len() > 0	
	}

	fn parameter_count(& self) -> usize {
		return self.parameters.len();	
	}
}

impl Display for ModuleSignature {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		if(self.has_parameters()) {
			let mut is_first = true;
			write!(f, "{}(", self.identifier)?;

			for p in &self.parameters {
				if(is_first) {
					is_first = false;
					write!(f, "{}", p)?;
				}
				else {
					write!(f, ",{}", p)?;	
				}	
			}

			write!(f, ")")
		}
		else {
			write!(f, "{}", self.identifier)
		}
    }
}

/// A template for a module instance used as part of the right side of a iteration rule.
/// It uses expressions with parameter variables in it, like "A(x+1, y)".
#[derive(Debug, Clone)]
pub struct ModuleTemplate {
	pub identifier: char,
	pub parameter_expressions: Vec<ArithmeticExpression>
}

impl ModuleTemplate {
	/// Create an actual module instance based on this template. Any expressions
	/// contained in the template will be evaluated with given environment.
	pub fn instantiate(& self, env: &Environment) -> Module {
		Module {
			identifier: self.identifier,
			parameter_values: self.parameter_expressions.iter().map(|expr| expr.eval(env)).collect()
		}
	}

	pub fn has_parameters(& self) -> bool {
		return self.parameter_expressions.len() > 0;	
	}

	pub fn parameter_count(& self) -> usize {
		return self.parameter_expressions.len();	
	}
}

impl Display for ModuleTemplate {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		if(self.has_parameters()) {
			let mut is_first = true;
			write!(f, "{}(", self.identifier)?;

			for p in &self.parameter_expressions {
				if(is_first) {
					is_first = false;
					write!(f, "{}", p)?;
				}
				else {
					write!(f, ",{}", p)?;	
				}	
			}

			write!(f, ")")
		}
		else {
			write!(f, "{}", self.identifier)
		}
    }
}

/// An annotation that can be part of a module, for example to create a bezier
/// patch with name A and scaling f, the module would look like "~A(f)"
/// We use hardcoded annotations, since just interpreting any character in front of a module
/// identifier as an annotation would be ambiguous; we want to allow parameterless module strings such as
/// "+++---A(f)" which contain special characters like '+' and '-'. 
#[derive(Debug, Clone)]
pub enum ModuleAnnotation {
	/// Interpret the module as a command to create a bezier patch at this position.
	CreatePatch
}

impl Display for ModuleAnnotation {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		return match self {
			ModuleAnnotation::CreatePatch => write!(f, "~")
		};
    }
}

/// A module as its appearing in an actual iteration string. Can have parameter values, like "A(1, 3)", or not,
/// like "A".
#[derive(Debug, Clone)]
pub struct Module {
	/// The character used as the identifier of this module.
	pub identifier: char,
	/// The actual parameters values.
	pub parameter_values: Vec<f64>,
	/// A possible module annotation.
	pub annotation: Option<ModuleAnnotation>
}

impl Module {
	pub fn has_parameters(& self) -> bool {
		return self.parameter_values.len() > 0;	
	}

	pub fn parameter_count(& self) -> usize {
		return self.parameter_values.len();	
	}

	pub fn has_annotation(& self) -> bool {
		return self.annotation.is_some();
	}
}

impl Display for Module {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		if(self.has_annotation()) {
			write!(f, "{}", self.annotation.unwrap())?;
		}

		if(self.has_parameters()) {
			let mut is_first = true;
			write!(f, "{}(", self.identifier)?;

			for p in &self.parameter_values {
				if(is_first) {
					is_first = false;
					write!(f, "{}", p)?;
				}
				else {
					write!(f, ",{}", p)?;	
				}	
			}

			write!(f, ")")
		}
		else {
			write!(f, "{}", self.identifier)
		}
    }
}


/// A module and its immediate surounding modules, which might or might not actually exist.
#[derive(Debug, Clone)]
pub struct ModuleContext {
	pub left: Option<Module>,
	pub center: Module,
	pub right: Option<Module>
}

impl ModuleContext {
	pub fn new(center: Module) -> ModuleContext {
		ModuleContext {
			left: None,
			center: center,
			right: None	
		}
	}

	pub fn new_with_left(center: Module, left: Module) -> ModuleContext {
		ModuleContext {
			left: Some(left),
			center: center,
			right: None	
		}
	}

	pub fn new_with_right(center: Module, right: Module) -> ModuleContext {
		ModuleContext {
			left: None,
			center: center,
			right: Some(right)
		}
	}

	pub fn new_complete(center: Module, left: Module, right: Module) -> ModuleContext {
		ModuleContext {
			left: Some(left),
			center: center,
			right: Some(right)
		}
	}
}


/// The left side of a iteration rule. Supports imposing conditions on the parameters, as well as the
/// surounding modules. Parameter conditions can include parameters of both the module and side modules.
/// A succesfull match will bind variables in the pattern to the actual values.
#[derive(Debug, Clone)]
pub struct ModulePattern {
	pub match_left: Option<ModuleSignature>,
	pub match_center: ModuleSignature,
	pub match_right: Option<ModuleSignature>,
	pub condition: BooleanExpression
}

impl ModulePattern {
	/// Check whether the given module context matches this pattern.
	pub fn does_match(& self, context: &ModuleContext) -> bool {
		let mut env = Environment::new();

		if(self.match_center.identifier != context.center.identifier) {
			return false;
		}

		// Both centers have to have the exact same number of parameters
		if(self.match_center.parameter_count() != context.center.parameter_count()) {
			return false;		
		}

		Self::extract_parameters(&self.match_center, &context.center, &mut env);

		if(self.match_left.is_some()) {
			if(context.left.is_none()) {
				return false;			
			}
		
			// Check if parameter counts match
			let match_left = self.match_left.as_ref().unwrap();
			let left = context.left.as_ref().unwrap();

			if(match_left.identifier != left.identifier) {
				return false;
			}

			if(match_left.parameter_count() != left.parameter_count()) {
				return false;
			}

			Self::extract_parameters(&match_left, &left, &mut env);
		}

		if(self.match_right.is_some()) {
			if(context.right.is_none()) {
				return false;			
			}
		
			// Check if parameter counts match
			let match_right = self.match_right.as_ref().unwrap();
			let right = context.right.as_ref().unwrap();

			if(match_right.identifier != right.identifier) {
				return false;
			}

			if(match_right.parameter_count() != right.parameter_count()) {
				return false;
			}

			Self::extract_parameters(&match_right, &right, &mut env);
		}

		return self.condition.eval(&env);
	}

	/// Create an environment in which the parameter variables in this pattern are bound to the values
	/// present in the given context. This is used to instantiate the module templates in the right side
	/// of a rule. This function requires that `does_match` returned true.
	pub fn bind(& self, context: &ModuleContext) -> Environment {
		let mut env = Environment::new();

		if(self.match_left.is_some()) {
			Self::extract_parameters(&self.match_left.as_ref().unwrap(), &context.left.as_ref().unwrap(), &mut env);	
		}

		if(self.match_right.is_some()) {
			Self::extract_parameters(&self.match_right.as_ref().unwrap(), &context.right.as_ref().unwrap(), &mut env);	
		}

		Self::extract_parameters(&self.match_center, &context.center, &mut env);	

		return env;
	}

	fn extract_parameters(signature: &ModuleSignature, module: &Module, env: &mut Environment) {
		// We expect both the signature and the module to have the same number of parameters.
		for (i, p) in signature.parameters.iter().enumerate() {
			let value = module.parameter_values[i];
			env.define_parameter(*p, value);	
		}
	}
}

impl Display for ModulePattern {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {

		if(self.match_left.is_some()) {
			write!(f, "{} < ", self.match_left.as_ref().unwrap())?;
		}
	
		write!(f, "{}", self.match_center)?;
		
		if(self.match_right.is_some()) {
			write!(f, " > {} ", self.match_right.as_ref().unwrap())?;
		}

		match self.condition {
			BooleanExpression::Const(true) => write!(f, ": *"),
			_ => write!(f, ": {}", self.condition)			
		}
		
    }
}



/// A rule consisting of a left side pattern and a right side sequence of templates
#[derive(Debug, Clone)]
pub struct Rule {
	pub pattern: ModulePattern,
	pub right_side: Vec<ModuleTemplate>,
	pub probability: f64
}

impl Rule {
	/// Checks whether this rule always applies, i.e. has "100% chance". Note that this is not the same as having a probability
	/// of 1.0, since the probability is more like a general weight, e.g. users could use integers aswell.	
	pub fn is_deterministic(&self) -> bool {
		self.probability < 0.0
	}
}

impl Display for Rule {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		write!(f, "{} ", self.pattern)?;
		
		if(!self.is_deterministic()) {
			write!(f, ": {} ", self.probability)?;
		}

		write!(f, "-> ")?;

		for template in &self.right_side {
			write!(f, "{} ", template)?;
		}

		write!(f, "")
    }
}

#[derive(Debug, Clone)]
pub struct IterationEngine {
	pub axiom: Vec<Module>,
	pub module_string: Vec<Module>,
	pub rules: Vec<Rule>,
	pub iteration_depth: u32,
	rng: StdRng
}

impl IterationEngine {
	pub fn set_iteration_depth(&mut self, depth: u32) {
		self.iteration_depth = depth;		
	}

	pub fn add_rule(&mut self, rule: Rule) {
		self.rules.push(rule);	
	}

	pub fn new() -> IterationEngine {
		IterationEngine {
			axiom: Vec::new(),
			module_string: Vec::new(),
			rules: Vec::new(),
			iteration_depth: 0,
			rng: StdRng::seed_from_u64(133742)
		}	
	}

	pub fn set_seed(&mut self, seed: u64) {
		self.rng = StdRng::seed_from_u64(seed)
	}

	pub fn iterate(&mut self) {
		self.module_string = self.axiom.clone();

		for i in 0..self.iteration_depth {
			let mut new_module_string: Vec<Module> = Vec::new();

			for (i, module) in self.module_string.iter().enumerate() {
				let mut context = ModuleContext::new(module.clone());
				
				// If we are not the first module, we can populate the left side of
				// the context
				if(i > 0) {
					context.left = Some(self.module_string[i-1].clone());
				}

				// If we are not the last module, there exists a right neighbour.
				if(i < self.module_string.len() - 1) {
					context.right = Some(self.module_string[i+1].clone());
				}

				// Collect all rules that match
				let mut matching_rules = Vec::new();

				for rule in &self.rules {
					if(rule.pattern.does_match(&context)) {
						matching_rules.push(rule.clone());
					}
				}

				// If its empty, we can do nothing
				if(matching_rules.len() == 0) {
					new_module_string.push(module.clone());
				} else {
					// Check if there are any rules that are deterministic. If so, apply first one of them.
					let deterministic_matches: Vec<Rule> = matching_rules.clone().into_iter()
						.filter(|r| r.is_deterministic())
						.collect();

					let chosen_match = match deterministic_matches.len() {
						0 => {
							let mut items: Vec<Weighted<Rule>> = matching_rules.into_iter().map(|r| Weighted{ weight: r.probability, item: r }).collect();
							let wc = WeightedChoice::new(&mut items);
							
							wc.sample(&mut self.rng)
						},
						_ => deterministic_matches.first().unwrap().clone()
					};

					// We now have a match. Instantiate right side.
					let env = chosen_match.pattern.bind(&context);

					for template in &chosen_match.right_side {
						new_module_string.push(template.instantiate(&env));						
					}
				}
			}

			self.module_string = new_module_string;
		}
	}
}
