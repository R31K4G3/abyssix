#![allow(dead_code)]

use std::collections::HashMap;

use crate::executor::{Register, Register::*};
use crate::parser::{Expression, FunctionData, OperandType, Program, Statement};

#[derive(Debug, Clone, Copy)]
pub struct Label(usize);

#[derive(Debug, Clone, Copy)]
pub struct FuncLabel(usize);

#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
#[rustfmt::skip]
pub enum OpCode {
    Push             (/** [IN] value pushed */      Register),
    LoadInt          (/** [CONST] value pushed */   i64,       /** [OUT] to */       Register),
    LoadFloat        (/** [CONST] value pushed */   f64,       /** [OUT] to */       Register),
    Pop              (/** [OUT] value poped */      Register),
    Mov              (/** [IN] from */              Register,  /** [OUT] to */       Register),
    LogiNot          (/** [IN] original value */    Register,  /** [OUT] result */   Register),
    BitNot           (/** [IN] original value */    Register,  /** [OUT] result */   Register),
    NegInt           (/** [IN] original value */    Register,  /** [OUT] result */   Register),
    NegFloat         (/** [IN] original value */    Register,  /** [OUT] result */   Register),
    AddFloat         (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    AddInt           (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    SubFloat         (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    SubInt           (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    MulFloat         (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    MulInt           (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    DivFloat         (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    DivInt           (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    RemFloat         (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    RemInt           (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    NeInt            (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    EqInt            (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    NeFloat          (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    EqFloat          (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    And              (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    Or               (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    Xor              (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    Shr              (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    ShrUnsigned      (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    Shl              (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    LtInt            (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    LtFloat          (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    LeInt            (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    LeFloat          (/** [IN] from0 */             Register,  /** [IN] from1 */     Register,      /** [OUT] to */ Register),
    SetVar           (/** [CONST] variable index */ usize,     /** [IN] value */     Register),
    SetVarComputed   (/** [IN] variable index */    Register,  /** [IN] value */     Register),
    GetVar           (/** [CONST] variable index */ usize,     /** [OUT] result */   Register),
    GetVarComputed   (/** [IN] variable index */    Register,  /** [OUT] result */   Register),
    GetParam         (/** [CONST] param index */    usize,     /** [OUT] result */   Register),
    FloatToInt       (/** [IN] original float */    Register,  /** [OUT] result */   Register),
    IntToFloat       (/** [IN] original int */      Register,  /** [OUT] result */   Register),
    __JL_0515__      (/** [LABEL] target */         Label),
    __JLIZ_2505__    (/** [IN] value compared */    Register,  /** [LABEL] target */ Label),
    __FNCALL_2255__  (/** [FUNC_LABEL] target */    FuncLabel),
    JmpAddr          (/** [CONST] target */         usize),
    JmpAddrIfZero    (/** [IN] value compared */    Register,  /** [CONST] target */ usize),
    FunctionCall     (/** [CONST] target */         usize),
    PutByte          (/** [IN] value for print */   Register),
    ReadByteFromStdin(/** [OUT] value from stdin */ Register),
    Alloc            (/** [CONST] alloc size */     usize),
    Free             (/** [CONST] alloc size */     usize),
    Ret,
    Nop,
    PushRbpAndMovEspToEbp,
    MovEbpToEspAndPopRbp,
    Exit,
}

fn generate_expression_code(
    expr: &Expression,
    state: &mut State,
    name_table: &HashMap<String, FuncMeta>,
) {
    match expr {
        Expression::Call(funcname, args) => {
            let Some(fmeta) = name_table.get(funcname) else {
                panic!("The function {}() is not defined", funcname);
            };
            let fmeta = *fmeta;
            assert!(
                fmeta.params_size == args.len(),
                "{}() requires {} argument(s), but {} argument(s) passed",
                funcname,
                fmeta.params_size,
                args.len()
            );
            for arg in args {
                generate_expression_code(arg, state, name_table);
            }
            state.push(LabeledOpCode::without_label(OpCode::__FNCALL_2255__(
                fmeta.index,
            )));
            state.push(LabeledOpCode::without_label(OpCode::Free(args.len())));
            state.push(LabeledOpCode::without_label(OpCode::Push(RAX)));
            state.push(LabeledOpCode::without_label(OpCode::LoadInt(0, RAX)));
        }
        Expression::ReadInputByte => {
            state.push(LabeledOpCode::without_label(OpCode::ReadByteFromStdin(R1)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R1)));
        }
        Expression::Neg(operand_type, operand) => {
            generate_expression_code(operand, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(match operand_type {
                OperandType::Float => OpCode::NegFloat(R1, R2),
                OperandType::Int => OpCode::NegInt(R1, R2),
            }));
            state.push(LabeledOpCode::without_label(OpCode::Push(R2)));
        }
        Expression::BitNot(operand) => {
            generate_expression_code(operand, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::BitNot(R1, R2)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R2)));
        }
        Expression::LogiNot(operand) => {
            generate_expression_code(operand, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::LogiNot(R1, R2)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R2)));
        }
        Expression::Itof(operand) => {
            generate_expression_code(operand, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::IntToFloat(R1, R2)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R2)));
        }
        Expression::Ftoi(operand) => {
            generate_expression_code(operand, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::FloatToInt(R1, R2)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R2)));
        }
        Expression::Shl(left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::Shl(R1, R2, R3)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Shr(left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::Shr(R1, R2, R3)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::ShrUnsigned(left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::ShrUnsigned(R1, R2, R3)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::And(left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::And(R1, R2, R3)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Or(left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::Or(R1, R2, R3)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Xor(left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::Xor(R1, R2, R3)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Lt(operand_type, left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(match operand_type {
                OperandType::Float => OpCode::LtFloat(R1, R2, R3),
                OperandType::Int => OpCode::LtInt(R1, R2, R3),
            }));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Le(operand_type, left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(match operand_type {
                OperandType::Float => OpCode::LeFloat(R1, R2, R3),
                OperandType::Int => OpCode::LeInt(R1, R2, R3),
            }));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Gt(operand_type, left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(match operand_type {
                OperandType::Float => OpCode::LtFloat(R2, R1, R3),
                OperandType::Int => OpCode::LtInt(R2, R1, R3),
            }));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Ge(operand_type, left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(match operand_type {
                OperandType::Float => OpCode::LeFloat(R2, R1, R3),
                OperandType::Int => OpCode::LeInt(R2, R1, R3),
            }));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Int(val) => {
            state.push(LabeledOpCode::without_label(OpCode::LoadInt(*val, R1)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R1)));
        }
        Expression::Float(val) => {
            state.push(LabeledOpCode::without_label(OpCode::LoadFloat(*val, R1)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R1)));
        }
        Expression::GetWithComputedIndex(index) => {
            generate_expression_code(index, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::GetVarComputed(R1, R2)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R2)));
        }
        Expression::GetParam(index) => {
            state.push(LabeledOpCode::without_label(OpCode::GetParam(*index, R1)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R1)));
        }
        Expression::GetWithLiteralIndex(index) => {
            state.push(LabeledOpCode::without_label(OpCode::GetVar(*index, R1)));
            state.push(LabeledOpCode::without_label(OpCode::Push(R1)));
        }
        Expression::Eq(operand_type, left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(match operand_type {
                OperandType::Float => OpCode::EqFloat(R1, R2, R3),
                OperandType::Int => OpCode::EqInt(R1, R2, R3),
            }));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Ne(operand_type, left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(match operand_type {
                OperandType::Float => OpCode::NeFloat(R1, R2, R3),
                OperandType::Int => OpCode::NeInt(R1, R2, R3),
            }));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Add(operand_type, left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(match operand_type {
                OperandType::Float => OpCode::AddFloat(R1, R2, R3),
                OperandType::Int => OpCode::AddInt(R1, R2, R3),
            }));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Sub(operand_type, left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(match operand_type {
                OperandType::Float => OpCode::SubFloat(R1, R2, R3),
                OperandType::Int => OpCode::SubInt(R1, R2, R3),
            }));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Mul(operand_type, left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(match operand_type {
                OperandType::Float => OpCode::MulFloat(R1, R2, R3),
                OperandType::Int => OpCode::MulInt(R1, R2, R3),
            }));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Div(operand_type, left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(match operand_type {
                OperandType::Float => OpCode::DivFloat(R1, R2, R3),
                OperandType::Int => OpCode::DivInt(R1, R2, R3),
            }));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
        Expression::Rem(operand_type, left, right) => {
            generate_expression_code(left, state, name_table);
            generate_expression_code(right, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(match operand_type {
                OperandType::Float => OpCode::RemFloat(R1, R2, R3),
                OperandType::Int => OpCode::RemInt(R1, R2, R3),
            }));
            state.push(LabeledOpCode::without_label(OpCode::Push(R3)));
        }
    }
}

fn generate_statement_code(
    stmt: &Statement,
    state: &mut State,
    name_table: &HashMap<String, FuncMeta>,
) {
    match stmt {
        Statement::If {
            cond,
            then_branch,
            unless_branch,
        } => {
            let else_label = state.new_label();
            let end_label = state.new_label();
            generate_expression_code(cond, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::__JLIZ_2505__(
                R1, else_label,
            )));
            generate_statement_code(then_branch, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::__JL_0515__(end_label)));
            state.push(LabeledOpCode::with_one_label(else_label, OpCode::Nop));
            generate_statement_code(unless_branch, state, name_table);
            state.push(LabeledOpCode::with_one_label(end_label, OpCode::Nop));
        }
        Statement::While { cond, body } => {
            let begin_label = state.new_label();
            let end_label = state.new_label();
            state.push(LabeledOpCode::with_one_label(begin_label, OpCode::Nop));
            generate_expression_code(cond, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::__JLIZ_2505__(
                R1, end_label,
            )));
            generate_statement_code(body, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::__JL_0515__(
                begin_label,
            )));
            state.push(LabeledOpCode::with_one_label(end_label, OpCode::Nop));
        }
        Statement::SetWithComputedIndex { index, val } => {
            generate_expression_code(index, state, name_table);
            generate_expression_code(val, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R2)));
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::SetVarComputed(R1, R2)));
        }
        Statement::SetWithLiteralIndex { index, val } => {
            generate_expression_code(val, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::SetVar(*index, R1)));
        }
        Statement::Block { stmts } => {
            for stmt in stmts {
                generate_statement_code(stmt, state, name_table);
            }
        }
        // Statement::PrintFloat { val } => {
        //     generate_expression_code(val, state, name_table);
        //     state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
        //     state.push(LabeledOpCode::without_label(OpCode::PrintFloat(R1)));
        // }
        // Statement::PrintInt { val } => {
        //     generate_expression_code(val, state, name_table);
        //     state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
        //     state.push(LabeledOpCode::without_label(OpCode::PrintInt(R1)));
        // }
        Statement::PutByte { val } => {
            generate_expression_code(val, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(R1)));
            state.push(LabeledOpCode::without_label(OpCode::PutByte(R1)));
        }
        Statement::Return { val } => {
            generate_expression_code(val, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Pop(RAX)));
            state.push(LabeledOpCode::without_label(OpCode::MovEbpToEspAndPopRbp));
            state.push(LabeledOpCode::without_label(OpCode::Ret));
        }
        Statement::Expr { expr } => {
            generate_expression_code(expr, state, name_table);
            state.push(LabeledOpCode::without_label(OpCode::Free(1)));
        }
    }
}

struct LabeledOpCode {
    labels: Vec<Label>,
    code: OpCode,
}

impl LabeledOpCode {
    fn without_label(code: OpCode) -> Self {
        Self {
            labels: Vec::new(),
            code,
        }
    }
    fn with_one_label(label: Label, code: OpCode) -> Self {
        Self {
            labels: vec![label],
            code,
        }
    }
    fn with_labels(labels: Vec<Label>, code: OpCode) -> Self {
        Self { labels, code }
    }
}

#[derive(Clone, Copy)]
struct FuncMeta {
    params_size: usize,
    index: FuncLabel,
}

struct State<'a> {
    codes: Vec<LabeledOpCode>,
    next_label_val: &'a mut usize,
}

impl<'a> State<'a> {
    fn new(next_label_val: &'a mut usize) -> Self {
        Self {
            codes: Vec::new(),
            next_label_val,
        }
    }
    fn push(&mut self, code: LabeledOpCode) {
        self.codes.push(code);
    }
    fn new_label(&mut self) -> Label {
        let l = Label(*self.next_label_val);
        *self.next_label_val += 1;
        l
    }
    fn into_inner(self) -> Vec<LabeledOpCode> {
        self.codes
    }
}

#[derive(Debug)]
pub struct Codes {
    pub entry_point: usize,
    pub opcodes: Vec<OpCode>,
}

fn generate_function_code(
    f: &FunctionData,
    next_label_val: &mut usize,
    name_table: &HashMap<String, FuncMeta>,
) -> Vec<LabeledOpCode> {
    let mut state = State::new(next_label_val);

    state.push(LabeledOpCode::without_label(OpCode::PushRbpAndMovEspToEbp));
    state.push(LabeledOpCode::without_label(OpCode::Alloc(f.alloc_size)));
    generate_statement_code(&f.body, &mut state, &name_table);

    if f.name == "main" {
        state.push(LabeledOpCode::without_label(OpCode::Exit));
    } else {
        state.push(LabeledOpCode::without_label(OpCode::MovEbpToEspAndPopRbp));
        state.push(LabeledOpCode::without_label(OpCode::Ret));
    }

    let mut opcodes = state.into_inner();

    // Replace: `Push(R1); Push(R2)` to `Mov(R1, R2); Nop`,  `Push(R1); Push(R1)` to `Nop; Nop`
    for i in 1..opcodes.len() {
        let split = opcodes.split_at_mut(i);
        let (first, second) = (split.0.last_mut().unwrap(), split.1.first_mut().unwrap());
        if !second.labels.is_empty() {
            continue;
        }
        if let (OpCode::Push(r1), OpCode::Pop(r2)) = (first.code, second.code) {
            if r1 == r2 {
                first.code = OpCode::Nop;
                second.code = OpCode::Nop;
            } else {
                first.code = OpCode::Mov(r1, r2);
                second.code = OpCode::Nop;
            }
        }
    }

    // Move Nop's labels to following code
    let mut label_tmp = Vec::new();
    for op in &mut opcodes {
        if matches!(op.code, OpCode::Nop) {
            label_tmp.append(&mut op.labels);
        } else if !label_tmp.is_empty() {
            op.labels.append(&mut label_tmp);
        }
    }
    opcodes.push(LabeledOpCode::with_labels(label_tmp, OpCode::Nop));

    // Remove unlabeled Nop
    let opcodes: Vec<LabeledOpCode> = opcodes
        .into_iter()
        .filter(|op| {
            !(op.labels.is_empty()
                && match op.code {
                    OpCode::Nop => true,
                    OpCode::Alloc(0) => true,
                    OpCode::Free(0) => true,
                    _ => false,
                })
        })
        .collect();

    opcodes
}

pub fn generate(prog: Program) -> Codes {
    let name_table: HashMap<String, FuncMeta> = prog
        .funcs
        .iter()
        .enumerate()
        .map(|(index, f)| {
            (
                f.name.clone(),
                FuncMeta {
                    index: FuncLabel(index),
                    params_size: f.params_size,
                },
            )
        })
        .collect();

    let mut next_label_val = 0;
    let func_codes: Vec<Vec<LabeledOpCode>> = prog
        .funcs
        .iter()
        .map(|f| generate_function_code(f, &mut next_label_val, &name_table))
        .collect();

    let func_addrs = func_codes
        .iter()
        .fold((Vec::<usize>::new(), 0usize), |(mut m, mut acc), codes| {
            m.push(acc);
            acc += codes.len();
            (m, acc)
        })
        .0;

    // /* for debug */ {
    //     use std::io::Write as _;
    //     let mut ffffffff = std::fs::File::create("codes.txt").unwrap();
    //     for (i, fcodes) in func_codes.iter().enumerate() {
    //         writeln!(ffffffff, "{:?}:", FuncLabel(i)).unwrap();
    //         for c in fcodes {
    //             for l in &c.labels {
    //                 writeln!(ffffffff, "{:?}:", l).unwrap();
    //             }
    //             writeln!(ffffffff, "  {:?}", c.code).unwrap();
    //         }
    //     }
    // }

    let opcodes: Vec<LabeledOpCode> = func_codes.into_iter().flatten().collect();

    // Resolve labels as indices
    let mut label_indices =
        vec![usize::MAX /* MAX for marker that means it's uninitialized */; next_label_val];
    for (index, op) in opcodes.iter().enumerate() {
        for label in &op.labels {
            assert!(label_indices[label.0] == usize::MAX);
            label_indices[label.0] = index;
        }
    }
    let mut opcodes: Vec<OpCode> = opcodes
        .into_iter()
        .map(|op| match op.code {
            OpCode::__JL_0515__(label) => OpCode::JmpAddr(label_indices[label.0]),
            OpCode::__JLIZ_2505__(r1, label) => OpCode::JmpAddrIfZero(r1, label_indices[label.0]),
            OpCode::__FNCALL_2255__(index) => OpCode::FunctionCall(func_addrs[index.0]),
            _ => op.code,
        })
        .collect();

    // Remove last Opcode if it's Nop
    if matches!(opcodes.last(), Some(OpCode::Nop)) {
        opcodes.pop();
    }

    let entry_point = func_addrs[name_table["main"].index.0];

    Codes {
        entry_point,
        opcodes,
    }
}
