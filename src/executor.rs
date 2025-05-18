use crate::code_generator::{Codes, OpCode};
use core::num::Wrapping;

#[derive(Clone, Copy)]
pub union Value {
    __as_f64: f64,
    __as_i64: Wrapping<i64>,
}

impl Value {
    #[inline(always)]
    pub fn from_f64(v: f64) -> Self {
        Self { __as_f64: v }
    }
    #[inline(always)]
    pub fn from_i64(v: Wrapping<i64>) -> Self {
        Self { __as_i64: v }
    }
    #[inline(always)]
    pub fn as_f64(&self) -> f64 {
        unsafe { self.__as_f64 }
    }
    #[inline(always)]
    pub fn as_i64(&self) -> Wrapping<i64> {
        unsafe { self.__as_i64 }
    }
}

impl core::fmt::Debug for Value {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}({:?})", self.as_i64(), self.as_f64())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    R1 = 0,
    R2 = 1,
    R3 = 2,
    RAX = 3,
}

#[derive(Debug, Clone, Copy)]
struct Registers([Value; 4]);

impl Registers {
    #[inline(always)]
    pub fn new() -> Self {
        Self([Value::from_i64(Wrapping(0)); 4])
    }
}

impl core::ops::Index<Register> for Registers {
    type Output = Value;
    #[inline(always)]
    fn index<'a>(&'a self, r: Register) -> &'a Value {
        unsafe { &*self.0.as_ptr().add(r as usize) }
    }
}

impl core::ops::IndexMut<Register> for Registers {
    #[inline(always)]
    fn index_mut<'a>(&'a mut self, r: Register) -> &'a mut Value {
        unsafe { &mut *self.0.as_mut_ptr().add(r as usize) }
    }
}

macro_rules! bool_to_int {
    ($b:expr) => {
        if $b { 1 } else { 0 }
    };
}

