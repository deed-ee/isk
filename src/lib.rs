use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Type {
    U64,
    Bool,
    Unit,
}

impl Type {
    fn source_name(self) -> &'static str {
        match self {
            Type::U64 => "u64",
            Type::Bool => "bool",
            Type::Unit => "()",
        }
    }
}

#[derive(Clone, Debug)]
struct Diagnostic {
    code: &'static str,
    message: String,
    line: usize,
    column: usize,
}

impl Diagnostic {
    fn new(code: &'static str, message: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            code,
            message: message.into(),
            line,
            column,
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}: {}",
            self.line, self.column, self.code, self.message
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
enum TokenKind {
    Ident(String),
    Number(u64),
    Fn,
    Let,
    Mut,
    If,
    Else,
    True,
    False,
    U64,
    Bool,
    Arrow,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Colon,
    Comma,
    Semicolon,
    Plus,
    Minus,
    Star,
    Slash,
    Less,
    LessEqual,
    EqualEqual,
    Equal,
    Eof,
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    line: usize,
    column: usize,
}

fn lex(source: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();
    let (mut line, mut column) = (1, 1);
    while let Some((_, ch)) = chars.next() {
        let token_line = line;
        let token_column = column;
        if ch == '\n' {
            line += 1;
            column = 1;
            continue;
        }
        if ch.is_whitespace() {
            column += 1;
            continue;
        }
        let kind = if ch.is_ascii_alphabetic() || ch == '_' {
            let mut word = String::from(ch);
            column += 1;
            while let Some((_, next)) = chars.peek() {
                if next.is_ascii_alphanumeric() || *next == '_' {
                    word.push(*next);
                    chars.next();
                    column += 1;
                } else {
                    break;
                }
            }
            match word.as_str() {
                "fn" => TokenKind::Fn,
                "let" => TokenKind::Let,
                "mut" => TokenKind::Mut,
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                "u64" => TokenKind::U64,
                "bool" => TokenKind::Bool,
                _ => TokenKind::Ident(word),
            }
        } else if ch.is_ascii_digit() {
            let mut digits = String::from(ch);
            column += 1;
            while let Some((_, next)) = chars.peek() {
                if next.is_ascii_digit() {
                    digits.push(*next);
                    chars.next();
                    column += 1;
                } else {
                    break;
                }
            }
            let value = digits.parse::<u64>().map_err(|_| {
                Diagnostic::new(
                    "ISK-LEX-001",
                    "integer literal is out of range",
                    line,
                    token_column,
                )
            })?;
            TokenKind::Number(value)
        } else {
            column += 1;
            match ch {
                '(' => TokenKind::LParen,
                ')' => TokenKind::RParen,
                '{' => TokenKind::LBrace,
                '}' => TokenKind::RBrace,
                ':' => TokenKind::Colon,
                ',' => TokenKind::Comma,
                ';' => TokenKind::Semicolon,
                '+' => TokenKind::Plus,
                '-' if chars.peek().is_some_and(|(_, next)| *next == '>') => {
                    chars.next();
                    column += 1;
                    TokenKind::Arrow
                }
                '-' => TokenKind::Minus,
                '*' => TokenKind::Star,
                '/' => TokenKind::Slash,
                '<' if chars.peek().is_some_and(|(_, next)| *next == '=') => {
                    chars.next();
                    column += 1;
                    TokenKind::LessEqual
                }
                '<' => TokenKind::Less,
                '=' if chars.peek().is_some_and(|(_, next)| *next == '=') => {
                    chars.next();
                    column += 1;
                    TokenKind::EqualEqual
                }
                '=' => TokenKind::Equal,
                _ => {
                    return Err(Diagnostic::new(
                        "ISK-LEX-002",
                        format!("unexpected character `{ch}`"),
                        line,
                        token_column,
                    ))
                }
            }
        };
        tokens.push(Token {
            kind,
            line: token_line,
            column: token_column,
        });
    }
    tokens.push(Token {
        kind: TokenKind::Eof,
        line,
        column,
    });
    Ok(tokens)
}

#[derive(Clone, Debug)]
struct Program {
    functions: Vec<Function>,
}

#[derive(Clone, Debug)]
struct Function {
    name: String,
    params: Vec<(String, Type)>,
    result: Type,
    body: Block,
    line: usize,
    column: usize,
}

#[derive(Clone, Debug)]
struct Block {
    statements: Vec<Statement>,
    result: Option<Box<Expr>>,
}

#[derive(Clone, Debug)]
struct Statement {
    name: String,
    mutable: bool,
    annotation: Option<Type>,
    value: Expr,
    line: usize,
    column: usize,
}

#[derive(Clone, Debug)]
struct Expr {
    kind: ExprKind,
    line: usize,
    column: usize,
}

#[derive(Clone, Debug)]
enum ExprKind {
    Number(u64),
    Bool(bool),
    Variable(String),
    Call(String, Vec<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    If(Box<Expr>, Block, Block),
}

#[derive(Clone, Copy, Debug)]
enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Less,
    LessEqual,
    Equal,
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse(mut self) -> Result<Program, Diagnostic> {
        let mut functions = Vec::new();
        while !self.at(&TokenKind::Eof) {
            functions.push(self.function()?);
        }
        Ok(Program { functions })
    }

