use crate::ast::*;
use crate::lexer::Token;
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
        if let Expression::Literal(Literal::Int(i)) = expr {
            return Ok((rest, Expression::Literal(Literal::Int(-i))));
        }
        if let Expression::Literal(Literal::Float(f)) = expr {
            return Ok((rest, Expression::Literal(Literal::Float(-f))));
        }
        return Ok((
            rest,
            Expression::BinaryOp(
                Box::new(Expression::Literal(Literal::Int(0))),
                BinaryOp::Sub,
                Box::new(expr),
            ),
        ));
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

        let mut has_indent = false;
        if let Ok((next_input, _)) = token(Token::Indent)(current_input) {
            current_input = next_input;
            has_indent = true;
        }

        while let Ok((next_input, expr)) = expression(current_input) {
            items.push(expr);
            current_input = next_input;
            if let Ok((next_input, _)) = token(Token::Comma)(current_input) {
                current_input = next_input;
            } else {
                break;
            }
        }

        if has_indent {
            let (next_input, _) = token(Token::Dedent)(current_input)?;
            current_input = next_input;
        }

        let (input, _) = token(Token::RBracket)(current_input)?;
        return Ok((input, Expression::Literal(Literal::List(items))));
    }
    if let Ok((rest, _)) = token(Token::Dollar)(input) {
        let (rest, name) = ident(rest)?;
        let (rest, _) = token(Token::LParen)(rest)?;
        let mut args = Vec::new();
        let mut current_input = rest;

        let mut has_indent = false;
        if let Ok((next_input, _)) = token(Token::Indent)(current_input) {
            current_input = next_input;
            has_indent = true;
        }

        while let Ok((next_input, arg)) = expression(current_input) {
            args.push(arg);
            current_input = next_input;
            if let Ok((next_input, _)) = token(Token::Comma)(current_input) {
                current_input = next_input;
            } else {
                break;
            }
        }

        if has_indent {
            let (next_input, _) = token(Token::Dedent)(current_input)?;
            current_input = next_input;
        }

        let (input, _) = token(Token::RParen)(current_input)?;
        return Ok((input, Expression::MacroCall(name, args)));
    }
    if let Ok((rest, _)) = token(Token::Ampersand)(input) {
        let (rest, expr) = expression(rest)?;
        return Ok((rest, Expression::SharedRef(Box::new(expr))));
    }
    if let Ok((rest, _)) = token(Token::Tilde)(input) {
        let (rest, expr) = expression(rest)?;
        return Ok((rest, Expression::UniqueRef(Box::new(expr))));
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
                // Try GenericCall first
                let mut types = Vec::new();
                let mut inner_input = next_input;
                let mut generic_call_parsed = false;

                while let Ok((next_input, ty)) = parse_type(inner_input) {
                    types.push(ty);
                    inner_input = next_input;
                    if let Ok((next_input, _)) = token(Token::Comma)(inner_input) {
                        inner_input = next_input;
                    } else {
                        break;
                    }
                }

                if !types.is_empty()
                    && let Ok((after_bracket, _)) = token(Token::RBracket)(inner_input)
                    && let Ok((after_paren, _)) = token(Token::LParen)(after_bracket)
                {
                    let mut args = Vec::new();
                    let mut arg_input = after_paren;
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
                    generic_call_parsed = true;
                }

                if generic_call_parsed {
                    continue;
                }

                // If not a GenericCall, try Indexing
                if let Ok((inner_input, index_expr)) = expression(next_input) {
                    let (final_input, _) = token(Token::RBracket)(inner_input)?;
                    expr = Expression::Index(Box::new(expr), Box::new(index_expr));
                    current_input = final_input;
                    continue;
                }
            }
            if let Ok((next_input, _)) = token(Token::LParen)(current_input) {
                let mut args = Vec::new();
                let mut arg_input = next_input;

                let mut has_indent = false;
                if let Ok((next_input, _)) = token(Token::Indent)(arg_input) {
                    arg_input = next_input;
                    has_indent = true;
                }

                while let Ok((next_input, arg)) = expression(arg_input) {
                    args.push(arg);
                    arg_input = next_input;
                    if let Ok((next_input, _)) = token(Token::Comma)(arg_input) {
                        arg_input = next_input;
                    } else {
                        break;
                    }
                }

                if has_indent {
                    let (next_input, _) = token(Token::Dedent)(arg_input)?;
                    arg_input = next_input;
                }

                let (final_input, _) = token(Token::RParen)(arg_input)?;
                expr = Expression::Call(Box::new(expr), args);
                current_input = final_input;
                continue;
            }
            if let Ok((next_input, _)) = token(Token::Question)(current_input) {
                expr = Expression::Question(Box::new(expr));
                current_input = next_input;
                continue;
            }
            if let Ok((next_input, _)) = token(Token::BangBang)(current_input) {
                expr = Expression::Unwrap(Box::new(expr));
                current_input = next_input;
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

fn assignment_expression(input: &[TokenSpan]) -> ParseResult<'_, Expression> {
    let (input, left) = comparison_expression(input)?;
    if let Ok((rest, _)) = token(Token::Assign)(input) {
        let (rest, right) = assignment_expression(rest)?;
        return Ok((
            rest,
            Expression::BinaryOp(Box::new(left), BinaryOp::Assign, Box::new(right)),
        ));
    }
    Ok((input, left))
}

