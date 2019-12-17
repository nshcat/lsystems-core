use std::collections::HashMap;
use crate::drawing::TurtleCommand;
use crate::iteration::*;

/// A struct implementing the interpretation of a iterated module string as a series
/// of drawing commands. A drawing command is either a direct turtle command, or a special
/// command based on module annotations, such as the creation of a patch. This struct allows
/// the association of module identifiers with basic turtle commands.
pub struct InterpretationEngine {
    /// Mapping between identifiers and turtle commands.
    internal_map: HashMap<char, TurtleCommand>
}

impl InterpretationEngine {
    /// Associate given identifier with given turtle command
    pub fn associate(&mut self, character:char, operation: TurtleCommand) {
		self.internal_map.remove(&character);
		self.internal_map.insert(character, operation);
	}

    /// Retrieve the associated interpretation for given identifier. This will panic
    /// if no such association exists.
	fn retrieve(&self, character: char) -> TurtleCommand {
		match self.internal_map.get(&character) {
			Some(operation) => return operation.clone(),
			None => panic!("Interpretation map does not contain definition for character {}", character)
		}
	}

    /// Check whether an interpretation for given identifier exists within this engine.
	fn has_interpretation(&self, character: char) -> bool {
		return self.internal_map.contains_key(&character);
	}

    /// Clear all stored associations.
	pub fn clear(&mut self) {
		self.internal_map.clear();	
	}

    /// Create a new, empty interpretation engine.
	pub fn new() -> InterpretationEngine {
		return InterpretationEngine {
            internal_map: HashMap::new()
        };
    }

    /// Interpret given module string based on the associations stored within this
    /// engine and derive a sequence of drawing commands.
    pub fn interpret(&self, module_string: &[Module]) -> Vec<DrawingCommand> {
        let mut commands = Vec::new();

        for module in module_string {
            match module.annotation {
                Some(ModuleAnnotation::CreatePatch) => {
                    let scaling = match module.parameter_count() {
                        0 => 1.0,
                        1 => module.parameter_values[0],
                        _ => panic!("Found module annotated as CreatePatch, but has more than one parameter value: {}", module)
                    };
                                 
                    commands.push(
                        DrawingCommand::SpawnPatch{
                            patch_id: module.identifier,
                            scaling: scaling
                        }
                    );
                },
                None =>  {
                    if self.has_interpretation(module.identifier) {
                        let operation = self.retrieve(module.identifier);

                        let param = match module.parameter_count() {
                            0 => None,
                            1 => Some(module.parameter_values[0]),
                            _ => panic!("Turtle commands can't have more than one parameters, but has {}: {}", module.parameter_count(), module)	
                        };

                        commands.push(
                            DrawingCommand::BasicCommand{
                                operation: operation,
                                parameter: param 
                            }
                        );
                    }
                }
            }
        }

        commands
    }
}


/// A drawing command derived from a module of an iterated module string.
#[derive(Debug, Clone)]
pub enum DrawingCommand {
    /// A basic turtle command, with an optional argument
    BasicCommand { operation: TurtleCommand, parameter: Option<f64> },
    /// Spawn a patch at this position.
    SpawnPatch { patch_id: char, scaling: f64 }
}