    fn function(&mut self) -> Result<Function, Diagnostic> {
        let start = self.expect(&TokenKind::Fn, "expected `fn`")?;
        let name = self.ident("expected function name")?;
        self.expect(&TokenKind::LParen, "expected `(` after function name")?;
        let mut params = Vec::new();
        if !self.at(&TokenKind::RParen) {
            loop {
                let param = self.ident("expected parameter name")?;
                self.expect(&TokenKind::Colon, "expected `:` after parameter name")?;
                params.push((param, self.type_name()?));
                if !self.consume(&TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen, "expected `)` after parameters")?;
        let result = if self.consume(&TokenKind::Arrow) {
            self.type_name()?
        } else {
            Type::Unit
        };
        let body = self.block()?;
        Ok(Function {
            name,
            params,
            result,
            body,
            line: start.line,
            column: start.column,
        })
    }

    fn type_name(&mut self) -> Result<Type, Diagnostic> {
        let token = self.advance().clone();
        match token.kind {
            TokenKind::U64 => Ok(Type::U64),
            TokenKind::Bool => Ok(Type::Bool),
            _ => Err(Diagnostic::new(
                "ISK-PARSE-001",
                "expected type `u64` or `bool`",
                token.line,
                token.column,
            )),
        }
    }

    fn block(&mut self) -> Result<Block, Diagnostic> {
        self.expect(&TokenKind::LBrace, "expected `{`")?;
        let mut statements = Vec::new();
        while self.at(&TokenKind::Let) || self.at(&TokenKind::Mut) {
            let start = self.advance().clone();
            let mutable = start.kind == TokenKind::Mut;
            let name = self.ident("expected binding name")?;
            let annotation = if self.consume(&TokenKind::Colon) {
                Some(self.type_name()?)
            } else {
                None
            };
            self.expect(&TokenKind::Equal, "expected `=` in binding")?;
            let value = self.expression()?;
            self.expect(&TokenKind::Semicolon, "expected `;` after binding")?;
            statements.push(Statement {
                name,
                mutable,
                annotation,
                value,
                line: start.line,
                column: start.column,
            });
        }
        let result = if self.at(&TokenKind::RBrace) {
            None
        } else {
            Some(Box::new(self.expression()?))
        };
        self.expect(&TokenKind::RBrace, "expected `}` after block")?;
        Ok(Block { statements, result })
    }

    fn expression(&mut self) -> Result<Expr, Diagnostic> {
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.additive()?;
        loop {
            let op = if self.consume(&TokenKind::Less) {
                Some(BinaryOp::Less)
            } else if self.consume(&TokenKind::LessEqual) {
                Some(BinaryOp::LessEqual)
            } else if self.consume(&TokenKind::EqualEqual) {
                Some(BinaryOp::Equal)
            } else {
                None
            };
            let Some(op) = op else { break };
            let right = self.additive()?;
            expr = Self::binary(expr, op, right);
        }
        Ok(expr)
    }

    fn additive(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.multiplicative()?;
        loop {
            let op = if self.consume(&TokenKind::Plus) {
                Some(BinaryOp::Add)
            } else if self.consume(&TokenKind::Minus) {
                Some(BinaryOp::Subtract)
            } else {
                None
            };
            let Some(op) = op else { break };
            let right = self.multiplicative()?;
            expr = Self::binary(expr, op, right);
        }
        Ok(expr)
    }

    fn multiplicative(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.primary()?;
        loop {
            let op = if self.consume(&TokenKind::Star) {
                Some(BinaryOp::Multiply)
            } else if self.consume(&TokenKind::Slash) {
                Some(BinaryOp::Divide)
            } else {
                None
            };
            let Some(op) = op else { break };
            let right = self.primary()?;
            expr = Self::binary(expr, op, right);
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.advance().clone();
        let kind = match token.kind {
            TokenKind::Number(value) => ExprKind::Number(value),
            TokenKind::True => ExprKind::Bool(true),
            TokenKind::False => ExprKind::Bool(false),
            TokenKind::Ident(name) => {
                if self.consume(&TokenKind::LParen) {
                    let mut args = Vec::new();
                    if !self.at(&TokenKind::RParen) {
                        loop {
                            args.push(self.expression()?);
                            if !self.consume(&TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(&TokenKind::RParen, "expected `)` after arguments")?;
                    ExprKind::Call(name, args)
                } else {
                    ExprKind::Variable(name)
                }
            }
            TokenKind::If => {
                let condition = self.expression()?;
                let then_block = self.block()?;
                self.expect(&TokenKind::Else, "expected `else` after `if` block")?;
                let else_block = self.block()?;
                ExprKind::If(Box::new(condition), then_block, else_block)
            }
            TokenKind::LParen => {
                let expr = self.expression()?;
                self.expect(&TokenKind::RParen, "expected `)`")?;
                return Ok(expr);
            }
            _ => {
                return Err(Diagnostic::new(
                    "ISK-PARSE-002",
                    "expected an expression",
                    token.line,
                    token.column,
                ))
            }
        };
        Ok(Expr {
            kind,
            line: token.line,
            column: token.column,
        })
    }

    fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
        Expr {
            line: left.line,
            column: left.column,
            kind: ExprKind::Binary(Box::new(left), op, Box::new(right)),
        }
    }

    fn ident(&mut self, message: &str) -> Result<String, Diagnostic> {
        let token = self.advance().clone();
        if let TokenKind::Ident(name) = token.kind {
            Ok(name)
        } else {
            Err(Diagnostic::new(
                "ISK-PARSE-003",
                message,
                token.line,
                token.column,
            ))
        }
    }

    fn expect(&mut self, kind: &TokenKind, message: &str) -> Result<Token, Diagnostic> {
        if self.at(kind) {
            Ok(self.advance().clone())
        } else {
            let token = self.peek();
            Err(Diagnostic::new(
                "ISK-PARSE-004",
                message,
                token.line,
                token.column,
            ))
        }
    }

    fn consume(&mut self, kind: &TokenKind) -> bool {
        if self.at(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn at(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    fn advance(&mut self) -> &Token {
        let index = self.current;
        if !self.at(&TokenKind::Eof) {
            self.current += 1;
        }
        &self.tokens[index]
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
}

#[derive(Clone)]
struct FunctionSignature {
    params: Vec<Type>,
    result: Type,
}

fn check(program: &Program) -> Result<(), Diagnostic> {
    let mut signatures = BTreeMap::new();
    for function in &program.functions {
        if signatures
            .insert(
                function.name.clone(),
                FunctionSignature {
                    params: function.params.iter().map(|(_, ty)| *ty).collect(),
                    result: function.result,
                },
            )
            .is_some()
        {
            return Err(Diagnostic::new(
                "ISK-SEMA-001",
                format!("function `{}` is defined more than once", function.name),
                function.line,
                function.column,
            ));
        }
    }
    if !signatures.contains_key("main") {
        return Err(Diagnostic::new(
            "ISK-SEMA-002",
            "program must define `fn main()`",
            1,
            1,
        ));
    }
    let mut checker = Checker { signatures };
    for function in &program.functions {
        if function.name == "main" && (!function.params.is_empty() || function.result != Type::Unit)
        {
            return Err(Diagnostic::new(
                "ISK-SEMA-003",
                "`main` must have no parameters and return no value",
                function.line,
                function.column,
            ));
        }
        let mut scope = BTreeMap::new();
        for (name, ty) in &function.params {
            if scope.insert(name.clone(), *ty).is_some() {
                return Err(Diagnostic::new(
                    "ISK-SEMA-004",
                    format!("duplicate parameter `{name}`"),
                    function.line,
                    function.column,
                ));
            }
        }
        let actual = checker.block(&function.body, &mut scope)?;
        if actual != function.result {
            return Err(Diagnostic::new(
                "ISK-TYPE-001",
                format!(
                    "function `{}` returns `{}`, expected `{}`",
                    function.name,
                    actual.source_name(),
                    function.result.source_name()
                ),
                function.line,
                function.column,
            ));
        }
    }
    Ok(())
}

struct Checker {
    signatures: BTreeMap<String, FunctionSignature>,
}

impl Checker {
    fn block(
        &mut self,
        block: &Block,
        scope: &mut BTreeMap<String, Type>,
    ) -> Result<Type, Diagnostic> {
        for statement in &block.statements {
            let actual = self.expression(&statement.value, scope)?;
            let ty = statement.annotation.unwrap_or(actual);
            if actual != ty {
                return Err(Diagnostic::new(
                    "ISK-TYPE-002",
                    format!(
                        "binding annotation expects `{}`, found `{}`",
                        ty.source_name(),
                        actual.source_name()
                    ),
                    statement.line,
                    statement.column,
                ));
            }
            if scope.insert(statement.name.clone(), ty).is_some() {
                return Err(Diagnostic::new(
                    "ISK-SEMA-005",
                    format!(
                        "binding `{}` is already defined in this scope",
                        statement.name
                    ),
                    statement.line,
                    statement.column,
                ));
            }
        }
        match &block.result {
            Some(expr) => self.expression(expr, scope),
            None => Ok(Type::Unit),
        }
    }

    fn expression(
        &mut self,
        expr: &Expr,
        scope: &mut BTreeMap<String, Type>,
    ) -> Result<Type, Diagnostic> {
        match &expr.kind {
            ExprKind::Number(_) => Ok(Type::U64),
            ExprKind::Bool(_) => Ok(Type::Bool),
            ExprKind::Variable(name) => scope.get(name).copied().ok_or_else(|| {
                Diagnostic::new(
                    "ISK-NAME-001",
                    format!("unknown variable `{name}`"),
                    expr.line,
                    expr.column,
                )
            }),
            ExprKind::Call(name, args) if name == "print" => {
                if args.len() != 1 {
                    return Err(Diagnostic::new(
                        "ISK-TYPE-003",
                        "`print` expects exactly one argument",
                        expr.line,
                        expr.column,
                    ));
                }
                let ty = self.expression(&args[0], scope)?;
                if !matches!(ty, Type::U64 | Type::Bool) {
                    return Err(Diagnostic::new(
                        "ISK-TYPE-004",
                        "`print` supports `u64` and `bool`",
                        expr.line,
                        expr.column,
                    ));
                }
                Ok(Type::Unit)
            }
            ExprKind::Call(name, args) => {
                let Some(signature) = self.signatures.get(name).cloned() else {
                    return Err(Diagnostic::new(
                        "ISK-NAME-002",
                        format!("unknown function `{name}`"),
                        expr.line,
                        expr.column,
                    ));
                };
                if args.len() != signature.params.len() {
                    return Err(Diagnostic::new(
                        "ISK-TYPE-005",
                        format!(
                            "function `{name}` expects {} arguments, found {}",
                            signature.params.len(),
                            args.len()
                        ),
                        expr.line,
                        expr.column,
                    ));
                }
                for (arg, expected) in args.iter().zip(signature.params) {
                    let actual = self.expression(arg, scope)?;
                    if actual != expected {
                        return Err(Diagnostic::new(
                            "ISK-TYPE-006",
                            format!(
                                "argument to `{name}` has type `{}`, expected `{}`",
                                actual.source_name(),
                                expected.source_name()
                            ),
                            arg.line,
                            arg.column,
                        ));
                    }
                }
                Ok(signature.result)
            }
            ExprKind::Binary(left, op, right) => {
                let left_type = self.expression(left, scope)?;
                let right_type = self.expression(right, scope)?;
                if left_type != right_type {
                    return Err(Diagnostic::new(
                        "ISK-TYPE-007",
                        format!(
                            "operator operands have different types: `{}` and `{}`",
                            left_type.source_name(),
                            right_type.source_name()
                        ),
                        expr.line,
                        expr.column,
                    ));
                }
                match op {
                    BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide
                        if left_type == Type::U64 =>
                    {
                        Ok(Type::U64)
                    }
                    BinaryOp::Less | BinaryOp::LessEqual if left_type == Type::U64 => {
                        Ok(Type::Bool)
                    }
                    BinaryOp::Equal if matches!(left_type, Type::U64 | Type::Bool) => {
                        Ok(Type::Bool)
                    }
                    _ => Err(Diagnostic::new(
                        "ISK-TYPE-008",
                        "operator is not defined for these operand types",
                        expr.line,
                        expr.column,
                    )),
                }
            }
            ExprKind::If(condition, then_block, else_block) => {
                let condition_type = self.expression(condition, scope)?;
                if condition_type != Type::Bool {
                    return Err(Diagnostic::new(
                        "ISK-TYPE-009",
                        "if condition must have type `bool`",
                        condition.line,
                        condition.column,
                    ));
                }
                let then_type = self.block(then_block, &mut scope.clone())?;
                let else_type = self.block(else_block, &mut scope.clone())?;
                if then_type != else_type {
                    return Err(Diagnostic::new(
                        "ISK-TYPE-010",
                        "if branches must have the same type",
                        expr.line,
                        expr.column,
                    ));
                }
                Ok(then_type)
            }
        }
    }
}

fn generate(program: &Program) -> String {
    let mut output = String::from("// Generated by isk; do not edit.\n");
    for function in &program.functions {
        output.push_str("fn ");
        output.push_str(&function.name);
        output.push('(');
        for (index, (name, ty)) in function.params.iter().enumerate() {
            if index != 0 {
                output.push_str(", ");
            }
            output.push_str(name);
            output.push_str(": ");
            output.push_str(ty.source_name());
        }
        output.push(')');
        if function.result != Type::Unit {
            output.push_str(" -> ");
            output.push_str(function.result.source_name());
        }
        output.push(' ');
        emit_block(&function.body, &mut output, 0);
        output.push('\n');
    }
    output
}

fn emit_block(block: &Block, output: &mut String, indent: usize) {
    output.push_str("{\n");
    for statement in &block.statements {
        output.push_str(&" ".repeat(indent + 4));
        output.push_str(if statement.mutable {
            "let mut "
        } else {
            "let "
        });
        output.push_str(&statement.name);
        if let Some(ty) = statement.annotation {
            output.push_str(": ");
            output.push_str(ty.source_name());
        }
        output.push_str(" = ");
        emit_expr(&statement.value, output, indent + 4);
        output.push_str(";\n");
    }
    if let Some(expr) = &block.result {
        output.push_str(&" ".repeat(indent + 4));
        emit_expr(expr, output, indent + 4);
        output.push('\n');
    }
    output.push_str(&" ".repeat(indent));
    output.push('}');
}

fn emit_expr(expr: &Expr, output: &mut String, indent: usize) {
    match &expr.kind {
        ExprKind::Number(value) => output.push_str(&value.to_string()),
        ExprKind::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        ExprKind::Variable(name) => output.push_str(name),
        ExprKind::Call(name, args) if name == "print" => {
            output.push_str("println!(\"{}\", ");
            emit_expr(&args[0], output, indent);
            output.push(')');
        }
        ExprKind::Call(name, args) => {
            output.push_str(name);
            output.push('(');
            for (index, arg) in args.iter().enumerate() {
                if index != 0 {
                    output.push_str(", ");
                }
                emit_expr(arg, output, indent);
            }
            output.push(')');
        }
        ExprKind::Binary(left, op, right) => match op {
            BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply | BinaryOp::Divide => {
                output.push('(');
                emit_expr(left, output, indent);
                output.push_str(match op {
                    BinaryOp::Add => ").checked_add(",
                    BinaryOp::Subtract => ").checked_sub(",
                    BinaryOp::Multiply => ").checked_mul(",
                    BinaryOp::Divide => ").checked_div(",
                    _ => unreachable!(),
                });
                emit_expr(right, output, indent);
                output.push_str(if matches!(op, BinaryOp::Divide) {
                    ").expect(\"ISK division by zero\")"
                } else {
                    ").expect(\"ISK integer overflow\")"
                });
            }
            _ => {
                output.push('(');
                emit_expr(left, output, indent);
                output.push_str(match op {
                    BinaryOp::Less => " < ",
                    BinaryOp::LessEqual => " <= ",
                    BinaryOp::Equal => " == ",
                    _ => unreachable!(),
                });
                emit_expr(right, output, indent);
                output.push(')');
            }
        },
        ExprKind::If(condition, then_block, else_block) => {
            output.push_str("if ");
            emit_expr(condition, output, indent);
            output.push(' ');
            emit_block(then_block, output, indent);
            output.push_str(" else ");
            emit_block(else_block, output, indent);
        }
    }
}

pub fn compile(source: &str) -> Result<String, String> {
    let tokens = lex(source).map_err(|diagnostic| diagnostic.to_string())?;
    let program = Parser::new(tokens)
        .parse()
        .map_err(|diagnostic| diagnostic.to_string())?;
    check(&program).map_err(|diagnostic| diagnostic.to_string())?;
    Ok(generate(&program))
}

#[cfg(test)]
mod tests {
    use super::compile;

    #[test]
    fn compiles_factorial_to_safe_rust() {
        let source = include_str!("../examples/factorial.isk");
        let rust = compile(source).unwrap();
        assert!(rust.contains("fn factorial(n: u64) -> u64"));
        assert!(rust.contains("println!(\"{}\", factorial(10))"));
        assert!(rust.contains("checked_mul"));
        assert!(rust.contains("checked_sub"));
        assert!(!rust.contains("unsafe"));
    }

    #[test]
    fn reports_type_errors_with_stable_codes() {
        let error = compile("fn main() { let answer: bool = 42; }").unwrap_err();
        assert!(error.contains("ISK-TYPE-002"));
        assert!(error.contains("expects `bool`, found `u64`"));
    }

    #[test]
    fn rejects_missing_entrypoint() {
        let error = compile("fn helper() {}").unwrap_err();
        assert!(error.contains("ISK-SEMA-002"));
    }
}
