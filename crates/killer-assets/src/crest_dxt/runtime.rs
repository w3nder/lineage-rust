use super::lifted;

pub type R<T> = Result<T, Fault>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault(pub &'static str, pub u32);

#[derive(Clone, Copy)]
pub enum AluOp {
    Add,
    Sub,
    And,
    Or,
    Xor,
    Adc,
    Sbb,
}

#[derive(Clone, Copy)]
pub enum ShiftOp {
    Shl,
    Shr,
    Sar,
}

#[derive(Clone, Copy)]
pub enum FOp {
    Add,
    Sub,
    Subr,
    Mul,
    Div,
    Divr,
}

const STACK_BASE: u32 = 0x7000_0000;
const STACK_SIZE: usize = 1 << 20;
const HEAP_BASE: u32 = 0x3000_0000;
pub const CALLBACK: u32 = 0x2100_0000;
const FSW_C0: u16 = 0x0100;
const FSW_C2: u16 = 0x0400;
const FSW_C3: u16 = 0x4000;

pub struct Cpu {
    pub r: [u32; 8],
    pub zf: bool,
    pub sf: bool,
    pub cf: bool,
    pub of: bool,
    pub pf: bool,
    pub fs0: u32,
    pub fcw: u16,
    pub fsw: u16,
    stack: Vec<f64>,
    ram: Vec<u8>,
    heap: Vec<u8>,
    globals: Vec<u8>,
    globals_base: u32,
    rodata: Vec<(u32, Vec<u8>)>,
    pub output: Vec<Vec<u8>>,
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

impl Cpu {
    pub fn new() -> Cpu {
        let (low, high) = lifted::GLOBALS;
        let mut cpu = Cpu {
            r: [0; 8],
            zf: false,
            sf: false,
            cf: false,
            of: false,
            pf: false,
            fs0: 0,
            fcw: 0x027f,
            fsw: 0,
            stack: Vec::new(),
            ram: vec![0; STACK_SIZE],
            heap: Vec::new(),
            globals: vec![0; (high - low) as usize],
            globals_base: low,
            rodata: lifted::RODATA.iter().map(|(a, d)| (*a, d.to_vec())).collect(),
            output: Vec::new(),
        };
        cpu.r[4] = STACK_BASE + STACK_SIZE as u32 - 0x100;
        cpu.r[5] = cpu.r[4];
        cpu
    }

    pub fn alloc(&mut self, size: u32) -> u32 {
        let start = (self.heap.len() as u32 + 15) & !15;
        self.heap.resize(start as usize + size as usize + 16, 0);
        HEAP_BASE + start
    }

    pub fn alloc_bytes(&mut self, data: &[u8]) -> u32 {
        let pointer = self.alloc(data.len() as u32);
        let offset = (pointer - HEAP_BASE) as usize;
        self.heap[offset..offset + data.len()].copy_from_slice(data);
        pointer
    }

    pub fn read_bytes(&self, address: u32, length: usize) -> R<Vec<u8>> {
        Ok(self.slice(address, length)?.to_vec())
    }

    fn slice(&self, address: u32, length: usize) -> R<&[u8]> {
        let end = address as u64 + length as u64;
        if address >= STACK_BASE && end <= STACK_BASE as u64 + STACK_SIZE as u64 {
            let o = (address - STACK_BASE) as usize;
            return Ok(&self.ram[o..o + length]);
        }
        if address >= HEAP_BASE && end <= HEAP_BASE as u64 + self.heap.len() as u64 {
            let o = (address - HEAP_BASE) as usize;
            return Ok(&self.heap[o..o + length]);
        }
        if address >= self.globals_base && end <= self.globals_base as u64 + self.globals.len() as u64 {
            let o = (address - self.globals_base) as usize;
            return Ok(&self.globals[o..o + length]);
        }
        for (base, data) in &self.rodata {
            if address >= *base && end <= *base as u64 + data.len() as u64 {
                let o = (address - base) as usize;
                return Ok(&data[o..o + length]);
            }
        }
        Err(Fault("read", address))
    }

    fn slice_mut(&mut self, address: u32, length: usize) -> R<&mut [u8]> {
        let end = address as u64 + length as u64;
        if address >= STACK_BASE && end <= STACK_BASE as u64 + STACK_SIZE as u64 {
            let o = (address - STACK_BASE) as usize;
            return Ok(&mut self.ram[o..o + length]);
        }
        if address >= HEAP_BASE && end <= HEAP_BASE as u64 + self.heap.len() as u64 {
            let o = (address - HEAP_BASE) as usize;
            return Ok(&mut self.heap[o..o + length]);
        }
        if address >= self.globals_base && end <= self.globals_base as u64 + self.globals.len() as u64 {
            let o = (address - self.globals_base) as usize;
            return Ok(&mut self.globals[o..o + length]);
        }
        for (base, data) in &mut self.rodata {
            if address >= *base && end <= *base as u64 + data.len() as u64 {
                let o = (address - *base) as usize;
                return Ok(&mut data[o..o + length]);
            }
        }
        Err(Fault("write", address))
    }

    pub fn rd8(&self, a: u32) -> R<u8> {
        Ok(self.slice(a, 1)?[0])
    }

    pub fn rd16(&self, a: u32) -> R<u16> {
        Ok(u16::from_le_bytes(self.slice(a, 2)?.try_into().unwrap_or([0; 2])))
    }

    pub fn rd32(&self, a: u32) -> R<u32> {
        Ok(u32::from_le_bytes(self.slice(a, 4)?.try_into().unwrap_or([0; 4])))
    }

    pub fn rd64(&self, a: u32) -> R<u64> {
        Ok(u64::from_le_bytes(self.slice(a, 8)?.try_into().unwrap_or([0; 8])))
    }

    pub fn rdf32(&self, a: u32) -> R<f32> {
        Ok(f32::from_bits(self.rd32(a)?))
    }

    pub fn rdf64(&self, a: u32) -> R<f64> {
        Ok(f64::from_bits(self.rd64(a)?))
    }

    pub fn wr8(&mut self, a: u32, v: u8) -> R<()> {
        self.slice_mut(a, 1)?[0] = v;
        Ok(())
    }

    pub fn wr16(&mut self, a: u32, v: u16) -> R<()> {
        self.slice_mut(a, 2)?.copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    pub fn wr32(&mut self, a: u32, v: u32) -> R<()> {
        self.slice_mut(a, 4)?.copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    pub fn wr64(&mut self, a: u32, v: u64) -> R<()> {
        self.slice_mut(a, 8)?.copy_from_slice(&v.to_le_bytes());
        Ok(())
    }

    pub fn wrf32(&mut self, a: u32, v: f32) -> R<()> {
        self.wr32(a, v.to_bits())
    }

    pub fn wrf64(&mut self, a: u32, v: f64) -> R<()> {
        self.wr64(a, v.to_bits())
    }

    pub fn push(&mut self, v: u32) -> R<()> {
        self.r[4] = self.r[4].wrapping_sub(4);
        self.wr32(self.r[4], v)
    }

    pub fn pop(&mut self) -> R<u32> {
        let v = self.rd32(self.r[4])?;
        self.r[4] = self.r[4].wrapping_add(4);
        Ok(v)
    }

    pub fn arg(&self, index: u32) -> R<u32> {
        self.rd32(self.r[4].wrapping_add(4 + 4 * index))
    }

    pub fn set16(&mut self, i: usize, v: u32) {
        self.r[i] = (self.r[i] & 0xffff_0000) | (v & 0xffff);
    }

    pub fn set8(&mut self, i: usize, shift: u32, v: u32) {
        self.r[i] = (self.r[i] & !(0xff << shift)) | ((v & 0xff) << shift);
    }

    pub fn sext(&self, v: u32, bits: u32) -> u32 {
        match bits {
            8 => v as u8 as i8 as i32 as u32,
            16 => v as u16 as i16 as i32 as u32,
            _ => v,
        }
    }

    fn set_szp(&mut self, res: u32, w: u32) {
        let mask = if w == 32 { u32::MAX } else { (1u32 << w) - 1 };
        let res = res & mask;
        self.zf = res == 0;
        self.sf = (res >> (w - 1)) & 1 == 1;
        self.pf = (res as u8).count_ones().is_multiple_of(2);
    }

    pub fn alu(&mut self, op: AluOp, a: u32, b: u32, w: u32) -> u32 {
        let mask: u64 = if w == 32 { 0xffff_ffff } else { (1u64 << w) - 1 };
        let (a, b) = (a as u64 & mask, b as u64 & mask);
        let sign = 1u64 << (w - 1);
        let carry_in = u64::from(self.cf);
        let res = match op {
            AluOp::Add => {
                let r = a + b;
                self.cf = r > mask;
                self.of = (!(a ^ b) & (a ^ r) & sign) != 0;
                r
            }
            AluOp::Adc => {
                let r = a + b + carry_in;
                self.cf = r > mask;
                self.of = (!(a ^ b) & (a ^ r) & sign) != 0;
                r
            }
            AluOp::Sub => {
                let r = a.wrapping_sub(b) & mask;
                self.cf = a < b;
                self.of = ((a ^ b) & (a ^ r) & sign) != 0;
                r
            }
            AluOp::Sbb => {
                let r = a.wrapping_sub(b).wrapping_sub(carry_in) & mask;
                self.cf = a < b + carry_in;
                self.of = ((a ^ b) & (a ^ r) & sign) != 0;
                r
            }
            AluOp::And | AluOp::Or | AluOp::Xor => {
                self.cf = false;
                self.of = false;
                match op {
                    AluOp::And => a & b,
                    AluOp::Or => a | b,
                    _ => a ^ b,
                }
            }
        };
        self.set_szp(res as u32, w);
        (res & mask) as u32
    }

    pub fn incdec(&mut self, a: u32, w: u32, inc: bool) -> u32 {
        let cf = self.cf;
        let res = if inc { self.alu(AluOp::Add, a, 1, w) } else { self.alu(AluOp::Sub, a, 1, w) };
        self.cf = cf;
        res
    }

    pub fn neg(&mut self, a: u32, w: u32) -> u32 {
        let res = self.alu(AluOp::Sub, 0, a, w);
        self.cf = (a & if w == 32 { u32::MAX } else { (1 << w) - 1 }) != 0;
        res
    }

    pub fn shift(&mut self, op: ShiftOp, a: u32, count: u32, w: u32) -> u32 {
        let count = count & 31;
        if count == 0 {
            return a;
        }
        let mask: u64 = if w == 32 { 0xffff_ffff } else { (1u64 << w) - 1 };
        let a = a as u64 & mask;
        let res = match op {
            ShiftOp::Shl => {
                let r = (a << count) & mask;
                self.cf = if count <= w { (a >> (w - count)) & 1 == 1 } else { false };
                self.of = ((r >> (w - 1)) & 1 == 1) != self.cf;
                r
            }
            ShiftOp::Shr => {
                let r = a >> count;
                self.cf = (a >> (count - 1)) & 1 == 1;
                self.of = (a >> (w - 1)) & 1 == 1;
                r
            }
            ShiftOp::Sar => {
                let signed = if w == 32 { a as u32 as i32 as i64 } else { ((a << (64 - w)) as i64) >> (64 - w) };
                let r = ((signed >> count.min(63)) as u64) & mask;
                self.cf = (signed >> (count - 1).min(63)) & 1 == 1;
                self.of = false;
                r
            }
        };
        self.set_szp(res as u32, w);
        res as u32
    }

    pub fn imul(&mut self, a: u32, b: u32) -> u32 {
        let full = i64::from(a as i32) * i64::from(b as i32);
        let res = full as i32;
        self.cf = i64::from(res) != full;
        self.of = self.cf;
        res as u32
    }

    pub fn imul_wide(&mut self, a: u32, b: u32) -> (u32, u32) {
        let full = i64::from(a as i32) * i64::from(b as i32);
        self.cf = i64::from(full as i32) != full;
        self.of = self.cf;
        (full as u32, (full >> 32) as u32)
    }

    pub fn mul_wide(&mut self, a: u32, b: u32) -> (u32, u32) {
        let full = u64::from(a) * u64::from(b);
        self.cf = (full >> 32) != 0;
        self.of = self.cf;
        (full as u32, (full >> 32) as u32)
    }

    pub fn idiv(&mut self, divisor: u32) -> R<(u32, u32)> {
        let d = i64::from(divisor as i32);
        if d == 0 {
            return Err(Fault("idiv", 0));
        }
        let n = (i64::from(self.r[2] as i32) << 32) | i64::from(self.r[0]);
        Ok(((n / d) as u32, (n % d) as u32))
    }

    pub fn rep_stos(&mut self, size: u32) -> R<()> {
        let count = self.r[1];
        let (mut dest, value) = (self.r[7], self.r[0]);
        for _ in 0..count {
            if size == 4 {
                self.wr32(dest, value)?;
            } else {
                self.wr8(dest, value as u8)?;
            }
            dest = dest.wrapping_add(size);
        }
        self.r[7] = dest;
        self.r[1] = 0;
        Ok(())
    }

    pub fn rep_movs(&mut self, size: u32) -> R<()> {
        let count = self.r[1];
        let (mut src, mut dest) = (self.r[6], self.r[7]);
        for _ in 0..count {
            if size == 4 {
                let v = self.rd32(src)?;
                self.wr32(dest, v)?;
            } else {
                let v = self.rd8(src)?;
                self.wr8(dest, v)?;
            }
            src = src.wrapping_add(size);
            dest = dest.wrapping_add(size);
        }
        self.r[6] = src;
        self.r[7] = dest;
        self.r[1] = 0;
        Ok(())
    }

    pub fn fpush(&mut self, v: f64) {
        self.stack.push(v);
    }

    pub fn fpop(&mut self) {
        self.stack.pop();
    }

    pub fn st0(&self) -> f64 {
        self.stack.last().copied().unwrap_or(f64::NAN)
    }

    fn sti(&self, i: usize) -> f64 {
        let len = self.stack.len();
        if i < len { self.stack[len - 1 - i] } else { f64::NAN }
    }

    fn set_sti(&mut self, i: usize, v: f64) {
        let len = self.stack.len();
        if i < len {
            self.stack[len - 1 - i] = v;
        }
    }

    pub fn fld_st(&mut self, i: usize) {
        let v = self.sti(i);
        self.fpush(v);
    }

    pub fn fst_st(&mut self, i: usize, pop: bool) {
        let v = self.st0();
        self.set_sti(i, v);
        if pop {
            self.fpop();
        }
    }

    pub fn fxch(&mut self, i: usize) {
        let (a, b) = (self.st0(), self.sti(i));
        self.set_sti(0, b);
        self.set_sti(i, a);
    }

    pub fn fabs(&mut self) {
        let v = self.st0().abs();
        self.set_sti(0, v);
    }

    pub fn fchs(&mut self) {
        let v = -self.st0();
        self.set_sti(0, v);
    }

    pub fn fsqrt(&mut self) {
        let v = self.st0().sqrt();
        self.set_sti(0, v);
    }

    pub fn fround(&self, v: f64) -> f64 {
        match (self.fcw >> 10) & 3 {
            0 => v.round_ties_even(),
            1 => v.floor(),
            2 => v.ceil(),
            _ => v.trunc(),
        }
    }

    fn apply(op: FOp, dest: f64, src: f64) -> f64 {
        match op {
            FOp::Add => dest + src,
            FOp::Sub => dest - src,
            FOp::Subr => src - dest,
            FOp::Mul => dest * src,
            FOp::Div => dest / src,
            FOp::Divr => src / dest,
        }
    }

    pub fn farith(&mut self, op: FOp, dest: usize, src: usize, pop: bool) {
        let v = Self::apply(op, self.sti(dest), self.sti(src));
        self.set_sti(dest, v);
        if pop {
            self.fpop();
        }
    }

    pub fn farith_value(&mut self, op: FOp, value: f64) {
        let v = Self::apply(op, self.st0(), value);
        self.set_sti(0, v);
    }

    fn compare(&mut self, a: f64, b: f64) {
        self.fsw &= !(FSW_C0 | FSW_C2 | FSW_C3);
        if a.is_nan() || b.is_nan() {
            self.fsw |= FSW_C0 | FSW_C2 | FSW_C3;
        } else if a < b {
            self.fsw |= FSW_C0;
        } else if a == b {
            self.fsw |= FSW_C3;
        }
    }

    pub fn fcom(&mut self, i: usize, pop: bool) {
        let (a, b) = (self.st0(), self.sti(i));
        self.compare(a, b);
        if pop {
            self.fpop();
        }
    }

    pub fn fcom_value(&mut self, value: f64, pop: bool) {
        let a = self.st0();
        self.compare(a, value);
        if pop {
            self.fpop();
        }
    }

    pub fn fcompp(&mut self) {
        let (a, b) = (self.st0(), self.sti(1));
        self.compare(a, b);
        self.fpop();
        self.fpop();
    }

    pub fn call_indirect(&mut self, target: u32) -> R<()> {
        if target != CALLBACK {
            return Err(Fault("indirect", target));
        }
        let pointer = self.arg(0)?;
        let length = self.arg(2)? as usize;
        let data = self.read_bytes(pointer, length)?;
        self.output.push(data);
        self.r[4] = self.r[4].wrapping_add(4);
        Ok(())
    }
}

pub fn host_alloc(c: &mut Cpu, _target: u32) -> R<()> {
    let size = c.arg(0)?;
    c.r[0] = c.alloc(size);
    c.r[4] = c.r[4].wrapping_add(4);
    Ok(())
}

pub fn host_malloc(c: &mut Cpu, target: u32) -> R<()> {
    host_alloc(c, target)
}

pub fn host_calloc(c: &mut Cpu, _target: u32) -> R<()> {
    let size = c.arg(0)?.wrapping_mul(c.arg(1)?);
    c.r[0] = c.alloc(size);
    c.r[4] = c.r[4].wrapping_add(4);
    Ok(())
}

pub fn host_free(c: &mut Cpu, _target: u32) -> R<()> {
    c.r[4] = c.r[4].wrapping_add(4);
    Ok(())
}

pub fn host_floor(c: &mut Cpu, _target: u32) -> R<()> {
    let v = c.rdf64(c.r[4].wrapping_add(4))?;
    c.fpush(v.floor());
    c.r[4] = c.r[4].wrapping_add(4);
    Ok(())
}

pub fn host_fault(_c: &mut Cpu, target: u32) -> R<()> {
    Err(Fault("host", target))
}

