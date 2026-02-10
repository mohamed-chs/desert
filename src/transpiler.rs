use crate::ast::*;
use crate::resolver::Resolver;
use crate::sourcemap::SourceMap;

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

        self.transpile_statements(&program.statements, 0, source, &mut output, &mut source_map, &mut current_line);
        
        (output, source_map)
    }

    fn transpile_statements(&self, statements: &[Statement], indent: usize, source: &str, output: &mut String, source_map: &mut SourceMap, current_line: &mut usize) {
        for stmt in statements {
            let ds_line = source[..stmt.span.start].lines().count().saturating_sub(1);
            
            match &stmt.kind {
                StatementKind::Def { name, params, return_ty, body } => {
                    let indent_str = "    ".repeat(indent);
                    let params_str: Vec<String> = params.iter().map(|p| {
                        if let Some(t) = &p.ty {
                            format!("{}: {}", p.name, self.transpile_type(t))
                        } else {
                            p.name.clone()
                        }
                    }).collect();
                    let ret_str = if let Some(t) = return_ty {
                        format!(" -> {}", self.transpile_type(t))
                    } else {
                        String::new()
                    };
                    
                    let header = format!("{}fn {}({}){} {{\n", indent_str, name, params_str.join(", "), ret_str);
                    output.push_str(&header);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;

                    self.transpile_statements(body, indent + 1, source, output, source_map, current_line);

                    let footer = format!("{}}}\n", indent_str);
                    output.push_str(&footer);
                    source_map.add_mapping(*current_line, ds_line); // Best we can do for closing brace
                    *current_line += 1;
                }
                StatementKind::If { condition, then_block, else_block } => {
                    let indent_str = "    ".repeat(indent);
                    let header = format!("{}if {} {{\n", indent_str, self.transpile_expression(condition));
                    output.push_str(&header);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;

                    self.transpile_statements(then_block, indent + 1, source, output, source_map, current_line);

                    if let Some(else_b) = else_block {
                        let mid = format!("{}}} else {{\n", indent_str);
                        output.push_str(&mid);
                        source_map.add_mapping(*current_line, ds_line);
                        *current_line += 1;
                        self.transpile_statements(else_b, indent + 1, source, output, source_map, current_line);
                    }

                    let footer = format!("{}}}\n", indent_str);
                    output.push_str(&footer);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }
                StatementKind::For { var, iterable, body } => {
                    let indent_str = "    ".repeat(indent);
                    let header = format!("{}for {} in {} {{\n", indent_str, var, self.transpile_expression(iterable));
                    output.push_str(&header);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;

                    self.transpile_statements(body, indent + 1, source, output, source_map, current_line);

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
                        let f_ty = field.ty.as_ref().map(|t| self.transpile_type(t)).unwrap_or_else(|| "()".to_string());
                        output.push_str(&format!("{}    pub {}: {},\n", indent_str, field.name, f_ty));
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

                    self.transpile_statements(methods, indent + 1, source, output, source_map, current_line);

                    let footer = format!("{}}}\n", indent_str);
                    output.push_str(&footer);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }
                StatementKind::Impl { protocol, for_type, methods } => {
                    let indent_str = "    ".repeat(indent);
                    let header = if let Some(p) = protocol {
                        format!("{}impl {} for {} {{\n", indent_str, p, for_type)
                    } else {
                        format!("{}impl {} {{\n", indent_str, for_type)
                    };
                    output.push_str(&header);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;

                    self.transpile_statements(methods, indent + 1, source, output, source_map, current_line);

                    let footer = format!("{}}}\n", indent_str);
                    output.push_str(&footer);
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }
                StatementKind::PyImport(content) => {
                    let indent_str = "    ".repeat(indent);
                    output.push_str(&format!("{}// Python Import: {}\n", indent_str, content));
                    source_map.add_mapping(*current_line, ds_line);
                    *current_line += 1;
                }

                                _ => {

                    
                    let stmt_output = self.transpile_statement(stmt, indent);
                    for _ in stmt_output.lines() {
                        source_map.add_mapping(*current_line, ds_line);
                        *current_line += 1;
                    }
                    output.push_str(&stmt_output);
                }
            }
        }
    }

    fn transpile_statement(&self, stmt: &Statement, indent: usize) -> String {
        let indent_str = "    ".repeat(indent);
        match &stmt.kind {
            StatementKind::Let { name, ty, value } => {
                let ty_str = if let Some(t) = ty {
                    format!(": {}", self.transpile_type(t))
                } else {
                    String::new()
                };
                format!("{}let {}{} = {};\n", indent_str, name, ty_str, self.transpile_expression(value))
            }
            StatementKind::Mut { name, ty, value } => {
                let ty_str = if let Some(t) = ty {
                    format!(": {}", self.transpile_type(t))
                } else {
                    String::new()
                };
                format!("{}let mut {}{} = {};\n", indent_str, name, ty_str, self.transpile_expression(value))
            }
            StatementKind::Def { name, params, return_ty, body } => {
                let params_str: Vec<String> = params.iter().map(|p| {
                    if let Some(t) = &p.ty {
                        format!("{}: {}", p.name, self.transpile_type(t))
                    } else {
                        p.name.clone()
                    }
                }).collect();
                let ret_str = if let Some(t) = return_ty {
                    format!(" -> {}", self.transpile_type(t))
                } else {
                    String::new()
                };
                let mut def = format!("{}fn {}({}){} {{\n", indent_str, name, params_str.join(", "), ret_str);
                for s in body {
                    def.push_str(&self.transpile_statement(s, indent + 1));
                }
                def.push_str(&format!("{}}}\n", indent_str));
                def
            }
            StatementKind::If { condition, then_block, else_block } => {
                let mut output = format!("{}if {} {{\n", indent_str, self.transpile_expression(condition));
                for s in then_block {
                    output.push_str(&self.transpile_statement(s, indent + 1));
                }
                if let Some(else_b) = else_block {
                    output.push_str(&format!("{}}} else {{\n", indent_str));
                    for s in else_b {
                        output.push_str(&self.transpile_statement(s, indent + 1));
                    }
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            StatementKind::Return(Some(expr)) => {
                format!("{}return {};\n", indent_str, self.transpile_expression(expr))
            }
            StatementKind::Return(None) => {
                format!("{}return;\n", indent_str)
            }
            StatementKind::Struct { name, fields } => {
                let mut output = format!("{}struct {} {{\n", indent_str, name);
                for field in fields {
                    let f_ty = field.ty.as_ref().map(|t| self.transpile_type(t)).unwrap_or_else(|| "()".to_string());
                    output.push_str(&format!("{}    pub {}: {},\n", indent_str, field.name, f_ty));
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            StatementKind::Protocol { name, methods } => {
                let mut output = format!("{}trait {} {{\n", indent_str, name);
                for m in methods {
                    output.push_str(&self.transpile_statement(m, indent + 1));
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            StatementKind::Impl { protocol, for_type, methods } => {
                let mut output = if let Some(p) = protocol {
                    format!("{}impl {} for {} {{\n", indent_str, p, for_type)
                } else {
                    format!("{}impl {} {{\n", indent_str, for_type)
                };
                for m in methods {
                    output.push_str(&self.transpile_statement(m, indent + 1));
                }
                output.push_str(&format!("{}}}\n", indent_str));
                output
            }
            StatementKind::For { var, iterable, body: _ } => {
                format!("{}for {} in {} {{ ... }}\n", indent_str, var, self.transpile_expression(iterable))
            }
            StatementKind::PyImport(content) => {
                format!("{}// Python Import: {}\n", indent_str, content)
            }
            StatementKind::Expr(expr) => {
                format!("{}{};\n", indent_str, self.transpile_expression(expr))
            }
        }
    }

    fn transpile_expression(&self, expr: &Expression) -> String {
        match expr {
            Expression::Literal(Literal::Int(i)) => i.to_string(),
            Expression::Literal(Literal::Float(f)) => {
                let s = f.to_string();
                if s.contains('.') { s } else { format!("{}.0", s) }
            }
            Expression::Literal(Literal::String(s)) => format!("\"{}\".to_string()", s),
            Expression::Literal(Literal::List(items)) => {
                let items_str: Vec<String> = items.iter().map(|i| self.transpile_expression(i)).collect();
                format!("vec![{}]", items_str.join(", "))
            }
            Expression::Ident(name) => name.clone(),
            Expression::BinaryOp(left, op, right) => {
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
                                        BinaryOp::MatMul => "*",
                     // For now map to *
                };
                format!("({} {} {})", self.transpile_expression(left), op_str, self.transpile_expression(right))
            }
            Expression::Call(callee, args) => {
                let args_str: Vec<String> = args.iter().map(|a| self.transpile_expression(a)).collect();
                format!("{}({})", self.transpile_expression(callee), args_str.join(", "))
            }
            Expression::GenericCall(callee, types, args) => {
                let types_str: Vec<String> = types.iter().map(|t| self.transpile_type(t)).collect();
                let args_str: Vec<String> = args.iter().map(|a| self.transpile_expression(a)).collect();
                format!("{}::<{}>({})", self.transpile_expression(callee), types_str.join(", "), args_str.join(", "))
            }
            Expression::MacroCall(name, args) => {
                let rust_macro = match name.as_str() {
                    "print" => "println!",
                    _ => &format!("{}!", name),
                };
                let args_str: Vec<String> = args.iter().map(|a| self.transpile_expression(a)).collect();
                format!("{}({})", rust_macro, args_str.join(", "))
            }
            Expression::MemberAccess(expr, member) => {
                let expr_str = self.transpile_expression(expr);
                if self.resolver.is_type(&expr_str) {
                    format!("{}::{}", expr_str, member)
                } else {
                    format!("{}.{}", expr_str, member)
                }
            }
            Expression::Move(expr) => {
                self.transpile_expression(expr)
            }
            Expression::SharedRef(expr) => {
                format!("&{}", self.transpile_expression(expr))
            }
            Expression::UniqueRef(expr) => {
                format!("&mut {}", self.transpile_expression(expr))
            }
        }
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

            

            let expected = "fn factorial(n) {\n    if (n == 0) {\n        return 1;\n    }\n    return (n * factorial((n - 1)));\n}\n";

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

            

            let expected = "fn main() {\n    println!(\"Hello, Desert!\".to_string());\n}\n";

            assert_eq!(rust_code, expected);

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

            assert_eq!(rust_code, "fn foo(x: &i32, y: &mut Vec<f32>) -> i32 {\n    return 0;\n}\n");

        }

    

        #[test]

        fn test_transpile_unified_dot() {

            let input = "let x = Path.new(\"foo\")\nlet y = x.exists()";

            let lexer = Lexer::new(input);

            let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();

            let (_, program) = parse_program(&tokens).unwrap();

            let transpiler = Transpiler::new();

            let (rust_code, _) = transpiler.transpile(&program, input);

            assert_eq!(rust_code, "let x = Path::new(\"foo\".to_string());\nlet y = x.exists();\n");
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
        fn test_transpile_protocol() {
            let input = "protocol Speak:\n    def talk(self) -> Str:\n        return \"\"";
            let lexer = Lexer::new(input);
            let tokens: Vec<_> = lexer.map(|r| r.unwrap()).collect();
            let (_, program) = parse_program(&tokens).unwrap();
            let transpiler = Transpiler::new();
            let (rust_code, _) = transpiler.transpile(&program, input);
            let expected = "trait Speak {\n    fn talk(self) -> String {\n        return \"\".to_string();\n    }\n}\n";
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
            let expected = "impl Speak for Dog {\n    fn talk(self) -> String {\n        return \"Woof\".to_string();\n    }\n}\n";
            assert_eq!(rust_code, expected);
        }
    }