use crate::ast::*;

pub fn format_program(program: &Program) -> String {
    let mut out = String::new();
    for stmt in &program.statements {
        format_statement(stmt, 0, &mut out);
    }
    out
}

fn format_statement(stmt: &Statement, indent: usize, out: &mut String) {
    let indent_str = "    ".repeat(indent);
    match &stmt.kind {
        StatementKind::Import { path, alias } => {
            if let Some(alias) = alias {
                out.push_str(&format!("{}import \"{}\" as {}\n", indent_str, path, alias));
            } else {
                out.push_str(&format!("{}import \"{}\"\n", indent_str, path));
            }
        }
        StatementKind::FromImport { path, items } => {
            let rendered_items = items
                .iter()
                .map(|item| match &item.alias {
                    Some(alias) => format!("{} as {}", item.name, alias),
                    None => item.name.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            out.push_str(&format!(
                "{}from \"{}\" import {}\n",
                indent_str, path, rendered_items
            ));
        }
        StatementKind::Let { pattern, ty, value } => {
            out.push_str(&format!(
                "{}let {}{} = {}\n",
                indent_str,
                format_pattern(pattern),
                format_type_suffix(ty.as_ref()),
                format_expression(value, 0)
            ));
        }
        StatementKind::Mut { pattern, ty, value } => {
            out.push_str(&format!(
                "{}mut {}{} = {}\n",
                indent_str,
                format_pattern(pattern),
                format_type_suffix(ty.as_ref()),
                format_expression(value, 0)
            ));
        }
        StatementKind::Def {
            name,
            params,
            return_ty,
            body,
        } => {
            let params_str = params
                .iter()
                .map(format_param)
                .collect::<Vec<_>>()
                .join(", ");
            let ret = return_ty
                .as_ref()
                .map(|ty| format!(" -> {}", format_type(ty)))
                .unwrap_or_default();
            out.push_str(&format!(
                "{}def {}({}){}:\n",
                indent_str, name, params_str, ret
            ));
            format_block(body, indent + 1, out);
        }
        StatementKind::If {
            condition,
            then_block,
            else_block,
        } => {
            out.push_str(&format!(
                "{}if {}:\n",
                indent_str,
                format_expression(condition, 0)
            ));
            format_block(then_block, indent + 1, out);
            if let Some(else_block) = else_block {
                out.push_str(&format!("{}else:\n", indent_str));
                format_block(else_block, indent + 1, out);
            }
        }
        StatementKind::For {
            pattern,
            iterable,
            body,
        } => {
            out.push_str(&format!(
                "{}for {} in {}:\n",
                indent_str,
                format_pattern(pattern),
                format_expression(iterable, 0)
            ));
            format_block(body, indent + 1, out);
        }
        StatementKind::While { condition, body } => {
            out.push_str(&format!(
                "{}while {}:\n",
                indent_str,
                format_expression(condition, 0)
            ));
            format_block(body, indent + 1, out);
        }
        StatementKind::Struct { name, fields } => {
            out.push_str(&format!("{}struct {}:\n", indent_str, name));
            for field in fields {
                let ty = field
                    .ty
                    .as_ref()
                    .map(format_type)
                    .unwrap_or_else(|| "()".to_string());
                out.push_str(&format!(
                    "{}{}: {}\n",
                    "    ".repeat(indent + 1),
                    field.name,
                    ty
                ));
            }
        }
        StatementKind::Enum { name, variants } => {
            out.push_str(&format!("{}enum {}:\n", indent_str, name));
            for variant in variants {
                if variant.fields.is_empty() {
                    out.push_str(&format!("{}{}\n", "    ".repeat(indent + 1), variant.name));
                } else {
                    let types_str: Vec<String> = variant.fields.iter().map(format_type).collect();
                    out.push_str(&format!(
                        "{}{}({})\n",
                        "    ".repeat(indent + 1),
                        variant.name,
                        types_str.join(", ")
                    ));
                }
            }
        }
        StatementKind::Protocol { name, methods } => {
            out.push_str(&format!("{}protocol {}:\n", indent_str, name));
            format_block(methods, indent + 1, out);
        }
        StatementKind::Impl {
            protocol,
            for_type,
            methods,
        } => {
            if let Some(protocol) = protocol {
                out.push_str(&format!(
                    "{}impl {} for {}:\n",
                    indent_str, protocol, for_type
                ));
            } else {
                out.push_str(&format!("{}impl {}:\n", indent_str, for_type));
            }
            format_block(methods, indent + 1, out);
        }
        StatementKind::Match { expression, arms } => {
            out.push_str(&format!(
                "{}match {}:\n",
                indent_str,
                format_expression(expression, 0)
            ));
            for (pattern, body) in arms {
                out.push_str(&format!(
                    "{}{}:\n",
                    "    ".repeat(indent + 1),
                    format_expression(pattern, 0)
                ));
                format_block(body, indent + 2, out);
            }
        }
        StatementKind::PyImport(content) => {
            out.push_str(&format!("{}pyimport:\n", indent_str));
            out.push_str(&format!(
                "{}{}\n",
                "    ".repeat(indent + 1),
                content.trim()
            ));
        }
        StatementKind::Return(expr) => {
            if let Some(expr) = expr {
                out.push_str(&format!(
                    "{}return {}\n",
                    indent_str,
                    format_expression(expr, 0)
                ));
            } else {
                out.push_str(&format!("{}return\n", indent_str));
            }
        }
        StatementKind::Break => out.push_str(&format!("{}break\n", indent_str)),
        StatementKind::Continue => out.push_str(&format!("{}continue\n", indent_str)),
        StatementKind::Expr(expr) => {
            out.push_str(&format!("{}{}\n", indent_str, format_expression(expr, 0)));
        }
    }
}

fn format_block(stmts: &[Statement], indent: usize, out: &mut String) {
    for stmt in stmts {
        format_statement(stmt, indent, out);
    }
}

fn format_pattern(pattern: &Pattern) -> String {
    pattern.format_desert()
}

fn format_param(param: &Param) -> String {
    let mut out = String::new();
    if param.is_mut {
        out.push_str("mut ");
    }
    out.push_str(&param.name);
    if let Some(ty) = &param.ty {
        out.push_str(": ");
        out.push_str(&format_type(ty));
    }
    out
}

fn format_type_suffix(ty: Option<&Type>) -> String {
    ty.map(|t| format!(": {}", format_type(t)))
        .unwrap_or_default()
}

fn format_type(ty: &Type) -> String {
    match ty {
        Type::Simple(name) => name.clone(),
        Type::Generic(name, args) => format!(
            "{}[{}]",
            name,
            args.iter().map(format_type).collect::<Vec<_>>().join(", ")
        ),
        Type::Tuple(types) => format!(
            "({})",
            types.iter().map(format_type).collect::<Vec<_>>().join(", ")
        ),
        Type::SharedRef(inner) => format!("&{}", format_type(inner)),
        Type::UniqueRef(inner) => format!("~{}", format_type(inner)),
    }
}

fn format_expression(expr: &Expression, parent_prec: u8) -> String {
    let prec = expression_precedence(expr);
    let mut rendered = match expr {
        Expression::Literal(Literal::Int(i)) => i.to_string(),
        Expression::Literal(Literal::Float(f)) => {
            let s = f.to_string();
            if s.contains('.') {
                s
            } else {
                format!("{}.0", s)
            }
        }
        Expression::Literal(Literal::String(s)) => format!("\"{}\"", escape_string(s)),
        Expression::Literal(Literal::List(items)) => format!(
            "[{}]",
            items
                .iter()
                .map(|item| format_expression(item, 0))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expression::Literal(Literal::Dict(pairs)) => format!(
            "{{{}}}",
            pairs
                .iter()
                .map(|(k, v)| format!("{}: {}", format_expression(k, 0), format_expression(v, 0)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expression::Literal(Literal::Set(items)) => format!(
            "{{{}}}",
            items
                .iter()
                .map(|item| format_expression(item, 0))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expression::Ident(name) => name.clone(),
        Expression::BinaryOp(left, op, right) => format!(
            "{} {} {}",
            format_expression(left, prec),
            binary_op_text(op),
            format_expression(right, prec + u8::from(matches!(op, BinaryOp::Assign)))
        ),
        Expression::Call(callee, args) => format!(
            "{}({})",
            format_expression(callee, prec),
            args.iter()
                .map(|a| format_expression(a, 0))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expression::GenericCall(callee, types, args) => format!(
            "{}[{}]({})",
            format_expression(callee, prec),
            types.iter().map(format_type).collect::<Vec<_>>().join(", "),
            args.iter()
                .map(|a| format_expression(a, 0))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expression::MacroCall(name, args) => format!(
            "${}({})",
            name,
            args.iter()
                .map(|a| format_expression(a, 0))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expression::MemberAccess(target, member) => {
            format!("{}.{}", format_expression(target, prec), member)
        }
        Expression::Move(inner) => format!("move {}", format_expression(inner, prec)),
        Expression::SharedRef(inner) => format!("&{}", format_expression(inner, prec)),
        Expression::UniqueRef(inner) => format!("~{}", format_expression(inner, prec)),
        Expression::Not(inner) => format!("not {}", format_expression(inner, prec)),
        Expression::Question(inner) => format!("{}?", format_expression(inner, prec)),
        Expression::Unwrap(inner) => format!("{}!!", format_expression(inner, prec)),
        Expression::Index(target, index) => format!(
            "{}[{}]",
            format_expression(target, prec),
            format_expression(index, 0)
        ),
        Expression::Tuple(items) => {
            if items.len() == 1 {
                format!("({},)", format_expression(&items[0], 0))
            } else {
                format!(
                    "({})",
                    items
                        .iter()
                        .map(|i| format_expression(i, 0))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
        Expression::Range(start, end) => format!(
            "{}..{}",
            format_expression(start, prec),
            format_expression(end, prec)
        ),
        Expression::RangeInclusive(start, end) => format!(
            "{}..={}",
            format_expression(start, prec),
            format_expression(end, prec)
        ),
        Expression::Lambda { params, body } => {
            let params_str = params
                .iter()
                .map(format_param)
                .collect::<Vec<_>>()
                .join(", ");
            format!("|{}| {}", params_str, format_expression(body, 0))
        }
    };

    if prec < parent_prec {
        rendered = format!("({rendered})");
    }
    rendered
}

fn expression_precedence(expr: &Expression) -> u8 {
    match expr {
        Expression::BinaryOp(_, op, _) => match op {
            BinaryOp::Assign => 1,
            BinaryOp::Or => 2,
            BinaryOp::And => 3,
            BinaryOp::Eq
            | BinaryOp::Ne
            | BinaryOp::Gt
            | BinaryOp::Lt
            | BinaryOp::Ge
            | BinaryOp::Le => 4,
            BinaryOp::Add | BinaryOp::Sub => 5,
            BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod | BinaryOp::MatMul => 6,
        },
        Expression::Range(_, _) | Expression::RangeInclusive(_, _) => 2,
        Expression::Lambda { .. } => 1,
        Expression::Move(_)
        | Expression::SharedRef(_)
        | Expression::UniqueRef(_)
        | Expression::Not(_) => 7,
        Expression::Call(_, _)
        | Expression::GenericCall(_, _, _)
        | Expression::MemberAccess(_, _)
        | Expression::Question(_)
        | Expression::Unwrap(_)
        | Expression::Index(_, _) => 8,
        Expression::Literal(_)
        | Expression::Ident(_)
        | Expression::MacroCall(_, _)
        | Expression::Tuple(_) => 9,
    }
}

fn binary_op_text(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Mod => "%",
        BinaryOp::Div => "/",
        BinaryOp::And => "and",
        BinaryOp::Or => "or",
        BinaryOp::Assign => "=",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Gt => ">",
        BinaryOp::Lt => "<",
        BinaryOp::Ge => ">=",
        BinaryOp::Le => "<=",
        BinaryOp::MatMul => "@",
    }
}

fn escape_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::format_program;
    use crate::lexer::Lexer;
    use crate::parser::parse_program;

    #[test]
    fn formatter_round_trip_basic_program() {
        let input = "def main():\n    mut x=1\n    if x>0:\n        $print(\"ok\")\n";
        let tokens: Vec<_> = Lexer::new(input).map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let out = format_program(&program);
        assert_eq!(
            out,
            "def main():\n    mut x = 1\n    if x > 0:\n        $print(\"ok\")\n"
        );
    }
}
