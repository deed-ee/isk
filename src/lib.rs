pub mod project;

use std::collections::BTreeMap;
use std::fmt;
use std::ops::Range;

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
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    /// One-based source line.
    pub line: usize,
    /// One-based Unicode scalar column.
    pub column: usize,
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
    offset: usize,
}

fn lex(source: &str) -> Result<Vec<Token>, Diagnostic> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();
    let (mut line, mut column) = (1, 1);
    while let Some((offset, ch)) = chars.next() {
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
            offset,
        });
    }
    tokens.push(Token {
        kind: TokenKind::Eof,
        line,
        column,
        offset: source.len(),
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
    name_offset: usize,
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
    offset: usize,
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
        let name_offset = self.peek().offset;
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
            name_offset,
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
            offset: token.offset,
        })
    }

    fn binary(left: Expr, op: BinaryOp, right: Expr) -> Expr {
        Expr {
            line: left.line,
            column: left.column,
            offset: left.offset,
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

// Source-order identity within a single program; renaming does not reorder definitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct FunctionId(usize);

#[derive(Clone)]
struct FunctionSignature {
    id: FunctionId,
    params: Vec<Type>,
    result: Type,
}

fn check(program: &Program) -> Result<Checker, Diagnostic> {
    let mut signatures = BTreeMap::new();
    for (index, function) in program.functions.iter().enumerate() {
        if signatures
            .insert(
                function.name.clone(),
                FunctionSignature {
                    id: FunctionId(index),
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
    let mut checker = Checker {
        signatures,
        references: BTreeMap::new(),
        caller: FunctionId(0),
    };
    for (index, function) in program.functions.iter().enumerate() {
        checker.caller = FunctionId(index);
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
    Ok(checker)
}

struct Checker {
    signatures: BTreeMap<String, FunctionSignature>,
    references: BTreeMap<FunctionId, Vec<ResolvedCall>>,
    caller: FunctionId,
}

struct ResolvedCall {
    caller: FunctionId,
    offset: usize,
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
                self.references
                    .entry(signature.id)
                    .or_default()
                    .push(ResolvedCall {
                        caller: self.caller,
                        offset: expr.offset,
                    });
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

fn emit_function_name(name: &str, output: &mut String) {
    // Keep Rust's keywords out of generated function names without changing ISK names.
    if name != "main" {
        output.push_str("isk_fn_");
    }
    output.push_str(name);
}

fn emit_binding_name(name: &str, output: &mut String) {
    output.push_str("isk_var_");
    output.push_str(name);
}

fn generate(program: &Program) -> String {
    let mut output = String::from("// Generated by isk; do not edit.\n");
    for function in &program.functions {
        output.push_str("fn ");
        emit_function_name(&function.name, &mut output);
        output.push('(');
        for (index, (name, ty)) in function.params.iter().enumerate() {
            if index != 0 {
                output.push_str(", ");
            }
            emit_binding_name(name, &mut output);
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
        emit_binding_name(&statement.name, output);
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
        ExprKind::Number(value) => {
            output.push_str(&value.to_string());
            output.push_str("_u64");
        }
        ExprKind::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        ExprKind::Variable(name) => emit_binding_name(name, output),
        ExprKind::Call(name, args) if name == "print" => {
            output.push_str("println!(\"{}\", ");
            emit_expr(&args[0], output, indent);
            output.push(')');
        }
        ExprKind::Call(name, args) => {
            // ISK resolves functions independently of local variables.
            output.push_str("self::");
            emit_function_name(name, output);
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

fn parse(source: &str) -> Result<Program, Diagnostic> {
    Parser::new(lex(source)?).parse()
}

/// Validate source without generating backend code; return the first diagnostic.
pub fn check_source(source: &str) -> Result<(), Diagnostic> {
    CheckedProgram::validate(source).map(|_| ())
}

pub fn compile(source: &str) -> Result<String, String> {
    Ok(generate(&CheckedProgram::new(source)?.program))
}

pub struct TestRoot<'a> {
    pub id: usize,
    pub name: &'a str,
    pub declaration: Range<usize>,
}

/// Build a Rust driver from the same snapshot used to validate the roots.
pub fn compile_test_harness<'a>(
    program: &'a CheckedProgram<'_>,
    roots: &[&str],
) -> Result<(Vec<TestRoot<'a>>, String), String> {
    use std::fmt::Write as _;
    let tests = program.test_roots(roots)?;
    let mut rust = String::from("mod isk_test_program {\n");
    rust.push_str(&generate(&program.program));
    rust.push_str("pub(super) fn run(index: usize) -> bool {\nmatch index {\n");
    for (index, test) in tests.iter().enumerate() {
        write!(rust, "{index} => self::").unwrap();
        emit_function_name(test.name, &mut rust);
        rust.push_str("(),\n");
    }
    rust.push_str("_ => unreachable!(),\n}\n}\n}\n");
    rust.push_str(
        "fn main() {\n\
         let index = std::env::args().nth(1).unwrap().parse().unwrap();\n\
         if !isk_test_program::run(index) { std::process::exit(2); }\n\
         }\n",
    );
    Ok((tests, rust))
}

/// A validated snapshot of one source file, independent of a frontend or backend.
pub struct CheckedProgram<'a> {
    source: &'a str,
    program: Program,
    semantics: Checker,
}

/// A checked function and the locations of its name and resolved calls.
#[derive(Debug, PartialEq, Eq)]
pub struct FunctionInfo {
    /// Zero-based declaration order, scoped to this source file.
    pub id: usize,
    pub name: String,
    pub parameters: Vec<(String, &'static str)>,
    pub result: &'static str,
    /// Half-open UTF-8 byte range of the declared name.
    pub declaration: Range<usize>,
    /// Half-open UTF-8 byte ranges of callee names, in source order.
    pub calls: Vec<Range<usize>>,
}

#[derive(Debug)]
pub struct UnknownFunction<'a>(pub &'a str);

impl UnknownFunction<'_> {
    pub const CODE: &'static str = "ISK-INSPECT-001";

    pub fn message(&self) -> String {
        format!("unknown function `{}`", self.0)
    }
}

impl fmt::Display for UnknownFunction<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: unknown function `{}`", Self::CODE, self.0)
    }
}

/// A direct call and its enclosing function in the checked source snapshot.
#[derive(Debug, PartialEq, Eq)]
pub struct CallSite<'a> {
    pub caller_id: usize,
    pub caller_name: &'a str,
    /// Half-open UTF-8 byte range of the callee name.
    pub callee_range: Range<usize>,
}

/// Advisory verification scope for one changed function in a checked snapshot.
#[derive(Debug, PartialEq, Eq)]
pub enum TestSelection<'a> {
    Focused(Vec<&'a str>),
    Full(TestFallback),
}

#[derive(Debug, PartialEq, Eq)]
pub enum TestFallback {
    NoRoots,
    UnknownChange,
    UnknownRoot,
    NoReachableRoot,
}

impl TestFallback {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NoRoots => "no-roots",
            Self::UnknownChange => "unknown-change",
            Self::UnknownRoot => "unknown-root",
            Self::NoReachableRoot => "no-reachable-root",
        }
    }
}

/// A validated semantic rename tied to the exact source it was planned against.
pub struct RenamePlan<'a> {
    source: &'a str,
    replacement: String,
    edits: Vec<Range<usize>>,
    preview: String,
}

impl RenamePlan<'_> {
    pub fn edits(&self) -> &[Range<usize>] {
        &self.edits
    }

    pub fn replacement(&self) -> &str {
        &self.replacement
    }

    pub fn apply(self, current_source: &str) -> Result<String, String> {
        if current_source != self.source {
            return Err("ISK-EDIT-005: source changed since the rename was planned".to_owned());
        }
        Ok(self.preview)
    }
}

