use crate::lexer::Token;
use crate::ast::*;
use nom::{
    IResult,
    error::{Error, ErrorKind},
};
use std::ops::Range;

pub type TokenSpan = (Token, Range<usize>);
pub type ParseResult<'a, T> = IResult<&'a [TokenSpan], T>;

fn token<'a>(expected: Token) -> impl Fn(&'a [TokenSpan]) -> ParseResult<'a, Token> {
    move |input: &[TokenSpan]| {
        if let Some(((t, _), rest)) = input.split_first() {
            if *t == expected {
                Ok((rest, t.clone()))
            } else {
                Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)))
            }
        } else {
            Err(nom::Err::Error(Error::new(input, ErrorKind::Eof)))
        }
    }
}

fn ident(input: &[TokenSpan]) -> ParseResult<'_, String> {
    if let Some(((Token::Ident(name), _), rest)) = input.split_first() {
        Ok((rest, name.clone()))
    } else {
        Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn int_lit(input: &[TokenSpan]) -> ParseResult<'_, i64> {
    if let Some(((Token::Int(val), _), rest)) = input.split_first() {
        Ok((rest, *val))
    } else {
        Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn parse_type(input: &[TokenSpan]) -> ParseResult<'_, Type> {
    if let Ok((rest, _)) = token(Token::Ampersand)(input) {
        let (rest, inner) = parse_type(rest)?;
        return Ok((rest, Type::SharedRef(Box::new(inner))));
    }
    if let Ok((rest, _)) = token(Token::Tilde)(input) {
        let (rest, inner) = parse_type(rest)?;
        return Ok((rest, Type::UniqueRef(Box::new(inner))));
    }
    
    let (current_input, name) = ident(input)?;
    if let Ok((rest, _)) = token(Token::LBracket)(current_input) {
        let mut generics = Vec::new();
        let mut inner_input = rest;
        while let Ok((next_input, ty)) = parse_type(inner_input) {
            generics.push(ty);
            inner_input = next_input;
            if let Ok((next_input, _)) = token(Token::Comma)(inner_input) {
                inner_input = next_input;
            } else {
                break;
            }
        }
        let (input, _) = token(Token::RBracket)(inner_input)?;
        Ok((input, Type::Generic(name, generics)))
    } else {
        Ok((current_input, Type::Simple(name)))
    }
}

fn float_lit(input: &[TokenSpan]) -> ParseResult<'_, f64> {
    if let Some(((Token::Float(val), _), rest)) = input.split_first() {
        Ok((rest, *val))
    } else {
        Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)))
    }
}