fn expression(input: &[TokenSpan]) -> ParseResult<'_, Expression> {
    assignment_expression(input)
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

fn import_path(input: &[TokenSpan]) -> ParseResult<'_, String> {
    if let Some(((Token::String(path), _), rest)) = input.split_first() {
        return Ok((rest, path.clone()));
    }

    let (mut current_input, first) = ident(input)?;
    let mut parts = vec![first];
    while let Ok((next_input, _)) = token(Token::Dot)(current_input) {
        let (next_input, part) = ident(next_input)?;
        parts.push(part);
        current_input = next_input;
    }

    Ok((current_input, parts.join("/")))
}

fn import_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::Import)(input)?;
    let (input, path) = import_path(input)?;

    let mut alias = None;
    let mut current_input = input;
    if let Ok((next_input, _)) = token(Token::As)(current_input) {
        let (next_input, name) = ident(next_input)?;
        alias = Some(name);
        current_input = next_input;
    }

    Ok((
        current_input,
        StatementKind::Import {
            path,
            alias,
        },
    ))
}

fn from_import_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::From)(input)?;
    let (input, path) = import_path(input)?;
    let (input, _) = token(Token::Import)(input)?;

    let (mut current_input, first_name) = ident(input)?;
    let mut items = Vec::new();

    let mut first_alias = None;
    if let Ok((next_input, _)) = token(Token::As)(current_input) {
        let (next_input, alias) = ident(next_input)?;
        first_alias = Some(alias);
        current_input = next_input;
    }
    items.push(ImportItem {
        name: first_name,
        alias: first_alias,
    });

    while let Ok((next_input, _)) = token(Token::Comma)(current_input) {
        let (next_input, name) = ident(next_input)?;
        let mut alias = None;
        let mut next_cursor = next_input;
        if let Ok((alias_input, _)) = token(Token::As)(next_cursor) {
            let (alias_input, alias_name) = ident(alias_input)?;
            alias = Some(alias_name);
            next_cursor = alias_input;
        }
        items.push(ImportItem { name, alias });
        current_input = next_cursor;
    }

    Ok((current_input, StatementKind::FromImport { path, items }))
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
    while let Some(((first_token, _), _)) = current_input.split_first() {
        let mut is_mut = false;
        let mut inner_input = current_input;
        if *first_token == Token::Mut {
            is_mut = true;
            inner_input = &current_input[1..];
        }

        if let Ok((next_input, p_name)) = ident(inner_input) {
            let mut p_ty = None;
            let mut inner_input = next_input;
            if let Ok((next_input, _)) = token(Token::Colon)(inner_input) {
                let (next_input, ty) = parse_type(next_input)?;
                p_ty = Some(ty);
                inner_input = next_input;
            }
            params.push(Param {
                name: p_name,
                ty: p_ty,
                is_mut,
            });
            current_input = inner_input;
            if let Ok((next_input, _)) = token(Token::Comma)(current_input) {
                current_input = next_input;
            } else {
                break;
            }
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
    Ok((
        input,
        StatementKind::Def {
            name,
            params,
            return_ty,
            body,
        },
    ))
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

    let mut current_input = input;
    let mut else_block = None;
    if let Ok((next_input, _)) = token(Token::Else)(current_input) {
        let (next_input, _) = token(Token::Colon)(next_input)?;
        let (next_input, block) = block(next_input)?;
        else_block = Some(block);
        current_input = next_input;
    }

    Ok((
        current_input,
        StatementKind::If {
            condition,
            then_block,
            else_block,
        },
    ))
}

fn struct_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::Struct)(input)?;
    let (input, name) = ident(input)?;
    let (input, _) = token(Token::Colon)(input)?;
    let (input, _) = token(Token::Indent)(input)?;

    let mut fields = Vec::new();
    let mut current_input = input;
    while let Ok((next_input, f_name)) = ident(current_input) {
        let (next_input, _) = token(Token::Colon)(next_input)?;
        let (next_input, f_ty) = parse_type(next_input)?;
        fields.push(Param {
            name: f_name,
            ty: Some(f_ty),
            is_mut: false,
        });
        current_input = next_input;
    }

    let (input, _) = token(Token::Dedent)(current_input)?;
    Ok((input, StatementKind::Struct { name, fields }))
}

