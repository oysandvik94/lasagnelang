use std::sync::Once;

use tracing::error;
use tracing_subscriber::FmtSubscriber;

use crate::{
    eval::{
        self,
        objects::{Environment, Object},
        EvaledProgram,
    },
    parser::{
        ast::{BlockStatement, Identifier, Operator, Statement},
        expressions::{
            expression::Expression, expression_statement::ExpressionStatement,
            functions::FunctionLiteral, if_expression::IfExpression,
        },
        lexer::lexedtokens::LexedTokens,
    },
};

use super::{ast::PrefixOperator, ParsedProgram, Parser};

pub fn assert_list<T, K, F>(test_cases: Vec<(T, K)>, mut asserter: F)
where
    F: FnMut(&K, &T),
    K: PartialEq + std::fmt::Debug,
{
    test_cases.iter().for_each(|(input, expected)| {
        asserter(expected, input);
    });
}

pub fn has_parser_errors(program: &ParsedProgram) -> bool {
    match program {
        ParsedProgram::ValidProgram(_) => false,
        ParsedProgram::InvalidProgram(parse_errors) => {
            eprintln!("Found parser errors:");
            for parse_error in parse_errors {
                eprintln!("parser error: {parse_error}");
            }

            true
        }
    }
}

pub fn parse_program(source_code: &str) -> ParsedProgram {
    let tokens = LexedTokens::from(source_code);
    Parser::parse_tokens(tokens)
}

pub fn expect_evaled_program(source_code: &str) -> Object {
    match eval::eval(source_code, &mut Environment::new_env_reference()) {
        EvaledProgram::ParseError(parse_errors) => {
            parse_errors.into_iter().for_each(|ele| {
                eprintln!("{ele}");
            });
            panic!("Eval failed with parse errors");
        }
        EvaledProgram::EvalError(eval_errors) => {
            error!("{eval_errors}");
            panic!("Eval failed with runtime errors")
        }
        EvaledProgram::Valid(valid_program) => valid_program,
    }
}

pub fn expect_parsed_program(source_code: &str) -> Vec<Statement> {
    let tokens = LexedTokens::from(source_code);
    match Parser::parse_tokens(tokens) {
        ParsedProgram::ValidProgram(valid_statements) => valid_statements,
        ParsedProgram::InvalidProgram(parse_errors) => {
            parse_errors.into_iter().for_each(|ele| {
                error!("{ele}");
            });
            panic!("Eval failed with parse errors")
        }
    }
}

pub fn create_prefix_test_case(
    right_expression: Expression,
    operator: PrefixOperator,
) -> Statement {
    Statement::Expression(ExpressionStatement {
        expression: Expression::Prefix {
            right: Box::new(right_expression),
            operator,
        },
    })
}

pub fn create_infix_test_case(
    left_expression: Expression,
    right_expression: Expression,
    operator: Operator,
) -> Statement {
    Statement::Expression(ExpressionStatement {
        expression: Expression::Infix {
            left: Box::new(left_expression),
            right: Box::new(right_expression),
            operator,
        },
    })
}

pub fn create_function_expression(parameters: Vec<&str>, body: BlockStatement) -> Statement {
    Statement::Expression(ExpressionStatement {
        expression: Expression::Function(FunctionLiteral {
            parameters: parameters
                .iter()
                .map(|param| Identifier(param.to_string()))
                .collect(),
            body,
        }),
    })
}
pub fn create_if_condition(
    condition: Expression,
    consequence: BlockStatement,
    alternative: Option<BlockStatement>,
) -> Statement {
    use Expression::*;
    Statement::Expression(ExpressionStatement {
        expression: If(IfExpression {
            condition: Box::from(condition),
            consequence,
            alternative,
        }),
    })
}

pub fn create_infix_expression(
    left: Expression,
    right_expression: Expression,
    operator: Operator,
) -> Expression {
    Expression::Infix {
        right: Box::from(right_expression),
        left: Box::from(left),
        operator,
    }
}

pub fn create_identifierliteral(literal: &str) -> Expression {
    Expression::IdentifierLiteral(Identifier(literal.to_string()))
}

static TRACING: Once = Once::new();
pub fn setup_logger() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();

    TRACING.call_once(|| {
        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
    })
}