fn primary_expression(input: &[TokenSpan]) -> ParseResult<'_, Expression> {
    if let Ok((rest, _)) = token(Token::Minus)(input) {
        let (rest, expr) = primary_expression(rest)?;
        return Ok((rest, Expression::BinaryOp(Box::new(Expression::Literal(Literal::Int(0))), BinaryOp::Sub, Box::new(expr))));
    }
    if let Ok((rest, val)) = float_lit(input) {
        return Ok((rest, Expression::Literal(Literal::Float(val))));
    }
    if let Ok((rest, val)) = int_lit(input) {
        return Ok((rest, Expression::Literal(Literal::Int(val))));
    }
    if let Some(((Token::String(s), _), rest)) = input.split_first() {
        return Ok((rest, Expression::Literal(Literal::String(s.clone()))));
    }
    if let Ok((rest, _)) = token(Token::LBracket)(input) {
        let mut items = Vec::new();
        let mut current_input = rest;
        while let Ok((next_input, expr)) = expression(current_input) {
            items.push(expr);
            current_input = next_input;
            if let Ok((next_input, _)) = token(Token::Comma)(current_input) {
                current_input = next_input;
            } else {
                break;
            }
        }
        let (input, _) = token(Token::RBracket)(current_input)?;
        return Ok((input, Expression::Literal(Literal::List(items))));
    }
    if let Ok((rest, _)) = token(Token::Dollar)(input) {
        let (rest, name) = ident(rest)?;
        let (rest, _) = token(Token::LParen)(rest)?;
        let mut args = Vec::new();
        let mut current_input = rest;
        while let Ok((next_input, arg)) = expression(current_input) {
            args.push(arg);
            current_input = next_input;
            if let Ok((next_input, _)) = token(Token::Comma)(current_input) {
                current_input = next_input;
            } else {
                break;
            }
        }
        let (input, _) = token(Token::RParen)(current_input)?;
        return Ok((input, Expression::MacroCall(name, args)));
    }
    if let Ok((rest, _)) = token(Token::Move)(input) {
        let (rest, expr) = expression(rest)?;
        return Ok((rest, Expression::Move(Box::new(expr))));
    }
    if let Ok((rest, name)) = ident(input) {
        let mut current_input = rest;
        let mut expr = Expression::Ident(name);

        loop {
            if let Ok((next_input, _)) = token(Token::Dot)(current_input) {
                let (next_input, member) = ident(next_input)?;
                expr = Expression::MemberAccess(Box::new(expr), member);
                current_input = next_input;
                continue;
            }
            if let Ok((next_input, _)) = token(Token::LBracket)(current_input) {
                 let mut types = Vec::new();
                 let mut inner_input = next_input;
                 while let Ok((next_input, ty)) = parse_type(inner_input) {
                     types.push(ty);
                     inner_input = next_input;
                     if let Ok((next_input, _)) = token(Token::Comma)(inner_input) {
                         inner_input = next_input;
                     } else {
                         break;
                     }
                 }
                 if let Ok((next_input, _)) = token(Token::RBracket)(inner_input) {
                     if let Ok((next_input, _)) = token(Token::LParen)(next_input) {
                         let mut args = Vec::new();
                         let mut arg_input = next_input;
                         while let Ok((next_input, arg)) = expression(arg_input) {
                             args.push(arg);
                             arg_input = next_input;
                             if let Ok((next_input, _)) = token(Token::Comma)(arg_input) {
                                 arg_input = next_input;
                             } else {
                                 break;
                             }
                         }
                         let (final_input, _) = token(Token::RParen)(arg_input)?;
                         expr = Expression::GenericCall(Box::new(expr), types, args);
                         current_input = final_input;
                         continue;
                     }
                 }
            }
            if let Ok((next_input, _)) = token(Token::LParen)(current_input) {
                let mut args = Vec::new();
                let mut arg_input = next_input;
                while let Ok((next_input, arg)) = expression(arg_input) {
                    args.push(arg);
                    arg_input = next_input;
                    if let Ok((next_input, _)) = token(Token::Comma)(arg_input) {
                        arg_input = next_input;
                    } else {
                        break;
                    }
                }
                let (final_input, _) = token(Token::RParen)(arg_input)?;
                expr = Expression::Call(Box::new(expr), args);
                current_input = final_input;
                continue;
            }
            break;
        }
        return Ok((current_input, expr));
    }
    if let Ok((rest, _)) = token(Token::LParen)(input) {
        let (rest, expr) = expression(rest)?;
        let (rest, _) = token(Token::RParen)(rest)?;
        return Ok((rest, expr));
    }
    Err(nom::Err::Error(Error::new(input, ErrorKind::Tag)))
}

fn multiplicative_expression(input: &[TokenSpan]) -> ParseResult<'_, Expression> {
    let (mut current_input, mut left) = primary_expression(input)?;
    while let Some(((first, _), rest)) = current_input.split_first() {
        let op = match first {
            Token::Star => BinaryOp::Mul,
            Token::Slash => BinaryOp::Div,
            Token::At => BinaryOp::MatMul,
            _ => break,
        };
        let (next_input, right) = primary_expression(rest)?;
        left = Expression::BinaryOp(Box::new(left), op, Box::new(right));
        current_input = next_input;
    }
    Ok((current_input, left))
}

fn additive_expression(input: &[TokenSpan]) -> ParseResult<'_, Expression> {
    let (mut current_input, mut left) = multiplicative_expression(input)?;
    while let Some(((first, _), rest)) = current_input.split_first() {
        let op = match first {
            Token::Plus => BinaryOp::Add,
            Token::Minus => BinaryOp::Sub,
            _ => break,
        };
        let (next_input, right) = multiplicative_expression(rest)?;
        left = Expression::BinaryOp(Box::new(left), op, Box::new(right));
        current_input = next_input;
    }
    Ok((current_input, left))
}