fn protocol_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::Protocol)(input)?;
    let (input, name) = ident(input)?;
    let (input, _) = token(Token::Colon)(input)?;
    let (input, methods) = block(input)?;
    Ok((input, StatementKind::Protocol { name, methods }))
}

fn impl_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::Impl)(input)?;

    let (input, name1) = ident(input)?;
    let current_input = input;

    if let Ok((next_input, _)) = token(Token::For)(current_input) {
        let (next_input, name2) = ident(next_input)?;
        let (next_input, _) = token(Token::Colon)(next_input)?;
        let (next_input, methods) = block(next_input)?;
        Ok((
            next_input,
            StatementKind::Impl {
                protocol: Some(name1),
                for_type: name2,
                methods,
            },
        ))
    } else {
        let (input, _) = token(Token::Colon)(current_input)?;
        let (input, methods) = block(input)?;
        Ok((
            input,
            StatementKind::Impl {
                protocol: None,
                for_type: name1,
                methods,
            },
        ))
    }
}

fn pyimport_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::PyImport)(input)?;
    let (input, _) = token(Token::Colon)(input)?;
    let (input, _) = token(Token::Indent)(input)?;

    // Collect all tokens until Dedent.
    let mut current_input = input;
    let mut content = String::new();
    let mut first = true;
    while let Some(((t, _), next_tokens)) = current_input.split_first() {
        if *t == Token::Dedent {
            break;
        }
        if *t == Token::Indent {
            current_input = next_tokens;
            continue;
        }
        if !first {
            content.push(' ');
        }
        content.push_str(&token_text(t));
        current_input = next_tokens;
        first = false;
    }
    let (input, _) = token(Token::Dedent)(current_input)?;
    Ok((input, StatementKind::PyImport(content)))
}

fn token_text(token: &Token) -> String {
    match token {
        Token::Let => "let".to_string(),
        Token::Mut => "mut".to_string(),
        Token::Move => "move".to_string(),
        Token::Def => "def".to_string(),
        Token::Protocol => "protocol".to_string(),
        Token::Struct => "struct".to_string(),
        Token::Impl => "impl".to_string(),
        Token::For => "for".to_string(),
        Token::In => "in".to_string(),
        Token::If => "if".to_string(),
        Token::Else => "else".to_string(),
        Token::Return => "return".to_string(),
        Token::Import => "import".to_string(),
        Token::PyImport => "pyimport".to_string(),
        Token::Match => "match".to_string(),
        Token::As => "as".to_string(),
        Token::From => "from".to_string(),
        Token::Colon => ":".to_string(),
        Token::Arrow => "->".to_string(),
        Token::Question => "?".to_string(),
        Token::BangBang => "!!".to_string(),
        Token::Dot => ".".to_string(),
        Token::LBracket => "[".to_string(),
        Token::RBracket => "]".to_string(),
        Token::LParen => "(".to_string(),
        Token::RParen => ")".to_string(),
        Token::Comma => ",".to_string(),
        Token::Assign => "=".to_string(),
        Token::Eq => "==".to_string(),
        Token::Ne => "!=".to_string(),
        Token::Gt => ">".to_string(),
        Token::Lt => "<".to_string(),
        Token::Ge => ">=".to_string(),
        Token::Le => "<=".to_string(),
        Token::At => "@".to_string(),
        Token::Dollar => "$".to_string(),
        Token::Ampersand => "&".to_string(),
        Token::Tilde => "~".to_string(),
        Token::Plus => "+".to_string(),
        Token::Minus => "-".to_string(),
        Token::Star => "*".to_string(),
        Token::Slash => "/".to_string(),
        Token::Ident(name) => name.clone(),
        Token::Float(value) => value.to_string(),
        Token::Int(value) => value.to_string(),
        Token::String(value) => format!("\"{}\"", value),
        Token::NewlineWithIndent => "\\n".to_string(),
        Token::Whitespace => " ".to_string(),
        Token::Comment => "#".to_string(),
        Token::Indent => "<indent>".to_string(),
        Token::Dedent => "<dedent>".to_string(),
    }
}

