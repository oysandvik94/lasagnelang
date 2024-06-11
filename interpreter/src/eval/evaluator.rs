use crate::parser::{
    ast::{Operator, PrefixOperator, Program, Statement},
    expressions::{expression::Expression, expression_statement::ExpressionStatement},
};

use super::{eval_error::EvalError, objects::Object};

pub(crate) trait Evaluable {
    fn eval(&self) -> Result<Object, EvalError>;
}

impl Evaluable for Program {
    fn eval(&self) -> Result<Object, EvalError> {
        let mut object: Option<Object> = None;

        for statement in &self.statements {
            object = Some(statement.eval()?);
        }

        match object {
            Some(object) => Ok(object),
            None => Err(EvalError::EmptyProgram),
        }
    }
}

impl Evaluable for Statement {
    fn eval(&self) -> Result<Object, EvalError> {
        match self {
            Statement::Expression(ExpressionStatement { expression }) => expression.eval(),
            Statement::Assign(_) => todo!(),
            Statement::Return(_) => todo!(),
        }
    }
}

impl Evaluable for Expression {
    fn eval(&self) -> Result<Object, EvalError> {
        use Object::*;

        match self {
            Expression::IntegerLiteral(number) => Ok(Integer(*number)),
            Expression::IdentifierLiteral(_) => todo!(),
            Expression::BooleanLiteral(boolean) => Ok(Boolean(*boolean)),
            Expression::Prefix { right, operator } => eval_prefix_expression(right, operator),
            Expression::Infix {
                left,
                right,
                operator,
            } => {
                let left = left.eval()?;
                let right = right.eval()?;
                eval_infix_expression(operator, left, right)
            }
            Expression::If(_) => todo!(),
            Expression::Function(_) => todo!(),
            Expression::Call(_) => todo!(),
        }
    }
}

fn eval_infix_expression(
    operator: &crate::parser::ast::Operator,
    left: Object,
    right: Object,
) -> Result<Object, EvalError> {
    use Object::*;
    match (left, right) {
        (Integer(left_integer), Integer(right_integer)) => {
            eval_integer_infix_expression(left_integer, right_integer, operator)
        }
        (Boolean(left_boolean), Boolean(right_boolean)) => {
            eval_boolean_infix_expression(left_boolean, right_boolean, operator)
        }
        (unexpected_left, unexpected_right) => Err(EvalError::InfixRightLeft(
            unexpected_left.clone(),
            unexpected_right.clone(),
        )),
    }
}

fn eval_boolean_infix_expression(
    left_boolean: bool,
    right_boolean: bool,
    operator: &Operator,
) -> Result<Object, EvalError> {
    use Object::*;

    Ok(match operator {
        Operator::Equals => Boolean(left_boolean == right_boolean),
        Operator::NotEquals => Boolean(left_boolean != right_boolean),
        unsupported_operator => {
            return Err(EvalError::BooleanInfixOperator(
                unsupported_operator.clone(),
            ))
        }
    })
}

fn eval_integer_infix_expression(
    left_integer: i32,
    right_integer: i32,
    operator: &crate::parser::ast::Operator,
) -> Result<Object, EvalError> {
    use Object::*;

    Ok(match operator {
        Operator::Minus => Integer(left_integer - right_integer),
        Operator::Plus => Integer(left_integer + right_integer),
        Operator::Multiply => Integer(left_integer * right_integer),
        Operator::DividedBy => Integer(left_integer / right_integer),
        Operator::LessThan => Boolean(left_integer < right_integer),
        Operator::GreaterThan => Boolean(left_integer > right_integer),
        Operator::Equals => Boolean(left_integer == right_integer),
        Operator::NotEquals => Boolean(left_integer != right_integer),
        unexpected_operator => {
            return Err(EvalError::IntegerInfixOperatorError(
                unexpected_operator.clone(),
            ));
        }
    })
}

fn eval_prefix_expression(
    right: &Expression,
    operator: &PrefixOperator,
) -> Result<Object, EvalError> {
    let right = right.eval()?;
    match operator {
        PrefixOperator::Bang => eval_bang_operator_expression(&right),
        PrefixOperator::Minus => eval_minus_operator_expression(&right),
    }
}

fn eval_minus_operator_expression(right: &Object) -> Result<Object, EvalError> {
    match right {
        Object::Integer(integer_value) => Ok(Object::Integer(-integer_value)),
        unexpected_object => Err(EvalError::IncorrectBangSuffix(unexpected_object.clone())),
    }
}

fn eval_bang_operator_expression(right: &Object) -> Result<Object, EvalError> {
    match right {
        Object::Boolean(boolean_value) => Ok(Object::Boolean(!boolean_value)),
        unexpected_object => Err(EvalError::IncorrectBangSuffix(unexpected_object.clone())),
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        eval::{self, objects::Object},
        parser::test_util,
    };

    #[test]
    fn eval_integer_expression_test() {
        let input_expected: Vec<(&str, i32)> = vec![
            ("5", 5),
            ("10", 10),
            ("-5", -5),
            ("-10", -10),
            ("5 + 5 + 5 + 5 - 10", 10),
            ("2 * 2 * 2 * 2 * 2", 32),
            ("-50 + 100 + -50", 0),
            ("5 * 2 + 10", 20),
            ("5 + 2 * 10", 25),
            ("20 + 2 * -10", 0),
            ("50 / 2 * 2 + 10", 60),
            ("2 * (5 + 10)", 30),
            ("3 * 3 * 3 + 10", 37),
            ("3 * (3 * 3) + 10", 37),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
        ];

        let asserter = |expected: &i32, input: &&str| {
            let object = eval::eval(input).expect("Eval failed");

            match object {
                Object::Integer(number) => assert_eq!(&number, expected),
                Object::Boolean(boolean) => {
                    panic!("Should have returned a number, instead got {boolean}")
                }
            }
        };

        test_util::assert_list(input_expected, asserter);
    }

    #[test]
    fn eval_boolean_expression_test() {
        let input_expected: Vec<(&str, bool)> = vec![
            ("true", true),
            ("false", false),
            ("1 < 2", true),
            ("1 > 2", false),
            ("1 < 1", false),
            ("1 > 1", false),
            ("1 == 1", true),
            ("1 != 1", false),
            ("1 == 2", false),
            ("1 != 2", true),
            ("true == true", true),
            ("false == false", true),
            ("true == false", false),
            ("true != false", true),
            ("false != true", true),
            ("(1 < 2) == true", true),
            ("(1 < 2) == false", false),
            ("(1 > 2) == true", false),
            ("(1 > 2) == false", true),
        ];

        let asserter = |expected: &bool, input: &&str| {
            let object = eval::eval(input).expect("Eval failed");

            match object {
                Object::Boolean(boolean) => assert_eq!(expected, &boolean),
                something_else => panic!("Expected boolean, got {something_else}"),
            }
        };

        test_util::assert_list(input_expected, asserter);
    }

    #[test]
    fn eval_bang_operator_test() {
        let input_expected: Vec<(&str, bool)> = vec![
            ("!true", false),
            ("!false", true),
            ("!!true", true),
            ("!!false", false),
        ];

        test_util::assert_list(input_expected, |expected: &bool, input: &&str| {
            let object = eval::eval(input).expect("Eval failed");

            match object {
                Object::Boolean(boolean) => assert_eq!(expected, &boolean),
                something_else => panic!("Expected boolean, got {something_else}"),
            }
        });
    }
}