fn comparison_expression(input: &[TokenSpan]) -> ParseResult<'_, Expression> {
    let (mut current_input, mut left) = additive_expression(input)?;
    while let Some(((first, _), rest)) = current_input.split_first() {
        let op = match first {
            Token::Eq => BinaryOp::Eq,
            Token::Ne => BinaryOp::Ne,
            Token::Gt => BinaryOp::Gt,
            Token::Lt => BinaryOp::Lt,
            Token::Ge => BinaryOp::Ge,
            Token::Le => BinaryOp::Le,
            _ => break,
        };
        let (next_input, right) = additive_expression(rest)?;
        left = Expression::BinaryOp(Box::new(left), op, Box::new(right));
        current_input = next_input;
    }
    Ok((current_input, left))
}

fn expression(input: &[TokenSpan]) -> ParseResult<'_, Expression> {
    comparison_expression(input)
}

fn let_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::Let)(input)?;
    let (input, name) = ident(input)?;
    let mut current_input = input;
    let mut ty = None;
    if let Ok((next_input, _)) = token(Token::Colon)(current_input) {
        let (next_input, t) = parse_type(next_input)?;
        ty = Some(t);
        current_input = next_input;
    }
    let (input, _) = token(Token::Assign)(current_input)?;
    let (input, value) = expression(input)?;
    Ok((input, StatementKind::Let { name, ty, value }))
}

fn mut_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::Mut)(input)?;
    let (input, name) = ident(input)?;
    let mut current_input = input;
    let mut ty = None;
    if let Ok((next_input, _)) = token(Token::Colon)(current_input) {
        let (next_input, t) = parse_type(next_input)?;
        ty = Some(t);
        current_input = next_input;
    }
    let (input, _) = token(Token::Assign)(current_input)?;
    let (input, value) = expression(input)?;
    Ok((input, StatementKind::Mut { name, ty, value }))
}

fn block(input: &[TokenSpan]) -> ParseResult<'_, Vec<Statement>> {
    let (input, _) = token(Token::Indent)(input)?;
    let mut statements = Vec::new();
    let mut current_input = input;
    while let Some(((first, _), _)) = current_input.split_first() {
        if *first == Token::Dedent {
            break;
        }
        let (next_input, stmt) = statement(current_input)?;
        statements.push(stmt);
        current_input = next_input;
    }
    let (input, _) = token(Token::Dedent)(current_input)?;
    Ok((input, statements))
}

fn def_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::Def)(input)?;
    let (input, name) = ident(input)?;
    let (input, _) = token(Token::LParen)(input)?;
    
    let mut params = Vec::new();
    let mut current_input = input;
    while let Ok((next_input, p_name)) = ident(current_input) {
        let mut p_ty = None;
        let mut inner_input = next_input;
        if let Ok((next_input, _)) = token(Token::Colon)(inner_input) {
            let (next_input, ty) = parse_type(next_input)?;
            p_ty = Some(ty);
            inner_input = next_input;
        }
        params.push(Param { name: p_name, ty: p_ty });
        current_input = inner_input;
        if let Ok((next_input, _)) = token(Token::Comma)(current_input) {
            current_input = next_input;
        } else {
            break;
        }
    }
    let (input, _) = token(Token::RParen)(current_input)?;
    
    let mut current_input = input;
    let mut return_ty = None;
    if let Ok((next_input, _)) = token(Token::Arrow)(current_input) {
        let (next_input, ty) = parse_type(next_input)?;
        return_ty = Some(ty);
        current_input = next_input;
    }
    
    let (input, _) = token(Token::Colon)(current_input)?;
    let (input, body) = block(input)?;
    Ok((input, StatementKind::Def { name, params, return_ty, body }))
}

fn return_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::Return)(input)?;
    if let Ok((input, expr)) = expression(input) {
        Ok((input, StatementKind::Return(Some(expr))))
    } else {
        Ok((input, StatementKind::Return(None)))
    }
}

fn if_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::If)(input)?;
    let (input, condition) = expression(input)?;
    let (input, _) = token(Token::Colon)(input)?;
    let (input, then_block) = block(input)?;
    Ok((input, StatementKind::If { condition, then_block, else_block: None }))
}

