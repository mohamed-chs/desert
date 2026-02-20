use crate::ast::*;
use crate::resolver::Resolver;
use crate::sourcemap::SourceMap;
use std::collections::HashMap;
use std::collections::HashSet;

const MATMUL_PRELUDE: &str = "trait DesertMatMul<Rhs> {\n    type Output;\n    fn desert_matmul(self, rhs: Rhs) -> Self::Output;\n}\n\nimpl DesertMatMul<Vec<f32>> for Vec<f32> {\n    type Output = Vec<f32>;\n\n    fn desert_matmul(self, rhs: Vec<f32>) -> Self::Output {\n        vec![self.into_iter().zip(rhs).map(|(a, b)| a * b).sum()]\n    }\n}\n\nimpl DesertMatMul<Vec<f32>> for Vec<Vec<f32>> {\n    type Output = Vec<f32>;\n\n    fn desert_matmul(self, rhs: Vec<f32>) -> Self::Output {\n        self.into_iter()\n            .map(|row| row.into_iter().zip(rhs.iter().copied()).map(|(a, b)| a * b).sum())\n            .collect()\n    }\n}\n\nfn desert_matmul<L, R>(lhs: L, rhs: R) -> <L as DesertMatMul<R>>::Output\nwhere\n    L: DesertMatMul<R>,\n{\n    lhs.desert_matmul(rhs)\n}\n\n";

pub struct Transpiler {
    resolver: Resolver,
}

impl Transpiler {
    pub fn new() -> Self {
        Self {
            resolver: Resolver::new(),
        }
    }

    pub fn transpile(&self, program: &Program, source: &str) -> (String, SourceMap) {
        let mut output = String::new();
        let mut source_map = SourceMap::new();
        let mut current_line = 0;
        let struct_fields = self.collect_struct_fields(program);
        let protocol_names = self.collect_protocol_names(program);
        let uses_matmul = self.program_uses_matmul(program);

        if uses_matmul {
            output.push_str(MATMUL_PRELUDE);
            for _ in MATMUL_PRELUDE.lines() {
                source_map.add_mapping(current_line, 0);
                current_line += 1;
            }
        }

        self.transpile_statements(
            &program.statements,
            0,
            source,
            &struct_fields,
            &protocol_names,
            &mut output,
            &mut source_map,
            &mut current_line,
        );

        (output, source_map)
    }

