use peg::parser;
use crate::iteration::*;

fn boxed<T>(t: T) -> Box<T> {
    Box::new(t)
}

parser!{
	pub grammar lsystem_parser() for str {
		rule true_false() -> bool
			= b:$("true" / "false") { b.parse().unwrap() }
			/ "*" { true }

		pub rule boolean_expr() -> BooleanExpression
			= precedence!{
				l:(@) padding() "||" padding() r:@ { BooleanExpression::Or(boxed(l), boxed(r)) }
				--
				l:(@) padding() "&&" padding() r:@ { BooleanExpression::And(boxed(l), boxed(r)) }
				--
				"(" padding() c:(boolean_expr()) padding() ")" { c }
				--
				l:arith_expr() padding() "<" padding() r:arith_expr() { BooleanExpression::Lth(boxed(l), boxed(r)) }
				l:arith_expr() padding() "<=" padding() r:arith_expr() { BooleanExpression::Leq(boxed(l), boxed(r)) }
				l:arith_expr() padding() "==" padding() r:arith_expr() { BooleanExpression::Eq(boxed(l), boxed(r)) }					
				l:arith_expr() padding() ">=" padding() r:arith_expr() { BooleanExpression::Geq(boxed(l), boxed(r)) }
				l:arith_expr() padding() ">" padding() r:arith_expr() { BooleanExpression::Gth(boxed(l), boxed(r)) }
				--
				"!" r:(@) { BooleanExpression::Not(boxed(r)) }
				--
				b:true_false() { BooleanExpression::Const(b) }
			}

		rule arith_expr() -> ArithmeticExpression
			= precedence!{
				l:(@) padding() "+" padding() r:@ { ArithmeticExpression::Add(boxed(l), boxed(r)) }
				l:(@) padding() "-" padding() r:@ { ArithmeticExpression::Sub(boxed(l), boxed(r)) }
				--
				l:(@) padding() "*" padding() r:@ { ArithmeticExpression::Mul(boxed(l), boxed(r)) }
				l:(@) padding() "/" padding() r:@ { ArithmeticExpression::Div(boxed(l), boxed(r)) }
				--
				l:@ padding() "^"  padding()r:(@) { ArithmeticExpression::Pow(boxed(l), boxed(r)) }
				--
				"-" r:(@) { ArithmeticExpression::Neg(boxed(r)) }
				--
				n:number() { ArithmeticExpression::Const(n) }
				p:parameter_name() { ArithmeticExpression::Param(p) }
			}

		rule whitespace()
			= quiet!{[' ' | '\t']+}

		rule padding()
			= whitespace()*

		rule _()
			= padding()*

		rule number() -> f64
			= n:$(['+'|'-']?['0'..='9']+("." ['0'..='9']+)?) { n.parse().unwrap() }

		rule identifier() -> char
			= x:$(['a'..='z' | 'A'..='Z' | '0'..='9' | '!' | '^' | '+' | '\'' | '-' | '[' | ']' | '\\' | '/' | '|' | '~' | '#' | '&' | '{' | '}' | '.']) { x.parse().unwrap() }

		rule condition() -> BooleanExpression
			= expr:(condition_part())? { expr.unwrap_or(BooleanExpression::Const(true)) }

		rule condition_part() -> BooleanExpression
			= padding() ":" padding() expr:boolean_expr() { expr }
	
		rule parameter_name() -> char
			= x:$(['a'..='z']) { x.parse().unwrap() }

		rule simple_signature() -> ModuleSignature
			= x:identifier() { ModuleSignature{ identifier: x, parameters: Vec::new() } }

		rule signature_with_parameters() -> ModuleSignature
			= x:identifier() "(" padding() p:parameter_name() ** (padding() "," padding())  padding() ")" { ModuleSignature{ identifier: x, parameters: p } }

		rule signature() -> ModuleSignature
			= signature_with_parameters() / simple_signature()

		rule left_pattern() -> ModuleSignature
			= left:signature() padding() "<" padding() { left }

		rule right_pattern() -> ModuleSignature
			= padding() ">" padding() right:signature() { right }

		rule pattern() -> ModulePattern
			= left:(left_pattern())? center:signature() right:(right_pattern())? expr:condition() { ModulePattern{ match_left: left, match_center: center, match_right: right, condition: expr } }

		rule simple_template() -> ModuleTemplate
			= x:identifier() { ModuleTemplate{ identifier: x, parameter_expressions: Vec::new() } }

		rule template_with_parameters() -> ModuleTemplate
			= x:identifier() "(" expr:arith_expr() ** ","  ")" { ModuleTemplate{ identifier: x, parameter_expressions: expr } }

		pub rule template() -> ModuleTemplate
			= template_with_parameters() / simple_template()

		rule template_string_entry() -> ModuleTemplate
			= padding() t:template() padding() { t }

		rule template_string() -> Vec<ModuleTemplate>
			= template_string_entry()*

		rule probability() -> f64
			= ":" padding() p:number() { p }

		rule probability_suffix() -> f64
			= p:(probability())? { p.unwrap_or(-1.0) }

		pub rule lsystem_rule() -> Rule
			= p:pattern() padding()  prob:probability_suffix() padding() "->" padding() rightside:template_string() { Rule{ pattern: p, right_side: rightside, probability: prob } }

	    rule simple_module() -> Module
			= x:identifier() { Module{ identifier: x, parameter_values: Vec::new() } }
	
		rule module_with_parameters() -> Module
			= x:identifier() "(" n:number() ** ","  ")" { Module{ identifier: x, parameter_values: n } }

		pub rule module() -> Module
			= module_with_parameters() / simple_module()

		pub rule module_string() -> Vec<Module>
			= module_string_entry()*

		rule module_string_entry() -> Module
			= padding() m:module() padding() { m }

		pub rule rule_list() -> Vec<Rule>
			= rs:lsystem_rule() ** (padding() "\n" padding())* { rs }
	}
}
