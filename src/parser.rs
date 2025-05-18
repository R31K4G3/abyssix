#![allow(dead_code)]

use crate::lexer::Token;

macro_rules! expect_token {
    ($tokens:expr, $expected:ident) => {{
        let Some(next_token) = $tokens.pop() else {
            panic!(
                "The keyword '{:?}' is expected, but the end of the input is found",
                Token::$expected
            );
        };
        if !matches!(next_token, Token::$expected) {
            panic!(
                "The keyword '{:?}' is expected, but other token ({:?}) is found",
                Token::$expected,
                next_token
            );
        };
    }};
}

fn expect_int(tokens: &mut Vec<Token>) -> i64 {
    let Some(next_token) = tokens.pop() else {
        panic!("Int literal is expected, but the end of the input is found");
    };
    let Token::Int(val) = next_token else {
        panic!(
            "An integer is expected, but other token ({:?}) is found",
            next_token
        );
    };
    val
}

fn expect_ident(tokens: &mut Vec<Token>) -> String {
    let Some(next_token) = tokens.pop() else {
        panic!("Int literal is expected, but the end of the input is found");
    };
    let Token::Ident(val) = next_token else {
        panic!(
            "An identifier is expected, but other token ({:?}) is found",
            next_token
        );
    };
    val
}

macro_rules! consume_token {
    ($tokens:expr, $expected:ident) => {{
        if let Some(next_token) = $tokens.last() {
            if matches!(next_token, Token::$expected) {
                $tokens.pop();
                Some(())
            } else {
                None
            }
        } else {
            None
        }
    }};
}

fn consume_int(tokens: &mut Vec<Token>) -> Option<i64> {
    let Some(next_token) = tokens.last() else {
        return None;
    };
    let Token::Int(val) = next_token else {
        return None;
    };
    let val = *val;
    tokens.pop();
    Some(val)
}

fn consume_float(tokens: &mut Vec<Token>) -> Option<f64> {
    let Some(next_token) = tokens.last() else {
        return None;
    };
    let Token::Float(val) = next_token else {
        return None;
    };
    let val = *val;
    tokens.pop();
    Some(val)
}

fn consume_ident(tokens: &mut Vec<Token>) -> Option<String> {
    let Some(next_token) = tokens.last() else {
        return None;
    };
    let Token::Ident(val) = next_token else {
        return None;
    };
    let val = val.clone();
    tokens.pop();
    Some(val)
}

#[derive(Debug)]
pub enum OperandType {
    Float,
    Int,
}