    fn transpile_statements(
        &self,
        statements: &[Statement],
        indent: usize,
        source: &str,
        struct_fields: &HashMap<String, Vec<String>>,
        protocol_names: &HashSet<String>,
        output: &mut String,
        source_map: &mut SourceMap,
        current_line: &mut usize,
    ) {
        for stmt in statements {
            let ds_line = source[..stmt.span.start].lines().count().saturating_sub(1);

            match &stmt.kind {
                StatementKind::Def {
                    name,
                    params,
                    return_ty,
                    body,
                } => {
                    let indent_str = "    ".repeat(indent);
                    let params_str: Vec<String> = params
                        .iter()
                        .map(|p| {
                            if p.name == "self" && p.ty.is_none() {
                                return if p.is_mut {
                                    "&mut self".to_string()
                                } else {
                                    "&self".to_string()
                                };
                            }

                            let mut_str = if p.is_mut { "mut " } else { "" };
                            if let Some(t) = &p.ty {
                                format!(
                                    "{}{}: {}",
                                    mut_str,
                                    p.name,
                                    self.transpile_param_type(t, protocol_names)
                                )
                            } else {
                                format!("{}{}", mut_str, p.name)
                            }
                        })
                        .collect();
                    let ret_str = if let Some(t) = return_ty {
                        format!(" -> {}", self.transpile_type(t))
                    } else if self.statement_block_has_value_return(body) {
                        " -> impl std::fmt::Debug".to_string()
                    } else {
                        String::new()
                    };

                    let header = format!(
                        "{}fn {}({}){} {{\n",
                        indent_str,
                        name,
                        params_str.join(", "),
                        ret_str
                    );
                    output.push_str(&header);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;

                    self.transpile_statements(
                        body,
                        indent + 1,
                        source,
                        struct_fields,
                        protocol_names,
                        output,
                        source_map,
                        current_line,
                    );

                    let footer = format!("{}}}\n", indent_str);
                    output.push_str(&footer);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }
                StatementKind::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    let indent_str = "    ".repeat(indent);
                    let header = format!(
                        "{}if {} {{\n",
                        indent_str,
                        self.transpile_expression(condition, struct_fields)
                    );
                    output.push_str(&header);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;

                    self.transpile_statements(
                        then_block,
                        indent + 1,
                        source,
                        struct_fields,
                        protocol_names,
                        output,
                        source_map,
                        current_line,
                    );

                    if let Some(else_b) = else_block {
                        let mid = format!("{}}} else {{\n", indent_str);
                        output.push_str(&mid);
                        source_map.add_mapping(*current_line, ds_line);
                        *current_line += 1;
                        self.transpile_statements(
                            else_b,
                            indent + 1,
                            source,
                            struct_fields,
                            protocol_names,
                            output,
                            source_map,
                            current_line,
                        );
                    }

                    let footer = format!("{}}}\n", indent_str);
                    output.push_str(&footer);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }
                StatementKind::For {
                    var,
                    iterable,
                    body,
                } => {
                    let indent_str = "    ".repeat(indent);
                    let header = format!(
                        "{}for {} in {} {{\n",
                        indent_str,
                        var,
                        self.transpile_expression(iterable, struct_fields)
                    );
                    output.push_str(&header);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;

                    self.transpile_statements(
                        body,
                        indent + 1,
                        source,
                        struct_fields,
                        protocol_names,
                        output,
                        source_map,
                        current_line,
                    );

                    let footer = format!("{}}}\n", indent_str);
                    output.push_str(&footer);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }
                StatementKind::Struct { name, fields } => {
                    let indent_str = "    ".repeat(indent);
                    let header = format!("{}struct {} {{\n", indent_str, name);
                    output.push_str(&header);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;

                    for field in fields {
                        let f_ty = field
                            .ty
                            .as_ref()
                            .map(|t| self.transpile_type(t))
                            .unwrap_or_else(|| "()".to_string());
                        output.push_str(&format!(
                            "{}    pub {}: {},\n",
                            indent_str, field.name, f_ty
                        ));
                        source_map.add_mapping(*current_line, ds_line);
                        *current_line += 1;
                    }

                    let footer = format!("{}}}\n", indent_str);
                    output.push_str(&footer);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }
                StatementKind::Protocol { name, methods } => {
                    let indent_str = "    ".repeat(indent);
                    let header = format!("{}trait {} {{\n", indent_str, name);
                    output.push_str(&header);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;

                    self.transpile_statements(
                        methods,
                        indent + 1,
                        source,
                        struct_fields,
                        protocol_names,
                        output,
                        source_map,
                        current_line,
                    );

                    let footer = format!("{}}}\n", indent_str);
                    output.push_str(&footer);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }
                StatementKind::Impl {
                    protocol,
                    for_type,
                    methods,
                } => {
                    let indent_str = "    ".repeat(indent);
                    let header = if let Some(p) = protocol {
                        format!("{}impl {} for {} {{\n", indent_str, p, for_type)
                    } else {
                        format!("{}impl {} {{\n", indent_str, for_type)
                    };
                    output.push_str(&header);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;

                    self.transpile_statements(
                        methods,
                        indent + 1,
                        source,
                        struct_fields,
                        protocol_names,
                        output,
                        source_map,
                        current_line,
                    );

                    let footer = format!("{}}}\n", indent_str);
                    output.push_str(&footer);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }
                StatementKind::PyImport(content) => {
                    let indent_str = "    ".repeat(indent);
                    output.push_str(&format!("{}/* Desert PyImport block:\n", indent_str));
                    output.push_str(&format!("{}   {}\n", indent_str, content));
                    output.push_str(&format!("{}*/\n", indent_str));
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 3;
                }
                StatementKind::Ref {
                    name: _,
                    ty: _,
                    value: _,
                } => {
                    let stmt_output = self.transpile_statement(stmt, indent, struct_fields);
                    output.push_str(&stmt_output);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }
                StatementKind::MutRef {
                    name: _,
                    ty: _,
                    value: _,
                } => {
                    let stmt_output = self.transpile_statement(stmt, indent, struct_fields);
                    output.push_str(&stmt_output);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }
                StatementKind::Match { expression, arms } => {
                    let ds_line = source[..stmt.span.start].lines().count().saturating_sub(1);
                    let header = format!(
                        "{}match {} {{\n",
                        "    ".repeat(indent),
                        self.transpile_expression(expression, struct_fields)
                    );
                    output.push_str(&header);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;

                    for (pattern, body) in arms {
                        let arm_header = format!(
                            "{}    {} => {{\n",
                            "    ".repeat(indent),
                            self.transpile_expression(pattern, struct_fields)
                        );
                        output.push_str(&arm_header);
                        source_map.add_mapping(*current_line, ds_line);
                        *current_line += 1;

                        self.transpile_statements(
                            body,
                            indent + 2,
                            source,
                            struct_fields,
                            protocol_names,
                            output,
                            source_map,
                            current_line,
                        );

                        let arm_footer = format!("{}    }}\n", "    ".repeat(indent));
                        output.push_str(&arm_footer);
                        source_map.add_mapping(*current_line, ds_line);
                        *current_line += 1;
                    }

                    let footer = format!("{}}}\n", "    ".repeat(indent));
                    output.push_str(&footer);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }
                _ => {
                    let stmt_output = self.transpile_statement(stmt, indent, struct_fields);
                    for _ in stmt_output.lines() {
                        source_map.add_mapping(*current_line, ds_line);
                        *current_line += 1;
                    }
                    output.push_str(&stmt_output);
                }
            }
        }
    }

    fn transpile_statement(
        &self,
        stmt: &Statement,
        indent: usize,
        struct_fields: &HashMap<String, Vec<String>>,
    ) -> String {
        let indent_str = "    ".repeat(indent);
        match &stmt.kind {
            StatementKind::Let { name, ty, value } => {
                let ty_str = if let Some(t) = ty {
                    format!(": {}", self.transpile_type(t))
                } else {
                    String::new()
                };
                format!(
                    "{}let {}{} = {};\n",
                    indent_str,
                    name,
                    ty_str,
                    self.transpile_expression(value, struct_fields)
                )
            }
            StatementKind::Mut { name, ty, value } => {
                let ty_str = if let Some(t) = ty {
                    format!(": {}", self.transpile_type(t))
                } else {
                    String::new()
                };
                format!(
                    "{}let mut {}{} = {};\n",
                    indent_str,
                    name,
                    ty_str,
                    self.transpile_expression(value, struct_fields)
                )
            }
            StatementKind::Return(Some(expr)) => {
                format!(
                    "{}return {};\n",
                    indent_str,
                    self.transpile_expression(expr, struct_fields)
                )
            }
            StatementKind::Return(None) => {
                format!("{}return;\n", indent_str)
            }
            StatementKind::PyImport(content) => {
                format!("{}/* Desert PyImport block: {} */\n", indent_str, content)
            }
            StatementKind::Ref { name, ty, value } => {
                let ty_str = if let Some(t) = ty {
                    format!(": {}", self.transpile_type(t))
                } else {
                    String::new()
                };
                format!(
                    "{}let {} {} = &{};\n",
                    indent_str,
                    name,
                    ty_str,
                    self.transpile_expression(value, struct_fields)
                )
            }
            StatementKind::MutRef { name, ty, value } => {
                let ty_str = if let Some(t) = ty {
                    format!(": {}", self.transpile_type(t))
                } else {
                    String::new()
                };
                format!(
                    "{}let {} {} = &mut {};\n",
                    indent_str,
                    name,
                    ty_str,
                    self.transpile_expression(value, struct_fields)
                )
            }
            StatementKind::Match { expression, arms } => {
                let mut output = format!(
                    "{}match {} {{\n",
                    indent_str,
                    self.transpile_expression(expression, struct_fields)
                );
                for (pattern, body) in arms {
                    output.push_str(&format!(
                        "{}    {} => {{\n",
                        indent_str,
                        self.transpile_expression(pattern, struct_fields)
                    ));
                    for s in body {
                        output.push_str(&self.transpile_statement(s, indent + 2, struct_fields));
                    }
                    output.push_str(&format!("{}    }}\n", indent_str));
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            StatementKind::Expr(expr) => {
                format!("{}{};\n", indent_str, self.transpile_expression(expr, struct_fields))
            }
            _ => unreachable!("transpile_statement called with block-level statement"),
        }
    }

    fn transpile_expression(
        &self,
        expr: &Expression,
        struct_fields: &HashMap<String, Vec<String>>,
    ) -> String {
        match expr {
            Expression::Literal(Literal::Int(i)) => i.to_string(),
            Expression::Literal(Literal::Float(f)) => {
                let s = f.to_string();
                if s.contains('.') {
                    s
                } else {
                    format!("{}.0", s)
                }
            }
            Expression::Literal(Literal::String(s)) => format!("\"{}\".to_string()", s),
            Expression::Literal(Literal::List(items)) => {
                let items_str: Vec<String> =
                    items
                        .iter()
                        .map(|i| self.transpile_expression(i, struct_fields))
                        .collect();
                format!("vec![{}]", items_str.join(", "))
            }
            Expression::Ident(name) => name.clone(),
            Expression::BinaryOp(left, op, right) => {
                if matches!(op, BinaryOp::MatMul) {
                    return format!(
                        "desert_matmul(({}).clone(), ({}).clone())",
                        self.transpile_expression(left, struct_fields),
                        self.transpile_expression(right, struct_fields)
                    );
                }

                let op_str = match op {
                    BinaryOp::Add => "+",
                    BinaryOp::Sub => "-",
                    BinaryOp::Mul => "*",
                    BinaryOp::Div => "/",
                    BinaryOp::Assign => "=",
                    BinaryOp::Eq => "==",
                    BinaryOp::Ne => "!=",
                    BinaryOp::Gt => ">",
                    BinaryOp::Lt => "<",
                    BinaryOp::Ge => ">=",
                    BinaryOp::Le => "<=",
                    BinaryOp::MatMul => unreachable!("handled above"),
                };
                format!(
                    "({} {} {})",
                    self.transpile_expression(left, struct_fields),
                    op_str,
                    self.transpile_expression(right, struct_fields)
                )
            }
            Expression::Call(callee, args) => {
                if let Expression::Ident(name) = callee.as_ref()
                    && let Some(constructor) =
                        self.transpile_struct_constructor_call(name, args, struct_fields)
                {
                    return constructor;
                }

                let args_str: Vec<String> =
                    args.iter()
                        .map(|a| self.transpile_expression(a, struct_fields))
                        .collect();
                format!(
                    "{}({})",
                    self.transpile_expression(callee, struct_fields),
                    args_str.join(", ")
                )
            }
            Expression::GenericCall(callee, types, args) => {
                let types_str: Vec<String> = types.iter().map(|t| self.transpile_type(t)).collect();
                let args_str: Vec<String> =
                    args.iter()
                        .map(|a| self.transpile_expression(a, struct_fields))
                        .collect();
                format!(
                    "{}::<{}>({})",
                    self.transpile_expression(callee, struct_fields),
                    types_str.join(", "),
                    args_str.join(", ")
                )
            }
            Expression::MacroCall(name, args) => {
                if name == "print" {
                    return self.transpile_print_macro(args, struct_fields);
                }

                let rust_macro = format!("{}!", name);
                let args_str: Vec<String> =
                    args.iter()
                        .map(|a| self.transpile_expression(a, struct_fields))
                        .collect();
                format!("{}({})", rust_macro, args_str.join(", "))
            }
            Expression::MemberAccess(expr, member) => {
                let expr_str = self.transpile_expression(expr, struct_fields);
                if self.resolver.is_type(&expr_str) {
                    format!("{}::{}", expr_str, member)
                } else {
                    format!("{}.{}", expr_str, member)
                }
            }
            Expression::Move(expr) => {
                self.transpile_expression(expr, struct_fields) // In Rust, move is the default for many types
            }
            Expression::SharedRef(expr) => {
                format!("&{}", self.transpile_expression(expr, struct_fields))
            }
            Expression::UniqueRef(expr) => {
                format!("&mut {}", self.transpile_expression(expr, struct_fields))
            }
            Expression::Question(expr) => {
                format!("{}?", self.transpile_expression(expr, struct_fields))
            }
            Expression::Unwrap(expr) => {
                format!("{}.unwrap()", self.transpile_expression(expr, struct_fields))
            }
            Expression::Index(expr, index) => {
                format!(
                    "{}[{}]",
                    self.transpile_expression(expr, struct_fields),
                    self.transpile_expression(index, struct_fields)
                )
            }
        }
    }

    fn transpile_print_macro(
        &self,
        args: &[Expression],
        struct_fields: &HashMap<String, Vec<String>>,
    ) -> String {
        if args.is_empty() {
            return "println!()".to_string();
        }

        let parts: Vec<String> = args
            .iter()
            .map(|arg| match arg {
                Expression::Literal(Literal::String(s)) => self.transpile_print_string_literal(s),
                _ => format!(
                    "format!(\"{{:?}}\", {})",
                    self.transpile_expression(arg, struct_fields)
                ),
            })
            .collect();

        if parts.len() == 1 {
            format!("println!(\"{{}}\", {})", parts[0])
        } else {
            format!("println!(\"{{}}\", vec![{}].join(\" \"))", parts.join(", "))
        }
    }

    fn transpile_print_string_literal(&self, value: &str) -> String {
        let mut format_template = String::new();
        let mut interpolation_args = Vec::new();
        let mut idx = 0;

        while let Some(open_rel) = value[idx..].find('{') {
            let open = idx + open_rel;
            let prefix = &value[idx..open];
            format_template.push_str(prefix);

            if let Some(close_rel) = value[open + 1..].find('}') {
                let close = open + 1 + close_rel;
                let placeholder = value[open + 1..close].trim();

                if placeholder.is_empty() {
                    format_template.push('{');
                    format_template.push('}');
                } else {
                    format_template.push_str("{:?}");
                    interpolation_args.push(placeholder.to_string());
                }

                idx = close + 1;
            } else {
                format_template.push_str(&value[open..]);
                idx = value.len();
            }
        }

        if idx < value.len() {
            format_template.push_str(&value[idx..]);
        }

        if interpolation_args.is_empty() {
            return format!("\"{}\".to_string()", format_template);
        }

        format!(
            "format!(\"{}\", {})",
            format_template,
            interpolation_args.join(", ")
        )
    }

    fn collect_struct_fields(&self, program: &Program) -> HashMap<String, Vec<String>> {
        let mut struct_fields = HashMap::new();
        for stmt in &program.statements {
            if let StatementKind::Struct { name, fields } = &stmt.kind {
                let field_names = fields.iter().map(|field| field.name.clone()).collect();
                struct_fields.insert(name.clone(), field_names);
            }
        }
        struct_fields
    }

    fn collect_protocol_names(&self, program: &Program) -> HashSet<String> {
        let mut protocol_names = HashSet::new();
        for stmt in &program.statements {
            if let StatementKind::Protocol { name, .. } = &stmt.kind {
                protocol_names.insert(name.clone());
            }
        }
        protocol_names
    }

    fn transpile_param_type(&self, ty: &Type, protocol_names: &HashSet<String>) -> String {
        if let Type::Simple(name) = ty
            && protocol_names.contains(name)
        {
            return format!("impl {}", name);
        }
        self.transpile_type(ty)
    }

    fn transpile_struct_constructor_call(
        &self,
        name: &str,
        args: &[Expression],
        struct_fields: &HashMap<String, Vec<String>>,
    ) -> Option<String> {
        let fields = struct_fields.get(name)?;
        let mut assigned = HashMap::new();
        let mut positional = Vec::new();

        for arg in args {
            match arg {
                Expression::BinaryOp(left, BinaryOp::Assign, right) => {
                    if let Expression::Ident(field_name) = left.as_ref() {
                        assigned.insert(
                            field_name.clone(),
                            self.transpile_expression(right, struct_fields),
                        );
                    } else {
                        positional.push(self.transpile_expression(arg, struct_fields));
                    }
                }
                _ => positional.push(self.transpile_expression(arg, struct_fields)),
            }
        }

        let mut positional_idx = 0;
        let mut field_pairs = Vec::new();
        for field in fields {
            if let Some(value) = assigned.get(field) {
                field_pairs.push(format!("{field}: {value}"));
                continue;
            }

            if let Some(value) = positional.get(positional_idx) {
                field_pairs.push(format!("{field}: {value}"));
                positional_idx += 1;
            }
        }

        Some(format!("{name} {{ {} }}", field_pairs.join(", ")))
    }

    fn transpile_type(&self, ty: &Type) -> String {
        match ty {
            Type::Simple(name) => match name.as_str() {
                "List" => "Vec".to_string(),
                "Str" => "String".to_string(),
                _ => name.clone(),
            },
            Type::Generic(name, inner) => {
                let name = match name.as_str() {
                    "List" => "Vec",
                    _ => name,
                };
                let inner_str: Vec<String> = inner.iter().map(|t| self.transpile_type(t)).collect();
                format!("{}<{}>", name, inner_str.join(", "))
            }
            Type::SharedRef(inner) => format!("&{}", self.transpile_type(inner)),
            Type::UniqueRef(inner) => format!("&mut {}", self.transpile_type(inner)),
        }
    }

    fn statement_block_has_value_return(&self, statements: &[Statement]) -> bool {
        statements.iter().any(|statement| match &statement.kind {
            StatementKind::Return(Some(_)) => true,
            StatementKind::If {
                then_block,
                else_block,
                ..
            } => {
                self.statement_block_has_value_return(then_block)
                    || else_block
                        .as_ref()
                        .is_some_and(|block| self.statement_block_has_value_return(block))
            }
            StatementKind::For { body, .. }
            | StatementKind::Def { body, .. }
            | StatementKind::Protocol { methods: body, .. }
            | StatementKind::Impl { methods: body, .. } => self.statement_block_has_value_return(body),
            StatementKind::Match { arms, .. } => arms
                .iter()
                .any(|(_, body)| self.statement_block_has_value_return(body)),
            _ => false,
        })
    }

    fn program_uses_matmul(&self, program: &Program) -> bool {
        program
            .statements
            .iter()
            .any(|statement| self.statement_uses_matmul(statement))
    }

    fn statement_uses_matmul(&self, statement: &Statement) -> bool {
        match &statement.kind {
            StatementKind::Let { value, .. }
            | StatementKind::Mut { value, .. }
            | StatementKind::Ref { value, .. }
            | StatementKind::MutRef { value, .. } => self.expression_uses_matmul(value),
            StatementKind::Def { body, .. } => body.iter().any(|s| self.statement_uses_matmul(s)),
            StatementKind::If {
                condition,
                then_block,
                else_block,
            } => {
                self.expression_uses_matmul(condition)
                    || then_block.iter().any(|s| self.statement_uses_matmul(s))
                    || else_block
                        .as_ref()
                        .is_some_and(|b| b.iter().any(|s| self.statement_uses_matmul(s)))
            }
            StatementKind::For { iterable, body, .. } => {
                self.expression_uses_matmul(iterable)
                    || body.iter().any(|s| self.statement_uses_matmul(s))
            }
            StatementKind::Protocol { methods, .. } | StatementKind::Impl { methods, .. } => {
                methods.iter().any(|s| self.statement_uses_matmul(s))
            }
            StatementKind::Match { expression, arms } => {
                self.expression_uses_matmul(expression)
                    || arms
                        .iter()
                        .any(|(pat, body)| {
                            self.expression_uses_matmul(pat)
                                || body.iter().any(|s| self.statement_uses_matmul(s))
                        })
            }
            StatementKind::Return(Some(expr)) | StatementKind::Expr(expr) => {
                self.expression_uses_matmul(expr)
            }
            StatementKind::Struct { .. }
            | StatementKind::PyImport(_)
            | StatementKind::Return(None) => false,
        }
    }

    fn expression_uses_matmul(&self, expr: &Expression) -> bool {
        match expr {
            Expression::BinaryOp(left, op, right) => {
                matches!(op, BinaryOp::MatMul)
                    || self.expression_uses_matmul(left)
                    || self.expression_uses_matmul(right)
            }
            Expression::Call(callee, args) | Expression::GenericCall(callee, _, args) => {
                self.expression_uses_matmul(callee)
                    || args.iter().any(|arg| self.expression_uses_matmul(arg))
            }
            Expression::MacroCall(_, args) => args.iter().any(|arg| self.expression_uses_matmul(arg)),
            Expression::MemberAccess(expr, _)
            | Expression::Move(expr)
            | Expression::SharedRef(expr)
            | Expression::UniqueRef(expr)
            | Expression::Question(expr)
            | Expression::Unwrap(expr) => self.expression_uses_matmul(expr),
            Expression::Index(expr, idx) => {
                self.expression_uses_matmul(expr) || self.expression_uses_matmul(idx)
            }
            Expression::Literal(Literal::List(items)) => {
                items.iter().any(|item| self.expression_uses_matmul(item))
            }
            Expression::Literal(_) | Expression::Ident(_) => false,
        }
    }
}

impl Default for Transpiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::parse_program;

    #[test]

    fn test_transpile_basic() {
        let input = "let x = 10\ndef foo(y):\n    let z = y\n    z";

        let lexer = Lexer::new(input);

        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();

        let (_, program) = parse_program(&tokens).unwrap();

        let transpiler = Transpiler::new();

        let (rust_code, _) = transpiler.transpile(&program, input);

        let expected = "let x = 10;\nfn foo(y) {\n    let z = y;\n    z;\n}\n";

        assert_eq!(rust_code, expected);
    }

    #[test]

    fn test_transpile_factorial() {
        let input = "

    def factorial(n):

        if n == 0:

            return 1

        return n * factorial(n - 1)

    ";

        let lexer = Lexer::new(input.trim());

        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();

        let (_, program) = parse_program(&tokens).unwrap();

        let transpiler = Transpiler::new();

        let (rust_code, _) = transpiler.transpile(&program, input.trim());

        let expected = "fn factorial(n) -> impl std::fmt::Debug {\n    if (n == 0) {\n        return 1;\n    }\n    return (n * factorial((n - 1)));\n}\n";

        assert_eq!(rust_code, expected);
    }

    #[test]

    fn test_transpile_hello_world() {
        let input = "def main():\n    $print(\"Hello, Desert!\")";

        let lexer = Lexer::new(input);

        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();

        let (_, program) = parse_program(&tokens).unwrap();

        let transpiler = Transpiler::new();

        let (rust_code, _) = transpiler.transpile(&program, input);

        let expected = "fn main() {\n    println!(\"{}\", \"Hello, Desert!\".to_string());\n}\n";

        assert_eq!(rust_code, expected);
    }

    #[test]
    fn test_transpile_print_with_interpolation_and_field_access() {
        let input = "def main():\n    $print(\"port {cfg.port}\")";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        let expected = "fn main() {\n    println!(\"{}\", format!(\"port {:?}\", cfg.port));\n}\n";
        assert_eq!(rust_code, expected);
    }

    #[test]
    fn test_transpile_print_with_multiple_arguments() {
        let input = "def main():\n    $print(\"x\", val, list)";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        let expected = "fn main() {\n    println!(\"{}\", vec![\"x\".to_string(), format!(\"{:?}\", val), format!(\"{:?}\", list)].join(\" \"));\n}\n";
        assert_eq!(rust_code, expected);
    }

    #[test]
    fn test_transpile_matmul_operator() {
        let input = "def main():\n    let out = a @ b";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        assert!(rust_code.contains("fn desert_matmul"));
        assert!(rust_code.contains("let out = desert_matmul((a).clone(), (b).clone());"));
    }

    #[test]

    fn test_transpile_types() {
        let input = "let x: List[i32] = [1, 2, 3]";

        let lexer = Lexer::new(input);

        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();

        let (_, program) = parse_program(&tokens).unwrap();

        let transpiler = Transpiler::new();

        let (rust_code, _) = transpiler.transpile(&program, input);

        assert_eq!(rust_code, "let x: Vec<i32> = vec![1, 2, 3];\n");
    }

    #[test]

    fn test_transpile_borrows() {
        let input = "def foo(x: &i32, y: ~List[f32]) -> i32:\n    return 0";

        let lexer = Lexer::new(input);

        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();

        let (_, program) = parse_program(&tokens).unwrap();

        let transpiler = Transpiler::new();

        let (rust_code, _) = transpiler.transpile(&program, input);

        assert_eq!(
            rust_code,
            "fn foo(x: &i32, y: &mut Vec<f32>) -> i32 {\n    return 0;\n}\n"
        );
    }

    #[test]

    fn test_transpile_unified_dot() {
        let input = "let x = Path.new(\"foo\")\nlet y = x.exists()";

        let lexer = Lexer::new(input);

        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();

        let (_, program) = parse_program(&tokens).unwrap();

        let transpiler = Transpiler::new();

        let (rust_code, _) = transpiler.transpile(&program, input);

        assert_eq!(
            rust_code,
            "let x = Path::new(\"foo\".to_string());\nlet y = x.exists();\n"
        );
    }

    #[test]
    fn test_transpile_if_else() {
        let input = "if x > 0:\n    return 1\nelse:\n    return 0";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        let expected = "if (x > 0) {\n    return 1;\n} else {\n    return 0;\n}\n";
        assert_eq!(rust_code, expected);
    }

    #[test]
    fn test_transpile_struct() {
        let input = "struct Point:\n    x: i32\n    y: i32";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        let expected = "struct Point {\n    pub x: i32,\n    pub y: i32,\n}\n";
        assert_eq!(rust_code, expected);
    }

    #[test]
    fn test_transpile_struct_constructor_positional() {
        let input = "struct Point:\n    x: i32\n    y: i32\nlet p = Point(1, 2)";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        let expected =
            "struct Point {\n    pub x: i32,\n    pub y: i32,\n}\nlet p = Point { x: 1, y: 2 };\n";
        assert_eq!(rust_code, expected);
    }

    #[test]
    fn test_transpile_struct_constructor_named() {
        let input = "struct Scale:\n    factor: f64\nlet s = Scale(factor=1.5)";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        let expected = "struct Scale {\n    pub factor: f64,\n}\nlet s = Scale { factor: 1.5 };\n";
        assert_eq!(rust_code, expected);
    }

    #[test]
    fn test_transpile_protocol() {
        let input = "protocol Speak:\n    def talk(self) -> Str:\n        return \"\"";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        let expected = "trait Speak {\n    fn talk(&self) -> String {\n        return \"\".to_string();\n    }\n}\n";
        assert_eq!(rust_code, expected);
    }

    #[test]
    fn test_transpile_impl() {
        let input = "impl Speak for Dog:\n    def talk(self) -> Str:\n        return \"Woof\"";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        let expected =
            "impl Speak for Dog {\n    fn talk(&self) -> String {\n        return \"Woof\".to_string();\n    }\n}\n";
        assert_eq!(rust_code, expected);
    }

    #[test]
    fn test_transpile_mut_self_receiver() {
        let input = "impl Counter:\n    def inc(mut self):\n        self.value = self.value + 1";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        let expected = "impl Counter {\n    fn inc(&mut self) {\n        (self.value = (self.value + 1));\n    }\n}\n";
        assert_eq!(rust_code, expected);
    }

    #[test]
    fn test_transpile_error_ops() {
        let input = "let x = foo()?\nlet y = bar()!!";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        let expected = "let x = foo()?;\nlet y = bar().unwrap();\n";
        assert_eq!(rust_code, expected);
    }

    #[test]
    fn test_transpile_pyimport() {
        let input = "pyimport:\n    import torch";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        assert!(rust_code.contains("/* Desert PyImport block:"));
        assert!(rust_code.contains("import torch"));
    }

    #[test]
    fn test_transpile_nested_control_flow() {
        let input = "if x:\n    for i in list:\n        $print(i)";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        assert!(rust_code.contains("if x {"));
        assert!(rust_code.contains("for i in list {"));
    }

    #[test]
    fn test_transpile_complex_generic_call() {
        let input = "obj.method[T1, T2](arg1, arg2)";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
        let (_, program) = parse_program(&tokens).unwrap();
        let transpiler = Transpiler::new();
        let (rust_code, _) = transpiler.transpile(&program, input);
        assert!(rust_code.contains("obj.method::<T1, T2>(arg1, arg2)"));
    }
}