fn pyimport_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::PyImport)(input)?;
    let (input, _) = token(Token::Colon)(input)?;
    let (input, _) = token(Token::Indent)(input)?;
    
    // For now, just collect all tokens until Dedent
    let mut current_input = input;
    while let Some(((t, _), _)) = current_input.split_first() {
        if *t == Token::Dedent {
            break;
        }
        current_input = &current_input[1..];
    }
    let (input, _) = token(Token::Dedent)(current_input)?;
    Ok((input, StatementKind::PyImport("TODO: PyO3 integration".to_string())))
}

fn for_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::For)(input)?;
    let (input, var) = ident(input)?;
    let (input, _) = token(Token::In)(input)?;
    let (input, iterable) = expression(input)?;
    let (input, _) = token(Token::Colon)(input)?;
    let (input, body) = block(input)?;
    Ok((input, StatementKind::For { var, iterable, body }))
}

fn statement(input: &[TokenSpan]) -> ParseResult<'_, Statement> {
    let start_span = input.first().map(|(_, s)| s.clone()).unwrap_or(0..0);
    
    let (rest, kind) = if let Ok((rest, kind)) = let_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = mut_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = def_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = return_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = if_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = for_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = pyimport_statement(input) {
        (rest, kind)
    } else {
        let (rest, expr) = expression(input)?;
        (rest, StatementKind::Expr(expr))
    };

    let end_span = input.get(input.len() - rest.len() - 1).map(|(_, s)| s.clone()).unwrap_or(start_span.clone());
    let span = start_span.start..end_span.end;

    Ok((rest, Statement { kind, span }))
}

pub fn parse_program(input: &[TokenSpan]) -> ParseResult<'_, Program> {
    let mut statements = Vec::new();
    let mut current_input = input;
    while !current_input.is_empty() {
        if let Some(((Token::Dedent, _), rest)) = current_input.split_first() {
            current_input = rest;
            continue;
        }
        match statement(current_input) {
            Ok((next_input, stmt)) => {
                statements.push(stmt);
                current_input = next_input;
            }
            Err(e) => return Err(e),
        }
    }
    Ok((current_input, Program { statements }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

        #[test]

        fn test_parse_let() {

            let input = "let x = 10";

            let lexer = Lexer::new(input);

            let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();

            let (_, program) = parse_program(&tokens).unwrap();

            assert_eq!(program.statements.len(), 1);

            assert_eq!(program.statements[0].span, 0..10);

        }

    

        #[test]

        fn test_parse_def() {

            let input = "def foo(x, y):\n    let z = x\n    z";

            let lexer = Lexer::new(input);

            let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();

            let (_, program) = parse_program(&tokens).unwrap();

            assert_eq!(program.statements.len(), 1);

        }

    

        #[test]

        fn test_parse_factorial() {

            let input = "

    def factorial(n):

        if n == 0:

            return 1

        return n * factorial(n - 1)

    ";

            let lexer = Lexer::new(input.trim());

            let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();

            let (_, program) = parse_program(&tokens).unwrap();

            assert_eq!(program.statements.len(), 1);

        }

    

        #[test]

        fn test_parse_types() {

            let input = "let x: List[i32] = [1, 2, 3]";

            let lexer = Lexer::new(input);

            let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();

            let (_, program) = parse_program(&tokens).unwrap();

            match &program.statements[0].kind {

                StatementKind::Let { ty, .. } => {

                    assert!(matches!(ty, Some(Type::Generic(name, _)) if name == "List"));

                }

                _ => panic!("Expected Let statement"),

            }

        }

    

        #[test]

        fn test_parse_borrows() {

            let input = "def foo(x: &i32, y: ~List[f32]) -> i32:\n    return 0";

            let lexer = Lexer::new(input);

            let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();

            let (_, program) = parse_program(&tokens).unwrap();

            match &program.statements[0].kind {

                StatementKind::Def { params, .. } => {

                    assert!(matches!(params[0].ty, Some(Type::SharedRef(_))));

                    assert!(matches!(params[1].ty, Some(Type::UniqueRef(_))));

                }

                _ => panic!("Expected Def statement"),

            }

        }

    }

    