#[derive(Debug)]
pub enum Expression {
    Int(i64),
    Float(f64),
    GetWithLiteralIndex(usize),
    GetParam(usize),
    GetWithComputedIndex(Box<Expression>),
    Call(String, Vec<Expression>),
    Add(OperandType, Box<Expression>, Box<Expression>),
    Sub(OperandType, Box<Expression>, Box<Expression>),
    Mul(OperandType, Box<Expression>, Box<Expression>),
    Div(OperandType, Box<Expression>, Box<Expression>),
    Rem(OperandType, Box<Expression>, Box<Expression>),
    Eq(OperandType, Box<Expression>, Box<Expression>),
    Ne(OperandType, Box<Expression>, Box<Expression>),
    Lt(OperandType, Box<Expression>, Box<Expression>),
    Gt(OperandType, Box<Expression>, Box<Expression>),
    Le(OperandType, Box<Expression>, Box<Expression>),
    Ge(OperandType, Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Xor(Box<Expression>, Box<Expression>),
    Shl(Box<Expression>, Box<Expression>),
    Shr(Box<Expression>, Box<Expression>),
    ShrUnsigned(Box<Expression>, Box<Expression>),
    Itof(Box<Expression>),
    Ftoi(Box<Expression>),
    Neg(OperandType, Box<Expression>),
    BitNot(Box<Expression>),
    LogiNot(Box<Expression>),
    ReadInputByte,
}

fn parse_primary(tokens: &mut Vec<Token>, d: &FuncSizeData) -> Expression {
    if consume_token!(tokens, GetKeyword).is_some() {
        if consume_token!(tokens, Dot).is_some() {
            let index = expect_int(tokens);
            assert!(index >= 0);
            let index: usize = index.try_into().unwrap();
            assert!(index < d.alloc_size);
            Expression::GetWithLiteralIndex(index)
        } else if consume_token!(tokens, OpeningBracket).is_some() {
            let index = parse_expr(tokens, d);
            expect_token!(tokens, ClosingBracket);
            Expression::GetWithComputedIndex(Box::new(index))
        } else if tokens.is_empty() {
            panic!("'.' or '[' is expected, but the end of the input is found");
        } else {
            panic!(
                "'.' or '[' is expected, but other token ({:?}) is found",
                tokens.pop().unwrap()
            );
        }
    } else if consume_token!(tokens, ParamKeyword).is_some() {
        expect_token!(tokens, Dot);
        let index = expect_int(tokens);
        assert!(index >= 0);
        let index: usize = index.try_into().unwrap();
        assert!(index < d.params_size);
        Expression::GetParam(d.params_size - index)
    } else if let Some(val) = consume_int(tokens) {
        Expression::Int(val)
    } else if let Some(val) = consume_float(tokens) {
        Expression::Float(val)
    } else if let Some(fname) = consume_ident(tokens) {
        expect_token!(tokens, OpeningParens);
        let args = if consume_token!(tokens, ClosingParens).is_some() {
            Vec::new()
        } else {
            let mut args = vec![parse_expr(tokens, d)];
            while consume_token!(tokens, Comma).is_some() {
                args.push(parse_expr(tokens, d));
            }
            expect_token!(tokens, ClosingParens);
            args
        };
        Expression::Call(fname, args)
    } else if consume_token!(tokens, GetcKeyword).is_some() {
        Expression::ReadInputByte
    } else if consume_token!(tokens, OpeningParens).is_some() {
        let node = parse_expr(tokens, d);
        expect_token!(tokens, ClosingParens);
        node
    } else if tokens.is_empty() {
        panic!(
            "integer, decimal, identifier, 'getc', 'get' or '(' is expected, but the end of the input is found"
        );
    } else {
        panic!(
            "integer, decimal, identifier, 'getc', 'get' or '(' is expected, but other token ({:?}) is found",
            tokens.pop().unwrap()
        );
    }
}

fn parse_unary(tokens: &mut Vec<Token>, d: &FuncSizeData) -> Expression {
    if consume_token!(tokens, Excl).is_some() {
        return Expression::LogiNot(Box::new(parse_unary(tokens, d)));
    }
    if consume_token!(tokens, Tilde).is_some() {
        return Expression::BitNot(Box::new(parse_unary(tokens, d)));
    }
    if consume_token!(tokens, FtoiKeyword).is_some() {
        return Expression::Ftoi(Box::new(parse_unary(tokens, d)));
    }
    if consume_token!(tokens, ItofKeyword).is_some() {
        return Expression::Itof(Box::new(parse_unary(tokens, d)));
    }
    if tokens.len() >= 3
        && matches!(
            &tokens[tokens.len() - 3..],
            [
                Token::Minus,
                Token::Dot,
                Token::IntKeyword | Token::FloatKeyword
            ]
        )
    {
        let operand_type = if consume_token!(tokens, IntKeyword).is_some() {
            OperandType::Int
        } else if consume_token!(tokens, FloatKeyword).is_some() {
            OperandType::Float
        } else {
            unreachable!()
        };
        expect_token!(tokens, Dot);
        expect_token!(tokens, Minus);
        return Expression::Neg(operand_type, Box::new(parse_unary(tokens, d)));
    }
    return parse_primary(tokens, d);
}

fn parse_multiplicative(tokens: &mut Vec<Token>, d: &FuncSizeData) -> Expression {
    let mut node = parse_unary(tokens, d);
    loop {
        if tokens.len() >= 3
            && matches!(
                &tokens[tokens.len() - 3..],
                [
                    Token::Asterisk | Token::Slash | Token::Percent,
                    Token::Dot,
                    Token::IntKeyword | Token::FloatKeyword
                ]
            )
        {
            // e.g.) int.*
        } else {
            return node;
        }
        let operand_type = if consume_token!(tokens, IntKeyword).is_some() {
            OperandType::Int
        } else if consume_token!(tokens, FloatKeyword).is_some() {
            OperandType::Float
        } else {
            unreachable!()
        };
        expect_token!(tokens, Dot);
        if consume_token!(tokens, Asterisk).is_some() {
            node = Expression::Mul(
                operand_type,
                Box::new(node),
                Box::new(parse_unary(tokens, d)),
            );
        } else if consume_token!(tokens, Slash).is_some() {
            node = Expression::Div(
                operand_type,
                Box::new(node),
                Box::new(parse_unary(tokens, d)),
            );
        } else if consume_token!(tokens, Percent).is_some() {
            node = Expression::Rem(
                operand_type,
                Box::new(node),
                Box::new(parse_unary(tokens, d)),
            );
        } else {
            unreachable!();
        }
    }
}

fn parse_additive(tokens: &mut Vec<Token>, d: &FuncSizeData) -> Expression {
    let mut node = parse_multiplicative(tokens, d);
    loop {
        if tokens.len() >= 3
            && matches!(
                &tokens[tokens.len() - 3..],
                [
                    Token::Plus | Token::Minus,
                    Token::Dot,
                    Token::IntKeyword | Token::FloatKeyword
                ]
            )
        {
            // e.g.) int.+
        } else {
            return node;
        }
        let operand_type = if consume_token!(tokens, IntKeyword).is_some() {
            OperandType::Int
        } else if consume_token!(tokens, FloatKeyword).is_some() {
            OperandType::Float
        } else {
            unreachable!()
        };
        expect_token!(tokens, Dot);
        if consume_token!(tokens, Plus).is_some() {
            node = Expression::Add(
                operand_type,
                Box::new(node),
                Box::new(parse_multiplicative(tokens, d)),
            );
        } else if consume_token!(tokens, Minus).is_some() {
            node = Expression::Sub(
                operand_type,
                Box::new(node),
                Box::new(parse_multiplicative(tokens, d)),
            );
        } else {
            unreachable!();
        }
    }
}

fn parse_bitshift(tokens: &mut Vec<Token>, d: &FuncSizeData) -> Expression {
    let mut node = parse_additive(tokens, d);
    loop {
        if consume_token!(tokens, DoubleLt).is_some() {
            node = Expression::Shl(Box::new(node), Box::new(parse_additive(tokens, d)));
        } else if consume_token!(tokens, DoubleGt).is_some() {
            node = Expression::Shr(Box::new(node), Box::new(parse_additive(tokens, d)));
        } else if consume_token!(tokens, TripleGt).is_some() {
            node = Expression::ShrUnsigned(Box::new(node), Box::new(parse_additive(tokens, d)));
        } else {
            return node;
        }
    }
}

fn parse_bit_and(tokens: &mut Vec<Token>, d: &FuncSizeData) -> Expression {
    let mut node = parse_bitshift(tokens, d);
    loop {
        if consume_token!(tokens, Ampersand).is_some() {
            let right = parse_bitshift(tokens, d);
            node = Expression::And(Box::new(node), Box::new(right))
        } else {
            return node;
        }
    }
}
fn parse_bit_xor(tokens: &mut Vec<Token>, d: &FuncSizeData) -> Expression {
    let mut node = parse_bit_and(tokens, d);
    loop {
        if consume_token!(tokens, Circumflex).is_some() {
            let right = parse_bit_and(tokens, d);
            node = Expression::Xor(Box::new(node), Box::new(right))
        } else {
            return node;
        }
    }
}
fn parse_bit_or(tokens: &mut Vec<Token>, d: &FuncSizeData) -> Expression {
    let mut node = parse_bit_xor(tokens, d);
    loop {
        if consume_token!(tokens, Pipe).is_some() {
            let right = parse_bit_xor(tokens, d);
            node = Expression::Or(Box::new(node), Box::new(right))
        } else {
            return node;
        }
    }
}

fn parse_relational(tokens: &mut Vec<Token>, d: &FuncSizeData) -> Expression {
    let mut node = parse_bit_or(tokens, d);

    loop {
        if tokens.len() >= 3
            && matches!(
                &tokens[tokens.len() - 3..],
                [
                    Token::GreaterThan | Token::LessThan | Token::GtEq | Token::LtEq,
                    Token::Dot,
                    Token::IntKeyword | Token::FloatKeyword
                ]
            )
        {
            // e.g.) int.>=
        } else {
            return node;
        }
        let operand_type = if consume_token!(tokens, IntKeyword).is_some() {
            OperandType::Int
        } else if consume_token!(tokens, FloatKeyword).is_some() {
            OperandType::Float
        } else {
            unreachable!()
        };
        expect_token!(tokens, Dot);
        if consume_token!(tokens, GreaterThan).is_some() {
            node = Expression::Gt(
                operand_type,
                Box::new(node),
                Box::new(parse_bit_or(tokens, d)),
            );
        } else if consume_token!(tokens, LessThan).is_some() {
            node = Expression::Lt(
                operand_type,
                Box::new(node),
                Box::new(parse_bit_or(tokens, d)),
            );
        } else if consume_token!(tokens, GtEq).is_some() {
            node = Expression::Ge(
                operand_type,
                Box::new(node),
                Box::new(parse_bit_or(tokens, d)),
            );
        } else if consume_token!(tokens, LtEq).is_some() {
            node = Expression::Le(
                operand_type,
                Box::new(node),
                Box::new(parse_bit_or(tokens, d)),
            );
        } else {
            unreachable!();
        }
    }
}

fn parse_equality(tokens: &mut Vec<Token>, d: &FuncSizeData) -> Expression {
    let mut node = parse_relational(tokens, d);

    loop {
        if tokens.len() >= 3
            && matches!(
                &tokens[tokens.len() - 3..],
                [
                    Token::DoubleEq | Token::ExclEq,
                    Token::Dot,
                    Token::IntKeyword | Token::FloatKeyword
                ]
            )
        {
            // e.g.) int.==
        } else {
            return node;
        }
        let operand_type = if consume_token!(tokens, IntKeyword).is_some() {
            OperandType::Int
        } else if consume_token!(tokens, FloatKeyword).is_some() {
            OperandType::Float
        } else {
            unreachable!()
        };
        expect_token!(tokens, Dot);
        if consume_token!(tokens, DoubleEq).is_some() {
            node = Expression::Eq(
                operand_type,
                Box::new(node),
                Box::new(parse_relational(tokens, d)),
            );
        } else if consume_token!(tokens, ExclEq).is_some() {
            node = Expression::Ne(
                operand_type,
                Box::new(node),
                Box::new(parse_relational(tokens, d)),
            );
        } else {
            unreachable!();
        }
    }
}

#[inline(always)]
fn parse_expr(tokens: &mut Vec<Token>, d: &FuncSizeData) -> Expression {
    parse_equality(tokens, d)
}

#[derive(Debug)]
pub enum Statement {
    While {
        cond: Box<Expression>,
        body: Box<Statement>,
    },
    If {
        cond: Box<Expression>,
        then_branch: Box<Statement>,
        unless_branch: Box<Statement>,
    },
    SetWithLiteralIndex {
        index: usize,
        val: Box<Expression>,
    },
    SetWithComputedIndex {
        index: Box<Expression>,
        val: Box<Expression>,
    },
    Block {
        stmts: Vec<Statement>,
    },
    PutByte {
        val: Box<Expression>,
    },
    Return {
        val: Box<Expression>,
    },
    Expr {
        expr: Box<Expression>,
    },
}

fn parse_stmt(tokens: &mut Vec<Token>, d: &FuncSizeData) -> Statement {
    if consume_token!(tokens, WhileKeyword).is_some() {
        let cond = parse_expr(tokens, d);
        expect_token!(tokens, Colon);
        let body = parse_stmt(tokens, d);
        Statement::While {
            cond: Box::new(cond),
            body: Box::new(body),
        }
    } else if consume_token!(tokens, SetKeyword).is_some() {
        if consume_token!(tokens, Dot).is_some() {
            let index = expect_int(tokens);
            assert!(index >= 0);
            let index: usize = index.try_into().unwrap();
            assert!(index < d.alloc_size);
            expect_token!(tokens, Equal);
            let val = parse_expr(tokens, d);
            expect_token!(tokens, Semicolon);
            Statement::SetWithLiteralIndex {
                index,
                val: Box::new(val),
            }
        } else if consume_token!(tokens, OpeningBracket).is_some() {
            let index = parse_expr(tokens, d);
            expect_token!(tokens, ClosingBracket);
            expect_token!(tokens, Equal);
            let val = parse_expr(tokens, d);
            expect_token!(tokens, Semicolon);
            Statement::SetWithComputedIndex {
                index: Box::new(index),
                val: Box::new(val),
            }
        } else if tokens.is_empty() {
            panic!("'.' or '[' is expected, but the end of the input is found");
        } else {
            panic!(
                "'.' or '[' is expected, but other token ({:?}) is found",
                tokens.pop().unwrap()
            );
        }
    } else if consume_token!(tokens, IfKeyword).is_some() {
        let cond = parse_expr(tokens, d);
        expect_token!(tokens, Colon);
        let then_branch = parse_stmt(tokens, d);
        expect_token!(tokens, ElseKeyword);
        let unless_branch = parse_stmt(tokens, d);
        Statement::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            unless_branch: Box::new(unless_branch),
        }
    } else if consume_token!(tokens, OpeningBrace).is_some() {
        let mut stmts = Vec::new();
        while consume_token!(tokens, ClosingBrace).is_none() {
            stmts.push(parse_stmt(tokens, d));
        }
        Statement::Block { stmts }
    } else if consume_token!(tokens, PutcKeyword).is_some() {
        let val = parse_expr(tokens, d);
        expect_token!(tokens, Semicolon);
        Statement::PutByte { val: Box::new(val) }
    // } else if consume_token!(tokens, PrintFloatKeyword).is_some() {
    //     let val = parse_expr(tokens, d);
    //     expect_token!(tokens, Semicolon);
    //     Statement::PrintFloat { val: Box::new(val) }
    // } else if consume_token!(tokens, PrintIntKeyword).is_some() {
    //     let val = parse_expr(tokens, d);
    //     expect_token!(tokens, Semicolon);
    //     Statement::PrintInt { val: Box::new(val) }
    } else if consume_token!(tokens, ReturnKeyword).is_some() {
        let val = parse_expr(tokens, d);
        expect_token!(tokens, Semicolon);
        Statement::Return { val: Box::new(val) }
    } else {
        let val = parse_expr(tokens, d);
        expect_token!(tokens, Semicolon);
        Statement::Expr {
            expr: Box::new(val),
        }
    }
}

