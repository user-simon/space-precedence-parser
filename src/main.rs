use std::{
    fmt::{self, Display, Formatter}, 
    env, 
    cmp::Ordering, 
};
use lexer::*;

mod lexer;

/// The AST structure being parsed
#[derive(Debug)]
enum Ast {
    /// Literal numerical value
    Literal(f64), 
    /// Unary operation
    Unary(String, Box<Ast>), 
    /// Binary operation
    Binary(String, Box<(Ast, Ast)>), 
}

impl Display for Ast {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Ast::Literal(value) => write!(f, "{value}"),
            Ast::Unary(op, x) => write!(f, "({op} {x})"), 
            Ast::Binary(op, args) => {
                let x = &args.0;
                let y = &args.1;
                write!(f, "({x} {op} {y})")
            }, 
        }
    }
}

/// Operation precedence. In addition to the regular algebraic operator precedence, the distance between the
/// operator and the operand is also used. 
#[derive(Clone, Copy, PartialEq)]
struct Precedence {
    spacing: usize, 
    algebraic: usize, 
}

/// If the space between an operand and two operators are equal, the operator with the greatest algebraic
/// precedence is chosen.  
impl PartialOrd for Precedence {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let spacing = other.spacing.cmp(&self.spacing);
        let algebraic = other.algebraic.cmp(&self.algebraic);
        let ordering = match spacing {
            Ordering::Equal => algebraic, 
            _ => spacing, 
        };
        Some(ordering)
    }
}

/// Entry-point to the parsing algorithm. Parses a string into our AST
fn parse(string: &str) -> Option<Ast> {
    let mut tokens = Tokens::from(string);
    let min_precedence = Precedence {
        spacing: usize::MAX,
        algebraic: usize::MAX,
    };
    let expr = parse_expression(&mut tokens, min_precedence)?;
    tokens.next()
        .is_none()
        .then_some(expr)
}

/// Parses our AST from a set of lexical tokens. Based on the operator-precedence parser detailed in 
/// https://en.wikipedia.org/wiki/Operator-precedence_parser
fn parse_expression(tokens: &mut Tokens, min: Precedence) -> Option<Ast> {
    parse_primary(tokens).and_then(|lhs| parse_precedence(lhs, tokens, min))
}

/// Attempts to parse a binary operation from a left-hand side. If the lhs is not proceeded by a binary
/// operation, lhs is transparently returned
fn parse_precedence(mut lhs: Ast, tokens: &mut Tokens, min: Precedence) -> Option<Ast> {
    // attempts to read a binary operator including its precedence from the tokens
    let peek_op = |tokens: &mut Tokens| {
        let &Token::Symbol(op, spacing) = tokens.peek()? else {
            return None
        };
        let algebraic = match op {
            '+' => 2, 
            '-' => 2, 
            '*' => 1, 
            '/' => 1, 
            _ => return None, 
        };
        let prec = Precedence{ spacing, algebraic };
        Some((op, prec))
    };

    // parse all operations above the minimum precedence
    while let Some((op, prec)) = peek_op(tokens).filter(|(_, prec)| prec >= &min) {
        let _ = tokens.next();

        // compute the precedence of the current operator to the rhs parsed below. if the rhs is proceeded by
        // another operator, this is precedence that must be exceeded
        let rhs_prec = Precedence {
            spacing: tokens.peek().map(Token::spacing)?, 
            algebraic: prec.algebraic, 
        };
        let mut rhs = parse_primary(tokens)?;

        // parse all operations proceeding the rhs that are above `rhs_prec`; this becomes the new rhs
        while let Some(_) = peek_op(tokens).filter(|(_, sub_prec)| sub_prec > &rhs_prec) {
            rhs = parse_precedence(rhs, tokens, rhs_prec)?;
        }
        lhs = Ast::Binary(op.into(), Box::new((lhs, rhs)))
    }
    Some(lhs)
}

/// Parses literals and unary operations
fn parse_primary(tokens: &mut Tokens) -> Option<Ast> {
    let token = tokens.next()?;
    let mut parse_unary = |op: &str| {
        let arg_precedence = Precedence {
            spacing: tokens.peek().map(Token::spacing)?,
            algebraic: 0,
        };
        let arg = parse_expression(tokens, arg_precedence)?;
        Some(Ast::Unary(op.into(), Box::new(arg)))
    };
    let expr = match token {
        Token::Number(num, _) => Ast::Literal(num),
        Token::Symbol('-', _) => parse_unary("-")?, 
        Token::Word("sqrt", _) => parse_unary("sqrt")?, 
        _ => return None, 
    };
    Some(expr)
}

fn main() {
    let input = env::args().nth(1).unwrap();
    let expr = parse(&input).unwrap();
    println!("{expr}");
}

#[test]
fn test() {
    fn assert_eq(input: &str, expected: &str) {
        let expr = parse(&input).unwrap();
        let output = format!("{expr}");
        assert_eq!(output, expected);
    }

    assert_eq("1.2 + 3.4", "(1.2 + 3.4)");
    assert_eq("1 * 2+3", "(1 * (2 + 3))");
    assert_eq("1* 2+ 3", "(1 * (2 + 3))");

    assert_eq("1*    3+4   -   5/6",  "(1 * ((3 + 4) - (5 / 6)))");
    assert_eq("1*    3+4    -   5/6", "((1 * (3 + 4)) - (5 / 6))");

    assert_eq("sqrt 1", "(sqrt 1)");
    assert_eq("sqrt sqrt 1 + 1", "((sqrt (sqrt 1)) + 1)");
    assert_eq("sqrt sqrt  1 + 1", "(sqrt (sqrt (1 + 1)))");
    assert_eq("sqrt   sqrt 1 + 1", "(sqrt ((sqrt 1) + 1))");
}