fn match_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::Match)(input)?;
    let (input, expr) = expression(input)?;
    let (input, _) = token(Token::Colon)(input)?;
    let (input, _) = token(Token::Indent)(input)?;

    let mut arms = Vec::new();
    let mut current_input = input;

    while let Ok((next_input, pattern)) = expression(current_input) {
        let (next_input, _) = token(Token::Colon)(next_input)?;
        let (next_input, body) = block(next_input)?;
        arms.push((pattern, body));
        current_input = next_input;

        if let Some(((Token::Dedent, _), _)) = current_input.split_first() {
            break;
        }
    }

    let (input, _) = token(Token::Dedent)(current_input)?;
    Ok((
        input,
        StatementKind::Match {
            expression: expr,
            arms,
        },
    ))
}

fn for_statement(input: &[TokenSpan]) -> ParseResult<'_, StatementKind> {
    let (input, _) = token(Token::For)(input)?;
    let (input, var) = ident(input)?;
    let (input, _) = token(Token::In)(input)?;
    let (input, iterable) = expression(input)?;
    let (input, _) = token(Token::Colon)(input)?;
    let (input, body) = block(input)?;
    Ok((
        input,
        StatementKind::For {
            var,
            iterable,
            body,
        },
    ))
}

fn statement(input: &[TokenSpan]) -> ParseResult<'_, Statement> {
    let start_span = input.first().map(|(_, s)| s.clone()).unwrap_or(0..0);

    let (rest, kind) = if let Ok((rest, kind)) = let_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = mut_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = from_import_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = import_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = def_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = return_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = if_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = for_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = struct_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = protocol_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = impl_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = pyimport_statement(input) {
        (rest, kind)
    } else if let Ok((rest, kind)) = match_statement(input) {
        (rest, kind)
    } else {
        let (rest, expr) = expression(input)?;
        (rest, StatementKind::Expr(expr))
    };

    let end_span = input
        .get(input.len() - rest.len() - 1)
        .map(|(_, s)| s.clone())
        .unwrap_or(start_span.clone());
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

    #[test]
    fn test_parse_if_else() {
        let input = "if x > 0:\n    return 1\nelse:\n    return 0";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        match &program.statements[0].kind {
            StatementKind::If { else_block, .. } => {
                assert!(else_block.is_some());
            }
            _ => panic!("Expected If statement"),
        }
    }

    #[test]
    fn test_parse_import_string_path() {
        let input = "import \"utils/math.ds\"";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();

        match &program.statements[0].kind {
            StatementKind::Import { path, alias } => {
                assert_eq!(path, "utils/math.ds");
                assert!(alias.is_none());
            }
            _ => panic!("Expected Import statement"),
        }
    }

    #[test]
    fn test_parse_import_dotted_path() {
        let input = "import utils.math";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();

        match &program.statements[0].kind {
            StatementKind::Import { path, alias } => {
                assert_eq!(path, "utils/math");
                assert!(alias.is_none());
            }
            _ => panic!("Expected Import statement"),
        }
    }

    #[test]
    fn test_parse_import_alias() {
        let input = "import rust.std.collections.HashMap as Map";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();

        match &program.statements[0].kind {
            StatementKind::Import { path, alias } => {
                assert_eq!(path, "rust/std/collections/HashMap");
                assert_eq!(alias.as_deref(), Some("Map"));
            }
            _ => panic!("Expected Import statement"),
        }
    }

    #[test]
    fn test_parse_from_import_with_aliases() {
        let input = "from rust.std.cmp import max as maximum, min";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();

        match &program.statements[0].kind {
            StatementKind::FromImport { path, items } => {
                assert_eq!(path, "rust/std/cmp");
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].name, "max");
                assert_eq!(items[0].alias.as_deref(), Some("maximum"));
                assert_eq!(items[1].name, "min");
                assert!(items[1].alias.is_none());
            }
            _ => panic!("Expected Import statement"),
        }
    }

    #[test]
    fn test_parse_struct() {
        let input = "struct Point:\n    x: i32\n    y: i32";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        match &program.statements[0].kind {
            StatementKind::Struct { name, fields } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
            }
            _ => panic!("Expected Struct statement"),
        }
    }

    #[test]
    fn test_parse_protocol() {
        let input = "protocol Speak:\n    def talk(self) -> Str:\n        return \"\"";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        match &program.statements[0].kind {
            StatementKind::Protocol { name, .. } => {
                assert_eq!(name, "Speak");
            }
            _ => panic!("Expected Protocol statement"),
        }
    }

    #[test]
    fn test_parse_impl() {
        let input = "impl Speak for Dog:\n    def talk(self) -> Str:\n        return \"Woof\"";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        match &program.statements[0].kind {
            StatementKind::Impl {
                protocol, for_type, ..
            } => {
                assert_eq!(protocol, &Some("Speak".to_string()));
                assert_eq!(for_type, "Dog");
            }
            _ => panic!("Expected Impl statement"),
        }
    }

    #[test]
    fn test_parse_error_ops() {
        let input = "let x = foo()?\nlet y = bar()!!";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        match &program.statements[0].kind {
            StatementKind::Let { value, .. } => {
                assert!(matches!(value, Expression::Question(_)));
            }
            _ => panic!("Expected Let with Question"),
        }
        match &program.statements[1].kind {
            StatementKind::Let { value, .. } => {
                assert!(matches!(value, Expression::Unwrap(_)));
            }
            _ => panic!("Expected Let with Unwrap"),
        }
    }
    #[test]
    fn test_parse_pyimport() {
        let input = "pyimport:\n    import torch\n    import numpy as np";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        match &program.statements[0].kind {
            StatementKind::PyImport(content) => {
                assert!(content.contains("import torch"));
                assert!(content.contains("import numpy as np"));
            }
            _ => panic!("Expected PyImport statement"),
        }
    }

    #[test]
    fn test_parse_complex_expressions() {
        let input = "let x = (a + b) * (c - d) @ e.f[T](g)";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_negative_float_literal() {
        let input = "let x = -0.2";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        match &program.statements[0].kind {
            StatementKind::Let { value, .. } => {
                assert!(
                    matches!(value, Expression::Literal(Literal::Float(v)) if (*v - (-0.2)).abs() < f64::EPSILON)
                );
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_parse_nested_control_flow() {
        let input = "
if x:
    for i in list:
        if i > 0:
            $print(i)
else:
    $print(\"none\")
";
        let lexer = Lexer::new(input.trim());
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_keywords() {
        let input = "
let x = move data
let y = &x
let z = ~y
mut w = 10
";
        let lexer = Lexer::new(input.trim());
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        assert_eq!(program.statements.len(), 4);
    }

    #[test]
    fn test_parse_impl_basic() {
        let input = "impl Foo:\n    def bar(self):\n        return 1";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_match_basic() {
        let input = "match x:\n    Some(v):\n        $print(v)\n    None:\n        return";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_parse_borrow_binding_expression() {
        let input = "let x: &i32 = &y\nlet y: ~i32 = ~z";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        assert_eq!(program.statements.len(), 2);
    }

    #[test]
    fn test_parse_ref_as_identifier() {
        let input = "let ref = 1";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        match &program.statements[0].kind {
            StatementKind::Let { name, .. } => assert_eq!(name, "ref"),
            _ => panic!("Expected Let statement"),
        }
    }
}