pub fn execute(codes: Codes) {
    let mut r = Registers::new();
    let mut stack = Vec::with_capacity(4096);

    #[cfg(debug_assertions)]
    let mut stdout = std::io::stdout();
    #[cfg(not(debug_assertions))]
    let mut stdout = std::io::stdout().lock();

    #[cfg(debug_assertions)]
    let mut stdin = std::io::stdin();
    #[cfg(not(debug_assertions))]
    let mut stdin = std::io::stdin().lock();

    let mut rip = codes.entry_point;
    let mut rbp = 0;

    loop {
        match codes.opcodes[rip] {
            OpCode::Exit => break,
            OpCode::Alloc(size) => {
                stack.resize(stack.len() + size, Value::from_i64(Wrapping(0)));
            }
            OpCode::Free(size) => {
                #[cfg(debug_assertions)]
                {
                    debug_assert!(stack.len() >= rbp);
                    stack.truncate(stack.len() - size);
                }
                #[cfg(not(debug_assertions))]
                unsafe {
                    stack.set_len(stack.len() - size);
                }
            }
            OpCode::FunctionCall(faddr) => {
                stack.push(Value::from_i64(Wrapping(rip as i64) + Wrapping(1)));
                rip = faddr;
                continue;
            }
            OpCode::MovEbpToEspAndPopRbp => {
                #[cfg(debug_assertions)]
                {
                    debug_assert!(stack.len() >= rbp);
                    stack.truncate(rbp);
                    rbp = stack.pop().unwrap().as_i64().0 as usize;
                }
                #[cfg(not(debug_assertions))]
                unsafe {
                    stack.set_len(rbp);
                    rbp = stack.pop().unwrap_unchecked().as_i64().0 as usize;
                }
            }
            OpCode::PushRbpAndMovEspToEbp => {
                stack.push(Value::from_i64(Wrapping(rbp as i64)));
                rbp = stack.len();
            }
            OpCode::Ret => {
                #[cfg(debug_assertions)]
                {
                    rip = stack.pop().unwrap().as_i64().0 as usize;
                }
                #[cfg(not(debug_assertions))]
                {
                    rip = unsafe { stack.pop().unwrap_unchecked() }.as_i64().0 as usize;
                }
                continue;
            }
            OpCode::LoadInt(v, r1) => {
                r[r1] = Value::from_i64(Wrapping(v));
            }
            OpCode::LoadFloat(v, r1) => {
                r[r1] = Value::from_f64(v);
            }
            OpCode::Pop(r1) => {
                #[cfg(debug_assertions)]
                {
                    r[r1] = stack.pop().unwrap();
                }
                #[cfg(not(debug_assertions))]
                {
                    r[r1] = unsafe { stack.pop().unwrap_unchecked() };
                }
            }
            OpCode::Push(r1) => {
                stack.push(r[r1]);
            }
            // OpCode::PrintFloat(r1) => {
            //     use std::io::Write as _;
            //     write!(std::io::stdout(), "{:?}", r[r1].as_f64()).unwrap();
            // }
            // OpCode::PrintInt(r1) => {
            //     use std::io::Write as _;
            //     write!(std::io::stdout(), "{}", r[r1].as_i64().0).unwrap();
            // }
            OpCode::PutByte(r1) => {
                use std::io::Write as _;

                #[cfg(debug_assertions)]
                {
                    stdout
                        .write_all(&[r[r1].as_i64().0.rem_euclid(0xFF) as u8])
                        .unwrap();
                }
                #[cfg(not(debug_assertions))]
                {
                    let _ = stdout.write_all(&[r[r1].as_i64().0.rem_euclid(0xFF) as u8]);
                }
            }
            OpCode::ReadByteFromStdin(r1) => {
                use std::io::{Read as _, Write as _};

                #[cfg(debug_assertions)]
                {
                    stdout
                        .write_all(&[r[r1].as_i64().0.rem_euclid(0xFF) as u8])
                        .unwrap();
                    let mut buf: [u8; 1] = [0; 1];
                    stdout.flush().unwrap();
                    let read_len = stdin
                        .read(&mut buf)
                        .unwrap();
                    debug_assert!(read_len == 1, "failed to read from standard input");
                    r[r1] = Value::from_i64(Wrapping(buf[0] as i64));
                }
                #[cfg(not(debug_assertions))]
                {
                    use core::mem::MaybeUninit;

                    let _ = stdout.write_all(&[r[r1].as_i64().0.rem_euclid(0xFF) as u8]);
                    let mut buf: MaybeUninit<u8> = MaybeUninit::uninit();
                    let _ = stdout.flush();
                    let _ = stdin
                        .read(unsafe { core::slice::from_raw_parts_mut(buf.as_mut_ptr(), 1) });
                    r[r1] = Value::from_i64(Wrapping(unsafe { buf.assume_init() } as i64));
                }
            }
            OpCode::Mov(r1, r2) => {
                r[r2] = r[r1];
            }
            OpCode::LogiNot(r1, r2) => {
                r[r2] = Value::from_i64(Wrapping(bool_to_int!(r[r1].as_i64().0 == 0)));
            }
            OpCode::FloatToInt(r1, r2) => {
                r[r2] = Value::from_i64(Wrapping(r[r1].as_f64() as i64));
            }
            OpCode::IntToFloat(r1, r2) => {
                r[r2] = Value::from_f64(r[r1].as_i64().0 as f64);
            }
            OpCode::BitNot(r1, r2) => {
                r[r2] = Value::from_i64(!r[r1].as_i64());
            }
            OpCode::NegInt(r1, r2) => {
                r[r2] = Value::from_i64(-r[r1].as_i64());
            }
            OpCode::NegFloat(r1, r2) => {
                r[r2] = Value::from_f64(-r[r1].as_f64());
            }

            OpCode::Shl(r1, r2, r3) => {
                // TODO: `<<` の右辺どうする？
                r[r3] = Value::from_i64(r[r1].as_i64() << (r[r2].as_i64().0 as usize));
            }
            OpCode::Shr(r1, r2, r3) => {
                // TODO: `>>` の右辺どうする？
                r[r3] = Value::from_i64(r[r1].as_i64() >> (r[r2].as_i64().0 as usize));
            }
            OpCode::ShrUnsigned(r1, r2, r3) => {
                // TODO: `>>` の右辺どうする？
                r[r3] = Value::from_i64(Wrapping((Wrapping(r[r1].as_i64().0 as u64) >> (r[r2].as_i64().0 as usize)).0 as i64));
            }

            OpCode::LtInt(r1, r2, r3) => {
                r[r3] = Value::from_i64(Wrapping(bool_to_int!(r[r1].as_i64() < r[r2].as_i64())));
            }
            OpCode::LeInt(r1, r2, r3) => {
                r[r3] = Value::from_i64(Wrapping(bool_to_int!(r[r1].as_i64() <= r[r2].as_i64())));
            }
            OpCode::EqInt(r1, r2, r3) => {
                r[r3] = Value::from_i64(Wrapping(bool_to_int!(r[r1].as_i64() == r[r2].as_i64())));
            }
            OpCode::NeInt(r1, r2, r3) => {
                r[r3] = Value::from_i64(Wrapping(bool_to_int!(r[r1].as_i64() != r[r2].as_i64())));
            }
            OpCode::LtFloat(r1, r2, r3) => {
                r[r3] = Value::from_i64(Wrapping(bool_to_int!(r[r1].as_f64() < r[r2].as_f64())));
            }
            OpCode::LeFloat(r1, r2, r3) => {
                r[r3] = Value::from_i64(Wrapping(bool_to_int!(r[r1].as_f64() <= r[r2].as_f64())));
            }
            OpCode::EqFloat(r1, r2, r3) => {
                r[r3] = Value::from_i64(Wrapping(bool_to_int!(r[r1].as_f64() == r[r2].as_f64())));
            }
            OpCode::NeFloat(r1, r2, r3) => {
                r[r3] = Value::from_i64(Wrapping(bool_to_int!(r[r1].as_f64() != r[r2].as_f64())));
            }

            OpCode::AddFloat(r1, r2, r3) => {
                r[r3] = Value::from_f64(r[r1].as_f64() + r[r2].as_f64());
            }
            OpCode::AddInt(r1, r2, r3) => {
                r[r3] = Value::from_i64(r[r1].as_i64() + r[r2].as_i64());
            }
            OpCode::SubFloat(r1, r2, r3) => {
                r[r3] = Value::from_f64(r[r1].as_f64() - r[r2].as_f64());
            }
            OpCode::SubInt(r1, r2, r3) => {
                r[r3] = Value::from_i64(r[r1].as_i64() - r[r2].as_i64());
            }
            OpCode::MulFloat(r1, r2, r3) => {
                r[r3] = Value::from_f64(r[r1].as_f64() * r[r2].as_f64());
            }
            OpCode::MulInt(r1, r2, r3) => {
                r[r3] = Value::from_i64(r[r1].as_i64() * r[r2].as_i64());
            }
            OpCode::DivFloat(r1, r2, r3) => {
                r[r3] = Value::from_f64(r[r1].as_f64() / r[r2].as_f64());
            }
            OpCode::DivInt(r1, r2, r3) => {
                r[r3] = Value::from_i64(r[r1].as_i64() / r[r2].as_i64());
            }
            OpCode::RemFloat(r1, r2, r3) => {
                r[r3] = Value::from_f64(r[r1].as_f64() % r[r2].as_f64());
            }
            OpCode::RemInt(r1, r2, r3) => {
                r[r3] = Value::from_i64(r[r1].as_i64() % r[r2].as_i64());
            }
            OpCode::And(r1, r2, r3) => {
                r[r3] = Value::from_i64(r[r1].as_i64() & r[r2].as_i64());
            }
            OpCode::Or(r1, r2, r3) => {
                r[r3] = Value::from_i64(r[r1].as_i64() | r[r2].as_i64());
            }
            OpCode::Xor(r1, r2, r3) => {
                r[r3] = Value::from_i64(r[r1].as_i64() ^ r[r2].as_i64());
            }

            OpCode::GetParam(index, r1) => {
                #[cfg(debug_assertions)]
                {
                    r[r1] = stack[rbp - 2 - index];
                }
                #[cfg(not(debug_assertions))]
                unsafe {
                    r[r1] = stack.as_ptr().add(rbp - 2 - index).read();
                }
            }
            OpCode::GetVar(index, r1) => {
                #[cfg(debug_assertions)]
                {
                    r[r1] = stack[rbp + index];
                }
                #[cfg(not(debug_assertions))]
                unsafe {
                    r[r1] = stack.as_ptr().add(rbp + index).read();
                }
            }
            OpCode::GetVarComputed(index, r1) => {
                #[cfg(debug_assertions)]
                {
                    debug_assert!(r[index].as_i64().0 >= 0);
                    let variables = &mut stack[rbp..];
                    r[r1] = variables[r[index].as_i64().0 as usize];
                }
                #[cfg(not(debug_assertions))]
                unsafe {
                    r[r1] = stack.as_ptr().add(rbp + (r[index].as_i64().0 as usize)).read();
                }
            }
            OpCode::SetVar(index, r1) => {
                #[cfg(debug_assertions)]
                {
                    stack[rbp + index] = r[r1];
                }
                #[cfg(not(debug_assertions))]
                unsafe {
                    stack.as_mut_ptr().add(rbp + index).write(r[r1]);
                }
            }
            OpCode::SetVarComputed(index, r1) => {
                #[cfg(debug_assertions)]
                {
                    debug_assert!(r[index].as_i64().0 >= 0);
                    let variables = &mut stack[rbp..];
                    variables[r[index].as_i64().0 as usize] = r[r1];
                }
                #[cfg(not(debug_assertions))]
                unsafe {
                    stack.as_mut_ptr().add(rbp + (r[index].as_i64().0 as usize)).write(r[r1]);
                }
            }
            OpCode::JmpAddr(addr) => {
                rip = addr;
                continue;
            }
            OpCode::JmpAddrIfZero(c, addr) => {
                if r[c].as_i64().0 == 0 {
                    rip = addr;
                    continue;
                }
            }

            // Expected behavior: removed in code_generator::generate()
            OpCode::Nop
            | OpCode::__FNCALL_2255__(_)
            | OpCode::__JL_0515__(_)
            | OpCode::__JLIZ_2505__(_, _) => {
                #[cfg(debug_assertions)]
                unreachable!();
                #[cfg(not(debug_assertions))]
                unsafe {
                    core::hint::unreachable_unchecked()
                };
            }
        }
        rip += 1;
    }
}