#[derive(Debug)]
pub struct FunctionData {
    pub body: Statement,
    pub name: String,
    pub params_size: usize,
    pub alloc_size: usize,
}

struct FuncSizeData {
    params_size: usize,
    alloc_size: usize,
}

#[derive(Debug)]
pub struct Program {
    pub funcs: Vec<FunctionData>,
}

fn parse_func(tokens: &mut Vec<Token>) -> FunctionData {
    expect_token!(tokens, FuncKeyword);
    let funcname = expect_ident(tokens);
    expect_token!(tokens, OpeningBrace);
    expect_token!(tokens, ParamsKeyword);
    let params_size = expect_int(tokens);
    assert!(params_size >= 0);
    let params_size: usize = params_size.try_into().unwrap();
    expect_token!(tokens, Semicolon);
    expect_token!(tokens, AllocKeyword);
    let alloc_size = expect_int(tokens);
    assert!(alloc_size >= 0);
    let alloc_size: usize = alloc_size.try_into().unwrap();
    expect_token!(tokens, Semicolon);
    let d = FuncSizeData {
        params_size,
        alloc_size,
    };
    let body = {
        let mut stmts = Vec::new();
        while consume_token!(tokens, ClosingBrace).is_none() {
            stmts.push(parse_stmt(tokens, &d));
        }
        Statement::Block { stmts }
    };
    FunctionData {
        body,
        name: funcname,
        params_size,
        alloc_size,
    }
}

pub fn parse_program(mut tokens: Vec<Token>) -> Program {
    tokens.reverse();
    let tokens = &mut tokens;

    let mut funcs: Vec<FunctionData> = Vec::new();
    while !tokens.is_empty() {
        let parsed_func = parse_func(tokens);
        assert!(
            funcs.iter().all(|f| f.name != parsed_func.name),
            "The function {}() is defined twice or more",
            parsed_func.name
        );
        funcs.push(parsed_func);
    }
    assert!(
        funcs.iter().any(|f| f.name == "main"),
        "The function main() is missing"
    );
    Program { funcs }
}