pub fn inspect_function(source: &str, name: &str) -> Result<FunctionInfo, String> {
    CheckedProgram::new(source)?
        .inspect_function(name)
        .map_err(|error| error.to_string())
}

/// Return a checked copy with a function declaration and its resolved calls renamed.
pub fn rename_function(source: &str, from: &str, to: &str) -> Result<String, String> {
    CheckedProgram::new(source)?.rename_function(from, to)
}

impl<'a> CheckedProgram<'a> {
    pub fn new(source: &'a str) -> Result<Self, String> {
        Self::validate(source).map_err(|diagnostic| diagnostic.to_string())
    }

    /// Create a checked snapshot, retaining the structured source diagnostic on failure.
    pub fn validate(source: &'a str) -> Result<Self, Diagnostic> {
        let program = parse(source)?;
        let semantics = check(&program)?;
        Ok(Self {
            source,
            program,
            semantics,
        })
    }

    /// Resolve explicit zero-argument boolean roots in declaration order.
    pub fn test_roots(&self, roots: &[&str]) -> Result<Vec<TestRoot<'_>>, String> {
        if roots.is_empty() {
            return Err("at least one test root is required".to_owned());
        }
        let mut tests = Vec::new();
        for name in roots {
            let signature = self
                .function_signature(name)
                .map_err(|error| error.to_string())?;
            if !signature.params.is_empty() || signature.result != Type::Bool {
                return Err(format!(
                    "test root `{name}` must have signature `fn {name}() -> bool`"
                ));
            }
            let id = signature.id.0;
            let function = &self.program.functions[id];
            tests.push(TestRoot {
                id,
                name: &function.name,
                declaration: function.name_offset..function.name_offset + function.name.len(),
            });
        }
        tests.sort_unstable_by_key(|test| test.id);
        tests.dedup_by_key(|test| test.id);

        Ok(tests)
    }

    pub fn inspect_function<'n>(&self, name: &'n str) -> Result<FunctionInfo, UnknownFunction<'n>> {
        let signature = self.function_signature(name)?;
        let function = &self.program.functions[signature.id.0];
        let name_range = |offset| offset..offset + function.name.len();
        Ok(FunctionInfo {
            id: signature.id.0,
            name: function.name.clone(),
            parameters: function
                .params
                .iter()
                .map(|(name, ty)| (name.clone(), ty.source_name()))
                .collect(),
            result: signature.result.source_name(),
            declaration: name_range(function.name_offset),
            calls: self
                .references(signature.id)
                .map(|call| name_range(call.offset))
                .collect(),
        })
    }

    /// Return direct call sites in source order, including recursive calls.
    pub fn call_sites(&self, name: &str) -> Result<impl Iterator<Item = CallSite<'_>>, String> {
        let signature = self
            .function_signature(name)
            .map_err(|error| error.to_string())?;
        let name_len = self.program.functions[signature.id.0].name.len();
        Ok(self.references(signature.id).map(move |call| CallSite {
            caller_id: call.caller.0,
            caller_name: &self.program.functions[call.caller.0].name,
            callee_range: call.offset..call.offset + name_len,
        }))
    }

    /// Find declared test roots that can reach a changed function via static calls.
    /// The caller must supply all known roots and a body-only change in this file.
    /// This does not discover tests, measure coverage, or establish correctness.
    pub fn affected_tests(&self, changed: &str, roots: &[&str]) -> TestSelection<'_> {
        if roots.is_empty() {
            return TestSelection::Full(TestFallback::NoRoots);
        }
        let Some(changed) = self.semantics.signatures.get(changed) else {
            return TestSelection::Full(TestFallback::UnknownChange);
        };
        let mut is_root = vec![false; self.program.functions.len()];
        for root in roots {
            let Some(signature) = self.semantics.signatures.get(*root) else {
                return TestSelection::Full(TestFallback::UnknownRoot);
            };
            is_root[signature.id.0] = true;
        }

        let mut pending = vec![changed.id];
        let mut affected = vec![false; self.program.functions.len()];
        while let Some(id) = pending.pop() {
            if !affected[id.0] {
                affected[id.0] = true;
                pending.extend(self.references(id).map(|call| call.caller));
            }
        }
        let selected: Vec<_> = self
            .program
            .functions
            .iter()
            .zip(affected.into_iter().zip(is_root))
            .filter_map(|(function, (affected, is_root))| {
                (affected && is_root).then_some(function.name.as_str())
            })
            .collect();
        if selected.is_empty() {
            TestSelection::Full(TestFallback::NoReachableRoot)
        } else {
            TestSelection::Focused(selected)
        }
    }

    fn function_signature<'n>(
        &self,
        name: &'n str,
    ) -> Result<&FunctionSignature, UnknownFunction<'n>> {
        self.semantics
            .signatures
            .get(name)
            .ok_or(UnknownFunction(name))
    }

    fn references(&self, id: FunctionId) -> impl Iterator<Item = &ResolvedCall> {
        self.semantics.references.get(&id).into_iter().flatten()
    }

    /// Preview a validated rename without modifying this snapshot.
    pub fn rename_function(&self, from: &str, to: &str) -> Result<String, String> {
        self.plan_rename(from, to)?.apply(self.source)
    }

    pub fn plan_rename(&self, from: &str, to: &str) -> Result<RenamePlan<'a>, String> {
        for name in [from, to] {
            if matches!(name, "main" | "print") {
                return Err(format!("ISK-EDIT-001: `{name}` is reserved"));
            }
        }
        let Some(signature) = self.semantics.signatures.get(from) else {
            return Err(format!("ISK-EDIT-002: unknown function `{from}`"));
        };
        if !matches!(
            lex(to).as_deref(),
            Ok([Token { kind: TokenKind::Ident(name), .. }, Token { kind: TokenKind::Eof, .. }])
                if name == to
        ) {
            return Err(format!("ISK-EDIT-003: invalid function name `{to}`"));
        }
        if from != to && self.semantics.signatures.contains_key(to) {
            return Err(format!("ISK-EDIT-004: function `{to}` already exists"));
        }

        let id = signature.id;
        let mut edits = Vec::new();
        if from != to {
            let name_range = |offset| offset..offset + from.len();
            edits.push(name_range(self.program.functions[id.0].name_offset));
            edits.extend(self.references(id).map(|call| name_range(call.offset)));
            edits.sort_unstable_by_key(|range| range.start);
        }

        let mut edited = String::with_capacity(self.source.len());
        let mut cursor = 0;
        for range in &edits {
            edited.push_str(&self.source[cursor..range.start]);
            edited.push_str(to);
            cursor = range.end;
        }
        edited.push_str(&self.source[cursor..]);
        if !edits.is_empty() {
            CheckedProgram::new(&edited)?;
        }
        Ok(RenamePlan {
            source: self.source,
            replacement: to.to_owned(),
            edits,
            preview: edited,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{check, compile, parse, rename_function, CheckedProgram};

    #[test]
    fn checked_snapshot_supports_queries_and_rename_previews_without_mutation() {
        let source = "fn step() {} fn other() {} fn main() { step() }";
        let snapshot = CheckedProgram::new(source).unwrap();
        let before = snapshot.inspect_function("step").unwrap();
        assert!(snapshot.rename_function("step", "other").is_err());
        let preview = snapshot.rename_function("step", "advance").unwrap();
        assert_eq!(snapshot.inspect_function("step").unwrap(), before);
        assert!(snapshot.inspect_function("advance").is_err());
        let updated = CheckedProgram::new(&preview).unwrap();
        let renamed = updated.inspect_function("advance").unwrap();
        assert_eq!(renamed.id, before.id);
        assert_eq!(renamed.calls.len(), 1);
        assert_eq!(&preview[renamed.calls[0].clone()], "advance");
    }

    #[test]
    fn compiles_factorial_to_safe_rust() {
        let source = include_str!("../examples/factorial.isk");
        let rust = compile(source).unwrap();
        assert!(rust.contains("fn isk_fn_factorial(isk_var_n: u64) -> u64"));
        assert!(rust.contains("println!(\"{}\", self::isk_fn_factorial(10_u64))"));
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

    #[test]
    fn renaming_preserves_ids_and_uses_resolved_calls_instead_of_text() {
        let source =
            "fn step(n: u64) -> u64 {\r\n    if n <= 1 { n } else { step(n - 1) }\r\n}\r\n\
                      fn step_limit(step: u64) -> u64 { step }\r\n\
                      fn main() { let step = 2; print(\u{2003}step(step_limit(step))) }\r\n";
        let expected =
            "fn advance(n: u64) -> u64 {\r\n    if n <= 1 { n } else { advance(n - 1) }\r\n}\r\n\
                        fn step_limit(step: u64) -> u64 { step }\r\n\
                        fn main() { let step = 2; print(\u{2003}advance(step_limit(step))) }\r\n";
        let edited = rename_function(source, "step", "advance").unwrap();
        assert_eq!(edited, expected);
        assert_ne!(source.replace("step", "advance"), edited);

        let before = check(&parse(source).unwrap()).unwrap();
        let repeated = check(&parse(source).unwrap()).unwrap();
        let after = check(&parse(&edited).unwrap()).unwrap();
        let id = before.signatures["step"].id;
        assert_eq!(id, repeated.signatures["step"].id);
        assert_eq!(id, after.signatures["advance"].id);
        assert_eq!(before.references[&id].len(), after.references[&id].len());
    }

    #[test]
    fn renaming_rejects_invalid_names_and_invalid_programs() {
        let source = "fn helper() {} fn main() { helper() }";
        for (from, to, code) in [
            ("helper", "main", "ISK-EDIT-001"),
            ("print", "helper", "ISK-EDIT-001"),
            ("missing", "other", "ISK-EDIT-002"),
            ("helper", "if", "ISK-EDIT-003"),
            ("helper", " two ", "ISK-EDIT-003"),
        ] {
            assert!(rename_function(source, from, to)
                .unwrap_err()
                .starts_with(code));
        }
        assert_eq!(rename_function(source, "helper", "helper").unwrap(), source);
        assert!(rename_function("fn main() { missing() }", "main", "other")
            .unwrap_err()
            .contains("ISK-NAME-002"));
    }
}
