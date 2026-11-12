use super::runtime::*;

pub fn f_1072c270(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x1072c270;
    loop {
        match pc {
            0x1072c270 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(4u32))?;
                c.alu(AluOp::Sub, c.r[0], 1u32, 32);
                if !c.cf { pc = 0x1072c27c; continue; }
            pc = 0x1072c279; continue;
            }
            0x1072c279 => {
                { let v = c.alu(AluOp::Xor, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8); c.set8(0, 0, v); }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            0x1072c27c => {
                if !c.zf { pc = 0x1072c281; continue; }
            pc = 0x1072c27e; continue;
            }
            0x1072c27e => {
                { let v = 1u32; c.set8(0, 0, v); }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            0x1072c281 => {
                c.r[1] = c.r[0].wrapping_add(4294967295u32);
                c.r[1] = c.alu(AluOp::And, c.r[1], c.r[0], 32);
                c.r[1] = c.neg(c.r[1], 32);
                { let v = c.alu(AluOp::Sbb, ((c.r[1] >> 0) & 0xff), ((c.r[1] >> 0) & 0xff), 8); c.set8(1, 0, v); }
                { let v = c.incdec(((c.r[1] >> 0) & 0xff), 8, true); c.set8(1, 0, v); }
                { let v = ((c.r[1] >> 0) & 0xff); c.set8(0, 0, v); }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_1072c4f0(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x1072c4f0;
    loop {
        match pc {
            0x1072c4f0 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(4u32))?;
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 8u32, 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if !c.zf { pc = 0x1072c502; continue; }
            pc = 0x1072c4fb; continue;
            }
            0x1072c4fb => {
                c.r[0] = c.alu(AluOp::Or, c.r[0], 4294967295u32, 32);
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            0x1072c502 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.alu(AluOp::Sub, c.r[0], 3u32, 32);
                if c.zf { pc = 0x1072c519; continue; }
            pc = 0x1072c50b; continue;
            }
            0x1072c50b => {
                c.alu(AluOp::Sub, c.r[0], 4u32, 32);
                if c.zf { pc = 0x1072c519; continue; }
            pc = 0x1072c510; continue;
            }
            0x1072c510 => {
                c.r[0] = 4294967294u32;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            0x1072c519 => {
                c.push(c.r[3])?;
                c.r[3] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.push(c.r[5])?;
                c.r[5] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.push(c.r[6])?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[0] = c.sext(c.rd16(c.r[6].wrapping_add(50u32))? as u32, 16);
                c.alu(AluOp::Sub, c.r[0], 10u32, 32);
                if c.sf != c.of { pc = 0x1072c579; continue; }
            pc = 0x1072c531; continue;
            }
            0x1072c531 => {
                c.alu(AluOp::Sub, c.r[0], 13u32, 32);
                if !c.zf && c.sf == c.of { pc = 0x1072c579; continue; }
            pc = 0x1072c536; continue;
            }
            0x1072c536 => {
                c.alu(AluOp::Sub, c.rd16(c.r[6].wrapping_add(48u32))? as u32, 81u32, 16);
                c.r[2] = c.r[3];
                if !c.zf { pc = 0x1072c54d; continue; }
            pc = 0x1072c53f; continue;
            }
            0x1072c53f => {
                c.r[0] = 715827883u32;
                { let (lo, hi) = c.imul_wide(c.r[0], c.r[3]); c.r[0] = lo; c.r[2] = hi; }
                c.r[0] = c.r[2];
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 31u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], c.r[0], 32);
            pc = 0x1072c54d; continue;
            }
            0x1072c54d => {
                c.alu(AluOp::Sub, c.r[2], 1u32, 32);
                if c.cf { pc = 0x1072c568; continue; }
            pc = 0x1072c552; continue;
            }
            0x1072c552 => {
                if c.zf { pc = 0x1072c55b; continue; }
            pc = 0x1072c554; continue;
            }
            0x1072c554 => {
                c.r[1] = c.r[2].wrapping_add(4294967295u32);
                c.alu(AluOp::And, c.r[2], c.r[1], 32);
                if !c.zf { pc = 0x1072c568; continue; }
            pc = 0x1072c55b; continue;
            }
            0x1072c55b => {
                c.push(c.r[5])?;
                c.push(0x1072c561)?; dispatch(c, 0x1072c270)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if !c.zf { pc = 0x1072c579; continue; }
            pc = 0x1072c568; continue;
            }
            0x1072c568 => {
                c.alu(AluOp::Sub, c.rd32(c.r[6])?, 0u32, 32);
                if !c.zf { pc = 0x1072c579; continue; }
            pc = 0x1072c56d; continue;
            }
            0x1072c56d => {
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[5] = v; }
                c.r[0] = 4294967293u32;
                { let v = c.pop()?; c.r[3] = v; }
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            0x1072c579 => {
                c.push(c.r[7])?;
                c.push(9140u32)?;
                c.push(0x1072c584)?; dispatch(c, 0x10302860)?;
                c.r[7] = c.r[0];
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                { let v = c.r[7]; c.wr32(338496676u32, v)?; }
                c.r[1] = 2285u32;
                c.push(1244u32)?;
                c.rep_stos(4)?;
                c.push(0x1072c59f)?; dispatch(c, 0x10302860)?;
                c.r[2] = c.rd32(338496676u32)?;
                { let v = c.r[0]; c.wr32(c.r[2].wrapping_add(4u32), v)?; }
                c.r[2] = c.rd32(338496676u32)?;
                c.r[7] = c.rd32(c.r[2].wrapping_add(4u32))?;
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.r[1] = 311u32;
                c.rep_stos(4)?;
                c.r[7] = c.rd32(338496676u32)?;
                c.r[7] = c.alu(AluOp::Add, c.r[7], 9080u32, 32);
                c.r[1] = 14u32;
                c.rep_movs(4)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.alu(AluOp::Sub, c.r[3], 1u32, 32);
                if c.cf { pc = 0x1072c5f6; continue; }
            pc = 0x1072c5d7; continue;
            }
            0x1072c5d7 => {
                if c.zf { pc = 0x1072c5e0; continue; }
            pc = 0x1072c5d9; continue;
            }
            0x1072c5d9 => {
                c.r[0] = c.r[3].wrapping_add(4294967295u32);
                c.alu(AluOp::And, c.r[3], c.r[0], 32);
                if !c.zf { pc = 0x1072c5f6; continue; }
            pc = 0x1072c5e0; continue;
            }
            0x1072c5e0 => {
                c.alu(AluOp::Sub, c.r[5], 1u32, 32);
                if c.cf { pc = 0x1072c5f6; continue; }
            pc = 0x1072c5e5; continue;
            }
            0x1072c5e5 => {
                if c.zf { pc = 0x1072c701; continue; }
            pc = 0x1072c5eb; continue;
            }
            0x1072c5eb => {
                c.r[1] = c.r[5].wrapping_add(4294967295u32);
                c.alu(AluOp::And, c.r[5], c.r[1], 32);
                if c.zf { pc = 0x1072c701; continue; }
            pc = 0x1072c5f6; continue;
            }
            0x1072c5f6 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[0] = c.rd32(c.r[2])?;
                c.r[0] = c.incdec(c.r[0], 32, false);
                if c.zf { pc = 0x1072c6b2; continue; }
            pc = 0x1072c603; continue;
            }
            0x1072c603 => {
                c.r[0] = c.incdec(c.r[0], 32, false);
                if c.zf { pc = 0x1072c663; continue; }
            pc = 0x1072c606; continue;
            }
            0x1072c606 => {
                c.r[0] = c.incdec(c.r[0], 32, false);
                if !c.zf { pc = 0x1072c701; continue; }
            pc = 0x1072c60d; continue;
            }
            0x1072c60d => {
                c.r[0] = c.r[4].wrapping_add(20u32);
                c.push(c.r[0])?;
                c.r[1] = c.r[4].wrapping_add(20u32);
                c.push(c.r[1])?;
                c.push(c.r[5])?;
                c.push(c.r[3])?;
                c.push(0x1072c61e)?; dispatch(c, 0x1072c470)?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.push(c.r[2])?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.push(c.r[0])?;
                c.push(c.r[7])?;
                c.push(c.r[1])?;
                c.push(c.r[5])?;
                c.push(c.r[3])?;
                c.push(c.r[2])?;
                c.push(0x1072c63e)?; dispatch(c, 0x1072bf90)?;
                c.r[6] = c.r[0];
                c.r[4] = c.alu(AluOp::Add, c.r[4], 44u32, 32);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                if c.zf { pc = 0x1072c705; continue; }
            pc = 0x1072c64b; continue;
            }
            0x1072c64b => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[5] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[3] = c.r[7];
                c.r[7] = c.imul(c.r[7], c.rd32(c.r[4].wrapping_add(48u32))?);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                pc = 0x1072c705; continue;
            }
            0x1072c663 => {
                c.r[0] = c.r[4].wrapping_add(20u32);
                c.push(c.r[0])?;
                c.r[1] = c.r[4].wrapping_add(20u32);
                c.push(c.r[1])?;
                c.push(c.r[5])?;
                c.push(c.r[3])?;
                c.push(0x1072c674)?; dispatch(c, 0x1072c3f0)?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.push(c.r[2])?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.push(c.r[0])?;
                c.push(c.r[7])?;
                c.push(c.r[1])?;
                c.push(c.r[5])?;
                c.push(c.r[3])?;
                c.push(c.r[2])?;
                c.push(0x1072c694)?; dispatch(c, 0x1072bf90)?;
                c.r[6] = c.r[0];
                c.r[4] = c.alu(AluOp::Add, c.r[4], 44u32, 32);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                if c.zf { pc = 0x1072c705; continue; }
            pc = 0x1072c69d; continue;
            }
            0x1072c69d => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[5] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[3] = c.r[7];
                c.r[7] = c.imul(c.r[7], c.rd32(c.r[4].wrapping_add(48u32))?);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                pc = 0x1072c705; continue;
            }
            0x1072c6b2 => {
                c.r[0] = c.r[4].wrapping_add(20u32);
                c.push(c.r[0])?;
                c.r[1] = c.r[4].wrapping_add(20u32);
                c.push(c.r[1])?;
                c.push(c.r[5])?;
                c.push(c.r[3])?;
                c.push(0x1072c6c3)?; dispatch(c, 0x1072c370)?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.push(c.r[2])?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.push(c.r[0])?;
                c.push(c.r[7])?;
                c.push(c.r[1])?;
                c.push(c.r[5])?;
                c.push(c.r[3])?;
                c.push(c.r[2])?;
                c.push(0x1072c6e3)?; dispatch(c, 0x1072bf90)?;
                c.r[6] = c.r[0];
                c.r[4] = c.alu(AluOp::Add, c.r[4], 44u32, 32);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                if c.zf { pc = 0x1072c705; continue; }
            pc = 0x1072c6ec; continue;
            }
            0x1072c6ec => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[5] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[3] = c.r[7];
                c.r[7] = c.imul(c.r[7], c.rd32(c.r[4].wrapping_add(48u32))?);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                pc = 0x1072c705; continue;
            }
            0x1072c701 => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(40u32))?;
            pc = 0x1072c705; continue;
            }
            0x1072c705 => {
                c.r[0] = c.rd32(338496676u32)?;
                c.r[1] = c.rd32(c.r[0].wrapping_add(4u32))?;
                { let v = (c.r[3] & 0xffff); c.wr16(c.r[1].wrapping_add(48u32), v as u16)?; }
                c.r[2] = c.rd32(338496676u32)?;
                c.r[0] = c.rd32(c.r[2].wrapping_add(4u32))?;
                { let v = (c.r[5] & 0xffff); c.wr16(c.r[0].wrapping_add(46u32), v as u16)?; }
                c.r[1] = c.rd32(338496676u32)?;
                c.r[2] = c.rd32(c.r[1].wrapping_add(4u32))?;
                { let v = c.rd16(c.r[4].wrapping_add(48u32))? as u32; c.set16(0, v); }
                { let v = (c.r[0] & 0xffff); c.wr16(c.r[2].wrapping_add(52u32), v as u16)?; }
                c.r[1] = c.rd32(338496676u32)?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(28u32))?;
                { let v = c.r[2]; c.wr32(c.r[1].wrapping_add(8208u32), v)?; }
                c.r[0] = c.rd32(338496676u32)?;
                c.r[2] = c.rd32(c.r[0].wrapping_add(4u32))?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = (c.r[1] & 0xffff); c.wr16(c.r[2].wrapping_add(838u32), v as u16)?; }
                c.r[0] = c.rd32(338496676u32)?;
                c.r[0] = c.rd32(c.r[0].wrapping_add(4u32))?;
                { let v = c.rd16(c.r[0].wrapping_add(48u32))? as u32; c.set16(2, v); }
                { let v = (c.r[2] & 0xffff); c.wr16(c.r[0].wrapping_add(842u32), v as u16)?; }
                c.r[0] = c.rd32(338496676u32)?;
                c.r[2] = c.rd32(c.r[0].wrapping_add(4u32))?;
                { let v = (c.r[1] & 0xffff); c.wr16(c.r[2].wrapping_add(836u32), v as u16)?; }
                c.r[0] = c.rd32(338496676u32)?;
                c.r[0] = c.rd32(c.r[0].wrapping_add(4u32))?;
                { let v = c.rd16(c.r[0].wrapping_add(46u32))? as u32; c.set16(2, v); }
                { let v = (c.r[2] & 0xffff); c.wr16(c.r[0].wrapping_add(840u32), v as u16)?; }
                c.r[0] = c.rd32(338496676u32)?;
                c.r[0] = c.rd32(c.r[0].wrapping_add(4u32))?;
                { let v = c.rd16(c.r[0].wrapping_add(52u32))? as u32; c.set16(2, v); }
                { let v = (c.r[2] & 0xffff); c.wr16(c.r[0].wrapping_add(848u32), v as u16)?; }
                c.r[0] = c.rd32(338496676u32)?;
                c.r[2] = c.rd32(c.r[0].wrapping_add(4u32))?;
                { let v = c.r[7]; c.wr32(c.r[2].wrapping_add(852u32), v)?; }
                c.r[0] = c.rd32(338496676u32)?;
                c.r[2] = c.rd32(c.r[0].wrapping_add(4u32))?;
                { let v = 1u32; c.wr32(c.r[2].wrapping_add(856u32), v)?; }
                c.r[0] = c.rd32(338496676u32)?;
                c.r[2] = c.rd32(c.r[0].wrapping_add(4u32))?;
                c.r[0] = c.rd32(c.r[0].wrapping_add(8208u32))?;
                { let v = c.r[0]; c.wr32(c.r[2].wrapping_add(832u32), v)?; }
                c.r[2] = c.rd32(338496676u32)?;
                c.r[0] = c.rd32(c.r[2].wrapping_add(4u32))?;
                { let v = (c.r[1] & 0xffff); c.wr16(c.r[0].wrapping_add(844u32), v as u16)?; }
                c.r[1] = c.rd32(338496676u32)?;
                c.r[0] = c.rd32(c.r[1].wrapping_add(4u32))?;
                { let v = c.rd16(c.r[0].wrapping_add(52u32))? as u32; c.set16(2, v); }
                { let v = c.incdec((c.r[2] & 0xffff), 16, false); c.set16(2, v); }
                { let v = (c.r[2] & 0xffff); c.wr16(c.r[0].wrapping_add(846u32), v as u16)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(52u32))?;
                c.push(c.r[0])?;
                c.r[0] = c.rd32(338496676u32)?;
                c.r[1] = c.sext(c.rd16(c.r[0].wrapping_add(9130u32))? as u32, 16);
                c.push(c.r[1])?;
                c.push(c.r[0])?;
                c.push(0x1072c80b)?; dispatch(c, 0x1072fe80)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                { let v = c.pop()?; c.r[7] = v; }
                if c.zf { pc = 0x1072c81c; continue; }
            pc = 0x1072c813; continue;
            }
            0x1072c813 => {
                c.push(c.r[6])?;
                c.push(0x1072c819)?; dispatch(c, 0x10303283)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
            pc = 0x1072c81c; continue;
            }
            0x1072c81c => {
                c.r[2] = c.rd32(338496676u32)?;
                c.r[0] = c.rd32(c.r[2].wrapping_add(4u32))?;
                c.push(c.r[0])?;
                c.push(0x1072c82b)?; dispatch(c, 0x10306203)?;
                c.r[1] = c.rd32(338496676u32)?;
                c.push(c.r[1])?;
                c.push(0x1072c837)?; dispatch(c, 0x10306203)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[5] = v; }
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                { let v = c.pop()?; c.r[3] = v; }
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_1072dfa0(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x1072dfa0;
    loop {
        match pc {
            0x1072dfa0 => {
                c.fpush(c.rdf32(c.r[4].wrapping_add(4u32))? as f64);
                c.r[0] = c.rd32(c.r[4].wrapping_add(4u32))?;
                { let v = c.rdf32(c.r[4].wrapping_add(4u32))? as f64; c.farith_value(FOp::Mul, v); }
                c.r[1] = c.rd32(c.r[4].wrapping_add(8u32))?;
                c.fpush(c.rdf32(c.r[4].wrapping_add(8u32))? as f64);
                c.r[2] = c.rd32(c.r[4].wrapping_add(12u32))?;
                { let v = c.rdf32(c.r[4].wrapping_add(8u32))? as f64; c.farith_value(FOp::Mul, v); }
                { let v = c.r[0]; c.wr32(338496688u32, v)?; }
                { let v = c.r[1]; c.wr32(338496692u32, v)?; }
                { let v = c.r[2]; c.wr32(338496696u32, v)?; }
                c.farith(FOp::Add, 1, 0, true);
                c.fpush(c.rdf32(c.r[4].wrapping_add(12u32))? as f64);
                { let v = c.rdf32(c.r[4].wrapping_add(12u32))? as f64; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                c.fsqrt();
                c.fcom_value(c.rdf64(276379832u32)?, false);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if !c.pf { pc = 0x1072e01f; continue; }
            pc = 0x1072dfe8; continue;
            }
            0x1072dfe8 => {
                c.fcom_value(c.rdf64(276379816u32)?, false);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if !c.pf { pc = 0x1072e01f; continue; }
            pc = 0x1072dff5; continue;
            }
            0x1072dff5 => {
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Divr, v); }
                c.fpush(c.rdf32(c.r[4].wrapping_add(4u32))? as f64);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf32(338496688u32, v as f32)?; c.fpop(); }
                c.fpush(c.rdf32(c.r[4].wrapping_add(8u32))? as f64);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf32(338496692u32, v as f32)?; c.fpop(); }
                c.fpush(c.rdf32(c.r[4].wrapping_add(12u32))? as f64);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf32(338496696u32, v as f32)?; c.fpop(); }
            pc = 0x1072e01f; continue;
            }
            0x1072e01f => {
                c.fst_st(0, true);
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_1072e050(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x1072e050;
    loop {
        match pc {
            0x1072e050 => {
                c.push(1034416030u32)?;
                c.push(1058793325u32)?;
                c.push(1050555711u32)?;
                c.push(0x1072e064)?; dispatch(c, 0x1072dfa0)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_1072e070(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x1072e070;
    loop {
        match pc {
            0x1072e070 => {
                c.push(c.r[5])?;
                c.r[5] = c.r[4];
                c.r[4] = c.alu(AluOp::And, c.r[4], 4294967288u32, 32);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 256u32, 32);
                c.push(c.r[6])?;
                c.push(c.r[7])?;
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.r[1] = 31u32;
                c.r[7] = c.r[4].wrapping_add(8u32);
                c.rep_stos(4)?;
                c.r[0] = c.rd32(c.r[2])?;
                c.r[1] = c.rd32(c.r[2].wrapping_add(4u32))?;
                c.r[2] = c.rd32(c.r[2].wrapping_add(8u32))?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 2u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.r[1] = 31u32;
                c.r[7] = c.r[4].wrapping_add(136u32);
                c.rep_stos(4)?;
                c.push(c.r[0])?;
                c.r[1] = c.r[4].wrapping_add(12u32);
                c.r[6] = 124u32;
                c.push(c.r[1])?;
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
                { let v = 6158u32; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = 64u32; c.wr32(c.r[4].wrapping_add(92u32), v)?; }
                { let v = 32u32; c.wr32(c.r[4].wrapping_add(100u32), v)?; }
                { let v = 16711680u32; c.wr32(c.r[4].wrapping_add(104u32), v)?; }
                { let v = 65280u32; c.wr32(c.r[4].wrapping_add(108u32), v)?; }
                { let v = 255u32; c.wr32(c.r[4].wrapping_add(112u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(144u32), v)?; }
                c.push(0x1072e101)?; dispatch(c, 0x10731690)?;
                c.push(c.r[0])?;
                c.push(1u32)?;
                { let v = c.r[0]; c.wr32(c.r[3], v)?; }
                c.push(0x1072e10b)?; dispatch(c, 0x1074113a)?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[1] = c.rd32(c.r[3])?;
                c.push(c.r[2])?;
                c.r[6] = c.r[0];
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.push(c.r[0])?;
                c.push(c.r[1])?;
                c.push(338496688u32)?;
                c.push(0u32)?;
                c.push(c.r[6])?;
                c.r[2] = c.r[4].wrapping_add(176u32);
                c.push(c.r[2])?;
                c.r[0] = c.r[4].wrapping_add(52u32);
                c.push(0u32)?;
                c.push(c.r[0])?;
                c.push(0x1072e134)?; dispatch(c, 0x10731710)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 52u32, 32);
                { let v = c.pop()?; c.r[7] = v; }
                c.r[0] = c.r[6];
                { let v = c.pop()?; c.r[6] = v; }
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_1072e140(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x1072e140;
    loop {
        match pc {
            0x1072e140 => {
                c.push(c.r[5])?;
                c.r[5] = c.r[4];
                c.r[4] = c.alu(AluOp::And, c.r[4], 4294967288u32, 32);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 256u32, 32);
                c.push(c.r[6])?;
                c.push(c.r[7])?;
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.r[1] = 31u32;
                c.r[7] = c.r[4].wrapping_add(8u32);
                c.rep_stos(4)?;
                c.r[0] = c.rd32(c.r[2])?;
                c.r[1] = c.rd32(c.r[2].wrapping_add(4u32))?;
                c.r[2] = c.rd32(c.r[2].wrapping_add(8u32))?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 2u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.r[1] = 31u32;
                c.r[7] = c.r[4].wrapping_add(136u32);
                c.rep_stos(4)?;
                c.r[1] = c.r[4].wrapping_add(8u32);
                c.r[6] = 124u32;
                c.push(2u32)?;
                c.push(c.r[1])?;
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
                { let v = 6158u32; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = 65u32; c.wr32(c.r[4].wrapping_add(92u32), v)?; }
                { let v = 32u32; c.wr32(c.r[4].wrapping_add(100u32), v)?; }
                { let v = 16711680u32; c.wr32(c.r[4].wrapping_add(104u32), v)?; }
                { let v = 65280u32; c.wr32(c.r[4].wrapping_add(108u32), v)?; }
                { let v = 255u32; c.wr32(c.r[4].wrapping_add(112u32), v)?; }
                { let v = 4278190080u32; c.wr32(c.r[4].wrapping_add(116u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(144u32), v)?; }
                c.push(0x1072e1da)?; dispatch(c, 0x10731690)?;
                c.push(c.r[0])?;
                c.push(1u32)?;
                { let v = c.r[0]; c.wr32(c.r[3], v)?; }
                c.push(0x1072e1e4)?; dispatch(c, 0x1074113a)?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.push(0u32)?;
                c.push(c.r[2])?;
                c.r[6] = c.r[0];
                c.r[0] = c.rd32(c.r[3])?;
                c.push(c.r[0])?;
                c.push(338496688u32)?;
                c.push(2u32)?;
                c.push(c.r[6])?;
                c.r[1] = c.r[4].wrapping_add(176u32);
                c.push(c.r[1])?;
                c.r[2] = c.r[4].wrapping_add(52u32);
                c.push(0u32)?;
                c.push(c.r[2])?;
                c.push(0x1072e20b)?; dispatch(c, 0x10731710)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 52u32, 32);
                { let v = c.pop()?; c.r[7] = v; }
                c.r[0] = c.r[6];
                { let v = c.pop()?; c.r[6] = v; }
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_1072fe80(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x1072fe80;
    loop {
        match pc {
            0x1072fe80 => {
                c.push(c.r[5])?;
                c.r[5] = c.r[4];
                c.r[4] = c.alu(AluOp::And, c.r[4], 4294967288u32, 32);
                c.push(4294967295u32)?;
                c.push(276224913u32)?;
                c.r[0] = c.fs0;
                c.push(c.r[0])?;
                c.fs0 = c.r[4];
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 2336u32, 32);
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[0] = c.rd32(c.r[1].wrapping_add(4u32))?;
                c.r[2] = c.sext(c.rd16(c.r[0].wrapping_add(48u32))? as u32, 16);
                c.r[0] = c.sext(c.rd16(c.r[0].wrapping_add(46u32))? as u32, 16);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = 0u32; c.wr8(338496836u32, v as u8)?; }
                c.r[0] = c.sext(c.rd16(c.r[1].wrapping_add(9086u32))? as u32, 16);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], 32u32, 32);
                c.push(c.r[3])?;
                c.push(c.r[6])?;
                c.push(c.r[7])?;
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                if c.zf { pc = 0x1072fed9; continue; }
            pc = 0x1072fecd; continue;
            }
            0x1072fecd => {
                c.r[0] = c.incdec(c.r[0], 32, false);
                if !c.zf { pc = 0x1072fee6; continue; }
            pc = 0x1072fed0; continue;
            }
            0x1072fed0 => {
                { let v = 1u32; c.wr8(338496836u32, v as u8)?; }
                pc = 0x1072fee6; continue;
            }
            0x1072fed9 => {
                c.r[0] = c.r[2];
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], 1u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                c.r[2] = c.r[0];
            pc = 0x1072fee6; continue;
            }
            0x1072fee6 => {
                c.alu(AluOp::Sub, c.rd16(c.r[1].wrapping_add(9086u32))? as u32, 31u32, 16);
                if !c.zf { pc = 0x1072ff01; continue; }
            pc = 0x1072fef0; continue;
            }
            0x1072fef0 => {
                c.alu(AluOp::Sub, c.rd16(c.r[1].wrapping_add(9088u32))? as u32, 0u32, 16);
                if !c.zf { pc = 0x1072ff01; continue; }
            pc = 0x1072fefa; continue;
            }
            0x1072fefa => {
                { let v = 1u32; c.wr8(338496836u32, v as u8)?; }
            pc = 0x1072ff01; continue;
            }
            0x1072ff01 => {
                c.alu(AluOp::Sub, c.rd16(c.r[1].wrapping_add(9128u32))? as u32, 81u32, 16);
                { let v = 1u32; c.wr32(c.r[4].wrapping_add(68u32), v)?; }
                if !c.zf { pc = 0x1072ff2d; continue; }
            pc = 0x1072ff13; continue;
            }
            0x1072ff13 => {
                c.r[0] = 715827883u32;
                { let (lo, hi) = c.imul_wide(c.r[0], c.r[2]); c.r[0] = lo; c.r[2] = hi; }
                c.r[0] = c.r[2];
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 31u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], c.r[0], 32);
                { let v = 6u32; c.wr32(c.r[4].wrapping_add(68u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
            pc = 0x1072ff2d; continue;
            }
            0x1072ff2d => {
                { let v = c.rd8(c.r[1].wrapping_add(9126u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x1072ff3e; continue; }
            pc = 0x1072ff37; continue;
            }
            0x1072ff37 => {
                c.push(0x1072ff3c)?; dispatch(c, 0x1072e030)?;
                pc = 0x1072ff43; continue;
            }
            0x1072ff3e => {
                c.push(0x1072ff43)?; dispatch(c, 0x1072e050)?;
            pc = 0x1072ff43; continue;
            }
            0x1072ff43 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.alu(AluOp::Sub, c.r[0], 10u32, 32);
                if c.cf { pc = 0x1072ff55; continue; }
            pc = 0x1072ff4b; continue;
            }
            0x1072ff4b => {
                c.alu(AluOp::Sub, c.r[0], 13u32, 32);
                { let v = 1u32; c.wr8(c.r[4].wrapping_add(64u32), v as u8)?; }
                if c.cf || c.zf { pc = 0x1072ff5a; continue; }
            pc = 0x1072ff55; continue;
            }
            0x1072ff55 => {
                { let v = 0u32; c.wr8(c.r[4].wrapping_add(64u32), v as u8)?; }
            pc = 0x1072ff5a; continue;
            }
            0x1072ff5a => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.push(c.r[1])?;
                c.push(c.r[2])?;
                c.push(c.r[0])?;
                c.push(0x1072ff6e)?; dispatch(c, 0x10735c00)?;
                c.r[7] = c.r[0];
                c.r[0] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 4294967286u32, 32);
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                c.alu(AluOp::Sub, c.r[0], 12u32, 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(60u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(80u32), v)?; }
                if !c.cf && !c.zf { pc = 0x10730019; continue; }
            pc = 0x1072ff8e; continue;
            }
            0x1072ff8e => {
                pc = c.rd32(c.r[0].wrapping_mul(4).wrapping_add(275977692u32))?; if ![0x1072ffd4, 0x1072ff95, 0x1072ffaa, 0x1072ffbf, 0x1072ffe6, 0x1072ffe6, 0x1072ffe6, 0x1072fff0, 0x1072fffb, 0x1072ffe6, 0x1073000a, 0x1072fff0, 0x1072fff0, 0x1073015c, 0x107300f9, 0x1073011a, 0x1073013b, 0x10730185, 0x10730218, 0x10730322, 0x107303a1, 0x107303de, 0x107302a3, 0x1073043f, 0x107303a1, 0x107303a1, 0x10730746, 0x1073075b, 0x10730770, 0x10730787, 0x107308c3, 0x10730855, 0x10730891, 0x107308aa, 0x107308e7, 0x10730977, 0x10730a8a, 0x10730b0a, 0x10730b47, 0x10730a07, 0x10730b96].contains(&pc) { return Err(Fault("jmp", pc)); } continue;
            }
            0x1072ff95 => {
                { let v = 1u32; c.wr8(c.r[4].wrapping_add(23u32), v as u8)?; }
                { let v = 0u32; c.wr8(338496837u32, v as u8)?; }
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.r[3] = 2u32;
                pc = 0x10730019; continue;
            }
            0x1072ffaa => {
                { let v = 1u32; c.wr8(c.r[4].wrapping_add(23u32), v as u8)?; }
                { let v = 1u32; c.wr8(338496837u32, v as u8)?; }
                c.r[6] = 256u32;
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                pc = 0x10730019; continue;
            }
            0x1072ffbf => {
                { let v = 1u32; c.wr8(c.r[4].wrapping_add(23u32), v as u8)?; }
                { let v = 1u32; c.wr8(338496837u32, v as u8)?; }
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                c.r[6] = 512u32;
                pc = 0x10730019; continue;
            }
            0x1072ffd4 => {
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                { let v = 1u32; c.wr8(c.r[4].wrapping_add(23u32), v as u8)?; }
                { let v = 0u32; c.wr8(338496837u32, v as u8)?; }
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                pc = 0x10730019; continue;
            }
            0x1072ffe6 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 1u32, 32);
                pc = 0x10730010; continue;
            }
            0x1072fff0 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 2u32, 32);
                pc = 0x10730010; continue;
            }
            0x1072fffb => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[1] = c.r[0].wrapping_add(c.r[0].wrapping_mul(2));
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                pc = 0x10730014; continue;
            }
            0x1073000a => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
            pc = 0x10730010; continue;
            }
            0x10730010 => {
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
            pc = 0x10730014; continue;
            }
            0x10730014 => {
                { let v = 0u32; c.wr8(c.r[4].wrapping_add(23u32), v as u8)?; }
            pc = 0x10730019; continue;
            }
            0x10730019 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if !c.zf { pc = 0x10730080; continue; }
            pc = 0x10730020; continue;
            }
            0x10730020 => {
                { let v = c.rd8(c.r[4].wrapping_add(23u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10730061; continue; }
            pc = 0x10730028; continue;
            }
            0x10730028 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[2] = c.r[3];
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[6], 32);
                c.push(c.r[2])?;
                c.push(c.r[0])?;
                c.push(c.r[1])?;
                c.push(0x1073003c)?; dispatch(c, 0x107316c0)?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(60u32))?;
                c.push(c.r[0])?;
                c.push(c.r[3])?;
                c.push(c.r[6])?;
                c.r[6] = c.rd32(c.r[5].wrapping_add(8u32))?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(56u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.push(0u32)?;
                c.push(c.r[2])?;
                c.push(c.r[0])?;
                { let v = c.rd8(338496837u32)? as u32; c.set8(0, 0, v); }
                c.push(0x1073005c)?; dispatch(c, 0x1072e9e0)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 36u32, 32);
                pc = 0x10730080; continue;
            }
            0x10730061 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[3] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.push(c.r[1])?;
                c.push(c.r[2])?;
                c.push(0u32)?;
                c.push(c.r[0])?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.push(0x1073007d)?; dispatch(c, 0x1072f2d0)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 16u32, 32);
            pc = 0x10730080; continue;
            }
            0x10730080 => {
                { let v = c.rd8(338496836u32)? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10730602; continue; }
            pc = 0x1073008d; continue;
            }
            0x1073008d => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(68u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x107305f7; continue; }
            pc = 0x107300a1; continue;
            }
            0x107300a1 => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(16u32))?;
            pc = 0x107300a5; continue;
            }
            0x107300a5 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[0] = c.rd32(c.r[1].wrapping_add(4u32))?;
                c.r[0] = c.sext(c.rd16(c.r[0].wrapping_add(52u32))? as u32, 16);
                c.r[6] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[0] = c.imul(c.r[0], c.r[6]);
                c.r[0] = c.imul(c.r[0], c.r[2]);
                c.alu(AluOp::Sub, c.rd16(c.r[1].wrapping_add(9086u32))? as u32, 32u32, 16);
                if !c.zf { pc = 0x107300c9; continue; }
            pc = 0x107300c7; continue;
            }
            0x107300c7 => {
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 1u32, 32);
            pc = 0x107300c9; continue;
            }
            0x107300c9 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.push(c.r[6])?;
                c.r[6] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.push(c.r[6])?;
                c.push(c.r[0])?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(60u32))?;
                c.push(c.r[0])?;
                c.push(c.r[2])?;
                c.push(c.r[1])?;
                c.push(c.r[7])?;
                c.push(0x107300e0)?; dispatch(c, 0x10735c60)?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(108u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 28u32, 32);
                c.alu(AluOp::Sub, c.r[0], 12u32, 32);
                if !c.cf && !c.zf { pc = 0x1073059c; continue; }
            pc = 0x107300f0; continue;
            }
            0x107300f0 => {
                c.r[1] = c.r[0];
                pc = c.rd32(c.r[1].wrapping_mul(4).wrapping_add(275977744u32))?; if ![0x1073015c, 0x107300f9, 0x1073011a, 0x1073013b, 0x10730185, 0x10730218, 0x10730322, 0x107303a1, 0x107303de, 0x107302a3, 0x1073043f, 0x107303a1, 0x107303a1, 0x10730746, 0x1073075b, 0x10730770, 0x10730787, 0x107308c3, 0x10730855, 0x10730891, 0x107308aa, 0x107308e7, 0x10730977, 0x10730a8a, 0x10730b0a, 0x10730b47, 0x10730a07, 0x10730b96].contains(&pc) { return Err(Fault("jmp", pc)); } continue;
            }
            0x107300f9 => {
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                { let v = c.rd8(c.r[2].wrapping_add(9126u32))? as u32; c.set8(0, 0, v); }
                c.r[3] = c.r[4].wrapping_add(32u32);
                c.r[2] = c.r[7];
                c.push(c.r[0])?;
                c.push(0x10730110)?; dispatch(c, 0x1072e140)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.r[3] = c.r[0];
                pc = 0x1073059c; continue;
            }
            0x1073011a => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                { let v = c.rd8(c.r[1].wrapping_add(9126u32))? as u32; c.set8(2, 0, v); }
                c.r[3] = c.r[4].wrapping_add(32u32);
                c.push(c.r[2])?;
                c.r[2] = c.r[7];
                c.push(0x10730131)?; dispatch(c, 0x1072e270)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.r[3] = c.r[0];
                pc = 0x1073059c; continue;
            }
            0x1073013b => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = c.rd8(c.r[0].wrapping_add(9126u32))? as u32; c.set8(1, 0, v); }
                c.r[3] = c.r[4].wrapping_add(32u32);
                c.r[2] = c.r[7];
                c.push(c.r[1])?;
                c.push(0x10730152)?; dispatch(c, 0x1072e350)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.r[3] = c.r[0];
                pc = 0x1073059c; continue;
            }
            0x1073015c => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                { let v = c.rd8(c.r[0].wrapping_add(9127u32))? as u32; c.set8(2, 0, v); }
                c.r[0] = c.rd8(c.r[0].wrapping_add(9126u32))? as u32;
                c.r[3] = c.r[4].wrapping_add(32u32);
                c.push(c.r[2])?;
                c.push(c.r[0])?;
                c.r[2] = c.r[7];
                c.push(0x1073017b)?; dispatch(c, 0x1072e070)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                c.r[3] = c.r[0];
                pc = 0x1073059c; continue;
            }
            0x10730185 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 1u32, 32);
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.push(0x10730197)?; dispatch(c, 0x10740e60)?;
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[7])?);
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                c.r[3] = c.r[0];
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073059c; continue; }
            pc = 0x107301b9; continue;
            }
            0x107301b9 => {
                c.r[4] = c.r[4];
            pc = 0x107301c0; continue;
            }
            0x107301c0 => {
                c.r[0] = c.rd32(c.r[1])?;
                c.r[2] = c.r[0];
                c.r[6] = c.r[0];
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], 16u32, 32);
                c.r[6] = c.shift(ShiftOp::Shr, c.r[6], 12u32, 32);
                c.r[2] = c.alu(AluOp::And, c.r[2], 61440u32, 32);
                c.r[6] = c.alu(AluOp::And, c.r[6], 3840u32, 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[6], 32);
                c.r[6] = c.r[0];
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 4u32, 32);
                c.r[6] = c.shift(ShiftOp::Shr, c.r[6], 8u32, 32);
                c.r[0] = c.alu(AluOp::And, c.r[0], 15u32, 32);
                c.r[6] = c.alu(AluOp::And, c.r[6], 240u32, 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[6], 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[0], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                { let v = (c.r[2] & 0xffff); c.wr16(c.r[0], v as u16)?; }
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[7])?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 2u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.sf != c.of { pc = 0x107301c0; continue; }
            pc = 0x10730213; continue;
            }
            0x10730213 => {
                pc = 0x1073059c; continue;
            }
            0x10730218 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 1u32, 32);
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.push(0x1073022a)?; dispatch(c, 0x10740e60)?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[3] = c.r[0];
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[2] = c.r[3];
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073059c; continue; }
            pc = 0x1073024e; continue;
            }
            0x1073024e => {
                c.r[7] = c.r[7];
            pc = 0x10730250; continue;
            }
            0x10730250 => {
                c.r[0] = c.rd32(c.r[1])?;
                c.r[6] = c.r[0];
                c.r[6] = c.shift(ShiftOp::Shr, c.r[6], 9u32, 32);
                c.r[3] = c.r[0];
                c.r[3] = c.shift(ShiftOp::Shr, c.r[3], 6u32, 32);
                c.r[6] = c.alu(AluOp::And, c.r[6], 31744u32, 32);
                c.r[3] = c.alu(AluOp::And, c.r[3], 992u32, 32);
                c.r[6] = c.alu(AluOp::Or, c.r[6], c.r[3], 32);
                c.r[3] = c.r[0];
                c.r[3] = c.shift(ShiftOp::Shr, c.r[3], 3u32, 32);
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 16u32, 32);
                c.r[3] = c.alu(AluOp::And, c.r[3], 31u32, 32);
                c.r[6] = c.alu(AluOp::Or, c.r[6], c.r[3], 32);
                c.r[0] = c.alu(AluOp::And, c.r[0], 32768u32, 32);
                c.r[6] = c.alu(AluOp::Or, c.r[6], c.r[0], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                { let v = (c.r[6] & 0xffff); c.wr16(c.r[2], v as u16)?; }
                c.r[6] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[6] = c.imul(c.r[6], c.rd32(c.r[7])?);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 2u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], c.r[6], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.sf != c.of { pc = 0x10730250; continue; }
            pc = 0x1073029a; continue;
            }
            0x1073029a => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(16u32))?;
                pc = 0x1073059c; continue;
            }
            0x107302a3 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 1u32, 32);
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.push(0x107302b5)?; dispatch(c, 0x10740e60)?;
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[7])?);
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                c.r[3] = c.r[0];
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073059c; continue; }
            pc = 0x107302d7; continue;
            }
            0x107302d7 => {
                c.r[0] = c.rd32(c.r[1])?;
                c.r[2] = c.r[0];
                c.r[6] = c.r[0];
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], 9u32, 32);
                c.r[2] = c.alu(AluOp::And, c.r[2], 31744u32, 32);
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 3u32, 32);
                c.r[6] = c.shift(ShiftOp::Shr, c.r[6], 6u32, 32);
                c.r[0] = c.alu(AluOp::And, c.r[0], 31u32, 32);
                c.r[6] = c.alu(AluOp::And, c.r[6], 992u32, 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[6], 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[0], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                { let v = (c.r[2] & 0xffff); c.wr16(c.r[0], v as u16)?; }
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[7])?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 2u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.sf != c.of { pc = 0x107302d7; continue; }
            pc = 0x1073031d; continue;
            }
            0x1073031d => {
                pc = 0x1073059c; continue;
            }
            0x10730322 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 1u32, 32);
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.push(0x10730334)?; dispatch(c, 0x10740e60)?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[3] = c.r[0];
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073059c; continue; }
            pc = 0x10730356; continue;
            }
            0x10730356 => {
                c.r[0] = c.rd32(c.r[1])?;
                c.r[2] = c.r[0];
                c.r[6] = c.r[0];
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], 8u32, 32);
                c.r[2] = c.alu(AluOp::And, c.r[2], 63488u32, 32);
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 3u32, 32);
                c.r[6] = c.shift(ShiftOp::Shr, c.r[6], 5u32, 32);
                c.r[0] = c.alu(AluOp::And, c.r[0], 31u32, 32);
                c.r[6] = c.alu(AluOp::And, c.r[6], 2016u32, 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[6], 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[0], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                { let v = (c.r[2] & 0xffff); c.wr16(c.r[0], v as u16)?; }
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[7])?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 2u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.sf != c.of { pc = 0x10730356; continue; }
            pc = 0x1073039c; continue;
            }
            0x1073039c => {
                pc = 0x1073059c; continue;
            }
            0x107303a1 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 2u32, 32);
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.push(0x107303b4)?; dispatch(c, 0x10740e60)?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[1] = c.imul(c.r[1], c.rd32(c.r[7])?);
                c.r[6] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 2u32, 32);
                c.r[3] = c.r[0];
                c.r[0] = c.r[1];
                c.r[1] = c.shift(ShiftOp::Shr, c.r[1], 2u32, 32);
                c.r[7] = c.r[3];
                c.rep_movs(4)?;
                c.r[1] = c.r[0];
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.r[1] = c.alu(AluOp::And, c.r[1], 3u32, 32);
                c.rep_movs(1)?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(60u32))?;
                pc = 0x1073059c; continue;
            }
            0x107303de => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.r[0].wrapping_add(c.r[0].wrapping_mul(2));
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.push(0x107303f1)?; dispatch(c, 0x10740e60)?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[1] = c.imul(c.r[1], c.rd32(c.r[7])?);
                c.r[6] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                c.r[3] = c.r[0];
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073059c; continue; }
            pc = 0x1073040f; continue;
            }
            0x1073040f => {
            pc = 0x10730410; continue;
            }
            0x10730410 => {
                { let v = c.rd8(c.r[6])? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[0], v as u8)?; }
                { let v = c.rd8(c.r[6].wrapping_add(1u32))? as u32; c.set8(1, 0, v); }
                { let v = ((c.r[1] >> 0) & 0xff); c.wr8(c.r[0].wrapping_add(1u32), v as u8)?; }
                { let v = c.rd8(c.r[6].wrapping_add(2u32))? as u32; c.set8(2, 0, v); }
                c.r[1] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[0] = c.incdec(c.r[0], 32, true);
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[0], v as u8)?; }
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 4u32, 32);
                c.r[1] = c.incdec(c.r[1], 32, true);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.r[1] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[1] = c.imul(c.r[1], c.rd32(c.r[7])?);
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(24u32))?, c.r[1], 32);
                if c.sf != c.of { pc = 0x10730410; continue; }
            pc = 0x1073043a; continue;
            }
            0x1073043a => {
                pc = 0x1073059c; continue;
            }
            0x1073043f => {
                c.r[1] = c.r[4].wrapping_add(88u32);
                c.push(0x10730448)?; dispatch(c, 0x1072fe40)?;
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.r[1] = c.r[4].wrapping_add(108u32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(2356u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(104u32), v)?; }
                c.push(0x1073045e)?; dispatch(c, 0x1072fb10)?;
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[2] = c.rd32(c.r[0].wrapping_add(4u32))?;
                c.alu(AluOp::Sub, c.rd16(c.r[2].wrapping_add(52u32))? as u32, 3u32, 16);
                c.r[0] = c.rd8(c.r[0].wrapping_add(9124u32))? as u32;
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(1, 0, v); }
                { let v = ((c.r[1] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(28u32), v as u8)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.push(c.r[2])?;
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.push(c.r[0])?;
                c.r[0] = c.rd32(c.r[7])?;
                c.push(4u32)?;
                c.push(8u32)?;
                c.r[1] = c.r[4].wrapping_add(104u32);
                c.push(c.r[1])?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.push(4u32)?;
                c.push(c.r[2])?;
                c.push(c.r[0])?;
                c.push(c.r[1])?;
                c.push(0x10730498)?; dispatch(c, 0x10736110)?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(140u32))?;
                c.r[3] = c.r[0];
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(68u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(136u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 36u32, 32);
                c.alu(AluOp::Sub, c.r[0], 3u32, 32);
                if !c.zf { pc = 0x107304ff; continue; }
            pc = 0x107304ba; continue;
            }
            0x107304ba => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::Sub, c.r[1], c.r[6], 32);
                if c.zf || c.sf != c.of { pc = 0x1073053f; continue; }
            pc = 0x107304c0; continue;
            }
            0x107304c0 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(112u32))?;
                c.r[6] = c.alu(AluOp::Add, c.r[6], 4u32, 32);
                pc = 0x107304d0; continue;
            }
            0x107304d0 => {
                { let v = c.rd8(c.r[6].wrapping_add(4u32))? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(298u32), v as u8)?; }
                { let v = c.rd8(c.r[6])? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(297u32), v as u8)?; }
                { let v = c.rd8(c.r[6].wrapping_add(4294967292u32))? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(296u32), v as u8)?; }
                { let v = 255u32; c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(299u32), v as u8)?; }
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 12u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                if c.sf != c.of { pc = 0x107304d0; continue; }
            pc = 0x107304fd; continue;
            }
            0x107304fd => {
                pc = 0x1073053f; continue;
            }
            0x107304ff => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::Sub, c.r[1], c.r[6], 32);
                if c.zf || c.sf != c.of { pc = 0x1073053f; continue; }
            pc = 0x10730505; continue;
            }
            0x10730505 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(112u32))?;
                c.r[6] = c.alu(AluOp::Add, c.r[6], 8u32, 32);
                c.r[4] = c.r[4];
            pc = 0x10730510; continue;
            }
            0x10730510 => {
                { let v = c.rd8(c.r[6].wrapping_add(4294967288u32))? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(298u32), v as u8)?; }
                { let v = c.rd8(c.r[6].wrapping_add(4294967292u32))? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(297u32), v as u8)?; }
                { let v = c.rd8(c.r[6])? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(296u32), v as u8)?; }
                { let v = c.rd8(c.r[6].wrapping_add(4u32))? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(299u32), v as u8)?; }
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 16u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                if c.sf != c.of { pc = 0x10730510; continue; }
            pc = 0x1073053f; continue;
            }
            0x1073053f => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                c.push(1024u32)?;
                if c.zf { pc = 0x1073055e; continue; }
            pc = 0x1073054b; continue;
            }
            0x1073054b => {
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.r[0] = c.r[4].wrapping_add(300u32);
                c.push(c.r[6])?;
                c.push(c.r[0])?;
                { let t = c.rd32(c.r[5].wrapping_add(16u32))?; c.push(0x10730559)?; c.call_indirect(t)?; }
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
                pc = 0x10730574; continue;
            }
            0x1073055e => {
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[1] = c.r[4].wrapping_add(300u32);
                c.push(c.r[1])?;
                c.push(c.r[2])?;
                c.push(0x1073056f)?; dispatch(c, 0x1072e5d0)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
            pc = 0x10730574; continue;
            }
            0x10730574 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(112u32))?;
                c.alu(AluOp::Sub, c.r[0], c.r[6], 32);
                { let v = 4294967295u32; c.wr32(c.r[4].wrapping_add(2356u32), v)?; }
                if c.zf { pc = 0x10730590; continue; }
            pc = 0x10730587; continue;
            }
            0x10730587 => {
                c.push(c.r[0])?;
                c.push(0x1073058d)?; dispatch(c, 0x10306203)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
            pc = 0x10730590; continue;
            }
            0x10730590 => {
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(112u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(116u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(120u32), v)?; }
            pc = 0x1073059c; continue;
            }
            0x1073059c => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x107305af; continue; }
            pc = 0x107305a3; continue;
            }
            0x107305a3 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.push(c.r[1])?;
                c.push(0u32)?;
                c.push(c.r[3])?;
                { let t = c.r[0]; c.push(0x107305ad)?; c.call_indirect(t)?; }
                pc = 0x107305d6; continue;
            }
            0x107305af => {
                { let v = c.rd8(c.r[4].wrapping_add(23u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x107305c7; continue; }
            pc = 0x107305b7; continue;
            }
            0x107305b7 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[0] = c.r[3];
                c.push(0x107305c5)?; dispatch(c, 0x1072e430)?;
                pc = 0x107305d9; continue;
            }
            0x107305c7 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.push(c.r[2])?;
                c.push(c.r[3])?;
                c.push(c.r[0])?;
                c.push(0x107305d6)?; dispatch(c, 0x1072e5d0)?;
            pc = 0x107305d6; continue;
            }
            0x107305d6 => {
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
            pc = 0x107305d9; continue;
            }
            0x107305d9 => {
                c.push(c.r[3])?;
                c.push(0x107305df)?; dispatch(c, 0x10740e54)?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(72u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                if c.sf != c.of { pc = 0x107300a5; continue; }
            pc = 0x107305f7; continue;
            }
            0x107305f7 => {
                c.push(c.r[7])?;
                c.push(0x107305fd)?; dispatch(c, 0x10735c30)?;
                pc = 0x107315bc; continue;
            }
            0x10730602 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(40u32))?;
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(84u32), v)?; }
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(160u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(172u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(176u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(180u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(184u32), v)?; }
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(68u32))?, c.r[1], 32);
                { let v = 1u32; c.wr32(c.r[4].wrapping_add(2356u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(196u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(200u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(204u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(188u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(76u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x107315a2; continue; }
            pc = 0x10730668; continue;
            }
            0x10730668 => {
                c.r[2] = c.r[0].wrapping_mul(4);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(168u32), v)?; }
                pc = 0x10730680; continue;
            }
            0x10730680 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(160u32))?;
                c.r[3] = c.rd32(c.r[4].wrapping_add(84u32))?;
                c.push(c.r[7])?;
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
                c.push(0x10730699)?; dispatch(c, 0x10735c30)?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(68u32))?;
                c.push(c.r[0])?;
                c.push(c.r[3])?;
                c.push(c.r[6])?;
                c.push(0x107306a5)?; dispatch(c, 0x10735c00)?;
                c.r[3] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 16u32, 32);
                c.alu(AluOp::Sub, c.rd16(c.r[3].wrapping_add(9086u32))? as u32, 32u32, 16);
                c.r[7] = c.r[0];
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(60u32), v)?; }
                if !c.zf { pc = 0x107306d2; continue; }
            pc = 0x107306bb; continue;
            }
            0x107306bb => {
                c.r[1] = c.rd32(c.r[3].wrapping_add(4u32))?;
                c.r[0] = c.sext(c.rd16(c.r[1].wrapping_add(52u32))? as u32, 16);
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[4].wrapping_add(76u32))?);
                c.r[0] = c.imul(c.r[0], c.r[6]);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 1u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(72u32), v)?; }
                pc = 0x107306e5; continue;
            }
            0x107306d2 => {
                c.r[2] = c.rd32(c.r[3].wrapping_add(4u32))?;
                c.r[0] = c.sext(c.rd16(c.r[2].wrapping_add(52u32))? as u32, 16);
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[4].wrapping_add(76u32))?);
                c.r[0] = c.imul(c.r[0], c.r[6]);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(72u32), v)?; }
            pc = 0x107306e5; continue;
            }
            0x107306e5 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.push(c.r[1])?;
                c.push(c.r[2])?;
                c.push(c.r[0])?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(96u32))?;
                c.push(c.r[0])?;
                c.push(c.r[6])?;
                c.push(c.r[3])?;
                c.push(c.r[7])?;
                c.push(0x107306fc)?; dispatch(c, 0x10735c60)?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(196u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(112u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 28u32, 32);
                c.push(4u32)?;
                c.push(c.r[1])?;
                c.push(c.r[2])?;
                { let v = 1u32; c.wr8(338496700u32, v as u8)?; }
                c.r[0] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.push(c.r[6])?;
                c.push(c.r[0])?;
                c.r[1] = c.r[4].wrapping_add(244u32);
                c.push(0x10730726)?; dispatch(c, 0x10735a30)?;
                c.r[0] = c.sext(c.rd16(c.r[3].wrapping_add(9090u32))? as u32, 16);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 4294967162u32, 32);
                c.alu(AluOp::Sub, c.r[0], 3u32, 32);
                { let v = 2u32; c.wr8(c.r[4].wrapping_add(2356u32), v as u8)?; }
                if !c.cf && !c.zf { pc = 0x1073079e; continue; }
            pc = 0x1073073f; continue;
            }
            0x1073073f => {
                pc = c.rd32(c.r[0].wrapping_mul(4).wrapping_add(275977796u32))?; if ![0x10730746, 0x1073075b, 0x10730770, 0x10730787, 0x107308c3, 0x10730855, 0x10730891, 0x107308aa, 0x107308e7, 0x10730977, 0x10730a8a, 0x10730b0a, 0x10730b47, 0x10730a07, 0x10730b96].contains(&pc) { return Err(Fault("jmp", pc)); } continue;
            }
            0x10730746 => {
                c.push(275995472u32)?;
                c.push(0u32)?;
                c.r[1] = c.r[4].wrapping_add(232u32);
                c.push(0x10730759)?; dispatch(c, 0x10735130)?;
                pc = 0x107307b1; continue;
            }
            0x1073075b => {
                c.push(275995472u32)?;
                c.push(0u32)?;
                c.r[1] = c.r[4].wrapping_add(232u32);
                c.push(0x1073076e)?; dispatch(c, 0x10734da0)?;
                pc = 0x107307b1; continue;
            }
            0x10730770 => {
                c.push(275995472u32)?;
                c.push(0u32)?;
                c.push(0u32)?;
                c.r[1] = c.r[4].wrapping_add(236u32);
                c.push(0x10730785)?; dispatch(c, 0x10734ec0)?;
                pc = 0x107307b1; continue;
            }
            0x10730787 => {
                c.push(275995472u32)?;
                c.push(1u32)?;
                c.push(0u32)?;
                c.r[1] = c.r[4].wrapping_add(236u32);
                c.push(0x1073079c)?; dispatch(c, 0x10734ec0)?;
                pc = 0x107307b1; continue;
            }
            0x1073079e => {
                c.push(275995472u32)?;
                c.push(0u32)?;
                c.r[1] = c.r[4].wrapping_add(232u32);
                c.push(0x107307b1)?; dispatch(c, 0x10734c80)?;
            pc = 0x107307b1; continue;
            }
            0x107307b1 => {
                { let v = 0u32; c.wr8(338496700u32, v as u8)?; }
                c.fpush(c.rd32(c.r[3].wrapping_add(9120u32))? as i32 as f64);
                { let v = 0u32; c.wr8(c.r[4].wrapping_add(55u32), v as u8)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                { let v = c.rdf32(276406188u32)? as f64; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf32(276370776u32)? as f64; c.farith_value(FOp::Subr, v); }
                { let v = c.st0(); c.wrf32(c.r[4].wrapping_add(156u32), v as f32)?;  }
                { let v = c.st0(); c.wrf32(c.r[4].wrapping_add(56u32), v as f32)?; c.fpop(); }
            pc = 0x107307e2; continue;
            }
            0x107307e2 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                { let v = c.rd8(c.r[1].wrapping_add(9125u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10730843; continue; }
            pc = 0x107307ef; continue;
            }
            0x107307ef => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                c.r[6] = c.rd32(c.r[7].wrapping_add(8u32))?;
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10730843; continue; }
            pc = 0x10730801; continue;
            }
            0x10730801 => {
                c.r[0] = c.rd32(c.r[7])?;
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf || c.sf != c.of { pc = 0x10730833; continue; }
            pc = 0x10730809; continue;
            }
            0x10730809 => {
                c.r[4] = c.r[4];
            pc = 0x10730810; continue;
            }
            0x10730810 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[0] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.push(c.r[2])?;
                c.push(c.r[3])?;
                c.push(c.r[6])?;
                c.push(c.r[0])?;
                c.push(c.r[1])?;
                c.push(0x10730824)?; dispatch(c, 0x1072e5f0)?;
                { let v = c.r[0]; c.wr32(c.r[6], v)?; }
                c.r[0] = c.rd32(c.r[7])?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 20u32, 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 4u32, 32);
                c.r[3] = c.incdec(c.r[3], 32, true);
                c.alu(AluOp::Sub, c.r[3], c.r[0], 32);
                if c.sf != c.of { pc = 0x10730810; continue; }
            pc = 0x10730833; continue;
            }
            0x10730833 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.sf != c.of { pc = 0x10730801; continue; }
            pc = 0x10730843; continue;
            }
            0x10730843 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(80u32))?;
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                c.alu(AluOp::Sub, c.r[0], 10u32, 32);
                if !c.cf && !c.zf { pc = 0x10730871; continue; }
            pc = 0x1073084e; continue;
            }
            0x1073084e => {
                pc = c.rd32(c.r[0].wrapping_mul(4).wrapping_add(275977812u32))?; if ![0x107308c3, 0x10730855, 0x10730891, 0x107308aa, 0x107308e7, 0x10730977, 0x10730a8a, 0x10730b0a, 0x10730b47, 0x10730a07, 0x10730b96].contains(&pc) { return Err(Fault("jmp", pc)); } continue;
            }
            0x10730855 => {
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                { let v = c.rd8(c.r[2].wrapping_add(9126u32))? as u32; c.set8(0, 0, v); }
                c.r[3] = c.r[4].wrapping_add(32u32);
                c.r[2] = c.r[7];
                c.push(c.r[0])?;
                c.push(0x1073086c)?; dispatch(c, 0x1072e140)?;
            pc = 0x1073086c; continue;
            }
            0x1073086c => {
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
            pc = 0x1073086f; continue;
            }
            0x1073086f => {
                c.r[3] = c.r[0];
            pc = 0x10730871; continue;
            }
            0x10730871 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10730cc3; continue; }
            pc = 0x1073087c; continue;
            }
            0x1073087c => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.push(c.r[1])?;
                c.push(c.r[2])?;
                c.push(c.r[3])?;
                { let t = c.r[0]; c.push(0x10730889)?; c.call_indirect(t)?; }
                c.r[6] = c.rd32(c.r[5].wrapping_add(8u32))?;
                pc = 0x10730ce9; continue;
            }
            0x10730891 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                { let v = c.rd8(c.r[1].wrapping_add(9126u32))? as u32; c.set8(2, 0, v); }
                c.r[3] = c.r[4].wrapping_add(32u32);
                c.push(c.r[2])?;
                c.r[2] = c.r[7];
                c.push(0x107308a8)?; dispatch(c, 0x1072e270)?;
                pc = 0x1073086c; continue;
            }
            0x107308aa => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = c.rd8(c.r[0].wrapping_add(9126u32))? as u32; c.set8(1, 0, v); }
                c.r[3] = c.r[4].wrapping_add(32u32);
                c.r[2] = c.r[7];
                c.push(c.r[1])?;
                c.push(0x107308c1)?; dispatch(c, 0x1072e350)?;
                pc = 0x1073086c; continue;
            }
            0x107308c3 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                { let v = c.rd8(c.r[0].wrapping_add(9127u32))? as u32; c.set8(2, 0, v); }
                c.r[0] = c.rd8(c.r[0].wrapping_add(9126u32))? as u32;
                c.r[3] = c.r[4].wrapping_add(32u32);
                c.push(c.r[2])?;
                c.push(c.r[0])?;
                c.r[2] = c.r[7];
                c.push(0x107308e2)?; dispatch(c, 0x1072e070)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                pc = 0x1073086f; continue;
            }
            0x107308e7 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 1u32, 32);
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.push(0x107308f9)?; dispatch(c, 0x10740e60)?;
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[7])?);
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                c.r[3] = c.r[0];
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10730871; continue; }
            pc = 0x1073091b; continue;
            }
            0x1073091b => {
                pc = 0x10730920; continue;
            }
            0x10730920 => {
                c.r[0] = c.rd32(c.r[1])?;
                c.r[2] = c.r[0];
                c.r[6] = c.r[0];
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], 12u32, 32);
                c.r[6] = c.shift(ShiftOp::Shr, c.r[6], 8u32, 32);
                c.r[2] = c.alu(AluOp::And, c.r[2], 3840u32, 32);
                c.r[6] = c.alu(AluOp::And, c.r[6], 240u32, 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[6], 32);
                c.r[6] = c.r[0];
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 16u32, 32);
                c.r[6] = c.shift(ShiftOp::Shr, c.r[6], 4u32, 32);
                c.r[0] = c.alu(AluOp::And, c.r[0], 61440u32, 32);
                c.r[6] = c.alu(AluOp::And, c.r[6], 15u32, 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[6], 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[0], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                { let v = (c.r[2] & 0xffff); c.wr16(c.r[0], v as u16)?; }
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[7])?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 2u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.sf != c.of { pc = 0x10730920; continue; }
            pc = 0x10730972; continue;
            }
            0x10730972 => {
                pc = 0x10730871; continue;
            }
            0x10730977 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 1u32, 32);
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.push(0x10730989)?; dispatch(c, 0x10740e60)?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[3] = c.r[0];
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10730871; continue; }
            pc = 0x107309ab; continue;
            }
            0x107309ab => {
                pc = 0x107309b0; continue;
            }
            0x107309b0 => {
                c.r[0] = c.rd32(c.r[1])?;
                c.r[2] = c.r[0];
                c.r[6] = c.r[0];
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], 6u32, 32);
                c.r[6] = c.shift(ShiftOp::Shr, c.r[6], 3u32, 32);
                c.r[2] = c.alu(AluOp::And, c.r[2], 992u32, 32);
                c.r[6] = c.alu(AluOp::And, c.r[6], 31u32, 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[6], 32);
                c.r[6] = c.r[0];
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 9u32, 32);
                c.r[6] = c.shift(ShiftOp::Shr, c.r[6], 16u32, 32);
                c.r[0] = c.alu(AluOp::And, c.r[0], 31744u32, 32);
                c.r[6] = c.alu(AluOp::And, c.r[6], 32768u32, 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[6], 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[0], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                { let v = (c.r[2] & 0xffff); c.wr16(c.r[0], v as u16)?; }
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[7])?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 2u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.sf != c.of { pc = 0x107309b0; continue; }
            pc = 0x10730a02; continue;
            }
            0x10730a02 => {
                pc = 0x10730871; continue;
            }
            0x10730a07 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 1u32, 32);
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.push(0x10730a19)?; dispatch(c, 0x10740e60)?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[3] = c.r[0];
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10730871; continue; }
            pc = 0x10730a3b; continue;
            }
            0x10730a3b => {
                pc = 0x10730a40; continue;
            }
            0x10730a40 => {
                c.r[0] = c.rd32(c.r[1])?;
                c.r[2] = c.r[0];
                c.r[6] = c.r[0];
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], 6u32, 32);
                c.r[2] = c.alu(AluOp::And, c.r[2], 992u32, 32);
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 9u32, 32);
                c.r[6] = c.shift(ShiftOp::Shr, c.r[6], 3u32, 32);
                c.r[0] = c.alu(AluOp::And, c.r[0], 31744u32, 32);
                c.r[6] = c.alu(AluOp::And, c.r[6], 31u32, 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[6], 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[0], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                { let v = (c.r[2] & 0xffff); c.wr16(c.r[0], v as u16)?; }
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[7])?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 2u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.sf != c.of { pc = 0x10730a40; continue; }
            pc = 0x10730a85; continue;
            }
            0x10730a85 => {
                pc = 0x10730871; continue;
            }
            0x10730a8a => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 1u32, 32);
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.push(0x10730a9c)?; dispatch(c, 0x10740e60)?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[3] = c.r[0];
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10730871; continue; }
            pc = 0x10730abe; continue;
            }
            0x10730abe => {
                c.r[7] = c.r[7];
            pc = 0x10730ac0; continue;
            }
            0x10730ac0 => {
                c.r[0] = c.rd32(c.r[1])?;
                c.r[2] = c.r[0];
                c.r[6] = c.r[0];
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], 5u32, 32);
                c.r[2] = c.alu(AluOp::And, c.r[2], 2016u32, 32);
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 8u32, 32);
                c.r[6] = c.shift(ShiftOp::Shr, c.r[6], 3u32, 32);
                c.r[0] = c.alu(AluOp::And, c.r[0], 63488u32, 32);
                c.r[6] = c.alu(AluOp::And, c.r[6], 31u32, 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[6], 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[0], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                { let v = (c.r[2] & 0xffff); c.wr16(c.r[0], v as u16)?; }
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[7])?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 2u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.sf != c.of { pc = 0x10730ac0; continue; }
            pc = 0x10730b05; continue;
            }
            0x10730b05 => {
                pc = 0x10730871; continue;
            }
            0x10730b0a => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 2u32, 32);
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.push(0x10730b1d)?; dispatch(c, 0x10740e60)?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[1] = c.imul(c.r[1], c.rd32(c.r[7])?);
                c.r[6] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 2u32, 32);
                c.r[3] = c.r[0];
                c.r[0] = c.r[1];
                c.r[1] = c.shift(ShiftOp::Shr, c.r[1], 2u32, 32);
                c.r[7] = c.r[3];
                c.rep_movs(4)?;
                c.r[1] = c.r[0];
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.r[1] = c.alu(AluOp::And, c.r[1], 3u32, 32);
                c.rep_movs(1)?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(60u32))?;
                pc = 0x10730871; continue;
            }
            0x10730b47 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                c.r[0] = c.r[0].wrapping_add(c.r[0].wrapping_mul(2));
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.push(0x10730b5a)?; dispatch(c, 0x10740e60)?;
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[7])?);
                c.r[6] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                c.r[3] = c.r[0];
                if c.zf || c.sf != c.of { pc = 0x10730871; continue; }
            pc = 0x10730b72; continue;
            }
            0x10730b72 => {
                { let v = c.rd8(c.r[6])? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[0], v as u8)?; }
                { let v = c.rd8(c.r[6].wrapping_add(1u32))? as u32; c.set8(2, 0, v); }
                c.r[0] = c.incdec(c.r[0], 32, true);
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[0], v as u8)?; }
                { let v = c.rd8(c.r[6].wrapping_add(2u32))? as u32; c.set8(2, 0, v); }
                c.r[0] = c.incdec(c.r[0], 32, true);
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[0], v as u8)?; }
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[7])?);
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 4u32, 32);
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                if c.sf != c.of { pc = 0x10730b72; continue; }
            pc = 0x10730b91; continue;
            }
            0x10730b91 => {
                pc = 0x10730871; continue;
            }
            0x10730b96 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[1] = c.rd32(c.r[0].wrapping_add(4u32))?;
                c.r[6] = 3u32;
                c.alu(AluOp::Sub, c.rd16(c.r[1].wrapping_add(52u32))? as u32, (c.r[6] & 0xffff), 16);
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(164u32), v as u8)?; }
                c.r[1] = c.rd32(c.r[4].wrapping_add(164u32))?;
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                { let v = c.rd8(c.r[0].wrapping_add(9124u32))? as u32; c.set8(2, 0, v); }
                c.push(c.r[1])?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.r[4].wrapping_add(176u32);
                c.push(c.r[2])?;
                c.r[2] = c.rd32(c.r[7])?;
                c.push(4u32)?;
                c.push(8u32)?;
                c.push(c.r[0])?;
                c.r[0] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.push(4u32)?;
                c.push(c.r[1])?;
                c.push(c.r[2])?;
                c.push(c.r[0])?;
                c.push(0x10730bde)?; dispatch(c, 0x10736110)?;
                c.r[3] = c.r[0];
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.imul(c.r[0], c.rd32(c.r[7])?);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(68u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(80u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 36u32, 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if !c.zf { pc = 0x10730871; continue; }
            pc = 0x10730bf9; continue;
            }
            0x10730bf9 => {
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(184u32))?, c.r[6], 32);
                c.r[1] = c.rd32(c.r[4].wrapping_add(188u32))?;
                if !c.zf { pc = 0x10730c4f; continue; }
            pc = 0x10730c09; continue;
            }
            0x10730c09 => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf || c.sf != c.of { pc = 0x10730c8f; continue; }
            pc = 0x10730c13; continue;
            }
            0x10730c13 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(196u32))?;
                c.r[6] = c.alu(AluOp::Add, c.r[6], 4u32, 32);
                c.r[1] = c.r[1];
            pc = 0x10730c20; continue;
            }
            0x10730c20 => {
                { let v = c.rd8(c.r[6].wrapping_add(4u32))? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(1322u32), v as u8)?; }
                { let v = c.rd8(c.r[6])? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(1321u32), v as u8)?; }
                { let v = c.rd8(c.r[6].wrapping_add(4294967292u32))? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(1320u32), v as u8)?; }
                { let v = 255u32; c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(1323u32), v as u8)?; }
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 12u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                if c.sf != c.of { pc = 0x10730c20; continue; }
            pc = 0x10730c4d; continue;
            }
            0x10730c4d => {
                pc = 0x10730c8f; continue;
            }
            0x10730c4f => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf || c.sf != c.of { pc = 0x10730c8f; continue; }
            pc = 0x10730c55; continue;
            }
            0x10730c55 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(196u32))?;
                c.r[6] = c.alu(AluOp::Add, c.r[6], 8u32, 32);
            pc = 0x10730c60; continue;
            }
            0x10730c60 => {
                { let v = c.rd8(c.r[6].wrapping_add(4294967288u32))? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(1322u32), v as u8)?; }
                { let v = c.rd8(c.r[6].wrapping_add(4294967292u32))? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(1321u32), v as u8)?; }
                { let v = c.rd8(c.r[6])? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(1320u32), v as u8)?; }
                { let v = c.rd8(c.r[6].wrapping_add(4u32))? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(1323u32), v as u8)?; }
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 16u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                if c.sf != c.of { pc = 0x10730c60; continue; }
            pc = 0x10730c8f; continue;
            }
            0x10730c8f => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                c.push(1024u32)?;
                if c.zf { pc = 0x10730caf; continue; }
            pc = 0x10730c9b; continue;
            }
            0x10730c9b => {
                c.r[1] = c.r[4].wrapping_add(1324u32);
                c.push(0u32)?;
                c.push(c.r[1])?;
                { let t = c.r[0]; c.push(0x10730ca7)?; c.call_indirect(t)?; }
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
                pc = 0x10730871; continue;
            }
            0x10730caf => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[2] = c.r[4].wrapping_add(1324u32);
                c.push(c.r[2])?;
                c.push(c.r[0])?;
                c.push(0x10730cc0)?; dispatch(c, 0x1072e5d0)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
            pc = 0x10730cc3; continue;
            }
            0x10730cc3 => {
                { let v = c.rd8(c.r[4].wrapping_add(23u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                c.r[6] = c.rd32(c.r[5].wrapping_add(8u32))?;
                if c.zf { pc = 0x10730cdd; continue; }
            pc = 0x10730cce; continue;
            }
            0x10730cce => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[0] = c.r[3];
                c.r[2] = c.r[6];
                c.push(0x10730cdb)?; dispatch(c, 0x1072e430)?;
                pc = 0x10730cec; continue;
            }
            0x10730cdd => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.push(c.r[0])?;
                c.push(c.r[3])?;
                c.push(c.r[6])?;
                c.push(0x10730ce9)?; dispatch(c, 0x1072e5d0)?;
            pc = 0x10730ce9; continue;
            }
            0x10730ce9 => {
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
            pc = 0x10730cec; continue;
            }
            0x10730cec => {
                c.push(c.r[3])?;
                c.push(0x10730cf2)?; dispatch(c, 0x10740e54)?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(48u32))?;
                { let v = c.rd8(c.r[6].wrapping_add(9005u32))? as u32; c.set8(0, 0, v); }
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, ((c.r[0] >> 0) & 0xff), 1u32, 8);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                if !c.zf { pc = 0x10730d13; continue; }
            pc = 0x10730d08; continue;
            }
            0x10730d08 => {
                c.fpush(c.rdf32(c.r[6].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(9008u32))? as f64);
                { let v = c.st0(); c.wrf32(c.r[4].wrapping_add(56u32), v as f32)?; c.fpop(); }
            pc = 0x10730d13; continue;
            }
            0x10730d13 => {
                c.fpush(c.rdf32(276370776u32)? as f64);
                c.r[3] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.alu(AluOp::Sub, c.r[3], 1u32, 32);
                { let v = c.rdf32(c.r[4].wrapping_add(56u32))? as f64; c.farith_value(FOp::Sub, v); }
                { let v = c.st0(); c.wrf32(c.r[4].wrapping_add(24u32), v as f32)?; c.fpop(); }
                if !c.zf && c.sf == c.of { pc = 0x10730d3b; continue; }
            pc = 0x10730d2a; continue;
            }
            0x10730d2a => {
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(48u32))?, 1u32, 32);
                if !c.zf && c.sf == c.of { pc = 0x10730d3b; continue; }
            pc = 0x10730d31; continue;
            }
            0x10730d31 => {
                { let v = 1u32; c.wr8(c.r[4].wrapping_add(55u32), v as u8)?; }
                pc = 0x1073154f; continue;
            }
            0x10730d3b => {
                c.alu(AluOp::Sub, c.rd16(c.r[6].wrapping_add(9086u32))? as u32, 32u32, 16);
                if !c.zf { pc = 0x10730db2; continue; }
            pc = 0x10730d45; continue;
            }
            0x10730d45 => {
                c.push(c.r[7])?;
                c.push(0x10730d4b)?; dispatch(c, 0x10735c30)?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[0] = c.rd32(c.r[2].wrapping_add(4u32))?;
                c.r[1] = c.sext(c.rd16(c.r[0].wrapping_add(52u32))? as u32, 16);
                c.r[6] = c.rd32(c.r[4].wrapping_add(76u32))?;
                c.r[1] = c.imul(c.r[1], c.r[3]);
                c.r[0] = c.r[3].wrapping_add(1u32);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], 1u32, 32);
                c.r[7] = c.r[0];
                c.r[0] = c.rd32(c.r[4].wrapping_add(52u32))?;
                c.r[6] = c.alu(AluOp::Add, c.r[6], c.r[1], 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.r[2] = c.rd32(c.r[4].wrapping_add(68u32))?;
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], 1u32, 32);
                c.r[3] = c.r[0];
                c.push(c.r[2])?;
                c.push(c.r[3])?;
                c.push(c.r[7])?;
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(88u32), v)?; }
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(56u32), v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(64u32), v)?; }
                c.push(0x10730d8c)?; dispatch(c, 0x10735c00)?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(80u32))?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.push(c.r[1])?;
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.push(c.r[2])?;
                c.push(c.r[6])?;
                c.push(c.r[3])?;
                c.push(c.r[7])?;
                c.push(c.r[1])?;
                c.push(c.r[0])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(104u32), v)?; }
                c.push(0x10730da6)?; dispatch(c, 0x10735c60)?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(104u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 44u32, 32);
                pc = 0x10730ec5; continue;
            }
            0x10730db2 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.alu(AluOp::Sub, c.r[0], 2u32, 32);
                if c.zf || c.sf != c.of { pc = 0x10730df3; continue; }
            pc = 0x10730dba; continue;
            }
            0x10730dba => {
                c.alu(AluOp::Sub, c.rd32(c.r[7])?, 2u32, 32);
                if c.zf || c.sf != c.of { pc = 0x10730df3; continue; }
            pc = 0x10730dbf; continue;
            }
            0x10730dbf => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(284u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[0] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[1] = c.rd32(c.r[0].wrapping_add(4u32))?;
                c.r[1] = c.imul(c.r[1], c.rd32(c.r[0])?);
                c.r[7] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[6] = c.rd32(c.r[0].wrapping_add(8u32))?;
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 2u32, 32);
                c.r[2] = c.r[1];
                c.r[1] = c.shift(ShiftOp::Shr, c.r[1], 2u32, 32);
                c.rep_movs(4)?;
                c.r[1] = c.r[2];
                c.r[1] = c.alu(AluOp::And, c.r[1], 3u32, 32);
                c.rep_movs(1)?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(60u32))?;
                pc = 0x10730e93; continue;
            }
            0x10730df3 => {
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10730e93; continue; }
            pc = 0x10730e0a; continue;
            }
            0x10730e0a => {
                c.r[3] = c.r[3];
            pc = 0x10730e10; continue;
            }
            0x10730e10 => {
                c.r[0] = c.rd32(c.r[7])?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[2] = c.r[0];
                c.r[2] = c.imul(c.r[2], c.rd32(c.r[4].wrapping_add(16u32))?);
                c.r[3] = c.r[1].wrapping_add(c.r[2].wrapping_mul(4));
                c.alu(AluOp::Sub, c.rd32(c.r[7].wrapping_add(4u32))?, 1u32, 32);
                c.r[6] = c.r[3];
                if c.zf { pc = 0x10730e2a; continue; }
            pc = 0x10730e27; continue;
            }
            0x10730e27 => {
                c.r[6] = c.r[3].wrapping_add(c.r[0].wrapping_mul(4));
            pc = 0x10730e2a; continue;
            }
            0x10730e2a => {
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10730e7d; continue; }
            pc = 0x10730e36; continue;
            }
            0x10730e36 => {
                c.r[0] = c.rd32(c.r[7])?;
                c.alu(AluOp::Sub, c.r[0], 1u32, 32);
                c.r[1] = c.r[6];
                if c.zf { pc = 0x10731182; continue; }
            pc = 0x10730e43; continue;
            }
            0x10730e43 => {
                c.r[1] = c.r[6].wrapping_add(4u32);
                c.r[0] = c.r[3].wrapping_add(4u32);
            pc = 0x10730e49; continue;
            }
            0x10730e49 => {
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.push(c.r[1])?;
                c.push(c.r[6])?;
                c.push(c.r[0])?;
                c.push(c.r[3])?;
                c.push(c.r[2])?;
                c.push(0x10730e56)?; dispatch(c, 0x1072e8f0)?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(56u32))?;
                { let v = c.r[0]; c.wr32(c.r[1], v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(56u32), v)?; }
                c.r[1] = c.rd32(c.r[7])?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 2u32, 32);
                c.r[4] = c.alu(AluOp::Add, c.r[4], 20u32, 32);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 8u32, 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 8u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.sf != c.of { pc = 0x10730e36; continue; }
            pc = 0x10730e7d; continue;
            }
            0x10730e7d => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 2u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.sf != c.of { pc = 0x10730e10; continue; }
            pc = 0x10730e93; continue;
            }
            0x10730e93 => {
                c.r[0] = c.rd32(c.r[7])?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], 1u32, 32);
                { let v = c.r[0]; c.wr32(c.r[7], v)?; }
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], 1u32, 32);
                { let v = c.r[0]; c.wr32(c.r[7].wrapping_add(4u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], 1u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], 1u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
            pc = 0x10730ec5; continue;
            }
            0x10730ec5 => {
                c.r[6] = c.rd32(c.r[5].wrapping_add(8u32))?;
                { let v = c.rd8(c.r[6].wrapping_add(9095u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10730f34; continue; }
            pc = 0x10730ed2; continue;
            }
            0x10730ed2 => {
                c.r[1] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf || c.sf != c.of { pc = 0x10730f08; continue; }
            pc = 0x10730edb; continue;
            }
            0x10730edb => {
                c.r[1] = 4u32;
            pc = 0x10730ee0; continue;
            }
            0x10730ee0 => {
                c.r[2] = c.rd32(c.r[7])?;
                c.r[3] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[2] = c.imul(c.r[2], c.r[0]);
                { let v = 0u32; c.wr8(c.r[3].wrapping_add(c.r[2].wrapping_mul(4)).wrapping_add(3u32), v as u8)?; }
                c.r[2] = c.rd32(c.r[7])?;
                c.r[3] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[2] = c.imul(c.r[2], c.r[1]);
                c.r[2] = c.r[2].wrapping_add(c.r[3]).wrapping_add(4294967292u32);
                { let v = 0u32; c.wr8(c.r[2].wrapping_add(3u32), v as u8)?; }
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                if c.sf != c.of { pc = 0x10730ee0; continue; }
            pc = 0x10730f08; continue;
            }
            0x10730f08 => {
                c.r[1] = c.rd32(c.r[7])?;
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf || c.sf != c.of { pc = 0x10730f34; continue; }
            pc = 0x10730f10; continue;
            }
            0x10730f10 => {
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                { let v = 0u32; c.wr8(c.r[1].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(3u32), v as u8)?; }
                c.r[1] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.rd32(c.r[7])?;
                c.r[1] = c.incdec(c.r[1], 32, false);
                c.r[2] = c.imul(c.r[2], c.r[1]);
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[2] = c.alu(AluOp::Add, c.r[2], c.r[0], 32);
                c.r[1] = c.r[1].wrapping_add(c.r[2].wrapping_mul(4));
                { let v = 0u32; c.wr8(c.r[1].wrapping_add(3u32), v as u8)?; }
                c.r[1] = c.rd32(c.r[7])?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                if c.sf != c.of { pc = 0x10730f10; continue; }
            pc = 0x10730f34; continue;
            }
            0x10730f34 => {
                { let v = c.rd8(c.r[6].wrapping_add(9096u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10730feb; continue; }
            pc = 0x10730f42; continue;
            }
            0x10730f42 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 9100u32, 32);
                c.r[1] = c.r[1];
            pc = 0x10730f50; continue;
            }
            0x10730f50 => {
                c.r[6] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10730fa1; continue; }
            pc = 0x10730f5d; continue;
            }
            0x10730f5d => {
                { let v = 4u32; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
            pc = 0x10730f65; continue;
            }
            0x10730f65 => {
                c.r[6] = c.rd32(c.r[7])?;
                { let v = c.rd8(c.r[1])? as u32; c.set8(3, 0, v); }
                c.r[6] = c.imul(c.r[6], c.r[2]);
                c.r[2] = c.r[0].wrapping_add(c.r[6].wrapping_mul(4));
                c.r[6] = c.rd32(c.r[7].wrapping_add(8u32))?;
                { let v = ((c.r[3] >> 0) & 0xff); c.wr8(c.r[2].wrapping_add(c.r[6]), v as u8)?; }
                c.r[3] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[2] = c.rd32(c.r[7])?;
                c.r[6] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[2] = c.imul(c.r[2], c.r[3]);
                c.r[6] = c.r[2].wrapping_add(c.r[6]).wrapping_add(4294967292u32);
                { let v = c.rd8(c.r[1])? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[6].wrapping_add(c.r[0]), v as u8)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[6] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 4u32, 32);
                c.alu(AluOp::Sub, c.r[2], c.r[6], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                if c.sf != c.of { pc = 0x10730f65; continue; }
            pc = 0x10730fa1; continue;
            }
            0x10730fa1 => {
                c.r[2] = c.rd32(c.r[7])?;
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10730fe0; continue; }
            pc = 0x10730fad; continue;
            }
            0x10730fad => {
                c.r[1] = c.r[1];
            pc = 0x10730fb0; continue;
            }
            0x10730fb0 => {
                { let v = c.rd8(c.r[1])? as u32; c.set8(3, 0, v); }
                c.r[2] = c.r[0].wrapping_add(c.r[6].wrapping_mul(4));
                c.r[6] = c.rd32(c.r[7].wrapping_add(8u32))?;
                { let v = ((c.r[3] >> 0) & 0xff); c.wr8(c.r[2].wrapping_add(c.r[6]), v as u8)?; }
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[3] = c.rd32(c.r[7])?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[2] = c.incdec(c.r[2], 32, false);
                c.r[3] = c.imul(c.r[3], c.r[2]);
                c.r[2] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[3] = c.alu(AluOp::Add, c.r[3], c.r[6], 32);
                c.r[2] = c.r[2].wrapping_add(c.r[3].wrapping_mul(4));
                { let v = c.rd8(c.r[1])? as u32; c.set8(3, 0, v); }
                { let v = ((c.r[3] >> 0) & 0xff); c.wr8(c.r[2].wrapping_add(c.r[0]), v as u8)?; }
                c.r[2] = c.rd32(c.r[7])?;
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.alu(AluOp::Sub, c.r[6], c.r[2], 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.sf != c.of { pc = 0x10730fb0; continue; }
            pc = 0x10730fe0; continue;
            }
            0x10730fe0 => {
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[0], 2u32, 32);
                if c.zf || c.sf != c.of { pc = 0x10730f50; continue; }
            pc = 0x10730feb; continue;
            }
            0x10730feb => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.alu(AluOp::Sub, c.r[1], c.rd32(c.r[0].wrapping_add(9116u32))?, 32);
                if c.sf != c.of { pc = 0x107313fa; continue; }
            pc = 0x10730ffe; continue;
            }
            0x10730ffe => {
                { let v = c.rd8(c.r[0].wrapping_add(9104u32))? as u32; c.set8(1, 0, v); }
                c.alu(AluOp::And, ((c.r[1] >> 0) & 0xff), ((c.r[1] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10731372; continue; }
            pc = 0x1073100c; continue;
            }
            0x1073100c => {
                { let v = c.rd8(c.r[0].wrapping_add(9093u32))? as u32; c.set8(1, 0, v); }
                c.alu(AluOp::And, ((c.r[1] >> 0) & 0xff), ((c.r[1] >> 0) & 0xff), 8);
                if c.zf { pc = 0x107311d0; continue; }
            pc = 0x1073101a; continue;
            }
            0x1073101a => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10731372; continue; }
            pc = 0x1073102b; continue;
            }
            0x1073102b => {
                c.r[0] = c.rd32(c.r[7])?;
                c.r[1] = c.r[1];
            pc = 0x10731030; continue;
            }
            0x10731030 => {
                c.r[6] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[2] = c.r[1];
                c.r[2] = c.imul(c.r[2], c.r[0]);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                c.r[3] = c.r[6].wrapping_add(c.r[2].wrapping_mul(4));
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x107311bb; continue; }
            pc = 0x1073104b; continue;
            }
            0x1073104b => {
                c.fpush(c.rdf32(c.r[4].wrapping_add(56u32))? as f64);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(208u32), v)?; c.fpop(); }
                c.fpush(c.rdf32(c.r[4].wrapping_add(24u32))? as f64);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(216u32), v)?; c.fpop(); }
            pc = 0x10731061; continue;
            }
            0x10731061 => {
                c.r[2] = c.rd8(c.r[3].wrapping_add(2u32))? as u32;
                c.r[0] = c.rd32(c.r[3])?;
                c.r[1] = c.rd8(c.r[3].wrapping_add(3u32))? as u32;
                c.r[6] = c.rd8(c.r[3].wrapping_add(1u32))? as u32;
                c.r[0] = c.alu(AluOp::And, c.r[0], 255u32, 32);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(16u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731086; continue; }
            pc = 0x10731080; continue;
            }
            0x10731080 => {
                { let v = c.rdf32(276669560u32)? as f64; c.farith_value(FOp::Add, v); }
            pc = 0x10731086; continue;
            }
            0x10731086 => {
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                { let v = c.rdf64(276956120u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rd32(c.r[4].wrapping_add(16u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x107310a4; continue; }
            pc = 0x1073109e; continue;
            }
            0x1073109e => {
                { let v = c.rdf32(276669560u32)? as f64; c.farith_value(FOp::Add, v); }
            pc = 0x107310a4; continue;
            }
            0x107310a4 => {
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.rdf64(276956120u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rd32(c.r[4].wrapping_add(16u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x107310c2; continue; }
            pc = 0x107310bc; continue;
            }
            0x107310bc => {
                { let v = c.rdf32(276669560u32)? as f64; c.farith_value(FOp::Add, v); }
            pc = 0x107310c2; continue;
            }
            0x107310c2 => {
                { let v = c.rdf64(276956120u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Sub, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(144u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(208u32))?);
                c.farith(FOp::Mul, 0, 2, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(128u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(208u32))?);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(136u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[4].wrapping_add(216u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(144u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(144u32), v)?;  }
                { let v = c.rdf64(c.r[4].wrapping_add(144u32))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(136u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(136u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                c.fpush(c.rdf64(c.r[4].wrapping_add(128u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(128u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(0);
                c.fabs();
                c.fcom_value(c.rdf64(276956104u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.zf { pc = 0x10731189; continue; }
            pc = 0x1073114a; continue;
            }
            0x1073114a => {
                c.fsqrt();
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Divr, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(128u32))?);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(128u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(136u32))?);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(136u32), v)?; c.fpop(); }
                { let v = c.rdf64(c.r[4].wrapping_add(144u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(144u32), v)?; c.fpop(); }
                pc = 0x1073118b; continue;
            }
            0x10731182 => {
                c.r[0] = c.r[3];
                pc = 0x10730e49; continue;
            }
            0x10731189 => {
                c.fst_st(0, true);
            pc = 0x1073118b; continue;
            }
            0x1073118b => {
                c.push(c.r[3])?;
                c.r[0] = c.r[4].wrapping_add(132u32);
                c.push(c.r[0])?;
                c.r[1] = c.r[4].wrapping_add(24u32);
                c.push(c.r[1])?;
                c.push(0x1073119e)?; dispatch(c, 0x1072e4c0)?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[0] = c.rd32(c.r[7])?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 4u32, 32);
                c.alu(AluOp::Sub, c.r[1], c.r[0], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.sf != c.of { pc = 0x10731061; continue; }
            pc = 0x107311b7; continue;
            }
            0x107311b7 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(36u32))?;
            pc = 0x107311bb; continue;
            }
            0x107311bb => {
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                if c.sf != c.of { pc = 0x10731030; continue; }
            pc = 0x107311cb; continue;
            }
            0x107311cb => {
                pc = 0x10731372; continue;
            }
            0x107311d0 => {
                { let v = c.rd8(c.r[0].wrapping_add(9094u32))? as u32; c.set8(1, 0, v); }
                c.alu(AluOp::And, ((c.r[1] >> 0) & 0xff), ((c.r[1] >> 0) & 0xff), 8);
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                if c.zf { pc = 0x10731280; continue; }
            pc = 0x107311e1; continue;
            }
            0x107311e1 => {
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10731372; continue; }
            pc = 0x107311ef; continue;
            }
            0x107311ef => {
                c.r[0] = c.rd32(c.r[7])?;
            pc = 0x107311f1; continue;
            }
            0x107311f1 => {
                c.r[6] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[2] = c.r[1];
                c.r[2] = c.imul(c.r[2], c.r[0]);
                c.r[3] = c.r[6].wrapping_add(c.r[2].wrapping_mul(4));
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf || c.sf != c.of { pc = 0x1073126b; continue; }
            pc = 0x10731202; continue;
            }
            0x10731202 => {
                c.r[3] = c.incdec(c.r[3], 32, true);
            pc = 0x10731203; continue;
            }
            0x10731203 => {
                c.r[0] = c.rd8(c.r[3].wrapping_add(1u32))? as u32;
                c.r[1] = c.rd8(c.r[3])? as u32;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(36u32))? as i32 as f64);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.rdf32(276956292u32)? as f64; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf32(c.r[4].wrapping_add(56u32))? as f64; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rd32(c.r[4].wrapping_add(36u32))? as i32 as f64);
                { let v = c.rdf32(276956292u32)? as f64; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf32(c.r[4].wrapping_add(56u32))? as f64; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf32(c.r[4].wrapping_add(36u32), v as f32)?; c.fpop(); }
                { let v = c.rdf32(276677448u32)? as f64; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Add, v); }
                c.push(0x10731243)?; dispatch(c, 0x10740cb4)?;
                c.fpush(c.rdf32(c.r[4].wrapping_add(36u32))? as f64);
                { let v = c.rdf32(276677448u32)? as f64; c.farith_value(FOp::Mul, v); }
                { let v = ((c.r[0] >> 0) & 0xff); c.wr8(c.r[3].wrapping_add(1u32), v as u8)?; }
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Add, v); }
                c.push(0x1073125b)?; dispatch(c, 0x10740cb4)?;
                { let v = ((c.r[0] >> 0) & 0xff); c.wr8(c.r[3], v as u8)?; }
                c.r[0] = c.rd32(c.r[7])?;
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 4u32, 32);
                c.alu(AluOp::Sub, c.r[6], c.r[0], 32);
                if c.sf != c.of { pc = 0x10731203; continue; }
            pc = 0x10731267; continue;
            }
            0x10731267 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(28u32))?;
            pc = 0x1073126b; continue;
            }
            0x1073126b => {
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.sf != c.of { pc = 0x107311f1; continue; }
            pc = 0x1073127b; continue;
            }
            0x1073127b => {
                pc = 0x10731372; continue;
            }
            0x10731280 => {
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10731372; continue; }
            pc = 0x10731290; continue;
            }
            0x10731290 => {
                c.r[0] = c.rd32(c.r[7])?;
            pc = 0x10731292; continue;
            }
            0x10731292 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[1] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[2] = c.imul(c.r[2], c.r[0]);
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                c.r[3] = c.r[1].wrapping_add(c.r[2].wrapping_mul(4));
                if c.zf || c.sf != c.of { pc = 0x1073135e; continue; }
            pc = 0x107312a9; continue;
            }
            0x107312a9 => {
                c.r[4] = c.r[4];
            pc = 0x107312b0; continue;
            }
            0x107312b0 => {
                c.r[2] = c.rd8(c.r[3])? as u32;
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[1] = c.rd8(c.r[0].wrapping_add(9108u32))? as u32;
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                { let v = c.rdf32(c.r[4].wrapping_add(56u32))? as f64; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.rdf32(c.r[4].wrapping_add(24u32))? as f64; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Add, v); }
                c.push(0x107312e2)?; dispatch(c, 0x10740cb4)?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[1] = c.rd8(c.r[3].wrapping_add(1u32))? as u32;
                { let v = ((c.r[0] >> 0) & 0xff); c.wr8(c.r[3], v as u8)?; }
                c.r[0] = c.rd8(c.r[2].wrapping_add(9109u32))? as u32;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                { let v = c.rdf32(c.r[4].wrapping_add(24u32))? as f64; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.rdf32(c.r[4].wrapping_add(56u32))? as f64; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Add, v); }
                c.push(0x10731317)?; dispatch(c, 0x10740cb4)?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[1] = c.rd8(c.r[3].wrapping_add(2u32))? as u32;
                { let v = ((c.r[0] >> 0) & 0xff); c.wr8(c.r[3].wrapping_add(1u32), v as u8)?; }
                c.r[0] = c.rd8(c.r[2].wrapping_add(9110u32))? as u32;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                { let v = c.rdf32(c.r[4].wrapping_add(24u32))? as f64; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.rdf32(c.r[4].wrapping_add(56u32))? as f64; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Add, v); }
                c.push(0x1073134d)?; dispatch(c, 0x10740cb4)?;
                { let v = ((c.r[0] >> 0) & 0xff); c.wr8(c.r[3].wrapping_add(2u32), v as u8)?; }
                c.r[0] = c.rd32(c.r[7])?;
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 4u32, 32);
                c.alu(AluOp::Sub, c.r[6], c.r[0], 32);
                if c.sf != c.of { pc = 0x107312b0; continue; }
            pc = 0x1073135e; continue;
            }
            0x1073135e => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.sf != c.of { pc = 0x10731292; continue; }
            pc = 0x10731372; continue;
            }
            0x10731372 => {
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                { let v = c.rd8(c.r[2].wrapping_add(9105u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x107313eb; continue; }
            pc = 0x1073137f; continue;
            }
            0x1073137f => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x107313eb; continue; }
            pc = 0x1073138e; continue;
            }
            0x1073138e => {
                c.r[0] = c.rd32(c.r[7])?;
            pc = 0x10731390; continue;
            }
            0x10731390 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[1] = c.imul(c.r[1], c.r[0]);
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                c.r[3] = c.r[2].wrapping_add(c.r[1].wrapping_mul(4));
                if c.zf || c.sf != c.of { pc = 0x107313db; continue; }
            pc = 0x107313a3; continue;
            }
            0x107313a3 => {
                c.r[3] = c.alu(AluOp::Add, c.r[3], 3u32, 32);
            pc = 0x107313a6; continue;
            }
            0x107313a6 => {
                c.r[0] = c.rd8(c.r[3])? as u32;
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.rdf32(c.r[4].wrapping_add(56u32))? as f64; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rd32(c.r[1].wrapping_add(9112u32))? as i32 as f64);
                { let v = c.rdf32(c.r[4].wrapping_add(24u32))? as f64; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Add, v); }
                c.push(0x107313cf)?; dispatch(c, 0x10740cb4)?;
                { let v = ((c.r[0] >> 0) & 0xff); c.wr8(c.r[3], v as u8)?; }
                c.r[0] = c.rd32(c.r[7])?;
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 4u32, 32);
                c.alu(AluOp::Sub, c.r[6], c.r[0], 32);
                if c.sf != c.of { pc = 0x107313a6; continue; }
            pc = 0x107313db; continue;
            }
            0x107313db => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.sf != c.of { pc = 0x10731390; continue; }
            pc = 0x107313eb; continue;
            }
            0x107313eb => {
                c.fpush(c.rdf32(c.r[4].wrapping_add(56u32))? as f64);
                { let v = c.rdf32(c.r[4].wrapping_add(156u32))? as f64; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf32(c.r[4].wrapping_add(56u32), v as f32)?; c.fpop(); }
            pc = 0x107313fa; continue;
            }
            0x107313fa => {
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                { let v = c.rd8(c.r[2].wrapping_add(9093u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x1073154f; continue; }
            pc = 0x1073140b; continue;
            }
            0x1073140b => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073154f; continue; }
            pc = 0x1073141e; continue;
            }
            0x1073141e => {
                c.r[0] = c.rd32(c.r[7])?;
            pc = 0x10731420; continue;
            }
            0x10731420 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.rd32(c.r[7].wrapping_add(8u32))?;
                c.r[1] = c.imul(c.r[1], c.r[0]);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                c.r[3] = c.r[2].wrapping_add(c.r[1].wrapping_mul(4));
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073153b; continue; }
            pc = 0x1073143d; continue;
            }
            0x1073143d => {
                c.r[1] = c.r[1];
            pc = 0x10731440; continue;
            }
            0x10731440 => {
                c.r[2] = c.rd8(c.r[3].wrapping_add(2u32))? as u32;
                c.r[0] = c.rd32(c.r[3])?;
                c.r[1] = c.rd8(c.r[3].wrapping_add(3u32))? as u32;
                c.r[6] = c.rd8(c.r[3].wrapping_add(1u32))? as u32;
                c.r[0] = c.alu(AluOp::And, c.r[0], 255u32, 32);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(36u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731465; continue; }
            pc = 0x1073145f; continue;
            }
            0x1073145f => {
                { let v = c.rdf32(276669560u32)? as f64; c.farith_value(FOp::Add, v); }
            pc = 0x10731465; continue;
            }
            0x10731465 => {
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                { let v = c.rdf64(276956120u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Sub, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(88u32), v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[4].wrapping_add(36u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731487; continue; }
            pc = 0x10731481; continue;
            }
            0x10731481 => {
                { let v = c.rdf32(276669560u32)? as f64; c.farith_value(FOp::Add, v); }
            pc = 0x10731487; continue;
            }
            0x10731487 => {
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.rdf64(276956120u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Sub, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(96u32), v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[4].wrapping_add(36u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x107314a9; continue; }
            pc = 0x107314a3; continue;
            }
            0x107314a3 => {
                { let v = c.rdf32(276669560u32)? as f64; c.farith_value(FOp::Add, v); }
            pc = 0x107314a9; continue;
            }
            0x107314a9 => {
                { let v = c.rdf64(276956120u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Sub, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(104u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(88u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(88u32))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(104u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(104u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                c.fpush(c.rdf64(c.r[4].wrapping_add(96u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(96u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(0);
                c.fabs();
                c.fcom_value(c.rdf64(276956104u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.zf { pc = 0x10731510; continue; }
            pc = 0x107314ea; continue;
            }
            0x107314ea => {
                c.fsqrt();
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Divr, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(88u32))?);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(88u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(96u32))?);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(96u32), v)?; c.fpop(); }
                { let v = c.rdf64(c.r[4].wrapping_add(104u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(104u32), v)?; c.fpop(); }
                pc = 0x10731512; continue;
            }
            0x10731510 => {
                c.fst_st(0, true);
            pc = 0x10731512; continue;
            }
            0x10731512 => {
                c.push(c.r[3])?;
                c.r[0] = c.r[4].wrapping_add(92u32);
                c.push(c.r[0])?;
                c.r[1] = c.r[4].wrapping_add(44u32);
                c.push(c.r[1])?;
                c.push(0x10731522)?; dispatch(c, 0x1072e4c0)?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[0] = c.rd32(c.r[7])?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 4u32, 32);
                c.alu(AluOp::Sub, c.r[1], c.r[0], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.sf != c.of { pc = 0x10731440; continue; }
            pc = 0x1073153b; continue;
            }
            0x1073153b => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.sf != c.of { pc = 0x10731420; continue; }
            pc = 0x1073154f; continue;
            }
            0x1073154f => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.alu(AluOp::Sub, c.rd16(c.r[0].wrapping_add(9086u32))? as u32, 31u32, 16);
                if !c.zf { pc = 0x10731569; continue; }
            pc = 0x1073155c; continue;
            }
            0x1073155c => {
                c.r[2] = c.sext(c.rd16(c.r[0].wrapping_add(9088u32))? as u32, 16);
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(44u32))?, c.r[2], 32);
                if c.sf == c.of { pc = 0x10731575; continue; }
            pc = 0x10731569; continue;
            }
            0x10731569 => {
                { let v = c.rd8(c.r[4].wrapping_add(55u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x107307e2; continue; }
            pc = 0x10731575; continue;
            }
            0x10731575 => {
                c.r[1] = c.r[4].wrapping_add(224u32);
                { let v = 1u32; c.wr8(c.r[4].wrapping_add(2356u32), v as u8)?; }
                c.push(0x10731589)?; dispatch(c, 0x107353a0)?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(76u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(68u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(60u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(76u32), v)?; }
                if c.sf != c.of { pc = 0x10730680; continue; }
            pc = 0x107315a2; continue;
            }
            0x107315a2 => {
                c.push(c.r[7])?;
                c.push(0x107315a8)?; dispatch(c, 0x10735c30)?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(200u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x107315bf; continue; }
            pc = 0x107315b6; continue;
            }
            0x107315b6 => {
                c.push(c.r[0])?;
                c.push(0x107315bc)?; dispatch(c, 0x10306203)?;
            pc = 0x107315bc; continue;
            }
            0x107315bc => {
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
            pc = 0x107315bf; continue;
            }
            0x107315bf => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(2348u32))?;
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                c.r[0] = 1u32;
                c.fs0 = c.r[1];
                { let v = c.pop()?; c.r[3] = v; }
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10731690(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10731690;
    loop {
        match pc {
            0x10731690 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(8u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(4u32))?;
                c.r[2] = c.rd32(c.r[1].wrapping_add(12u32))?;
                c.r[1] = c.rd32(c.r[1].wrapping_add(8u32))?;
                c.r[0] = c.alu(AluOp::And, c.r[0], 65280u32, 32);
                c.r[0] = c.neg(c.r[0], 32);
                c.r[0] = c.alu(AluOp::Sbb, c.r[0], c.r[0], 32);
                c.r[0] = c.alu(AluOp::And, c.r[0], 8u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 3u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], 2u32, 32);
                c.r[0] = c.imul(c.r[0], c.r[2]);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 3u32, 32);
                c.r[1] = c.shift(ShiftOp::Shr, c.r[1], 2u32, 32);
                c.r[0] = c.imul(c.r[0], c.r[1]);
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10731710(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10731710;
    loop {
        match pc {
            0x10731710 => {
                c.push(c.r[5])?;
                c.r[5] = c.r[4];
                c.r[4] = c.alu(AluOp::And, c.r[4], 4294967288u32, 32);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 836u32, 32);
                c.push(c.r[3])?;
                c.r[3] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[0] = c.rd32(c.r[3].wrapping_add(12u32))?;
                c.r[1] = 4u32;
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                c.push(c.r[6])?;
                c.push(c.r[7])?;
                if !c.cf { pc = 0x10731731; continue; }
            pc = 0x1073172e; continue;
            }
            0x1073172e => {
                { let v = c.r[1]; c.wr32(c.r[3].wrapping_add(12u32), v)?; }
            pc = 0x10731731; continue;
            }
            0x10731731 => {
                c.alu(AluOp::Sub, c.rd32(c.r[3].wrapping_add(8u32))?, c.r[1], 32);
                if !c.cf { pc = 0x10731739; continue; }
            pc = 0x10731736; continue;
            }
            0x10731736 => {
                { let v = c.r[1]; c.wr32(c.r[3].wrapping_add(8u32), v)?; }
            pc = 0x10731739; continue;
            }
            0x10731739 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[7] = c.rd32(c.r[5].wrapping_add(20u32))?;
                { let v = 530438u32; c.wr32(c.r[0].wrapping_add(4u32), v)?; }
                c.r[2] = c.rd32(c.r[3].wrapping_add(12u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(12u32), v)?; }
                c.r[2] = c.rd32(c.r[3].wrapping_add(8u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(8u32), v)?; }
                c.r[2] = c.rd32(c.r[5].wrapping_add(32u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(16u32), v)?; }
                { let v = c.r[7]; c.wr32(c.r[0].wrapping_add(36u32), v)?; }
                { let v = 32u32; c.wr32(c.r[0].wrapping_add(72u32), v)?; }
                c.r[2] = c.rd32(c.r[3].wrapping_add(76u32))?;
                c.r[2] = c.alu(AluOp::And, c.r[2], 1u32, 32);
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[1], 32);
                c.r[1] = c.rd32(c.r[5].wrapping_add(24u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(76u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[0].wrapping_add(84u32), v)?; }
                { let v = 305419896u32; c.wr32(c.r[0].wrapping_add(80u32), v)?; }
                c.r[0] = c.rd32(c.r[3].wrapping_add(84u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 7u32, 32);
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 3u32, 32);
                c.r[6] = c.r[3].wrapping_add(88u32);
                c.r[2] = c.r[4].wrapping_add(64u32);
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                c.r[6] = c.alu(AluOp::Sub, c.r[6], c.r[2], 32);
            pc = 0x10731792; continue;
            }
            0x10731792 => {
                c.r[0] = c.r[6].wrapping_add(c.r[1].wrapping_mul(4));
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[0]).wrapping_add(64u32))?;
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), 1u32, 8);
                if !c.zf { pc = 0x107317a7; continue; }
            pc = 0x1073179f; continue;
            }
            0x1073179f => {
            pc = 0x107317a0; continue;
            }
            0x107317a0 => {
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 1u32, 32);
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), 1u32, 8);
                if c.zf { pc = 0x107317a0; continue; }
            pc = 0x107317a7; continue;
            }
            0x107317a7 => {
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(64u32), v)?; }
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], 3u32, 32);
                if c.sf != c.of { pc = 0x10731792; continue; }
            pc = 0x107317b1; continue;
            }
            0x107317b1 => {
                c.r[2] = c.rd32(c.r[3].wrapping_add(100u32))?;
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                if c.zf { pc = 0x107317e8; continue; }
            pc = 0x107317b8; continue;
            }
            0x107317b8 => {
                c.r[0] = c.r[2];
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), 1u32, 8);
                if !c.zf { pc = 0x107317c7; continue; }
            pc = 0x107317c0; continue;
            }
            0x107317c0 => {
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 1u32, 32);
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), 1u32, 8);
                if c.zf { pc = 0x107317c0; continue; }
            pc = 0x107317c7; continue;
            }
            0x107317c7 => {
                c.r[1] = c.r[6];
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], ((c.r[1] >> 0) & 0xff), 32);
                c.r[0] = 2155905153u32;
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(80u32), v)?; }
                c.r[2] = c.imul(c.r[2], c.rd32(278723184u32)?);
                { let (lo, hi) = c.mul_wide(c.r[0], c.r[2]); c.r[0] = lo; c.r[2] = hi; }
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], 7u32, 32);
                c.r[0] = c.r[2];
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(108u32), v)?; }
            pc = 0x107317e8; continue;
            }
            0x107317e8 => {
                c.r[0] = c.rd32(c.r[3].wrapping_add(8u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                c.r[1] = c.rd32(c.r[3].wrapping_add(36u32))?;
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(84u32), v)?; }
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(96u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x107325c0; continue; }
            pc = 0x10731806; continue;
            }
            0x10731806 => {
                c.r[0] = c.rd32(c.r[3].wrapping_add(8u32))?;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.rd32(c.r[4].wrapping_add(96u32))?, 32);
                c.alu(AluOp::Sub, c.r[0], 4u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                if c.cf { pc = 0x1073181e; continue; }
            pc = 0x10731816; continue;
            }
            0x10731816 => {
                { let v = 4u32; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
            pc = 0x1073181e; continue;
            }
            0x1073181e => {
                c.r[0] = c.rd32(c.r[3].wrapping_add(12u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                c.r[2] = c.rd32(c.r[4].wrapping_add(84u32))?;
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(60u32), v)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(88u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073259c; continue; }
            pc = 0x10731839; continue;
            }
            0x10731839 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(24u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[0] = c.r[1];
                c.r[0] = c.alu(AluOp::And, c.r[0], 255u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(92u32), v)?; }
                c.r[0] = c.r[1];
                c.r[2] = c.shift(ShiftOp::Shl, c.r[2], 2u32, 32);
                c.r[0] = c.alu(AluOp::And, c.r[0], 65280u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(100u32), v)?; }
                c.r[4] = c.r[4];
            pc = 0x10731860; continue;
            }
            0x10731860 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(28u32))?;
                c.fpush(c.rdf32(c.r[0])? as f64);
                c.r[6] = c.rd32(c.r[4].wrapping_add(88u32))?;
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(720u32), v)?; c.fpop(); }
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                c.fpush(c.rdf32(c.r[0].wrapping_add(4u32))? as f64);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(104u32), v)?; }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(728u32), v)?; c.fpop(); }
                c.fpush(c.rdf32(c.r[0].wrapping_add(8u32))? as f64);
                c.r[0] = c.rd32(c.r[3].wrapping_add(12u32))?;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[6], 32);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(736u32), v)?; c.fpop(); }
                c.alu(AluOp::Sub, c.r[0], 4u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
                if c.cf { pc = 0x107318a0; continue; }
            pc = 0x10731898; continue;
            }
            0x10731898 => {
                { let v = 4u32; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
            pc = 0x107318a0; continue;
            }
            0x107318a0 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::Sub, c.r[1], c.r[7], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(328u32), v)?; }
                if c.zf { pc = 0x10731a83; continue; }
            pc = 0x107318b4; continue;
            }
            0x107318b4 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(92u32))?;
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[7], 32);
                c.r[6] = c.rd32(c.r[4].wrapping_add(60u32))?;
                if c.zf { pc = 0x107319bd; continue; }
            pc = 0x107318c4; continue;
            }
            0x107318c4 => {
                c.r[1] = c.incdec(c.r[1], 32, false);
                if !c.zf { pc = 0x10731eb3; continue; }
            pc = 0x107318cb; continue;
            }
            0x107318cb => {
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                pc = 0x107318e0; continue;
            }
            0x107318e0 => {
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                pc = 0x107318f0; continue;
            }
            0x107318f0 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(52u32))?;
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(16u32))?, c.r[1], 32);
                if c.sf == c.of { pc = 0x10731983; continue; }
            pc = 0x107318fe; continue;
            }
            0x107318fe => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(20u32))?, c.r[2], 32);
                if c.sf == c.of { pc = 0x10731983; continue; }
            pc = 0x10731908; continue;
            }
            0x10731908 => {
                c.r[1] = c.rd8(c.r[6])? as u32;
                c.r[2] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[1] = c.r[2].wrapping_add(c.r[1].wrapping_mul(4));
                c.r[2] = c.rd32(c.r[1])?;
                c.r[2] = c.alu(AluOp::And, c.r[2], 16777215u32, 32);
                c.alu(AluOp::Sub, c.r[2], c.rd32(c.r[3].wrapping_add(64u32))?, 32);
                if c.cf { pc = 0x10731923; continue; }
            pc = 0x1073191e; continue;
            }
            0x1073191e => {
                c.alu(AluOp::Sub, c.r[2], c.rd32(c.r[3].wrapping_add(68u32))?, 32);
                if c.cf || c.zf { pc = 0x10731983; continue; }
            pc = 0x10731923; continue;
            }
            0x10731923 => {
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(c.r[7].wrapping_mul(4)).wrapping_add(112u32), v)?; }
                c.r[2] = c.r[0].wrapping_add(c.r[0].wrapping_mul(2));
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.r[0] = c.incdec(c.r[0], 32, true);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(328u32), v)?; }
                c.r[0] = c.rd8(c.r[1])? as u32;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.r[2] = c.r[4].wrapping_add(c.r[2].wrapping_mul(8)).wrapping_add(336u32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 8u32, 32);
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[2].wrapping_add(4294967288u32), v)?; c.fpop(); }
                c.r[0] = c.rd8(c.r[1].wrapping_add(1u32))? as u32;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[2], v)?; c.fpop(); }
                c.r[1] = c.rd8(c.r[1].wrapping_add(2u32))? as u32;
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[2].wrapping_add(8u32), v)?; c.fpop(); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(328u32))?;
                pc = 0x1073198c; continue;
            }
            0x10731983 => {
                { let v = 4294967295u32; c.wr32(c.r[4].wrapping_add(c.r[7].wrapping_mul(4)).wrapping_add(112u32), v)?; }
                c.r[7] = c.incdec(c.r[7], 32, true);
            pc = 0x1073198c; continue;
            }
            0x1073198c => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.alu(AluOp::Sub, c.r[1], 4u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.sf != c.of { pc = 0x107318f0; continue; }
            pc = 0x1073199f; continue;
            }
            0x1073199f => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[2] = c.rd32(c.r[3].wrapping_add(16u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], 4u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[6] = c.r[6].wrapping_add(c.r[2]).wrapping_add(4294967292u32);
                if c.sf != c.of { pc = 0x107318e0; continue; }
            pc = 0x107319b8; continue;
            }
            0x107319b8 => {
                pc = 0x10731eb3; continue;
            }
            0x107319bd => {
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
            pc = 0x107319c5; continue;
            }
            0x107319c5 => {
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                pc = 0x107319d0; continue;
            }
            0x107319d0 => {
                c.alu(AluOp::Sub, c.r[1], c.rd32(c.r[4].wrapping_add(52u32))?, 32);
                if c.sf == c.of { pc = 0x10731a49; continue; }
            pc = 0x107319d6; continue;
            }
            0x107319d6 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.alu(AluOp::Sub, c.r[2], c.rd32(c.r[4].wrapping_add(48u32))?, 32);
                if c.sf == c.of { pc = 0x10731a49; continue; }
            pc = 0x107319e0; continue;
            }
            0x107319e0 => {
                c.r[1] = c.rd8(c.r[6])? as u32;
                c.r[2] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[2] = c.r[2].wrapping_add(c.r[1].wrapping_mul(4));
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(c.r[7].wrapping_mul(4)).wrapping_add(112u32), v)?; }
                c.r[1] = c.r[0].wrapping_add(c.r[0].wrapping_mul(2));
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.r[0] = c.incdec(c.r[0], 32, true);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(328u32), v)?; }
                c.r[0] = c.rd8(c.r[2])? as u32;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.r[1] = c.r[4].wrapping_add(c.r[1].wrapping_mul(8)).wrapping_add(336u32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 8u32, 32);
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[1].wrapping_add(4294967288u32), v)?; c.fpop(); }
                c.r[0] = c.rd8(c.r[2].wrapping_add(1u32))? as u32;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[1], v)?; c.fpop(); }
                c.r[2] = c.rd8(c.r[2].wrapping_add(2u32))? as u32;
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64);
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[1].wrapping_add(8u32), v)?; c.fpop(); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(328u32))?;
                pc = 0x10731a52; continue;
            }
            0x10731a49 => {
                { let v = 4294967295u32; c.wr32(c.r[4].wrapping_add(c.r[7].wrapping_mul(4)).wrapping_add(112u32), v)?; }
                c.r[7] = c.incdec(c.r[7], 32, true);
            pc = 0x10731a52; continue;
            }
            0x10731a52 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.alu(AluOp::Sub, c.r[1], 4u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.sf != c.of { pc = 0x107319d0; continue; }
            pc = 0x10731a65; continue;
            }
            0x10731a65 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[1] = c.rd32(c.r[3].wrapping_add(16u32))?;
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.alu(AluOp::Sub, c.r[2], 4u32, 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[6] = c.r[6].wrapping_add(c.r[1]).wrapping_add(4294967292u32);
                if c.sf != c.of { pc = 0x107319c5; continue; }
            pc = 0x10731a7e; continue;
            }
            0x10731a7e => {
                pc = 0x10731eb3; continue;
            }
            0x10731a83 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(60u32))?;
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                c.r[1] = c.rd32(c.r[4].wrapping_add(92u32))?;
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[7], 32);
                if c.zf { pc = 0x10731d5c; continue; }
            pc = 0x10731a97; continue;
            }
            0x10731a97 => {
                c.r[1] = c.incdec(c.r[1], 32, false);
                if c.zf { pc = 0x10731c0a; continue; }
            pc = 0x10731a9e; continue;
            }
            0x10731a9e => {
                c.r[1] = c.incdec(c.r[1], 32, false);
                if !c.zf { pc = 0x10731eb3; continue; }
            pc = 0x10731aa5; continue;
            }
            0x10731aa5 => {
                c.r[6] = c.rd32(c.r[3].wrapping_add(16u32))?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.r[6] = c.alu(AluOp::Sub, c.r[6], c.r[2], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
            pc = 0x10731ab4; continue;
            }
            0x10731ab4 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[7] = c.r[7];
            pc = 0x10731ac0; continue;
            }
            0x10731ac0 => {
                c.alu(AluOp::Sub, c.r[2], c.rd32(c.r[4].wrapping_add(52u32))?, 32);
                if c.sf == c.of { pc = 0x10731bc4; continue; }
            pc = 0x10731aca; continue;
            }
            0x10731aca => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.alu(AluOp::Sub, c.r[7], c.rd32(c.r[4].wrapping_add(48u32))?, 32);
                if c.sf == c.of { pc = 0x10731bc4; continue; }
            pc = 0x10731ad8; continue;
            }
            0x10731ad8 => {
                c.r[6] = c.rd32(c.r[6])?;
                c.r[7] = c.rd32(c.r[3].wrapping_add(100u32))?;
                c.r[7] = c.alu(AluOp::And, c.r[7], c.r[6], 32);
                c.alu(AluOp::Sub, c.r[7], c.rd32(c.r[4].wrapping_add(108u32))?, 32);
                if c.cf || c.zf { pc = 0x10731bc4; continue; }
            pc = 0x10731ae9; continue;
            }
            0x10731ae9 => {
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(112u32), v)?; }
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.r[2] = c.r[0].wrapping_add(c.r[0].wrapping_mul(2));
                c.r[0] = c.incdec(c.r[0], 32, true);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(328u32), v)?; }
                c.r[0] = c.rd32(c.r[3].wrapping_add(88u32))?;
                c.r[7] = c.r[0];
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.r[1] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.r[7] = c.alu(AluOp::And, c.r[7], c.r[6], 32);
                c.r[7] = c.shift(ShiftOp::Shr, c.r[7], ((c.r[1] >> 0) & 0xff), 32);
                c.r[2] = c.r[4].wrapping_add(c.r[2].wrapping_mul(8)).wrapping_add(336u32);
                c.alu(AluOp::And, c.r[7], c.r[7], 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731b23; continue; }
            pc = 0x10731b1d; continue;
            }
            0x10731b1d => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731b23; continue;
            }
            0x10731b23 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731b3b; continue; }
            pc = 0x10731b35; continue;
            }
            0x10731b35 => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731b3b; continue;
            }
            0x10731b3b => {
                c.r[0] = c.rd32(c.r[3].wrapping_add(92u32))?;
                c.farith(FOp::Div, 1, 0, true);
                c.r[1] = c.rd32(c.r[4].wrapping_add(68u32))?;
                c.r[7] = c.r[0];
                c.r[7] = c.alu(AluOp::And, c.r[7], c.r[6], 32);
                c.r[7] = c.shift(ShiftOp::Shr, c.r[7], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[7], c.r[7], 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.st0(); c.wrf64(c.r[2], v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731b5e; continue; }
            pc = 0x10731b58; continue;
            }
            0x10731b58 => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731b5e; continue;
            }
            0x10731b5e => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(68u32))?;
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731b76; continue; }
            pc = 0x10731b70; continue;
            }
            0x10731b70 => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731b76; continue;
            }
            0x10731b76 => {
                c.r[0] = c.rd32(c.r[3].wrapping_add(96u32))?;
                c.farith(FOp::Div, 1, 0, true);
                c.r[1] = c.rd32(c.r[4].wrapping_add(72u32))?;
                c.r[7] = c.r[0];
                c.r[7] = c.alu(AluOp::And, c.r[7], c.r[6], 32);
                c.r[7] = c.shift(ShiftOp::Shr, c.r[7], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[7], c.r[7], 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.st0(); c.wrf64(c.r[2].wrapping_add(8u32), v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731b9a; continue; }
            pc = 0x10731b94; continue;
            }
            0x10731b94 => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731b9a; continue;
            }
            0x10731b9a => {
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731bae; continue; }
            pc = 0x10731ba8; continue;
            }
            0x10731ba8 => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731bae; continue;
            }
            0x10731bae => {
                c.farith(FOp::Div, 1, 0, true);
                c.r[1] = c.rd32(c.r[4].wrapping_add(28u32))?;
                { let v = c.st0(); c.wrf64(c.r[2].wrapping_add(16u32), v)?; c.fpop(); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(328u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(20u32))?;
                pc = 0x10731bcd; continue;
            }
            0x10731bc4 => {
                { let v = 4294967295u32; c.wr32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(112u32), v)?; }
                c.r[1] = c.incdec(c.r[1], 32, true);
            pc = 0x10731bcd; continue;
            }
            0x10731bcd => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], c.r[7], 32);
                c.alu(AluOp::Sub, c.r[2], 4u32, 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                if c.sf != c.of { pc = 0x10731ac0; continue; }
            pc = 0x10731be9; continue;
            }
            0x10731be9 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], c.r[7], 32);
                c.alu(AluOp::Sub, c.r[2], 4u32, 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                if c.sf != c.of { pc = 0x10731ab4; continue; }
            pc = 0x10731c05; continue;
            }
            0x10731c05 => {
                pc = 0x10731eb3; continue;
            }
            0x10731c0a => {
                c.r[6] = c.rd32(c.r[3].wrapping_add(16u32))?;
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.r[6] = c.alu(AluOp::Sub, c.r[6], c.r[2], 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[6] = c.rd32(c.r[4].wrapping_add(32u32))?;
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
            pc = 0x10731c20; continue;
            }
            0x10731c20 => {
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
            pc = 0x10731c26; continue;
            }
            0x10731c26 => {
                c.alu(AluOp::Sub, c.r[2], c.rd32(c.r[4].wrapping_add(52u32))?, 32);
                if c.sf == c.of { pc = 0x10731c42; continue; }
            pc = 0x10731c2c; continue;
            }
            0x10731c2c => {
                c.alu(AluOp::Sub, c.r[7], c.rd32(c.r[4].wrapping_add(48u32))?, 32);
                if c.sf == c.of { pc = 0x10731c42; continue; }
            pc = 0x10731c32; continue;
            }
            0x10731c32 => {
                c.r[6] = c.rd32(c.r[6])?;
                c.alu(AluOp::Sub, c.r[6], c.rd32(c.r[3].wrapping_add(64u32))?, 32);
                if c.cf { pc = 0x10731c76; continue; }
            pc = 0x10731c39; continue;
            }
            0x10731c39 => {
                c.alu(AluOp::Sub, c.r[6], c.rd32(c.r[3].wrapping_add(68u32))?, 32);
                if !c.cf && !c.zf { pc = 0x10731c76; continue; }
            pc = 0x10731c3e; continue;
            }
            0x10731c3e => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(32u32))?;
            pc = 0x10731c42; continue;
            }
            0x10731c42 => {
                { let v = 4294967295u32; c.wr32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(112u32), v)?; }
                c.r[1] = c.incdec(c.r[1], 32, true);
            pc = 0x10731c4b; continue;
            }
            0x10731c4b => {
                c.r[6] = c.alu(AluOp::Add, c.r[6], c.rd32(c.r[4].wrapping_add(44u32))?, 32);
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.alu(AluOp::Sub, c.r[2], 4u32, 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                if c.sf != c.of { pc = 0x10731c26; continue; }
            pc = 0x10731c5d; continue;
            }
            0x10731c5d => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[7], 4u32, 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                if c.sf != c.of { pc = 0x10731c20; continue; }
            pc = 0x10731c71; continue;
            }
            0x10731c71 => {
                pc = 0x10731eb3; continue;
            }
            0x10731c76 => {
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(112u32), v)?; }
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.r[2] = c.r[0].wrapping_add(c.r[0].wrapping_mul(2));
                c.r[0] = c.incdec(c.r[0], 32, true);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(328u32), v)?; }
                c.r[0] = c.rd32(c.r[3].wrapping_add(88u32))?;
                c.r[7] = c.r[0];
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.r[1] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.r[7] = c.alu(AluOp::And, c.r[7], c.r[6], 32);
                c.r[7] = c.shift(ShiftOp::Shr, c.r[7], ((c.r[1] >> 0) & 0xff), 32);
                c.r[2] = c.r[4].wrapping_add(c.r[2].wrapping_mul(8)).wrapping_add(336u32);
                c.alu(AluOp::And, c.r[7], c.r[7], 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731cb0; continue; }
            pc = 0x10731caa; continue;
            }
            0x10731caa => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731cb0; continue;
            }
            0x10731cb0 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731cc8; continue; }
            pc = 0x10731cc2; continue;
            }
            0x10731cc2 => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731cc8; continue;
            }
            0x10731cc8 => {
                c.r[0] = c.rd32(c.r[3].wrapping_add(92u32))?;
                c.farith(FOp::Div, 1, 0, true);
                c.r[1] = c.rd32(c.r[4].wrapping_add(68u32))?;
                c.r[7] = c.r[0];
                c.r[7] = c.alu(AluOp::And, c.r[7], c.r[6], 32);
                c.r[7] = c.shift(ShiftOp::Shr, c.r[7], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[7], c.r[7], 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.st0(); c.wrf64(c.r[2], v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731ceb; continue; }
            pc = 0x10731ce5; continue;
            }
            0x10731ce5 => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731ceb; continue;
            }
            0x10731ceb => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(68u32))?;
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731d03; continue; }
            pc = 0x10731cfd; continue;
            }
            0x10731cfd => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731d03; continue;
            }
            0x10731d03 => {
                c.r[0] = c.rd32(c.r[3].wrapping_add(96u32))?;
                c.farith(FOp::Div, 1, 0, true);
                c.r[1] = c.rd32(c.r[4].wrapping_add(72u32))?;
                c.r[7] = c.r[0];
                c.r[7] = c.alu(AluOp::And, c.r[7], c.r[6], 32);
                c.r[7] = c.shift(ShiftOp::Shr, c.r[7], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[7], c.r[7], 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.st0(); c.wrf64(c.r[2].wrapping_add(8u32), v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731d27; continue; }
            pc = 0x10731d21; continue;
            }
            0x10731d21 => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731d27; continue;
            }
            0x10731d27 => {
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731d3b; continue; }
            pc = 0x10731d35; continue;
            }
            0x10731d35 => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731d3b; continue;
            }
            0x10731d3b => {
                c.farith(FOp::Div, 1, 0, true);
                c.r[7] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(32u32))?;
                { let v = c.st0(); c.wrf64(c.r[2].wrapping_add(16u32), v)?; c.fpop(); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(328u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(20u32))?;
                pc = 0x10731c4b; continue;
            }
            0x10731d5c => {
                c.r[1] = c.rd32(c.r[3].wrapping_add(16u32))?;
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[1] = c.r[1];
            pc = 0x10731d70; continue;
            }
            0x10731d70 => {
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
            pc = 0x10731d76; continue;
            }
            0x10731d76 => {
                c.alu(AluOp::Sub, c.r[1], c.rd32(c.r[4].wrapping_add(52u32))?, 32);
                if c.sf == c.of { pc = 0x10731e78; continue; }
            pc = 0x10731d80; continue;
            }
            0x10731d80 => {
                c.alu(AluOp::Sub, c.r[6], c.rd32(c.r[4].wrapping_add(48u32))?, 32);
                if c.sf == c.of { pc = 0x10731e78; continue; }
            pc = 0x10731d8a; continue;
            }
            0x10731d8a => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[2] = c.rd32(c.r[2])?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(64u32))?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(c.r[7].wrapping_mul(4)).wrapping_add(112u32), v)?; }
                c.r[6] = c.r[0].wrapping_add(c.r[0].wrapping_mul(2));
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.r[0] = c.incdec(c.r[0], 32, true);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(328u32), v)?; }
                c.r[0] = c.r[3].wrapping_add(88u32);
                c.r[0] = c.rd32(c.r[0])?;
                c.r[3] = c.r[0];
                c.r[3] = c.alu(AluOp::And, c.r[3], c.r[2], 32);
                c.r[3] = c.shift(ShiftOp::Shr, c.r[3], ((c.r[1] >> 0) & 0xff), 32);
                c.r[6] = c.r[4].wrapping_add(c.r[6].wrapping_mul(8)).wrapping_add(336u32);
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731dc8; continue; }
            pc = 0x10731dc2; continue;
            }
            0x10731dc2 => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731dc8; continue;
            }
            0x10731dc8 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731de0; continue; }
            pc = 0x10731dda; continue;
            }
            0x10731dda => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731de0; continue;
            }
            0x10731de0 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.farith(FOp::Div, 1, 0, true);
                c.r[1] = c.rd32(c.r[4].wrapping_add(68u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 88u32, 32);
                c.r[0] = c.rd32(c.r[0].wrapping_add(4u32))?;
                c.r[3] = c.r[0];
                c.r[3] = c.alu(AluOp::And, c.r[3], c.r[2], 32);
                c.r[3] = c.shift(ShiftOp::Shr, c.r[3], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.st0(); c.wrf64(c.r[6], v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731e09; continue; }
            pc = 0x10731e03; continue;
            }
            0x10731e03 => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731e09; continue;
            }
            0x10731e09 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(68u32))?;
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731e21; continue; }
            pc = 0x10731e1b; continue;
            }
            0x10731e1b => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731e21; continue;
            }
            0x10731e21 => {
                c.r[3] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[0] = c.rd32(c.r[3].wrapping_add(96u32))?;
                c.farith(FOp::Div, 1, 0, true);
                c.r[1] = c.r[0];
                c.r[1] = c.alu(AluOp::And, c.r[1], c.r[2], 32);
                c.r[2] = c.r[1];
                c.r[1] = c.rd32(c.r[4].wrapping_add(72u32))?;
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_add(8u32), v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731e4a; continue; }
            pc = 0x10731e44; continue;
            }
            0x10731e44 => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731e4a; continue;
            }
            0x10731e4a => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(72u32))?;
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731e62; continue; }
            pc = 0x10731e5c; continue;
            }
            0x10731e5c => {
                { let v = c.rdf64(276582168u32)?; c.farith_value(FOp::Add, v); }
            pc = 0x10731e62; continue;
            }
            0x10731e62 => {
                c.farith(FOp::Div, 1, 0, true);
                c.r[1] = c.rd32(c.r[4].wrapping_add(20u32))?;
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_add(16u32), v)?; c.fpop(); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(328u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(36u32))?;
                pc = 0x10731e81; continue;
            }
            0x10731e78 => {
                { let v = 4294967295u32; c.wr32(c.r[4].wrapping_add(c.r[7].wrapping_mul(4)).wrapping_add(112u32), v)?; }
                c.r[7] = c.incdec(c.r[7], 32, true);
            pc = 0x10731e81; continue;
            }
            0x10731e81 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[2] = c.alu(AluOp::Add, c.r[2], c.rd32(c.r[4].wrapping_add(44u32))?, 32);
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], 4u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                if c.sf != c.of { pc = 0x10731d76; continue; }
            pc = 0x10731e9b; continue;
            }
            0x10731e9b => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.r[2] = c.alu(AluOp::Add, c.r[2], c.r[1], 32);
                c.alu(AluOp::Sub, c.r[6], 4u32, 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                if c.sf != c.of { pc = 0x10731d70; continue; }
            pc = 0x10731eb3; continue;
            }
            0x10731eb3 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(100u32))?;
                c.alu(AluOp::Sub, c.r[1], 256u32, 32);
                if c.zf { pc = 0x107322a7; continue; }
            pc = 0x10731ec3; continue;
            }
            0x10731ec3 => {
                c.alu(AluOp::Sub, c.r[1], 512u32, 32);
                if !c.zf { pc = 0x107323ad; continue; }
            pc = 0x10731ecf; continue;
            }
            0x10731ecf => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[6] = c.rd32(c.r[3].wrapping_add(16u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(60u32))?;
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                c.r[2] = c.shift(ShiftOp::Shl, c.r[2], 2u32, 32);
                c.r[6] = c.alu(AluOp::Sub, c.r[6], c.r[2], 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(176u32), v)?; }
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[6] = c.r[7];
                c.r[3] = c.r[3];
            pc = 0x10731f00; continue;
            }
            0x10731f00 => {
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
            pc = 0x10731f06; continue;
            }
            0x10731f06 => {
                c.alu(AluOp::Sub, c.r[2], c.rd32(c.r[4].wrapping_add(52u32))?, 32);
                if c.sf == c.of { pc = 0x10731f8f; continue; }
            pc = 0x10731f10; continue;
            }
            0x10731f10 => {
                c.alu(AluOp::Sub, c.r[6], c.rd32(c.r[4].wrapping_add(48u32))?, 32);
                if c.sf == c.of { pc = 0x10731f8f; continue; }
            pc = 0x10731f16; continue;
            }
            0x10731f16 => {
                c.r[2] = c.rd32(c.r[1])?;
                c.r[6] = c.rd32(c.r[3].wrapping_add(100u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(80u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[2] = c.alu(AluOp::And, c.r[2], c.r[6], 32);
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], ((c.r[1] >> 0) & 0xff), 32);
                c.r[6] = c.shift(ShiftOp::Shr, c.r[6], ((c.r[1] >> 0) & 0xff), 32);
                c.r[1] = c.rd32(c.r[4].wrapping_add(176u32))?;
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(c.r[7].wrapping_mul(4)).wrapping_add(112u32), v)?; }
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.alu(AluOp::Sub, c.r[2], c.r[6], 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if !c.zf { pc = 0x10731f45; continue; }
            pc = 0x10731f3d; continue;
            }
            0x10731f3d => {
                c.fpush(c.rdf32(276370776u32)? as f64);
                pc = 0x10731f6b; continue;
            }
            0x10731f45 => {
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731f57; continue; }
            pc = 0x10731f51; continue;
            }
            0x10731f51 => {
                { let v = c.rdf32(276669560u32)? as f64; c.farith_value(FOp::Add, v); }
            pc = 0x10731f57; continue;
            }
            0x10731f57 => {
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10731f69; continue; }
            pc = 0x10731f63; continue;
            }
            0x10731f63 => {
                { let v = c.rdf32(276669560u32)? as f64; c.farith_value(FOp::Add, v); }
            pc = 0x10731f69; continue;
            }
            0x10731f69 => {
                c.farith(FOp::Div, 1, 0, true);
            pc = 0x10731f6b; continue;
            }
            0x10731f6b => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                { let v = c.st0(); c.wrf32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(180u32), v as f32)?; c.fpop(); }
                c.r[1] = c.rd32(c.r[4].wrapping_add(176u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(176u32), v)?; }
                c.r[1] = c.rd32(c.r[4].wrapping_add(36u32))?;
                pc = 0x10731f9c; continue;
            }
            0x10731f8f => {
                { let v = 4294967295u32; c.wr32(c.r[4].wrapping_add(c.r[7].wrapping_mul(4)).wrapping_add(112u32), v)?; }
                c.r[7] = c.incdec(c.r[7], 32, true);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
            pc = 0x10731f9c; continue;
            }
            0x10731f9c => {
                c.r[1] = c.alu(AluOp::Add, c.r[1], c.rd32(c.r[4].wrapping_add(44u32))?, 32);
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.alu(AluOp::Sub, c.r[2], 4u32, 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                if c.sf != c.of { pc = 0x10731f06; continue; }
            pc = 0x10731fb2; continue;
            }
            0x10731fb2 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.r[1] = c.alu(AluOp::Add, c.r[1], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[6], 4u32, 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                if c.sf != c.of { pc = 0x10731f00; continue; }
            pc = 0x10731fca; continue;
            }
            0x10731fca => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(24u32))?;
                c.r[2] = c.r[1];
                c.r[1] = c.alu(AluOp::And, c.r[1], 131072u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(248u32), v)?; }
                c.r[1] = c.rd32(c.r[4].wrapping_add(176u32))?;
                c.r[2] = c.alu(AluOp::And, c.r[2], 65536u32, 32);
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                c.alu(AluOp::Sub, c.r[1], c.r[7], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(244u32), v)?; }
                if !c.zf { pc = 0x10732001; continue; }
            pc = 0x10731ff6; continue;
            }
            0x10731ff6 => {
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(256u32), v)?; }
                pc = 0x1073202a; continue;
            }
            0x10732001 => {
                c.r[0] = c.r[4].wrapping_add(176u32);
                c.push(c.r[0])?;
                c.push(0x1073200e)?; dispatch(c, 0x1073bdd0)?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(260u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(264u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[6], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(328u32))?;
                if !c.zf { pc = 0x10732065; continue; }
            pc = 0x1073202a; continue;
            }
            0x1073202a => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(176u32))?;
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::Sub, c.r[2], c.r[7], 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(260u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073205a; continue; }
            pc = 0x1073203f; continue;
            }
            0x1073203f => {
            pc = 0x10732040; continue;
            }
            0x10732040 => {
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(264u32), v)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(176u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                if c.sf != c.of { pc = 0x10732040; continue; }
            pc = 0x10732053; continue;
            }
            0x10732053 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(260u32))?;
            pc = 0x1073205a; continue;
            }
            0x1073205a => {
                { let v = 6u32; c.wr32(c.r[4].wrapping_add(252u32), v)?; }
            pc = 0x10732065; continue;
            }
            0x10732065 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(256u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(252u32))?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::Sub, c.r[2], c.r[6], 32);
                { let v = if !c.zf && c.sf == c.of { 1u32 } else { 0u32 }; c.set8(1, 0, v); }
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[7], 6u32, 32);
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(2, 0, v); }
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                if !c.zf { pc = 0x107320a5; continue; }
            pc = 0x10732086; continue;
            }
            0x10732086 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(256u32))?;
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(256u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(260u32), v)?; }
                { let v = 1u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                pc = 0x107320ad; continue;
            }
            0x107320a5 => {
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
            pc = 0x107320ad; continue;
            }
            0x107320ad => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                pc = 0x107320c0; continue;
            }
            0x107320c0 => {
                c.r[1] = 255u32;
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(c.r[6].wrapping_mul(4)).wrapping_add(256u32))?, c.r[1], 32);
                if c.zf || c.sf != c.of { pc = 0x107320d5; continue; }
            pc = 0x107320ce; continue;
            }
            0x107320ce => {
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(c.r[6].wrapping_mul(4)).wrapping_add(256u32), v)?; }
            pc = 0x107320d5; continue;
            }
            0x107320d5 => {
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(c.r[6].wrapping_mul(4)).wrapping_add(256u32))?, c.r[2], 32);
                if c.sf == c.of { pc = 0x107320e5; continue; }
            pc = 0x107320de; continue;
            }
            0x107320de => {
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(c.r[6].wrapping_mul(4)).wrapping_add(256u32), v)?; }
            pc = 0x107320e5; continue;
            }
            0x107320e5 => {
                { let v = c.rd8(c.r[4].wrapping_add(c.r[6].wrapping_mul(4)).wrapping_add(256u32))? as u32; c.set8(1, 0, v); }
                { let v = ((c.r[1] >> 0) & 0xff); c.wr8(c.r[7], v as u8)?; }
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.alu(AluOp::Sub, c.r[6], 2u32, 32);
                if c.sf != c.of { pc = 0x107320c0; continue; }
            pc = 0x107320f5; continue;
            }
            0x107320f5 => {
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.r[1] = c.r[7];
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[1], v)?; }
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                { let v = (c.r[2] & 0xffff); c.wr16(c.r[1].wrapping_add(4u32), v as u16)?; }
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[4] = c.r[4];
            pc = 0x10732110; continue;
            }
            0x10732110 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(c.r[2].wrapping_mul(4)).wrapping_add(112u32))?;
                c.r[7] = c.shift(ShiftOp::Shr, c.r[7], 3u32, 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.sf != c.of { pc = 0x1073216a; continue; }
            pc = 0x1073211b; continue;
            }
            0x1073211b => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(264u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf { pc = 0x10732165; continue; }
            pc = 0x1073212a; continue;
            }
            0x1073212a => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(252u32))?;
                c.alu(AluOp::Sub, c.r[1], 8u32, 32);
                if !c.zf { pc = 0x10732144; continue; }
            pc = 0x10732136; continue;
            }
            0x10732136 => {
                c.alu(AluOp::Sub, c.r[6], 1u32, 32);
                if c.zf || c.sf != c.of { pc = 0x1073215c; continue; }
            pc = 0x1073213b; continue;
            }
            0x1073213b => {
                c.r[1] = 9u32;
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[6], 32);
                pc = 0x10732163; continue;
            }
            0x10732144 => {
                c.alu(AluOp::Sub, c.r[1], 6u32, 32);
                if !c.zf { pc = 0x10732165; continue; }
            pc = 0x10732149; continue;
            }
            0x10732149 => {
                c.alu(AluOp::Sub, c.r[6], 5u32, 32);
                if !c.zf && c.sf == c.of { pc = 0x10732165; continue; }
            pc = 0x1073214e; continue;
            }
            0x1073214e => {
                c.alu(AluOp::Sub, c.r[6], 1u32, 32);
                if c.zf || c.sf != c.of { pc = 0x1073215c; continue; }
            pc = 0x10732153; continue;
            }
            0x10732153 => {
                c.r[1] = 7u32;
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[6], 32);
                pc = 0x10732163; continue;
            }
            0x1073215c => {
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(1, 0, v); }
            pc = 0x10732163; continue;
            }
            0x10732163 => {
                c.r[6] = c.r[1];
            pc = 0x10732165; continue;
            }
            0x10732165 => {
                c.r[6] = c.shift(ShiftOp::Shl, c.r[6], 21u32, 32);
                c.r[7] = c.alu(AluOp::Or, c.r[7], c.r[6], 32);
            pc = 0x1073216a; continue;
            }
            0x1073216a => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(c.r[2].wrapping_mul(4)).wrapping_add(116u32))?;
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.r[7] = c.shift(ShiftOp::Shr, c.r[7], 3u32, 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.sf != c.of { pc = 0x107321c5; continue; }
            pc = 0x10732176; continue;
            }
            0x10732176 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(264u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf { pc = 0x107321c0; continue; }
            pc = 0x10732185; continue;
            }
            0x10732185 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(252u32))?;
                c.alu(AluOp::Sub, c.r[1], 8u32, 32);
                if !c.zf { pc = 0x1073219f; continue; }
            pc = 0x10732191; continue;
            }
            0x10732191 => {
                c.alu(AluOp::Sub, c.r[6], 1u32, 32);
                if c.zf || c.sf != c.of { pc = 0x107321b7; continue; }
            pc = 0x10732196; continue;
            }
            0x10732196 => {
                c.r[1] = 9u32;
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[6], 32);
                pc = 0x107321be; continue;
            }
            0x1073219f => {
                c.alu(AluOp::Sub, c.r[1], 6u32, 32);
                if !c.zf { pc = 0x107321c0; continue; }
            pc = 0x107321a4; continue;
            }
            0x107321a4 => {
                c.alu(AluOp::Sub, c.r[6], 5u32, 32);
                if !c.zf && c.sf == c.of { pc = 0x107321c0; continue; }
            pc = 0x107321a9; continue;
            }
            0x107321a9 => {
                c.alu(AluOp::Sub, c.r[6], 1u32, 32);
                if c.zf || c.sf != c.of { pc = 0x107321b7; continue; }
            pc = 0x107321ae; continue;
            }
            0x107321ae => {
                c.r[1] = 7u32;
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[6], 32);
                pc = 0x107321be; continue;
            }
            0x107321b7 => {
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(1, 0, v); }
            pc = 0x107321be; continue;
            }
            0x107321be => {
                c.r[6] = c.r[1];
            pc = 0x107321c0; continue;
            }
            0x107321c0 => {
                c.r[6] = c.shift(ShiftOp::Shl, c.r[6], 21u32, 32);
                c.r[7] = c.alu(AluOp::Or, c.r[7], c.r[6], 32);
            pc = 0x107321c5; continue;
            }
            0x107321c5 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(c.r[2].wrapping_mul(4)).wrapping_add(116u32))?;
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.r[7] = c.shift(ShiftOp::Shr, c.r[7], 3u32, 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.sf != c.of { pc = 0x10732220; continue; }
            pc = 0x107321d1; continue;
            }
            0x107321d1 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(264u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf { pc = 0x1073221b; continue; }
            pc = 0x107321e0; continue;
            }
            0x107321e0 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(252u32))?;
                c.alu(AluOp::Sub, c.r[1], 8u32, 32);
                if !c.zf { pc = 0x107321fa; continue; }
            pc = 0x107321ec; continue;
            }
            0x107321ec => {
                c.alu(AluOp::Sub, c.r[6], 1u32, 32);
                if c.zf || c.sf != c.of { pc = 0x10732212; continue; }
            pc = 0x107321f1; continue;
            }
            0x107321f1 => {
                c.r[1] = 9u32;
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[6], 32);
                pc = 0x10732219; continue;
            }
            0x107321fa => {
                c.alu(AluOp::Sub, c.r[1], 6u32, 32);
                if !c.zf { pc = 0x1073221b; continue; }
            pc = 0x107321ff; continue;
            }
            0x107321ff => {
                c.alu(AluOp::Sub, c.r[6], 5u32, 32);
                if !c.zf && c.sf == c.of { pc = 0x1073221b; continue; }
            pc = 0x10732204; continue;
            }
            0x10732204 => {
                c.alu(AluOp::Sub, c.r[6], 1u32, 32);
                if c.zf || c.sf != c.of { pc = 0x10732212; continue; }
            pc = 0x10732209; continue;
            }
            0x10732209 => {
                c.r[1] = 7u32;
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[6], 32);
                pc = 0x10732219; continue;
            }
            0x10732212 => {
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(1, 0, v); }
            pc = 0x10732219; continue;
            }
            0x10732219 => {
                c.r[6] = c.r[1];
            pc = 0x1073221b; continue;
            }
            0x1073221b => {
                c.r[6] = c.shift(ShiftOp::Shl, c.r[6], 21u32, 32);
                c.r[7] = c.alu(AluOp::Or, c.r[7], c.r[6], 32);
            pc = 0x10732220; continue;
            }
            0x10732220 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(c.r[2].wrapping_mul(4)).wrapping_add(116u32))?;
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.r[7] = c.shift(ShiftOp::Shr, c.r[7], 3u32, 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.sf != c.of { pc = 0x1073227b; continue; }
            pc = 0x1073222c; continue;
            }
            0x1073222c => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(264u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf { pc = 0x10732276; continue; }
            pc = 0x1073223b; continue;
            }
            0x1073223b => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(252u32))?;
                c.alu(AluOp::Sub, c.r[1], 8u32, 32);
                if !c.zf { pc = 0x10732255; continue; }
            pc = 0x10732247; continue;
            }
            0x10732247 => {
                c.alu(AluOp::Sub, c.r[6], 1u32, 32);
                if c.zf || c.sf != c.of { pc = 0x1073226d; continue; }
            pc = 0x1073224c; continue;
            }
            0x1073224c => {
                c.r[1] = 9u32;
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[6], 32);
                pc = 0x10732274; continue;
            }
            0x10732255 => {
                c.alu(AluOp::Sub, c.r[1], 6u32, 32);
                if !c.zf { pc = 0x10732276; continue; }
            pc = 0x1073225a; continue;
            }
            0x1073225a => {
                c.alu(AluOp::Sub, c.r[6], 5u32, 32);
                if !c.zf && c.sf == c.of { pc = 0x10732276; continue; }
            pc = 0x1073225f; continue;
            }
            0x1073225f => {
                c.alu(AluOp::Sub, c.r[6], 1u32, 32);
                if c.zf || c.sf != c.of { pc = 0x1073226d; continue; }
            pc = 0x10732264; continue;
            }
            0x10732264 => {
                c.r[1] = 7u32;
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[6], 32);
                pc = 0x10732274; continue;
            }
            0x1073226d => {
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(1, 0, v); }
            pc = 0x10732274; continue;
            }
            0x10732274 => {
                c.r[6] = c.r[1];
            pc = 0x10732276; continue;
            }
            0x10732276 => {
                c.r[6] = c.shift(ShiftOp::Shl, c.r[6], 21u32, 32);
                c.r[7] = c.alu(AluOp::Or, c.r[7], c.r[6], 32);
            pc = 0x1073227b; continue;
            }
            0x1073227b => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.alu(AluOp::And, ((c.r[1] >> 0) & 0xff), 1u32, 8);
                if c.zf { pc = 0x10732294; continue; }
            pc = 0x10732285; continue;
            }
            0x10732285 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(40u32))?;
                { let v = c.alu(AluOp::Or, c.rd32(c.r[6])?, c.r[7], 32); c.wr32(c.r[6], v)?; }
                c.r[6] = c.alu(AluOp::Add, c.r[6], 3u32, 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
            pc = 0x10732294; continue;
            }
            0x10732294 => {
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], 4u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                if c.sf != c.of { pc = 0x10732110; continue; }
            pc = 0x107322a2; continue;
            }
            0x107322a2 => {
                pc = 0x107323ad; continue;
            }
            0x107322a7 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(60u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(44u32))?;
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                pc = 0x107322c0; continue;
            }
            0x107322c0 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(16u32))?, c.r[1], 32);
                if c.sf == c.of { pc = 0x10732375; continue; }
            pc = 0x107322ce; continue;
            }
            0x107322ce => {
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
            pc = 0x107322d0; continue;
            }
            0x107322d0 => {
                c.alu(AluOp::Sub, c.r[7], c.rd32(c.r[4].wrapping_add(52u32))?, 32);
                if c.sf == c.of { pc = 0x10732355; continue; }
            pc = 0x107322da; continue;
            }
            0x107322da => {
                c.r[0] = c.rd32(c.r[3].wrapping_add(100u32))?;
                c.r[1] = c.rd32(c.r[6])?;
                c.r[2] = c.r[0];
                c.r[2] = c.alu(AluOp::And, c.r[2], c.r[1], 32);
                c.r[1] = c.rd32(c.r[4].wrapping_add(80u32))?;
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x107322fb; continue; }
            pc = 0x107322f5; continue;
            }
            0x107322f5 => {
                { let v = c.rdf32(276669560u32)? as f64; c.farith_value(FOp::Add, v); }
            pc = 0x107322fb; continue;
            }
            0x107322fb => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(80u32))?;
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(24u32))? as i32 as f64);
                if c.sf == c.of { pc = 0x10732313; continue; }
            pc = 0x1073230d; continue;
            }
            0x1073230d => {
                { let v = c.rdf32(276669560u32)? as f64; c.farith_value(FOp::Add, v); }
            pc = 0x10732313; continue;
            }
            0x10732313 => {
                c.farith(FOp::Div, 1, 0, true);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 8u32, 32);
                { let v = c.rdf32(276400868u32)? as f64; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf32(276399672u32)? as f64; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[4], v)?; c.fpop(); }
                c.push(0x1073232c)?; dispatch(c, 0x10741158)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                c.push(0x10732334)?; dispatch(c, 0x10740cb4)?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = c.rd16(c.r[4].wrapping_add(76u32))? as u32; c.set16(1, v); }
                { let v = c.shift(ShiftOp::Shr, (c.r[1] & 0xffff), 4u32, 16); c.set16(1, v); }
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 12u32, 32);
                c.r[0] = c.alu(AluOp::Or, c.r[0], c.r[1], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(76u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(328u32))?;
                pc = 0x1073235b; continue;
            }
            0x10732355 => {
                { let v = c.shift(ShiftOp::Shr, c.rd16(c.r[4].wrapping_add(76u32))? as u32, 4u32, 16); c.wr16(c.r[4].wrapping_add(76u32), v as u16)?; }
            pc = 0x1073235b; continue;
            }
            0x1073235b => {
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[7], 4u32, 32);
                if c.sf != c.of { pc = 0x107322d0; continue; }
            pc = 0x10732367; continue;
            }
            0x10732367 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(40u32))?;
                { let v = c.rd16(c.r[4].wrapping_add(76u32))? as u32; c.set16(7, v); }
                { let v = (c.r[7] & 0xffff); c.wr16(c.r[1], v as u16)?; }
                pc = 0x1073237e; continue;
            }
            0x10732375 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(40u32))?;
                { let v = 0u32; c.wr16(c.r[1], v as u16)?; }
            pc = 0x1073237e; continue;
            }
            0x1073237e => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 2u32, 32);
                c.r[7] = c.incdec(c.r[7], 32, true);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[7] = c.rd32(c.r[3].wrapping_add(16u32))?;
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                c.r[1] = c.r[2].wrapping_mul(4);
                c.r[7] = c.alu(AluOp::Sub, c.r[7], c.r[1], 32);
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[6] = c.alu(AluOp::Add, c.r[6], c.r[7], 32);
                c.alu(AluOp::Sub, c.r[1], 4u32, 32);
                if c.sf != c.of { pc = 0x107322c0; continue; }
            pc = 0x107323ad; continue;
            }
            0x107323ad => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(100u32))?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::Sub, c.r[2], c.r[1], 32);
                if c.zf { pc = 0x107323c1; continue; }
            pc = 0x107323b7; continue;
            }
            0x107323b7 => {
                { let v = 1u32; c.wr8(c.r[4].wrapping_add(748u32), v as u8)?; }
                pc = 0x107323cb; continue;
            }
            0x107323c1 => {
                { let v = c.rd8(c.r[5].wrapping_add(40u32))? as u32; c.set8(2, 0, v); }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(748u32), v as u8)?; }
            pc = 0x107323cb; continue;
            }
            0x107323cb => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(52u32))?;
                c.r[6] = c.imul(c.r[6], c.rd32(c.r[4].wrapping_add(48u32))?);
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[0], c.r[6], 32);
                { let v = if c.sf == c.of { 1u32 } else { 0u32 }; c.set8(2, 0, v); }
                c.r[2] = c.alu(AluOp::Add, c.r[2], 3u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(744u32), v)?; }
                if !c.zf { pc = 0x10732405; continue; }
            pc = 0x107323e9; continue;
            }
            0x107323e9 => {
                c.r[0] = c.r[4].wrapping_add(760u32);
                c.r[2] = 2u32;
            pc = 0x107323f5; continue;
            }
            0x107323f5 => {
                { let v = c.r[1]; c.wr32(c.r[0].wrapping_add(4u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[0], v)?; }
                { let v = c.r[1]; c.wr32(c.r[0].wrapping_add(4294967292u32), v)?; }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 12u32, 32);
                c.r[2] = c.incdec(c.r[2], 32, false);
                if !c.zf { pc = 0x107323f5; continue; }
            pc = 0x10732403; continue;
            }
            0x10732403 => {
                pc = 0x10732417; continue;
            }
            0x10732405 => {
                c.r[0] = c.r[4].wrapping_add(328u32);
                c.push(c.r[0])?;
                c.push(0x10732412)?; dispatch(c, 0x1073a250)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
            pc = 0x10732417; continue;
            }
            0x10732417 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(756u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(760u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(764u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(772u32))?;
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 6u32, 32);
                c.r[0] = c.alu(AluOp::Or, c.r[0], c.r[2], 32);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 5u32, 32);
                c.r[0] = c.alu(AluOp::Or, c.r[0], c.r[6], 32);
                c.r[6] = c.rd32(c.r[4].wrapping_add(768u32))?;
                c.r[6] = c.shift(ShiftOp::Shl, c.r[6], 6u32, 32);
                c.r[6] = c.alu(AluOp::Or, c.r[6], c.r[7], 32);
                c.r[7] = c.rd32(c.r[4].wrapping_add(776u32))?;
                c.r[6] = c.shift(ShiftOp::Shl, c.r[6], 5u32, 32);
                c.r[6] = c.alu(AluOp::Or, c.r[6], c.r[7], 32);
                { let v = (c.r[6] & 0xffff); c.set16(7, v); }
                c.alu(AluOp::Sub, (c.r[7] & 0xffff), (c.r[0] & 0xffff), 16);
                { let v = (c.r[0] & 0xffff); c.wr16(c.r[4].wrapping_add(56u32), v as u16)?; }
                { let v = (c.r[6] & 0xffff); c.wr16(c.r[4].wrapping_add(58u32), v as u16)?; }
                if !c.zf { pc = 0x107324e4; continue; }
            pc = 0x10732467; continue;
            }
            0x10732467 => {
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(92u32))?, 2u32, 32);
                if !c.zf { pc = 0x107324b5; continue; }
            pc = 0x1073246e; continue;
            }
            0x1073246e => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(328u32))?;
                { let v = c.incdec((c.r[7] & 0xffff), 16, true); c.set16(7, v); }
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[6], c.r[1], 32);
                { let v = (c.r[7] & 0xffff); c.wr16(c.r[4].wrapping_add(58u32), v as u16)?; }
                if c.zf || c.sf != c.of { pc = 0x107324a3; continue; }
            pc = 0x10732482; continue;
            }
            0x10732482 => {
                pc = 0x10732490; continue;
            }
            0x10732490 => {
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(c.r[2].wrapping_mul(4)).wrapping_add(780u32), v)?; }
                c.r[6] = c.rd32(c.r[4].wrapping_add(328u32))?;
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.alu(AluOp::Sub, c.r[2], c.r[6], 32);
                if c.sf != c.of { pc = 0x10732490; continue; }
            pc = 0x107324a3; continue;
            }
            0x107324a3 => {
                { let v = c.rd16(c.r[4].wrapping_add(58u32))? as u32; c.set16(6, v); }
                { let v = 3u32; c.wr32(c.r[4].wrapping_add(752u32), v)?; }
                pc = 0x107324e1; continue;
            }
            0x107324b5 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(328u32))?;
                { let v = c.incdec((c.r[0] & 0xffff), 16, true); c.set16(0, v); }
                c.alu(AluOp::Sub, c.r[2], c.r[1], 32);
                { let v = (c.r[0] & 0xffff); c.wr16(c.r[4].wrapping_add(56u32), v as u16)?; }
                if c.zf || c.sf != c.of { pc = 0x107324e1; continue; }
            pc = 0x107324c7; continue;
            }
            0x107324c7 => {
                c.r[2] = 1u32;
                c.r[4] = c.r[4];
            pc = 0x107324d0; continue;
            }
            0x107324d0 => {
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(780u32), v)?; }
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.rd32(c.r[4].wrapping_add(328u32))?, 32);
                if c.sf != c.of { pc = 0x107324d0; continue; }
            pc = 0x107324e1; continue;
            }
            0x107324e1 => {
                c.alu(AluOp::Sub, (c.r[7] & 0xffff), (c.r[0] & 0xffff), 16);
            pc = 0x107324e4; continue;
            }
            0x107324e4 => {
                c.r[1] = c.alu(AluOp::Sbb, c.r[1], c.r[1], 32);
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.r[1] = c.neg(c.r[1], 32);
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(752u32))?, 3u32, 32);
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(2, 0, v); }
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                if !c.zf { pc = 0x1073250f; continue; }
            pc = 0x107324f9; continue;
            }
            0x107324f9 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(56u32))?;
                { let v = (c.r[7] & 0xffff); c.set16(0, v); }
                { let v = (c.r[1] & 0xffff); c.wr16(c.r[4].wrapping_add(58u32), v as u16)?; }
                c.r[7] = 1u32;
                { let v = (c.r[1] & 0xffff); c.set16(6, v); }
                pc = 0x10732511; continue;
            }
            0x1073250f => {
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
            pc = 0x10732511; continue;
            }
            0x10732511 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(40u32))?;
                { let v = (c.r[0] & 0xffff); c.wr16(c.r[1], v as u16)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(104u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 2u32, 32);
                { let v = (c.r[6] & 0xffff); c.wr16(c.r[1], v as u16)?; }
                c.r[6] = c.rd32(c.r[4].wrapping_add(752u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 2u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                c.r[2] = 15u32;
            pc = 0x10732535; continue;
            }
            0x10732535 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(c.r[2].wrapping_mul(4)).wrapping_add(112u32))?;
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 2u32, 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.sf == c.of { pc = 0x10732545; continue; }
            pc = 0x10732540; continue;
            }
            0x10732540 => {
                c.r[0] = c.alu(AluOp::Or, c.r[0], 3u32, 32);
                pc = 0x10732565; continue;
            }
            0x10732545 => {
                c.r[0] = c.alu(AluOp::Or, c.r[0], c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(780u32))?, 32);
                c.alu(AluOp::And, c.r[7], c.r[7], 32);
                if c.zf { pc = 0x10732565; continue; }
            pc = 0x10732550; continue;
            }
            0x10732550 => {
                c.alu(AluOp::Sub, c.r[6], 4u32, 32);
                if !c.zf { pc = 0x1073255a; continue; }
            pc = 0x10732555; continue;
            }
            0x10732555 => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], 1u32, 32);
                pc = 0x10732565; continue;
            }
            0x1073255a => {
                c.r[1] = c.r[0];
                c.r[1] = c.shift(ShiftOp::Shr, c.r[1], 1u32, 32);
                c.r[1] = !c.r[1] & 4294967295u32;
                c.r[1] = c.alu(AluOp::And, c.r[1], 1u32, 32);
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[1], 32);
            pc = 0x10732565; continue;
            }
            0x10732565 => {
                c.r[2] = c.incdec(c.r[2], 32, false);
                if !c.sf { pc = 0x10732535; continue; }
            pc = 0x10732568; continue;
            }
            0x10732568 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(60u32))?;
                { let v = c.r[0]; c.wr32(c.r[1], v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(88u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[2] = c.shift(ShiftOp::Shl, c.r[2], 2u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                c.r[1] = c.rd32(c.r[3].wrapping_add(12u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 4u32, 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(88u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(60u32), v)?; }
                if c.sf != c.of { pc = 0x10731860; continue; }
            pc = 0x1073259c; continue;
            }
            0x1073259c => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(84u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(96u32))?;
                c.r[2] = c.rd32(c.r[3].wrapping_add(16u32))?;
                c.r[2] = c.r[1].wrapping_add(c.r[2].wrapping_mul(4));
                c.r[1] = c.rd32(c.r[3].wrapping_add(8u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 4u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(96u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(84u32), v)?; }
                if c.sf != c.of { pc = 0x10731806; continue; }
            pc = 0x107325c0; continue;
            }
            0x107325c0 => {
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[3] = v; }
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10735b80(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10735b80;
    loop {
        match pc {
            0x10735b80 => {
                c.push(c.r[6])?;
                c.push(c.r[7])?;
                c.push(12u32)?;
                c.push(0x10735b89)?; dispatch(c, 0x10740e60)?;
                c.r[6] = c.r[0];
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                if !c.zf { pc = 0x10735ba3; continue; }
            pc = 0x10735b92; continue;
            }
            0x10735b92 => {
                c.r[0] = 338496860u32;
                c.r[1] = c.r[0];
                c.r[2] = 276956404u32;
                c.push(0x10735ba3)?; dispatch(c, 0x10735b60)?;
            pc = 0x10735ba3; continue;
            }
            0x10735ba3 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(12u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                { let v = c.r[0]; c.wr32(c.r[6], v)?; }
                c.r[0] = c.imul(c.r[0], c.r[1]);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 2u32, 32);
                c.r[7] = c.r[0];
                c.push(c.r[7])?;
                { let v = c.r[1]; c.wr32(c.r[6].wrapping_add(4u32), v)?; }
                c.push(0x10735bbe)?; dispatch(c, 0x10740e60)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[6].wrapping_add(8u32), v)?; }
                if !c.zf { pc = 0x10735bd9; continue; }
            pc = 0x10735bc8; continue;
            }
            0x10735bc8 => {
                c.r[0] = 338496860u32;
                c.r[1] = c.r[0];
                c.r[2] = 276956404u32;
                c.push(0x10735bd9)?; dispatch(c, 0x10735b60)?;
            pc = 0x10735bd9; continue;
            }
            0x10735bd9 => {
                c.r[1] = c.r[7];
                c.r[7] = c.rd32(c.r[6].wrapping_add(8u32))?;
                c.r[2] = c.r[1];
                c.r[1] = c.shift(ShiftOp::Shr, c.r[1], 2u32, 32);
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.rep_stos(4)?;
                c.r[1] = c.r[2];
                c.r[1] = c.alu(AluOp::And, c.r[1], 3u32, 32);
                c.rep_stos(1)?;
                { let v = c.pop()?; c.r[7] = v; }
                c.r[0] = c.r[6];
                { let v = c.pop()?; c.r[6] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10735c00(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10735c00;
    loop {
        match pc {
            0x10735c00 => {
                { let v = c.rd8(c.r[4].wrapping_add(12u32))? as u32; c.set8(2, 0, v); }
                c.alu(AluOp::And, ((c.r[2] >> 0) & 0xff), ((c.r[2] >> 0) & 0xff), 8);
                c.r[0] = c.rd32(c.r[4].wrapping_add(4u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(8u32))?;
                if c.zf { pc = 0x10735c24; continue; }
            pc = 0x10735c10; continue;
            }
            0x10735c10 => {
                c.alu(AluOp::Sub, c.r[0], 4u32, 32);
                if c.sf == c.of { pc = 0x10735c1a; continue; }
            pc = 0x10735c15; continue;
            }
            0x10735c15 => {
                c.r[0] = 4u32;
            pc = 0x10735c1a; continue;
            }
            0x10735c1a => {
                c.alu(AluOp::Sub, c.r[1], 4u32, 32);
                if c.sf == c.of { pc = 0x10735c24; continue; }
            pc = 0x10735c1f; continue;
            }
            0x10735c1f => {
                c.r[1] = 4u32;
            pc = 0x10735c24; continue;
            }
            0x10735c24 => {
                c.push(c.r[1])?;
                c.push(c.r[0])?;
                c.push(0x10735c2b)?; dispatch(c, 0x10735b80)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10735c30(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10735c30;
    loop {
        match pc {
            0x10735c30 => {
                c.push(c.r[6])?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(8u32))?;
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                if c.zf { pc = 0x10735c52; continue; }
            pc = 0x10735c39; continue;
            }
            0x10735c39 => {
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10735c49; continue; }
            pc = 0x10735c40; continue;
            }
            0x10735c40 => {
                c.push(c.r[0])?;
                c.push(0x10735c46)?; dispatch(c, 0x10740e54)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
            pc = 0x10735c49; continue;
            }
            0x10735c49 => {
                c.push(c.r[6])?;
                c.push(0x10735c4f)?; dispatch(c, 0x10740e54)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 4u32, 32);
            pc = 0x10735c52; continue;
            }
            0x10735c52 => {
                { let v = c.pop()?; c.r[6] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10735c60(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10735c60;
    loop {
        match pc {
            0x10735c60 => {
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 16u32, 32);
                c.r[2] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[0] = c.rd32(c.r[2].wrapping_add(4u32))?;
                { let v = c.rd8(c.r[4].wrapping_add(44u32))? as u32; c.set8(1, 0, v); }
                c.push(c.r[5])?;
                c.r[5] = c.rd32(c.r[0].wrapping_add(832u32))?;
                c.push(c.r[6])?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.push(c.r[7])?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[5] = c.alu(AluOp::Add, c.r[5], c.r[6], 32);
                c.alu(AluOp::And, ((c.r[1] >> 0) & 0xff), ((c.r[1] >> 0) & 0xff), 8);
                { let v = c.r[5]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[0] = c.r[7];
                if c.zf { pc = 0x10735c95; continue; }
            pc = 0x10735c8b; continue;
            }
            0x10735c8b => {
                c.alu(AluOp::Sub, c.r[7], 4u32, 32);
                if c.sf == c.of { pc = 0x10735c95; continue; }
            pc = 0x10735c90; continue;
            }
            0x10735c90 => {
                c.r[0] = 4u32;
            pc = 0x10735c95; continue;
            }
            0x10735c95 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::Sub, c.r[6], c.r[1], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10735f75; continue; }
            pc = 0x10735ca7; continue;
            }
            0x10735ca7 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 2u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.push(c.r[3])?;
                pc = 0x10735cc0; continue;
            }
            0x10735cc0 => {
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                c.alu(AluOp::And, c.r[7], c.r[7], 32);
                if c.zf || c.sf != c.of { pc = 0x10735f46; continue; }
            pc = 0x10735cca; continue;
            }
            0x10735cca => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[7] = c.r[7];
            pc = 0x10735cd0; continue;
            }
            0x10735cd0 => {
                { let v = c.rd8(c.r[2].wrapping_add(9124u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10735dee; continue; }
            pc = 0x10735cde; continue;
            }
            0x10735cde => {
                { let v = c.rd8(c.r[5])? as u32; c.set8(0, 0, v); }
                c.r[2] = c.rd32(c.r[2].wrapping_add(4u32))?;
                c.r[5] = c.incdec(c.r[5], 32, true);
                { let v = ((c.r[0] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(60u32), v as u8)?; }
                { let v = c.rd8(c.r[5])? as u32; c.set8(0, 0, v); }
                c.r[5] = c.incdec(c.r[5], 32, true);
                { let v = ((c.r[0] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(61u32), v as u8)?; }
                { let v = c.rd8(c.r[5])? as u32; c.set8(0, 0, v); }
                { let v = ((c.r[0] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(62u32), v as u8)?; }
                c.r[5] = c.incdec(c.r[5], 32, true);
                c.alu(AluOp::Sub, c.rd16(c.r[2].wrapping_add(52u32))? as u32, 4u32, 16);
                if !c.zf { pc = 0x10735d0a; continue; }
            pc = 0x10735d00; continue;
            }
            0x10735d00 => {
                { let v = c.rd8(c.r[5])? as u32; c.set8(0, 0, v); }
                { let v = ((c.r[0] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(63u32), v as u8)?; }
                c.r[5] = c.incdec(c.r[5], 32, true);
                pc = 0x10735d0f; continue;
            }
            0x10735d0a => {
                { let v = 255u32; c.wr8(c.r[4].wrapping_add(63u32), v as u8)?; }
            pc = 0x10735d0f; continue;
            }
            0x10735d0f => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 4294967286u32, 32);
                c.alu(AluOp::Sub, c.r[0], 10u32, 32);
                if !c.cf && !c.zf { pc = 0x10735dde; continue; }
            pc = 0x10735d1f; continue;
            }
            0x10735d1f => {
                c.r[2] = c.rd8(c.r[0].wrapping_add(275996560u32))? as u32;
                pc = c.rd32(c.r[2].wrapping_mul(4).wrapping_add(275996540u32))?; if ![0x10735dae, 0x10735d2d, 0x10735d58, 0x10735dd6, 0x10735d83].contains(&pc) { return Err(Fault("jmp", pc)); } continue;
            }
            0x10735d2d => {
                c.push(c.r[1])?;
                c.push(c.r[3])?;
                c.r[0] = c.r[4].wrapping_add(24u32);
                c.push(c.r[0])?;
                c.r[1] = c.r[4].wrapping_add(72u32);
                c.push(c.r[1])?;
                c.push(0x10735d3e)?; dispatch(c, 0x10733730)?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(68u32))?;
                { let v = c.r[2]; c.wr32(c.r[7].wrapping_add(c.r[0]), v)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 16u32, 32);
                pc = 0x10735e35; continue;
            }
            0x10735d58 => {
                c.push(c.r[1])?;
                c.push(c.r[3])?;
                c.r[2] = c.r[4].wrapping_add(24u32);
                c.push(c.r[2])?;
                c.r[0] = c.r[4].wrapping_add(72u32);
                c.push(c.r[0])?;
                c.push(0x10735d69)?; dispatch(c, 0x10733450)?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(68u32))?;
                { let v = c.r[2]; c.wr32(c.r[7].wrapping_add(c.r[0]), v)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 16u32, 32);
                pc = 0x10735e35; continue;
            }
            0x10735d83 => {
                c.push(c.r[1])?;
                c.push(c.r[3])?;
                c.r[1] = c.r[4].wrapping_add(24u32);
                c.push(c.r[1])?;
                c.r[2] = c.r[4].wrapping_add(72u32);
                c.push(c.r[2])?;
                c.push(0x10735d94)?; dispatch(c, 0x10733630)?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(68u32))?;
                { let v = c.r[2]; c.wr32(c.r[7].wrapping_add(c.r[0]), v)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 16u32, 32);
                pc = 0x10735e35; continue;
            }
            0x10735dae => {
                c.push(c.r[1])?;
                c.push(c.r[3])?;
                c.r[0] = c.r[4].wrapping_add(24u32);
                c.push(c.r[0])?;
                c.r[1] = c.r[4].wrapping_add(72u32);
                c.push(c.r[1])?;
                c.push(0x10735dbf)?; dispatch(c, 0x10733140)?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(68u32))?;
                { let v = c.r[2]; c.wr32(c.r[7].wrapping_add(c.r[0]), v)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 16u32, 32);
                pc = 0x10735e35; continue;
            }
            0x10735dd6 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(60u32))?;
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
            pc = 0x10735dde; continue;
            }
            0x10735dde => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                { let v = c.r[2]; c.wr32(c.r[7].wrapping_add(c.r[0]), v)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
                pc = 0x10735e35; continue;
            }
            0x10735dee => {
                { let v = c.rd8(c.r[5])? as u32; c.set8(2, 0, v); }
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[7].wrapping_add(c.r[0]), v as u8)?; }
                { let v = c.rd8(c.r[5].wrapping_add(1u32))? as u32; c.set8(2, 0, v); }
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                c.r[5] = c.incdec(c.r[5], 32, true);
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[7].wrapping_add(c.r[0]).wrapping_add(1u32), v as u8)?; }
                { let v = c.rd8(c.r[5].wrapping_add(1u32))? as u32; c.set8(2, 0, v); }
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                c.r[5] = c.incdec(c.r[5], 32, true);
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[7].wrapping_add(c.r[0]).wrapping_add(2u32), v as u8)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[0] = c.rd32(c.r[2].wrapping_add(4u32))?;
                c.r[5] = c.incdec(c.r[5], 32, true);
                c.alu(AluOp::Sub, c.rd16(c.r[0].wrapping_add(52u32))? as u32, 4u32, 16);
                if !c.zf { pc = 0x10735e2d; continue; }
            pc = 0x10735e1c; continue;
            }
            0x10735e1c => {
                c.r[2] = c.rd32(c.r[6].wrapping_add(8u32))?;
                { let v = c.rd8(c.r[5])? as u32; c.set8(0, 0, v); }
                { let v = ((c.r[0] >> 0) & 0xff); c.wr8(c.r[7].wrapping_add(c.r[2]).wrapping_add(3u32), v as u8)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[5] = c.incdec(c.r[5], 32, true);
                pc = 0x10735e35; continue;
            }
            0x10735e2d => {
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                { let v = 255u32; c.wr8(c.r[7].wrapping_add(c.r[0]).wrapping_add(3u32), v as u8)?; }
            pc = 0x10735e35; continue;
            }
            0x10735e35 => {
                { let v = c.rd8(c.r[2].wrapping_add(9095u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10735e5e; continue; }
            pc = 0x10735e3f; continue;
            }
            0x10735e3f => {
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if c.zf { pc = 0x10735e56; continue; }
            pc = 0x10735e43; continue;
            }
            0x10735e43 => {
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf { pc = 0x10735e56; continue; }
            pc = 0x10735e47; continue;
            }
            0x10735e47 => {
                c.r[0] = c.rd32(c.r[6])?;
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.alu(AluOp::Sub, c.r[3], c.r[0], 32);
                if c.zf { pc = 0x10735e56; continue; }
            pc = 0x10735e4e; continue;
            }
            0x10735e4e => {
                c.r[0] = c.rd32(c.r[6].wrapping_add(4u32))?;
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.alu(AluOp::Sub, c.r[1], c.r[0], 32);
                if !c.zf { pc = 0x10735e5e; continue; }
            pc = 0x10735e56; continue;
            }
            0x10735e56 => {
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                { let v = 0u32; c.wr8(c.r[7].wrapping_add(c.r[0]).wrapping_add(3u32), v as u8)?; }
            pc = 0x10735e5e; continue;
            }
            0x10735e5e => {
                { let v = c.rd8(c.r[2].wrapping_add(9096u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10735edf; continue; }
            pc = 0x10735e68; continue;
            }
            0x10735e68 => {
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if c.zf { pc = 0x10735e7f; continue; }
            pc = 0x10735e6c; continue;
            }
            0x10735e6c => {
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf { pc = 0x10735e7f; continue; }
            pc = 0x10735e70; continue;
            }
            0x10735e70 => {
                c.r[0] = c.rd32(c.r[6])?;
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.alu(AluOp::Sub, c.r[3], c.r[0], 32);
                if c.zf { pc = 0x10735e7f; continue; }
            pc = 0x10735e77; continue;
            }
            0x10735e77 => {
                c.r[0] = c.rd32(c.r[6].wrapping_add(4u32))?;
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.alu(AluOp::Sub, c.r[1], c.r[0], 32);
                if !c.zf { pc = 0x10735e8f; continue; }
            pc = 0x10735e7f; continue;
            }
            0x10735e7f => {
                { let v = c.rd8(c.r[2].wrapping_add(9100u32))? as u32; c.set8(2, 0, v); }
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[7].wrapping_add(c.r[0]), v as u8)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
            pc = 0x10735e8f; continue;
            }
            0x10735e8f => {
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if c.zf { pc = 0x10735ea6; continue; }
            pc = 0x10735e93; continue;
            }
            0x10735e93 => {
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf { pc = 0x10735ea6; continue; }
            pc = 0x10735e97; continue;
            }
            0x10735e97 => {
                c.r[0] = c.rd32(c.r[6])?;
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.alu(AluOp::Sub, c.r[3], c.r[0], 32);
                if c.zf { pc = 0x10735ea6; continue; }
            pc = 0x10735e9e; continue;
            }
            0x10735e9e => {
                c.r[0] = c.rd32(c.r[6].wrapping_add(4u32))?;
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.alu(AluOp::Sub, c.r[1], c.r[0], 32);
                if !c.zf { pc = 0x10735eb7; continue; }
            pc = 0x10735ea6; continue;
            }
            0x10735ea6 => {
                { let v = c.rd8(c.r[2].wrapping_add(9101u32))? as u32; c.set8(2, 0, v); }
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[7].wrapping_add(c.r[0]).wrapping_add(1u32), v as u8)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
            pc = 0x10735eb7; continue;
            }
            0x10735eb7 => {
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if c.zf { pc = 0x10735ece; continue; }
            pc = 0x10735ebb; continue;
            }
            0x10735ebb => {
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf { pc = 0x10735ece; continue; }
            pc = 0x10735ebf; continue;
            }
            0x10735ebf => {
                c.r[0] = c.rd32(c.r[6])?;
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.alu(AluOp::Sub, c.r[3], c.r[0], 32);
                if c.zf { pc = 0x10735ece; continue; }
            pc = 0x10735ec6; continue;
            }
            0x10735ec6 => {
                c.r[0] = c.rd32(c.r[6].wrapping_add(4u32))?;
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.alu(AluOp::Sub, c.r[1], c.r[0], 32);
                if !c.zf { pc = 0x10735edf; continue; }
            pc = 0x10735ece; continue;
            }
            0x10735ece => {
                { let v = c.rd8(c.r[2].wrapping_add(9102u32))? as u32; c.set8(2, 0, v); }
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[7].wrapping_add(c.r[0]).wrapping_add(2u32), v as u8)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
            pc = 0x10735edf; continue;
            }
            0x10735edf => {
                { let v = c.rd8(c.r[2].wrapping_add(9092u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10735f08; continue; }
            pc = 0x10735ee9; continue;
            }
            0x10735ee9 => {
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                { let v = c.rd8(c.r[7].wrapping_add(c.r[0]).wrapping_add(3u32))? as u32; c.set8(2, 0, v); }
                c.r[0] = c.r[7].wrapping_add(c.r[0]).wrapping_add(3u32);
                c.alu(AluOp::Sub, ((c.r[2] >> 0) & 0xff), 127u32, 8);
                { let v = if c.cf { 1u32 } else { 0u32 }; c.set8(2, 0, v); }
                { let v = c.incdec(((c.r[2] >> 0) & 0xff), 8, false); c.set8(2, 0, v); }
                c.r[2] = c.alu(AluOp::And, c.r[2], 255u32, 32);
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[0], v as u8)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
            pc = 0x10735f08; continue;
            }
            0x10735f08 => {
                { let v = c.rd8(c.r[2].wrapping_add(9132u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10735f30; continue; }
            pc = 0x10735f12; continue;
            }
            0x10735f12 => {
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                { let v = c.rd8(c.r[0].wrapping_add(c.r[7]).wrapping_add(2u32))? as u32; c.set8(2, 0, v); }
                c.r[1] = c.rd8(c.r[0].wrapping_add(c.r[7]))? as u32;
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[7], 32);
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[0], v as u8)?; }
                c.r[0] = c.rd32(c.r[6].wrapping_add(8u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
                { let v = ((c.r[1] >> 0) & 0xff); c.wr8(c.r[7].wrapping_add(c.r[0]).wrapping_add(2u32), v as u8)?; }
                c.r[1] = c.rd32(c.r[4].wrapping_add(52u32))?;
            pc = 0x10735f30; continue;
            }
            0x10735f30 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[3] = c.incdec(c.r[3], 32, true);
                c.r[7] = c.alu(AluOp::Add, c.r[7], 4u32, 32);
                c.alu(AluOp::Sub, c.r[3], c.r[0], 32);
                if c.sf != c.of { pc = 0x10735cd0; continue; }
            pc = 0x10735f40; continue;
            }
            0x10735f40 => {
                c.r[5] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[7] = c.r[0];
            pc = 0x10735f46; continue;
            }
            0x10735f46 => {
                c.r[0] = c.rd32(c.r[2].wrapping_add(4u32))?;
                c.r[3] = c.rd32(c.r[0].wrapping_add(852u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[5] = c.alu(AluOp::Add, c.r[5], c.r[3], 32);
                c.r[3] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[3] = c.alu(AluOp::Add, c.r[3], c.r[0], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.r[0], 32);
                { let v = c.r[5]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                if c.sf != c.of { pc = 0x10735cc0; continue; }
            pc = 0x10735f74; continue;
            }
            0x10735f74 => {
                { let v = c.pop()?; c.r[3] = v; }
            pc = 0x10735f75; continue;
            }
            0x10735f75 => {
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.alu(AluOp::Add, c.r[4], 16u32, 32);
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10736430(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10736430;
    loop {
        match pc {
            0x10736430 => {
                c.push(c.r[5])?;
                c.r[5] = c.r[4];
                c.r[4] = c.alu(AluOp::And, c.r[4], 4294967288u32, 32);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 8u32, 32);
                c.push(c.r[6])?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::Sub, c.r[2], 4u32, 32);
                c.push(c.r[7])?;
                c.r[7] = c.r[0];
                { let v = 0u32; c.wr32(338497016u32, v)?; }
                { let v = 0u32; c.wr32(338497020u32, v)?; }
                { let v = 0u32; c.wr32(338496992u32, v)?; }
                { let v = 0u32; c.wr32(338496996u32, v)?; }
                { let v = 0u32; c.wr32(338496984u32, v)?; }
                { let v = 0u32; c.wr32(338496988u32, v)?; }
                { let v = 0u32; c.wr32(338496968u32, v)?; }
                { let v = 0u32; c.wr32(338496972u32, v)?; }
                { let v = 0u32; c.wr32(338496960u32, v)?; }
                { let v = 0u32; c.wr32(338496964u32, v)?; }
                { let v = 0u32; c.wr32(338496952u32, v)?; }
                { let v = 0u32; c.wr32(338496956u32, v)?; }
                if c.sf != c.of { pc = 0x107366aa; continue; }
            pc = 0x107364c0; continue;
            }
            0x107364c0 => {
                c.r[6] = 3u32;
                c.r[0] = c.r[7].wrapping_add(16u32);
            pc = 0x107364c8; continue;
            }
            0x107364c8 => {
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967280u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338496952u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496952u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967288u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967280u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496960u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496960u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0])?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967280u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496968u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496968u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967288u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338496984u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496984u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0])?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496992u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496992u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0])?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338497016u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497016u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(8u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338496952u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496952u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(16u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(8u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496960u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496960u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(24u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(8u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496968u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496968u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(16u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338496984u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496984u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(24u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(16u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496992u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496992u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(24u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338497016u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497016u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(32u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338496952u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496952u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(32u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(40u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496960u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496960u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(32u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(48u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496968u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496968u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(40u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338496984u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496984u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(48u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(40u32))?; c.farith_value(FOp::Mul, v); }
                c.r[6] = c.alu(AluOp::Add, c.r[6], 4u32, 32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 96u32, 32);
                c.alu(AluOp::Sub, c.r[6], c.r[2], 32);
                { let v = c.rdf64(338496992u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496992u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967248u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338497016u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497016u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967256u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338496952u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496952u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967256u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967264u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496960u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496960u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967256u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967272u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496968u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496968u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967264u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338496984u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496984u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967264u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496992u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496992u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967272u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338497016u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497016u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                if c.sf != c.of { pc = 0x107364c8; continue; }
            pc = 0x107366aa; continue;
            }
            0x107366aa => {
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                if c.sf == c.of { pc = 0x1073673b; continue; }
            pc = 0x107366b2; continue;
            }
            0x107366b2 => {
                c.r[0] = c.r[1].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[0] = c.r[7].wrapping_add(c.r[0].wrapping_mul(8)).wrapping_add(16u32);
                c.r[2] = c.alu(AluOp::Sub, c.r[2], c.r[1], 32);
                pc = 0x107366c0; continue;
            }
            0x107366c0 => {
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967280u32))?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 24u32, 32);
                c.r[2] = c.incdec(c.r[2], 32, false);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338496952u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496952u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967264u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967256u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496960u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496960u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967256u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496968u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496968u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967264u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338496984u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496984u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967264u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338496992u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496992u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967272u32))?);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338497016u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497016u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                if !c.zf { pc = 0x107366c0; continue; }
            pc = 0x1073673b; continue;
            }
            0x1073673b => {
                c.r[7] = 3u32;
            pc = 0x10736740; continue;
            }
            0x10736740 => {
                c.fpush(c.rdf64(338496952u32)?);
                c.r[2] = 338496952u32;
                { let v = c.rdf64(338496984u32)?; c.farith_value(FOp::Add, v); }
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.alu(AluOp::Sub, c.r[2], 338497048u32, 32);
                { let v = c.rdf64(338497016u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338496944u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(276864864u32)?);
                { let v = c.rdf64(338496944u32)?; c.farith_value(FOp::Div, v); }
                if c.sf == c.of { pc = 0x1073678e; continue; }
            pc = 0x10736773; continue;
            }
            0x10736773 => {
                c.r[1] = 3u32;
                c.r[0] = c.r[2];
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[6], 32);
                c.r[4] = c.r[4];
            pc = 0x10736780; continue;
            }
            0x10736780 => {
                c.fld_st(0);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[1] = c.incdec(c.r[1], 32, false);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967288u32), v)?; c.fpop(); }
                if !c.zf { pc = 0x10736780; continue; }
            pc = 0x1073678e; continue;
            }
            0x1073678e => {
                c.r[2] = c.alu(AluOp::Add, c.r[2], 32u32, 32);
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.alu(AluOp::Sub, c.r[2], 338497048u32, 32);
                if c.sf != c.of { pc = 0x10736773; continue; }
            pc = 0x1073679a; continue;
            }
            0x1073679a => {
                c.fst_st(0, true);
                c.r[0] = 4u32;
            pc = 0x107367a1; continue;
            }
            0x107367a1 => {
                c.fpush(c.rdf64(338496960u32)?);
                { let v = c.rdf64(338496960u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338496968u32)?);
                { let v = c.rdf64(338496968u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338496952u32)?);
                { let v = c.rdf64(338496952u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 0, 1, false);
                c.farith(FOp::Add, 0, 2, false);
                { let v = c.st0(); c.wrf64(338496872u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338496952u32)?);
                { let v = c.rdf64(338496984u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(338496960u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338496968u32)?);
                { let v = c.rdf64(338496992u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(338496880u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338496952u32)?);
                { let v = c.rdf64(338497016u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(338496968u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338496960u32)?);
                { let v = c.rdf64(338496992u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(338496888u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338496992u32)?);
                { let v = c.rdf64(338496992u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338496984u32)?);
                { let v = c.rdf64(338496984u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 0, 1, false);
                c.farith(FOp::Add, 0, 3, false);
                { let v = c.st0(); c.wrf64(338496904u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338496984u32)?);
                { let v = c.rdf64(338497016u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(338496992u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338496960u32)?);
                { let v = c.rdf64(338496968u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(338496912u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338497016u32)?);
                { let v = c.rdf64(338497016u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 0, 1, false);
                c.farith(FOp::Add, 0, 2, false);
                { let v = c.st0(); c.wrf64(338496936u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(338496888u32)?);
                { let v = c.rdf64(338496888u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338496880u32)?);
                { let v = c.rdf64(338496880u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338496872u32)?);
                { let v = c.rdf64(338496872u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 0, 1, false);
                c.farith(FOp::Add, 0, 2, false);
                { let v = c.st0(); c.wrf64(338496952u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338496904u32)?);
                { let v = c.rdf64(338496872u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(338496880u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338496912u32)?);
                { let v = c.rdf64(338496888u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(338496960u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338496936u32)?);
                { let v = c.rdf64(338496872u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(338496888u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338496912u32)?);
                { let v = c.rdf64(338496880u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(338496968u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338496912u32)?);
                { let v = c.rdf64(338496912u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(8u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(338496904u32)?);
                { let v = c.rdf64(338496904u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(8u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 1, false);
                { let v = c.st0(); c.wrf64(338496984u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(338496936u32)?);
                { let v = c.rdf64(338496904u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(338496912u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338496888u32)?);
                c.r[0] = c.incdec(c.r[0], 32, false);
                { let v = c.rdf64(338496880u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(338496992u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338496936u32)?);
                { let v = c.rdf64(338496936u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(8u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 1, false);
                { let v = c.st0(); c.wrf64(338497016u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                if !c.zf { pc = 0x107367a1; continue; }
            pc = 0x1073696a; continue;
            }
            0x1073696a => {
                c.r[7] = c.incdec(c.r[7], 32, false);
                if !c.zf { pc = 0x10736740; continue; }
            pc = 0x10736971; continue;
            }
            0x10736971 => {
                c.fpush(c.rdf64(338496952u32)?);
                c.fcom_value(c.rdf64(338496984u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.zf { pc = 0x1073699b; continue; }
            pc = 0x10736984; continue;
            }
            0x10736984 => {
                c.fpush(c.rdf64(338496952u32)?);
                c.fcom_value(c.rdf64(338497016u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.zf { pc = 0x107369b3; continue; }
            pc = 0x10736997; continue;
            }
            0x10736997 => {
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                pc = 0x107369b8; continue;
            }
            0x1073699b => {
                c.fpush(c.rdf64(338496984u32)?);
                c.r[1] = 1u32;
                c.fcom_value(c.rdf64(338497016u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.zf { pc = 0x107369b8; continue; }
            pc = 0x107369b3; continue;
            }
            0x107369b3 => {
                c.r[1] = 2u32;
            pc = 0x107369b8; continue;
            }
            0x107369b8 => {
                c.r[2] = c.r[1];
                c.r[2] = c.shift(ShiftOp::Shl, c.r[2], 5u32, 32);
                c.fpush(c.rdf64(c.r[2].wrapping_add(338496952u32))?);
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                c.fsqrt();
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Divr, v); }
                { let v = c.st0(); c.wrf64(338496864u32, v)?; c.fpop(); }
                if c.zf || c.sf != c.of { pc = 0x107369f9; continue; }
            pc = 0x107369d7; continue;
            }
            0x107369d7 => {
                c.r[2] = c.r[1].wrapping_mul(8).wrapping_add(338496872u32);
                c.r[7] = c.r[7];
            pc = 0x107369e0; continue;
            }
            0x107369e0 => {
                c.fpush(c.rdf64(338496864u32)?);
                c.r[0] = c.incdec(c.r[0], 32, true);
                { let v = c.rdf64(c.r[2])?; c.farith_value(FOp::Mul, v); }
                c.r[2] = c.alu(AluOp::Add, c.r[2], 24u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = c.st0(); c.wrf64(c.r[3].wrapping_add(c.r[0].wrapping_mul(8)).wrapping_add(4294967288u32), v)?; c.fpop(); }
                if c.sf != c.of { pc = 0x107369e0; continue; }
            pc = 0x107369f4; continue;
            }
            0x107369f4 => {
                c.alu(AluOp::Sub, c.r[0], 3u32, 32);
                if c.sf == c.of { pc = 0x10736a1a; continue; }
            pc = 0x107369f9; continue;
            }
            0x107369f9 => {
                c.r[2] = c.r[0].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[1] = c.alu(AluOp::Add, c.r[1], c.r[2], 32);
                c.r[1] = c.r[1].wrapping_mul(8).wrapping_add(338496872u32);
            pc = 0x10736a05; continue;
            }
            0x10736a05 => {
                c.fpush(c.rdf64(338496864u32)?);
                c.r[0] = c.incdec(c.r[0], 32, true);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Mul, v); }
                c.r[1] = c.alu(AluOp::Add, c.r[1], 8u32, 32);
                c.alu(AluOp::Sub, c.r[0], 3u32, 32);
                { let v = c.st0(); c.wrf64(c.r[3].wrapping_add(c.r[0].wrapping_mul(8)).wrapping_add(4294967288u32), v)?; c.fpop(); }
                if c.sf != c.of { pc = 0x10736a05; continue; }
            pc = 0x10736a1a; continue;
            }
            0x10736a1a => {
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10736a20(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10736a20;
    loop {
        match pc {
            0x10736a20 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(4u32))?;
                c.fpush(c.rdf64(276379832u32)?);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 8u32, 32);
                c.push(c.r[5])?;
                c.r[5] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.push(c.r[6])?;
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf || c.sf != c.of { pc = 0x10736c0d; continue; }
            pc = 0x10736a3d; continue;
            }
            0x10736a3d => {
                c.push(c.r[3])?;
                c.push(c.r[7])?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(32u32))?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[7] = c.alu(AluOp::Add, c.r[7], 16u32, 32);
                c.r[3] = c.r[3];
            pc = 0x10736a50; continue;
            }
            0x10736a50 => {
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(20u32))?, 4u32, 32);
                c.r[3] = c.r[6];
                if c.sf != c.of { pc = 0x10736b92; continue; }
            pc = 0x10736a5d; continue;
            }
            0x10736a5d => {
                c.r[2] = c.r[6].wrapping_add(3u32);
                c.r[1] = c.r[7];
            pc = 0x10736a62; continue;
            }
            0x10736a62 => {
                c.fpush(c.rdf64(c.r[7].wrapping_add(4294967280u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967280u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[7].wrapping_add(4294967288u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[7])?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(338497032u32, v)?;  }
                c.fst_st(3, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fcom(1, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.zf { pc = 0x10736aa5; continue; }
            pc = 0x10736a97; continue;
            }
            0x10736a97 => {
                c.fst_st(0, true);
                c.r[5] = c.r[6];
                c.fpush(c.rdf64(338497032u32)?);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
            pc = 0x10736aa5; continue;
            }
            0x10736aa5 => {
                c.fpush(c.rdf64(c.r[7].wrapping_add(4294967280u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[7].wrapping_add(4294967288u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(16u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[7])?);
                { let v = c.rdf64(c.r[1].wrapping_add(24u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(338497032u32, v)?;  }
                c.fst_st(3, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fcom(1, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.zf { pc = 0x10736aec; continue; }
            pc = 0x10736adb; continue;
            }
            0x10736adb => {
                c.r[0] = c.r[2].wrapping_add(4294967294u32);
                c.fst_st(0, true);
                c.fpush(c.rdf64(338497032u32)?);
                c.r[5] = c.r[6];
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
            pc = 0x10736aec; continue;
            }
            0x10736aec => {
                c.fpush(c.rdf64(c.r[7].wrapping_add(4294967280u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(32u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[7].wrapping_add(4294967288u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(40u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[7])?);
                { let v = c.rdf64(c.r[1].wrapping_add(48u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(338497032u32, v)?;  }
                c.fst_st(3, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fcom(1, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.zf { pc = 0x10736b33; continue; }
            pc = 0x10736b22; continue;
            }
            0x10736b22 => {
                c.r[0] = c.r[2].wrapping_add(4294967295u32);
                c.fst_st(0, true);
                c.fpush(c.rdf64(338497032u32)?);
                c.r[5] = c.r[6];
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
            pc = 0x10736b33; continue;
            }
            0x10736b33 => {
                c.fpush(c.rdf64(c.r[7].wrapping_add(4294967280u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(56u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[7].wrapping_add(4294967288u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(64u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[7])?);
                { let v = c.rdf64(c.r[1].wrapping_add(72u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(338497032u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(338497032u32)?);
                c.fcom(1, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.zf { pc = 0x10736b7d; continue; }
            pc = 0x10736b6f; continue;
            }
            0x10736b6f => {
                c.fst_st(0, true);
                c.r[5] = c.r[6];
                c.fpush(c.rdf64(338497032u32)?);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
            pc = 0x10736b7d; continue;
            }
            0x10736b7d => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[2] = c.alu(AluOp::Add, c.r[2], 4u32, 32);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 4u32, 32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 96u32, 32);
                c.alu(AluOp::Sub, c.r[2], c.r[0], 32);
                if c.sf != c.of { pc = 0x10736a62; continue; }
            pc = 0x10736b92; continue;
            }
            0x10736b92 => {
                c.alu(AluOp::Sub, c.r[3], c.r[0], 32);
                if c.sf == c.of { pc = 0x10736bf6; continue; }
            pc = 0x10736b96; continue;
            }
            0x10736b96 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[1] = c.r[3].wrapping_add(c.r[3].wrapping_mul(2));
                c.r[1] = c.r[2].wrapping_add(c.r[1].wrapping_mul(8)).wrapping_add(16u32);
            pc = 0x10736ba1; continue;
            }
            0x10736ba1 => {
                c.fpush(c.rdf64(c.r[7].wrapping_add(4294967280u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967280u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[7].wrapping_add(4294967288u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[7])?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(338497032u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(338497032u32)?);
                c.fcom(1, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.zf { pc = 0x10736bea; continue; }
            pc = 0x10736bdc; continue;
            }
            0x10736bdc => {
                c.fst_st(0, true);
                c.r[5] = c.r[6];
                c.fpush(c.rdf64(338497032u32)?);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
            pc = 0x10736bea; continue;
            }
            0x10736bea => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[3] = c.incdec(c.r[3], 32, true);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 24u32, 32);
                c.alu(AluOp::Sub, c.r[3], c.r[0], 32);
                if c.sf != c.of { pc = 0x10736ba1; continue; }
            pc = 0x10736bf6; continue;
            }
            0x10736bf6 => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.r[3] = c.incdec(c.r[3], 32, false);
                c.r[7] = c.alu(AluOp::Add, c.r[7], 24u32, 32);
                c.alu(AluOp::Sub, c.r[6], c.r[0], 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                if c.sf != c.of { pc = 0x10736a50; continue; }
            pc = 0x10736c0b; continue;
            }
            0x10736c0b => {
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[3] = v; }
            pc = 0x10736c0d; continue;
            }
            0x10736c0d => {
                c.fsqrt();
                c.r[2] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(8u32))?;
                c.r[0] = c.r[5].wrapping_add(c.r[5].wrapping_mul(2));
                c.r[0] = c.r[2].wrapping_add(c.r[0].wrapping_mul(8));
                c.r[1] = c.r[1].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[1] = c.r[2].wrapping_add(c.r[1].wrapping_mul(8));
                c.r[2] = c.rd32(c.r[4].wrapping_add(28u32))?;
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[5] = v; }
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Divr, v); }
                { let v = c.st0(); c.wrf64(338497024u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0])?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                { let v = c.rdf64(338497024u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[2], v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(8u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                { let v = c.rdf64(338497024u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[2].wrapping_add(8u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(16u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(16u32))?; c.farith_value(FOp::Sub, v); }
                { let v = c.rdf64(338497024u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[2].wrapping_add(16u32), v)?; c.fpop(); }
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10736c70(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10736c70;
    loop {
        match pc {
            0x10736c70 => {
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 72u32, 32);
                c.push(c.r[3])?;
                c.push(c.r[5])?;
                c.r[5] = c.rd32(c.r[4].wrapping_add(84u32))?;
                c.push(c.r[6])?;
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.alu(AluOp::And, c.r[5], c.r[5], 32);
                c.push(c.r[7])?;
                c.r[3] = c.r[0];
                if c.zf || c.sf != c.of { pc = 0x10736cb7; continue; }
            pc = 0x10736c83; continue;
            }
            0x10736c83 => {
                c.r[1] = c.r[5];
                c.r[0] = 1u32;
                c.r[7] = c.r[4].wrapping_add(24u32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 8u32, 32);
                c.rep_stos(4)?;
            pc = 0x10736c93; continue;
            }
            0x10736c93 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(8u32))?);
                c.r[6] = c.incdec(c.r[6], 32, true);
                { let v = c.rdf64(c.r[3].wrapping_add(16u32))?; c.farith_value(FOp::Mul, v); }
                c.r[2] = c.alu(AluOp::Add, c.r[2], 24u32, 32);
                c.alu(AluOp::Sub, c.r[6], c.r[5], 32);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967264u32))?);
                { let v = c.rdf64(c.r[3])?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[3].wrapping_add(8u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_mul(8).wrapping_add(338497032u32), v)?; c.fpop(); }
                if c.sf != c.of { pc = 0x10736c93; continue; }
            pc = 0x10736cb7; continue;
            }
            0x10736cb7 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(100u32))?;
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(100u32), v)?; }
                if c.zf { pc = 0x10736e2f; continue; }
            pc = 0x10736cc9; continue;
            }
            0x10736cc9 => {
                c.alu(AluOp::And, c.r[5], c.r[5], 32);
                if c.zf || c.sf != c.of { pc = 0x10736f97; continue; }
            pc = 0x10736cd1; continue;
            }
            0x10736cd1 => {
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
            pc = 0x10736cd3; continue;
            }
            0x10736cd3 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(24u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if !c.zf { pc = 0x10736ce0; continue; }
            pc = 0x10736cdb; continue;
            }
            0x10736cdb => {
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.r[5], 32);
                if c.sf != c.of { pc = 0x10736cd3; continue; }
            pc = 0x10736ce0; continue;
            }
            0x10736ce0 => {
                c.r[7] = c.r[1];
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.r[0] = c.r[5];
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                c.alu(AluOp::Sub, c.r[0], 4u32, 32);
                if c.sf != c.of { pc = 0x10736dd2; continue; }
            pc = 0x10736cf0; continue;
            }
            0x10736cf0 => {
                c.r[0] = 16u32;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], 338497048u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[0] = 24u32;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], 338497048u32, 32);
                c.r[6] = c.r[1].wrapping_add(3u32);
                c.r[2] = c.r[7].wrapping_mul(8);
                c.r[5] = c.r[1].wrapping_mul(8).wrapping_add(8u32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
            pc = 0x10736d1d; continue;
            }
            0x10736d1d => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(24u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10736d43; continue; }
            pc = 0x10736d25; continue;
            }
            0x10736d25 => {
                c.r[3] = c.r[1].wrapping_mul(8);
                c.fpush(c.rdf64(c.r[3].wrapping_add(338497040u32))?);
                c.fcom_value(c.rdf64(c.r[2].wrapping_add(338497040u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.pf { pc = 0x10736d43; continue; }
            pc = 0x10736d3f; continue;
            }
            0x10736d3f => {
                c.r[7] = c.r[1];
                c.r[2] = c.r[3];
            pc = 0x10736d43; continue;
            }
            0x10736d43 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(28u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10736d64; continue; }
            pc = 0x10736d4b; continue;
            }
            0x10736d4b => {
                c.fpush(c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497048u32))?);
                c.fcom_value(c.rdf64(c.r[2].wrapping_add(338497040u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.pf { pc = 0x10736d64; continue; }
            pc = 0x10736d5f; continue;
            }
            0x10736d5f => {
                c.r[7] = c.r[6].wrapping_add(4294967294u32);
                c.r[2] = c.r[5];
            pc = 0x10736d64; continue;
            }
            0x10736d64 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(32u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10736d8e; continue; }
            pc = 0x10736d6c; continue;
            }
            0x10736d6c => {
                c.fpush(c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497056u32))?);
                c.fcom_value(c.rdf64(c.r[2].wrapping_add(338497040u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.pf { pc = 0x10736d8e; continue; }
            pc = 0x10736d80; continue;
            }
            0x10736d80 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[7] = c.r[6].wrapping_add(4294967295u32);
                c.r[2] = c.r[2].wrapping_add(c.r[1].wrapping_mul(8)).wrapping_add(338497048u32);
            pc = 0x10736d8e; continue;
            }
            0x10736d8e => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(36u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10736db7; continue; }
            pc = 0x10736d96; continue;
            }
            0x10736d96 => {
                c.fpush(c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497064u32))?);
                c.fcom_value(c.rdf64(c.r[2].wrapping_add(338497040u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.pf { pc = 0x10736db7; continue; }
            pc = 0x10736daa; continue;
            }
            0x10736daa => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[7] = c.r[6];
                c.r[2] = c.r[0].wrapping_add(c.r[1].wrapping_mul(8)).wrapping_add(338497048u32);
            pc = 0x10736db7; continue;
            }
            0x10736db7 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(92u32))?;
                c.r[6] = c.alu(AluOp::Add, c.r[6], 4u32, 32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[5] = c.alu(AluOp::Add, c.r[5], 32u32, 32);
                c.alu(AluOp::Sub, c.r[6], c.r[0], 32);
                if c.sf != c.of { pc = 0x10736d1d; continue; }
            pc = 0x10736dcc; continue;
            }
            0x10736dcc => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(100u32))?;
                c.r[5] = c.r[0];
            pc = 0x10736dd2; continue;
            }
            0x10736dd2 => {
                c.alu(AluOp::Sub, c.r[1], c.r[5], 32);
                if c.sf == c.of { pc = 0x10736e0b; continue; }
            pc = 0x10736dd6; continue;
            }
            0x10736dd6 => {
                c.r[6] = c.r[7].wrapping_mul(8);
                c.r[1] = c.r[1];
            pc = 0x10736de0; continue;
            }
            0x10736de0 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(24u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10736e06; continue; }
            pc = 0x10736de8; continue;
            }
            0x10736de8 => {
                c.r[2] = c.r[1].wrapping_mul(8);
                c.fpush(c.rdf64(c.r[2].wrapping_add(338497040u32))?);
                c.fcom_value(c.rdf64(c.r[6].wrapping_add(338497040u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.pf { pc = 0x10736e06; continue; }
            pc = 0x10736e02; continue;
            }
            0x10736e02 => {
                c.r[7] = c.r[1];
                c.r[6] = c.r[2];
            pc = 0x10736e06; continue;
            }
            0x10736e06 => {
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.r[5], 32);
                if c.sf != c.of { pc = 0x10736de0; continue; }
            pc = 0x10736e0b; continue;
            }
            0x10736e0b => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(96u32))?;
                { let v = c.r[7]; c.wr32(c.r[1].wrapping_add(c.r[3].wrapping_mul(4)), v)?; }
                c.r[3] = c.incdec(c.r[3], 32, true);
                c.alu(AluOp::Sub, c.r[3], c.r[5], 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(c.r[7].wrapping_mul(4)).wrapping_add(24u32), v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(100u32), v)?; }
                if c.sf != c.of { pc = 0x10736cd1; continue; }
            pc = 0x10736e27; continue;
            }
            0x10736e27 => {
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[5] = v; }
                { let v = c.pop()?; c.r[3] = v; }
                c.r[4] = c.alu(AluOp::Add, c.r[4], 72u32, 32);
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            0x10736e2f => {
                c.alu(AluOp::And, c.r[5], c.r[5], 32);
                if c.zf || c.sf != c.of { pc = 0x10736f97; continue; }
            pc = 0x10736e37; continue;
            }
            0x10736e37 => {
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.r[4] = c.r[4];
            pc = 0x10736e40; continue;
            }
            0x10736e40 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(24u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if !c.zf { pc = 0x10736e4d; continue; }
            pc = 0x10736e48; continue;
            }
            0x10736e48 => {
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.r[5], 32);
                if c.sf != c.of { pc = 0x10736e40; continue; }
            pc = 0x10736e4d; continue;
            }
            0x10736e4d => {
                c.r[7] = c.r[1];
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.r[2] = c.r[5];
                c.r[2] = c.alu(AluOp::Sub, c.r[2], c.r[1], 32);
                c.alu(AluOp::Sub, c.r[2], 4u32, 32);
                if c.sf != c.of { pc = 0x10736f3f; continue; }
            pc = 0x10736e5d; continue;
            }
            0x10736e5d => {
                c.r[0] = 16u32;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], 338497048u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[0] = 24u32;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], 338497048u32, 32);
                c.r[6] = c.r[1].wrapping_add(3u32);
                c.r[2] = c.r[7].wrapping_mul(8);
                c.r[5] = c.r[1].wrapping_mul(8).wrapping_add(8u32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
            pc = 0x10736e8a; continue;
            }
            0x10736e8a => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(24u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10736eb0; continue; }
            pc = 0x10736e92; continue;
            }
            0x10736e92 => {
                c.r[3] = c.r[1].wrapping_mul(8);
                c.fpush(c.rdf64(c.r[3].wrapping_add(338497040u32))?);
                c.fcom_value(c.rdf64(c.r[2].wrapping_add(338497040u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x10736eb0; continue; }
            pc = 0x10736eac; continue;
            }
            0x10736eac => {
                c.r[7] = c.r[1];
                c.r[2] = c.r[3];
            pc = 0x10736eb0; continue;
            }
            0x10736eb0 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(28u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10736ed1; continue; }
            pc = 0x10736eb8; continue;
            }
            0x10736eb8 => {
                c.fpush(c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497048u32))?);
                c.fcom_value(c.rdf64(c.r[2].wrapping_add(338497040u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x10736ed1; continue; }
            pc = 0x10736ecc; continue;
            }
            0x10736ecc => {
                c.r[7] = c.r[6].wrapping_add(4294967294u32);
                c.r[2] = c.r[5];
            pc = 0x10736ed1; continue;
            }
            0x10736ed1 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(32u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10736efb; continue; }
            pc = 0x10736ed9; continue;
            }
            0x10736ed9 => {
                c.fpush(c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497056u32))?);
                c.fcom_value(c.rdf64(c.r[2].wrapping_add(338497040u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x10736efb; continue; }
            pc = 0x10736eed; continue;
            }
            0x10736eed => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[7] = c.r[6].wrapping_add(4294967295u32);
                c.r[2] = c.r[0].wrapping_add(c.r[1].wrapping_mul(8)).wrapping_add(338497048u32);
            pc = 0x10736efb; continue;
            }
            0x10736efb => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(36u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10736f24; continue; }
            pc = 0x10736f03; continue;
            }
            0x10736f03 => {
                c.fpush(c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497064u32))?);
                c.fcom_value(c.rdf64(c.r[2].wrapping_add(338497040u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x10736f24; continue; }
            pc = 0x10736f17; continue;
            }
            0x10736f17 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[7] = c.r[6];
                c.r[2] = c.r[2].wrapping_add(c.r[1].wrapping_mul(8)).wrapping_add(338497048u32);
            pc = 0x10736f24; continue;
            }
            0x10736f24 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(92u32))?;
                c.r[6] = c.alu(AluOp::Add, c.r[6], 4u32, 32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[5] = c.alu(AluOp::Add, c.r[5], 32u32, 32);
                c.alu(AluOp::Sub, c.r[6], c.r[0], 32);
                if c.sf != c.of { pc = 0x10736e8a; continue; }
            pc = 0x10736f39; continue;
            }
            0x10736f39 => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(100u32))?;
                c.r[5] = c.r[0];
            pc = 0x10736f3f; continue;
            }
            0x10736f3f => {
                c.alu(AluOp::Sub, c.r[1], c.r[5], 32);
                if c.sf == c.of { pc = 0x10736f7b; continue; }
            pc = 0x10736f43; continue;
            }
            0x10736f43 => {
                c.r[6] = c.r[7].wrapping_mul(8);
                c.r[3] = c.r[3];
            pc = 0x10736f50; continue;
            }
            0x10736f50 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(24u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10736f76; continue; }
            pc = 0x10736f58; continue;
            }
            0x10736f58 => {
                c.r[2] = c.r[1].wrapping_mul(8);
                c.fpush(c.rdf64(c.r[2].wrapping_add(338497040u32))?);
                c.fcom_value(c.rdf64(c.r[6].wrapping_add(338497040u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x10736f76; continue; }
            pc = 0x10736f72; continue;
            }
            0x10736f72 => {
                c.r[7] = c.r[1];
                c.r[6] = c.r[2];
            pc = 0x10736f76; continue;
            }
            0x10736f76 => {
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.alu(AluOp::Sub, c.r[1], c.r[5], 32);
                if c.sf != c.of { pc = 0x10736f50; continue; }
            pc = 0x10736f7b; continue;
            }
            0x10736f7b => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(96u32))?;
                { let v = c.r[7]; c.wr32(c.r[0].wrapping_add(c.r[3].wrapping_mul(4)), v)?; }
                c.r[3] = c.incdec(c.r[3], 32, true);
                c.alu(AluOp::Sub, c.r[3], c.r[5], 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(c.r[7].wrapping_mul(4)).wrapping_add(24u32), v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(100u32), v)?; }
                if c.sf != c.of { pc = 0x10736e37; continue; }
            pc = 0x10736f97; continue;
            }
            0x10736f97 => {
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[5] = v; }
                { let v = c.pop()?; c.r[3] = v; }
                c.r[4] = c.alu(AluOp::Add, c.r[4], 72u32, 32);
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10736fa0(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10736fa0;
    loop {
        match pc {
            0x10736fa0 => {
                c.push(c.r[7])?;
                c.r[7] = c.r[0];
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf || c.sf != c.of { pc = 0x10736fe8; continue; }
            pc = 0x10736fa9; continue;
            }
            0x10736fa9 => {
                c.push(c.r[5])?;
                c.push(c.r[6])?;
                c.r[6] = c.r[3];
                c.r[2] = c.r[7];
                c.r[6] = c.alu(AluOp::Sub, c.r[6], c.r[7], 32);
            pc = 0x10736fb1; continue;
            }
            0x10736fb1 => {
                c.r[5] = c.rd32(c.r[6].wrapping_add(c.r[2]))?;
                c.alu(AluOp::Sub, c.r[5], c.rd32(c.r[2])?, 32);
                if !c.zf { pc = 0x10736fcd; continue; }
            pc = 0x10736fb8; continue;
            }
            0x10736fb8 => {
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 4u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                if c.sf != c.of { pc = 0x10736fb1; continue; }
            pc = 0x10736fc0; continue;
            }
            0x10736fc0 => {
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(2, 0, v); }
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[5] = v; }
                { let v = c.pop()?; c.r[7] = v; }
                c.r[0] = c.r[2];
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            0x10736fcd => {
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                if c.sf == c.of { pc = 0x10736fe6; continue; }
            pc = 0x10736fd1; continue;
            }
            0x10736fd1 => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.r[2] = c.r[7].wrapping_add(c.r[1].wrapping_mul(4)).wrapping_add(4294967292u32);
            pc = 0x10736fd7; continue;
            }
            0x10736fd7 => {
                c.r[6] = c.rd32(c.r[3].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.alu(AluOp::Sub, c.r[6], c.rd32(c.r[2])?, 32);
                if !c.zf { pc = 0x10736fe6; continue; }
            pc = 0x10736fde; continue;
            }
            0x10736fde => {
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[2] = c.alu(AluOp::Sub, c.r[2], 4u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                if c.sf != c.of { pc = 0x10736fd7; continue; }
            pc = 0x10736fe6; continue;
            }
            0x10736fe6 => {
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[5] = v; }
            pc = 0x10736fe8; continue;
            }
            0x10736fe8 => {
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(2, 0, v); }
                { let v = c.pop()?; c.r[7] = v; }
                c.r[0] = c.r[2];
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10737000(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10737000;
    loop {
        match pc {
            0x10737000 => {
                c.push(c.r[5])?;
                c.r[5] = c.r[4];
                c.r[4] = c.alu(AluOp::And, c.r[4], 4294967288u32, 32);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 44u32, 32);
                c.push(c.r[3])?;
                c.r[3] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.push(c.r[6])?;
                c.r[6] = c.r[0];
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                c.push(c.r[7])?;
                if c.zf || c.sf != c.of { pc = 0x107370c3; continue; }
            pc = 0x1073701b; continue;
            }
            0x1073701b => {
                c.r[1] = 338497432u32;
            pc = 0x10737020; continue;
            }
            0x10737020 => {
                c.r[7] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[7] = c.r[7].wrapping_add(c.r[7].wrapping_mul(2));
                c.r[3] = c.rd32(c.r[3].wrapping_add(c.r[7].wrapping_mul(8)))?;
                { let v = c.r[3]; c.wr32(c.r[1].wrapping_add(4294967288u32), v)?; }
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[7] = c.rd32(c.r[3].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4u32))?;
                { let v = c.r[7]; c.wr32(c.r[1].wrapping_add(4294967292u32), v)?; }
                c.r[7] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[3] = c.rd32(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)))?;
                { let v = c.r[3]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338497296u32), v)?; }
                c.r[7] = c.rd32(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4u32))?;
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                { let v = c.r[7]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338497300u32), v)?; }
                c.r[7] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[7] = c.r[7].wrapping_add(c.r[7].wrapping_mul(2));
                c.r[3] = c.rd32(c.r[3].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(8u32))?;
                { let v = c.r[3]; c.wr32(c.r[1], v)?; }
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[7] = c.rd32(c.r[3].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(12u32))?;
                { let v = c.r[7]; c.wr32(c.r[1].wrapping_add(4u32), v)?; }
                c.r[7] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[3] = c.rd32(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)))?;
                { let v = c.r[3]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338497296u32), v)?; }
                c.r[7] = c.rd32(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4u32))?;
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                { let v = c.r[7]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338497300u32), v)?; }
                c.r[7] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[7] = c.r[7].wrapping_add(c.r[7].wrapping_mul(2));
                c.r[3] = c.rd32(c.r[3].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(16u32))?;
                { let v = c.r[3]; c.wr32(c.r[1].wrapping_add(8u32), v)?; }
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[7] = c.rd32(c.r[3].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(20u32))?;
                { let v = c.r[7]; c.wr32(c.r[1].wrapping_add(12u32), v)?; }
                c.r[7] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[3] = c.rd32(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)))?;
                { let v = c.r[3]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338497296u32), v)?; }
                c.r[7] = c.rd32(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4u32))?;
                c.r[3] = c.rd32(c.r[5].wrapping_add(12u32))?;
                { let v = c.r[7]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338497300u32), v)?; }
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 24u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[3], 32);
                if c.sf != c.of { pc = 0x10737020; continue; }
            pc = 0x107370c3; continue;
            }
            0x107370c3 => {
                c.r[0] = c.r[3].wrapping_add(c.r[3].wrapping_mul(2));
                c.r[6] = c.r[0].wrapping_mul(8).wrapping_add(338497432u32);
                { let v = 0u32; c.wr32(338497272u32, v)?; }
                { let v = 0u32; c.wr32(338497276u32, v)?; }
                { let v = 0u32; c.wr32(338497248u32, v)?; }
                { let v = 0u32; c.wr32(338497252u32, v)?; }
                { let v = 0u32; c.wr32(338497192u32, v)?; }
                { let v = 0u32; c.wr32(338497196u32, v)?; }
                { let v = 0u32; c.wr32(338497184u32, v)?; }
                { let v = 0u32; c.wr32(338497188u32, v)?; }
                { let v = 0u32; c.wr32(338497176u32, v)?; }
                { let v = 0u32; c.wr32(338497180u32, v)?; }
                { let v = 0u32; c.wr32(338497280u32, v)?; }
                { let v = 0u32; c.wr32(338497284u32, v)?; }
                { let v = 0u32; c.wr32(338497256u32, v)?; }
                { let v = 0u32; c.wr32(338497260u32, v)?; }
                { let v = 0u32; c.wr32(338497216u32, v)?; }
                { let v = 0u32; c.wr32(338497220u32, v)?; }
                { let v = 0u32; c.wr32(338497208u32, v)?; }
                { let v = 0u32; c.wr32(338497212u32, v)?; }
                { let v = 0u32; c.wr32(338497200u32, v)?; }
                { let v = 0u32; c.wr32(338497204u32, v)?; }
                { let v = 0u32; c.wr32(338497288u32, v)?; }
                { let v = 0u32; c.wr32(338497292u32, v)?; }
                { let v = 0u32; c.wr32(338497264u32, v)?; }
                { let v = 0u32; c.wr32(338497268u32, v)?; }
                { let v = 0u32; c.wr32(338497240u32, v)?; }
                { let v = 0u32; c.wr32(338497244u32, v)?; }
                { let v = 0u32; c.wr32(338497232u32, v)?; }
                { let v = 0u32; c.wr32(338497236u32, v)?; }
                { let v = 0u32; c.wr32(338497224u32, v)?; }
                { let v = 0u32; c.wr32(338497228u32, v)?; }
                c.r[7] = c.r[3];
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
            pc = 0x10737203; continue;
            }
            0x10737203 => {
                c.r[2] = c.r[3];
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[4] = c.r[4];
            pc = 0x10737210; continue;
            }
            0x10737210 => {
                c.r[1] = c.r[3];
            pc = 0x10737212; continue;
            }
            0x10737212 => {
                c.fpush(c.rdf64(338497224u32)?);
                { let v = c.rdf64(338497224u32)?; c.farith_value(FOp::Mul, v); }
                c.fchs();
                c.fpush(c.rdf64(338497232u32)?);
                { let v = c.rdf64(338497232u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Sub, 1, 0, true);
                c.fpush(c.rdf64(338497240u32)?);
                { let v = c.rdf64(338497240u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Sub, 1, 0, true);
                c.fpush(c.rdf64(338497168u32)?);
                { let v = c.rdf64(338497264u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(278724192u32)?);
                c.farith(FOp::Mul, 0, 2, false);
                c.fcompp();
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.pf { pc = 0x10737286; continue; }
            pc = 0x10737259; continue;
            }
            0x10737259 => {
                c.r[3] = c.rd32(338497268u32)?;
                { let v = c.st0(); c.wrf64(338497168u32, v)?; c.fpop(); }
                c.r[0] = c.rd32(338497264u32)?;
                { let v = c.r[3]; c.wr32(278724196u32, v)?; }
                c.r[3] = c.rd32(c.r[5].wrapping_add(12u32))?;
                { let v = c.r[0]; c.wr32(278724192u32, v)?; }
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                pc = 0x10737288; continue;
            }
            0x10737286 => {
                c.fst_st(0, true);
            pc = 0x10737288; continue;
            }
            0x10737288 => {
                c.r[1] = c.incdec(c.r[1], 32, false);
                c.r[6] = c.alu(AluOp::Sub, c.r[6], 24u32, 32);
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                if c.sf != c.of { pc = 0x10737304; continue; }
            pc = 0x10737290; continue;
            }
            0x10737290 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.alu(AluOp::Sub, c.rd32(c.r[0])?, 3u32, 32);
                if c.sf != c.of { pc = 0x10737304; continue; }
            pc = 0x10737298; continue;
            }
            0x10737298 => {
                c.fpush(c.rdf64(338497224u32)?);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967288u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497224u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338497232u32)?);
                { let v = c.rdf64(c.r[6])?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497232u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338497240u32)?);
                { let v = c.rdf64(c.r[6].wrapping_add(8u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497240u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338497288u32)?);
                c.farith(FOp::Add, 0, 0, false);
                { let v = c.rdf64(276702320u32)?; c.farith_value(FOp::Subr, v); }
                { let v = c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497296u32))?; c.farith_value(FOp::Sub, v); }
                { let v = c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497296u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497264u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497264u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338497288u32)?);
                { let v = c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497296u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497288u32, v)?; c.fpop(); }
                pc = 0x10737212; continue;
            }
            0x10737304 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.incdec(c.r[2], 32, false);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], 24u32, 32);
                c.alu(AluOp::Sub, c.r[2], c.r[7], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.sf != c.of { pc = 0x107373fa; continue; }
            pc = 0x10737318; continue;
            }
            0x10737318 => {
                c.fpush(c.rdf64(338497200u32)?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967288u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497200u32, v)?; c.fpop(); }
                c.r[1] = c.rd32(338497200u32)?;
                c.fpush(c.rdf64(338497208u32)?);
                c.r[6] = c.rd32(338497204u32)?;
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Add, v); }
                { let v = c.r[1]; c.wr32(338497224u32, v)?; }
                { let v = c.r[6]; c.wr32(338497228u32, v)?; }
                { let v = c.st0(); c.wrf64(338497208u32, v)?; c.fpop(); }
                c.r[1] = c.rd32(338497208u32)?;
                c.fpush(c.rdf64(338497216u32)?);
                c.r[6] = c.rd32(338497212u32)?;
                { let v = c.rdf64(c.r[0].wrapping_add(8u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.r[1]; c.wr32(338497232u32, v)?; }
                { let v = c.r[6]; c.wr32(338497236u32, v)?; }
                c.r[6] = c.rd32(c.r[4].wrapping_add(32u32))?;
                { let v = c.st0(); c.wrf64(338497216u32, v)?; c.fpop(); }
                c.r[0] = c.rd32(338497216u32)?;
                c.fpush(c.rdf64(338497280u32)?);
                c.r[1] = c.rd32(338497220u32)?;
                c.farith(FOp::Add, 0, 0, false);
                { let v = c.r[0]; c.wr32(338497240u32, v)?; }
                { let v = c.r[1]; c.wr32(338497244u32, v)?; }
                { let v = c.rdf64(276812232u32)?; c.farith_value(FOp::Subr, v); }
                { let v = c.rdf64(c.r[2].wrapping_mul(8).wrapping_add(338497296u32))?; c.farith_value(FOp::Sub, v); }
                { let v = c.rdf64(c.r[2].wrapping_mul(8).wrapping_add(338497296u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497256u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497256u32, v)?; c.fpop(); }
                c.r[0] = c.rd32(338497256u32)?;
                c.fpush(c.rdf64(338497280u32)?);
                c.r[1] = c.rd32(338497260u32)?;
                { let v = c.rdf64(c.r[2].wrapping_mul(8).wrapping_add(338497296u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.r[0]; c.wr32(338497264u32, v)?; }
                { let v = c.r[1]; c.wr32(338497268u32, v)?; }
                { let v = c.st0(); c.wrf64(338497280u32, v)?; c.fpop(); }
                c.r[0] = c.rd32(338497280u32)?;
                c.r[1] = c.rd32(338497284u32)?;
                { let v = c.r[0]; c.wr32(338497288u32, v)?; }
                { let v = c.r[1]; c.wr32(338497292u32, v)?; }
                pc = 0x10737210; continue;
            }
            0x107373fa => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[7] = c.incdec(c.r[7], 32, false);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], 24u32, 32);
                c.alu(AluOp::Sub, c.r[7], 1u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.sf != c.of { pc = 0x10737539; continue; }
            pc = 0x1073740f; continue;
            }
            0x1073740f => {
                c.fpush(c.rdf64(338497176u32)?);
                c.r[6] = c.rd32(c.r[4].wrapping_add(32u32))?;
                { let v = c.rdf64(c.r[0].wrapping_add(4294967288u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497176u32, v)?; c.fpop(); }
                c.r[2] = c.rd32(338497176u32)?;
                c.fpush(c.rdf64(338497184u32)?);
                c.r[1] = c.rd32(338497180u32)?;
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Add, v); }
                { let v = c.r[2]; c.wr32(338497200u32, v)?; }
                { let v = c.r[2]; c.wr32(338497224u32, v)?; }
                { let v = c.r[1]; c.wr32(338497204u32, v)?; }
                { let v = c.st0(); c.wrf64(338497184u32, v)?; c.fpop(); }
                c.r[2] = c.rd32(338497184u32)?;
                c.fpush(c.rdf64(338497192u32)?);
                { let v = c.r[1]; c.wr32(338497228u32, v)?; }
                { let v = c.rdf64(c.r[0].wrapping_add(8u32))?; c.farith_value(FOp::Add, v); }
                c.r[1] = c.rd32(338497188u32)?;
                { let v = c.r[2]; c.wr32(338497208u32, v)?; }
                { let v = c.r[2]; c.wr32(338497232u32, v)?; }
                { let v = c.st0(); c.wrf64(338497192u32, v)?; c.fpop(); }
                c.r[2] = c.rd32(338497192u32)?;
                c.fpush(c.rdf64(338497272u32)?);
                c.r[0] = c.rd32(338497196u32)?;
                c.farith(FOp::Add, 0, 0, false);
                { let v = c.r[1]; c.wr32(338497212u32, v)?; }
                { let v = c.r[1]; c.wr32(338497236u32, v)?; }
                c.r[1] = c.r[2];
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Subr, v); }
                { let v = c.r[2]; c.wr32(338497216u32, v)?; }
                c.r[2] = c.r[0];
                { let v = c.r[0]; c.wr32(338497220u32, v)?; }
                { let v = c.rdf64(c.r[7].wrapping_mul(8).wrapping_add(338497296u32))?; c.farith_value(FOp::Sub, v); }
                { let v = c.r[1]; c.wr32(338497240u32, v)?; }
                { let v = c.r[2]; c.wr32(338497244u32, v)?; }
                { let v = c.rdf64(c.r[7].wrapping_mul(8).wrapping_add(338497296u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497248u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497248u32, v)?; c.fpop(); }
                c.r[0] = c.rd32(338497248u32)?;
                c.fpush(c.rdf64(338497272u32)?);
                c.r[1] = c.rd32(338497252u32)?;
                { let v = c.rdf64(c.r[7].wrapping_mul(8).wrapping_add(338497296u32))?; c.farith_value(FOp::Add, v); }
                c.r[2] = c.r[0];
                { let v = c.r[0]; c.wr32(338497256u32, v)?; }
                c.r[0] = c.r[1];
                { let v = c.st0(); c.wrf64(338497272u32, v)?; c.fpop(); }
                { let v = c.r[1]; c.wr32(338497260u32, v)?; }
                c.r[1] = c.rd32(338497272u32)?;
                { let v = c.r[2]; c.wr32(338497264u32, v)?; }
                c.r[2] = c.rd32(338497276u32)?;
                { let v = c.r[0]; c.wr32(338497268u32, v)?; }
                c.r[0] = c.r[1];
                { let v = c.r[1]; c.wr32(338497280u32, v)?; }
                c.r[1] = c.r[2];
                { let v = c.r[2]; c.wr32(338497284u32, v)?; }
                { let v = c.r[0]; c.wr32(338497288u32, v)?; }
                { let v = c.r[1]; c.wr32(338497292u32, v)?; }
                pc = 0x10737203; continue;
            }
            0x10737539 => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::Sub, c.r[7], c.r[6], 32);
                { let v = if !c.zf { 1u32 } else { 0u32 }; c.set8(0, 0, v); }
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[1], c.r[7], 32);
                { let v = if !c.zf { 1u32 } else { 0u32 }; c.set8(2, 0, v); }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[2], 32);
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[6], c.r[3], 32);
                { let v = if !c.zf { 1u32 } else { 0u32 }; c.set8(2, 0, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[0], 1u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if !c.zf { pc = 0x10737585; continue; }
            pc = 0x1073756b; continue;
            }
            0x1073756b => {
                c.alu(AluOp::Sub, c.r[6], c.r[3], 32);
                if c.zf { pc = 0x10737585; continue; }
            pc = 0x1073756f; continue;
            }
            0x1073756f => {
                c.alu(AluOp::Sub, c.r[7], c.r[6], 32);
                if !c.zf { pc = 0x10737579; continue; }
            pc = 0x10737573; continue;
            }
            0x10737573 => {
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                pc = 0x10737585; continue;
            }
            0x10737579 => {
                c.alu(AluOp::Sub, c.r[1], c.r[7], 32);
                if !c.zf { pc = 0x10737585; continue; }
            pc = 0x1073757d; continue;
            }
            0x1073757d => {
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
            pc = 0x10737585; continue;
            }
            0x10737585 => {
                c.fpush(c.rdf64(276379832u32)?);
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.fpush(c.rdf64(276379832u32)?);
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = c.st0(); c.wrf64(338497288u32, v)?;  }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                c.fpush(c.rdf64(276379832u32)?);
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                { let v = c.st0(); c.wrf64(338497240u32, v)?;  }
                c.fpush(c.rdf64(276379832u32)?);
                { let v = c.st0(); c.wrf64(338497232u32, v)?;  }
                c.fpush(c.rdf64(276379832u32)?);
                { let v = c.st0(); c.wrf64(338497224u32, v)?;  }
            pc = 0x107375c5; continue;
            }
            0x107375c5 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(c.r[6].wrapping_mul(4)).wrapping_add(40u32))?;
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                if c.sf == c.of { pc = 0x10737610; continue; }
            pc = 0x107375cd; continue;
            }
            0x107375cd => {
                c.fpush(c.rd32(c.r[4].wrapping_add(32u32))? as i32 as f64);
                c.r[0] = c.r[1].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[0] = c.r[0].wrapping_mul(8).wrapping_add(338497432u32);
                pc = 0x107375e0; continue;
            }
            0x107375e0 => {
                c.fld_st(0);
                c.r[1] = c.incdec(c.r[1], 32, true);
                { let v = c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497288u32))?; c.farith_value(FOp::Mul, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 24u32, 32);
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                c.fld_st(0);
                c.farith(FOp::Add, 6, 0, true);
                c.farith(FOp::Mul, 0, 1, false);
                c.farith(FOp::Add, 6, 0, true);
                c.fld_st(0);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967264u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 2, 0, true);
                c.fld_st(0);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967272u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 3, 0, true);
                c.fld_st(0);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967280u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 4, 0, true);
                if c.sf != c.of { pc = 0x107375e0; continue; }
            pc = 0x1073760e; continue;
            }
            0x1073760e => {
                c.fst_st(0, true);
            pc = 0x10737610; continue;
            }
            0x10737610 => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::Sub, c.r[2], c.r[3], 32);
                { let v = if !c.zf { 1u32 } else { 0u32 }; c.set8(0, 0, v); }
                c.r[7] = c.alu(AluOp::Add, c.r[7], c.r[0], 32);
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.alu(AluOp::Sub, c.r[6], 4u32, 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                if c.sf != c.of { pc = 0x107375c5; continue; }
            pc = 0x10737623; continue;
            }
            0x10737623 => {
                { let v = c.st0(); c.wrf64(338497224u32, v)?;  }
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                c.fxch(3);
                c.r[1] = c.rd32(c.r[5].wrapping_add(20u32))?;
                { let v = c.st0(); c.wrf64(338497288u32, v)?;  }
                c.r[0] = c.rd32(c.r[5].wrapping_add(24u32))?;
                c.fst_st(3, true);
                { let v = c.st0(); c.wrf64(338497232u32, v)?; c.fpop(); }
                { let v = c.st0(); c.wrf64(338497240u32, v)?; c.fpop(); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                c.farith(FOp::Subr, 0, 2, false);
                { let v = c.st0(); c.wrf64(338497264u32, v)?; c.fpop(); }
                c.fld_st(0);
                c.fchs();
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(16u32), v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[4].wrapping_add(32u32))? as i32 as f64);
                c.farith(FOp::Sub, 0, 1, false);
                c.fst_st(2, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(338497224u32)?);
                { let v = c.rdf64(338497224u32)?; c.farith_value(FOp::Mul, v); }
                c.fchs();
                c.fpush(c.rdf64(338497232u32)?);
                { let v = c.rdf64(338497232u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Sub, 1, 0, true);
                c.fpush(c.rdf64(338497240u32)?);
                { let v = c.rdf64(338497240u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Sub, 1, 0, true);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(32u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(338497224u32)?);
                { let v = c.rdf64(c.r[4].wrapping_add(16u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497264u32)?; c.farith_value(FOp::Div, v); }
                { let v = c.st0(); c.wrf64(c.r[1], v)?; c.fpop(); }
                c.fpush(c.rdf64(338497224u32)?);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338497264u32)?; c.farith_value(FOp::Div, v); }
                { let v = c.st0(); c.wrf64(c.r[1].wrapping_add(24u32), v)?; c.fpop(); }
                c.r[2] = c.rd32(338497224u32)?;
                c.r[6] = c.rd32(338497228u32)?;
                { let v = c.r[2]; c.wr32(c.r[0], v)?; }
                { let v = c.r[6]; c.wr32(c.r[0].wrapping_add(4u32), v)?; }
                c.fpush(c.rdf64(338497232u32)?);
                { let v = c.rdf64(c.r[4].wrapping_add(16u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497264u32)?; c.farith_value(FOp::Div, v); }
                { let v = c.st0(); c.wrf64(c.r[1].wrapping_add(8u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(338497232u32)?);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338497264u32)?; c.farith_value(FOp::Div, v); }
                { let v = c.st0(); c.wrf64(c.r[1].wrapping_add(32u32), v)?; c.fpop(); }
                c.r[2] = c.rd32(338497232u32)?;
                c.r[6] = c.rd32(338497236u32)?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(8u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[0].wrapping_add(12u32), v)?; }
                c.fpush(c.rdf64(338497240u32)?);
                { let v = c.rdf64(c.r[4].wrapping_add(16u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497264u32)?; c.farith_value(FOp::Div, v); }
                { let v = c.st0(); c.wrf64(c.r[1].wrapping_add(16u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(338497240u32)?);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338497264u32)?; c.farith_value(FOp::Div, v); }
                { let v = c.st0(); c.wrf64(c.r[1].wrapping_add(40u32), v)?; c.fpop(); }
                c.r[1] = c.rd32(338497240u32)?;
                c.r[2] = c.rd32(338497244u32)?;
                { let v = c.r[1]; c.wr32(c.r[0].wrapping_add(16u32), v)?; }
                c.fst_st(0, true);
                c.r[1] = c.rd32(c.r[4].wrapping_add(28u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(20u32), v)?; }
                c.r[0] = 1u32;
                c.alu(AluOp::Sub, c.r[1], c.r[0], 32);
                if c.zf { pc = 0x10737746; continue; }
            pc = 0x10737744; continue;
            }
            0x10737744 => {
                c.r[0] = c.r[7];
            pc = 0x10737746; continue;
            }
            0x10737746 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.fpush(c.rdf64(c.r[4].wrapping_add(32u32))?);
                { let v = c.r[0]; c.wr32(c.r[1], v)?; }
                { let v = c.rdf64(338497264u32)?; c.farith_value(FOp::Div, v); }
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[3] = v; }
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10737760(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10737760;
    loop {
        match pc {
            0x10737760 => {
                c.push(c.r[5])?;
                c.r[5] = c.r[4];
                c.r[4] = c.alu(AluOp::And, c.r[4], 4294967288u32, 32);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 140u32, 32);
                c.r[1] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.push(c.r[3])?;
                c.push(c.r[6])?;
                c.r[6] = c.r[0];
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::Sub, c.r[1], c.r[0], 32);
                c.push(c.r[7])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073782c; continue; }
            pc = 0x10737782; continue;
            }
            0x10737782 => {
                c.r[1] = 338498072u32;
            pc = 0x10737787; continue;
            }
            0x10737787 => {
                c.r[7] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[7] = c.r[7].wrapping_add(c.r[7].wrapping_mul(2));
                c.r[3] = c.rd32(c.r[3].wrapping_add(c.r[7].wrapping_mul(8)))?;
                { let v = c.r[3]; c.wr32(c.r[1].wrapping_add(4294967288u32), v)?; }
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[7] = c.rd32(c.r[3].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4u32))?;
                { let v = c.r[7]; c.wr32(c.r[1].wrapping_add(4294967292u32), v)?; }
                c.r[7] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[3] = c.rd32(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)))?;
                { let v = c.r[3]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338497936u32), v)?; }
                c.r[7] = c.rd32(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4u32))?;
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                { let v = c.r[7]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338497940u32), v)?; }
                c.r[7] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[7] = c.r[7].wrapping_add(c.r[7].wrapping_mul(2));
                c.r[3] = c.rd32(c.r[3].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(8u32))?;
                { let v = c.r[3]; c.wr32(c.r[1], v)?; }
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[7] = c.rd32(c.r[3].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(12u32))?;
                { let v = c.r[7]; c.wr32(c.r[1].wrapping_add(4u32), v)?; }
                c.r[7] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[3] = c.rd32(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)))?;
                { let v = c.r[3]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338497936u32), v)?; }
                c.r[7] = c.rd32(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4u32))?;
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                { let v = c.r[7]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338497940u32), v)?; }
                c.r[7] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[7] = c.r[7].wrapping_add(c.r[7].wrapping_mul(2));
                c.r[3] = c.rd32(c.r[3].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(16u32))?;
                { let v = c.r[3]; c.wr32(c.r[1].wrapping_add(8u32), v)?; }
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[7] = c.rd32(c.r[3].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(20u32))?;
                { let v = c.r[7]; c.wr32(c.r[1].wrapping_add(12u32), v)?; }
                c.r[7] = c.rd32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)))?;
                c.r[3] = c.rd32(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)))?;
                { let v = c.r[3]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338497936u32), v)?; }
                c.r[7] = c.rd32(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4u32))?;
                { let v = c.r[7]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338497940u32), v)?; }
                c.r[7] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 24u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[7], 32);
                if c.sf != c.of { pc = 0x10737787; continue; }
            pc = 0x1073782a; continue;
            }
            0x1073782a => {
                c.r[1] = c.r[7];
            pc = 0x1073782c; continue;
            }
            0x1073782c => {
                c.r[7] = c.rd32(c.r[5].wrapping_add(20u32))?;
                c.r[0] = c.r[1].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[6] = c.r[0].wrapping_mul(8).wrapping_add(338498072u32);
                c.r[3] = c.r[1];
                { let v = 0u32; c.wr32(338497912u32, v)?; }
                { let v = 0u32; c.wr32(338497916u32, v)?; }
                { let v = 0u32; c.wr32(338497888u32, v)?; }
                { let v = 0u32; c.wr32(338497892u32, v)?; }
                { let v = 0u32; c.wr32(338497832u32, v)?; }
                { let v = 0u32; c.wr32(338497836u32, v)?; }
                { let v = 0u32; c.wr32(338497824u32, v)?; }
                { let v = 0u32; c.wr32(338497828u32, v)?; }
                { let v = 0u32; c.wr32(338497816u32, v)?; }
                { let v = 0u32; c.wr32(338497820u32, v)?; }
                { let v = 0u32; c.wr32(338497920u32, v)?; }
                { let v = 0u32; c.wr32(338497924u32, v)?; }
                { let v = 0u32; c.wr32(338497896u32, v)?; }
                { let v = 0u32; c.wr32(338497900u32, v)?; }
                { let v = 0u32; c.wr32(338497856u32, v)?; }
                { let v = 0u32; c.wr32(338497860u32, v)?; }
                { let v = 0u32; c.wr32(338497848u32, v)?; }
                { let v = 0u32; c.wr32(338497852u32, v)?; }
                { let v = 0u32; c.wr32(338497840u32, v)?; }
                { let v = 0u32; c.wr32(338497844u32, v)?; }
                { let v = 0u32; c.wr32(338497928u32, v)?; }
                { let v = 0u32; c.wr32(338497932u32, v)?; }
                { let v = 0u32; c.wr32(338497904u32, v)?; }
                { let v = 0u32; c.wr32(338497908u32, v)?; }
                { let v = 0u32; c.wr32(338497880u32, v)?; }
                { let v = 0u32; c.wr32(338497884u32, v)?; }
                { let v = 0u32; c.wr32(338497872u32, v)?; }
                { let v = 0u32; c.wr32(338497876u32, v)?; }
                { let v = 0u32; c.wr32(338497864u32, v)?; }
                { let v = 0u32; c.wr32(338497868u32, v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                c.r[7] = c.alu(AluOp::Sub, c.r[7], 338497864u32, 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                c.r[4] = c.r[4];
            pc = 0x10737980; continue;
            }
            0x10737980 => {
                c.r[2] = c.r[1];
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                pc = 0x10737990; continue;
            }
            0x10737990 => {
                c.fpush(c.rdf64(338497928u32)?);
                c.r[0] = 338497864u32;
                c.fchs();
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(80u32), v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[4].wrapping_add(16u32))? as i32 as f64);
                c.fpush(c.rdf64(338497928u32)?);
                c.farith(FOp::Subr, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(88u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(80u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(80u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(104u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(88u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(80u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(120u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(88u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(88u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(128u32), v)?;  }
                { let v = c.rdf64(c.r[4].wrapping_add(104u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(96u32), v)?; c.fpop(); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(96u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(96u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(64u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(276379832u32)?);
            pc = 0x10737a10; continue;
            }
            0x10737a10 => {
                c.fpush(c.rdf64(338497904u32)?);
                { let v = c.rdf64(c.r[7].wrapping_add(c.r[0]))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338497904u32)?);
                { let v = c.rdf64(c.r[7].wrapping_add(c.r[0]).wrapping_add(24u32))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(80u32))?);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(88u32))?);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(72u32), v)?; c.fpop(); }
                c.fld_st(2);
                c.farith(FOp::Sub, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(16u32), v)?; c.fpop(); }
                c.fld_st(1);
                c.farith(FOp::Sub, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(56u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fxch(1);
                { let v = c.rdf64(c.r[4].wrapping_add(72u32))?; c.farith_value(FOp::Sub, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(104u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(112u32), v)?; c.fpop(); }
                { let v = c.rdf64(c.r[4].wrapping_add(72u32))?; c.farith_value(FOp::Sub, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(104u32))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(56u32))?);
                c.fabs();
                { let v = c.rdf64(c.r[4].wrapping_add(56u32))?; c.farith_value(FOp::Subr, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(16u32))?);
                c.fabs();
                { let v = c.rdf64(c.r[4].wrapping_add(16u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(120u32))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(16u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(96u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(16u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(56u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(96u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(56u32), v)?; c.fpop(); }
                c.fld_st(1);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fpush(c.rdf64(c.r[4].wrapping_add(112u32))?);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                { let v = c.rdf64(c.r[4].wrapping_add(112u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 2, 0, true);
                c.fxch(1);
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Mul, v); }
                c.fxch(1);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[4].wrapping_add(120u32))?);
                c.farith(FOp::Mul, 0, 1, false);
                c.fxch(1);
                { let v = c.rdf64(c.r[4].wrapping_add(128u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(72u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(56u32))?);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                { let v = c.rdf64(c.r[4].wrapping_add(56u32))?; c.farith_value(FOp::Subr, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(16u32))?);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                { let v = c.rdf64(c.r[4].wrapping_add(16u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Mul, v); }
                c.fxch(1);
                c.fst_st(0, true);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.alu(AluOp::Sub, c.r[0], 338497888u32, 32);
                c.fpush(c.rdf64(c.r[4].wrapping_add(72u32))?);
                c.farith(FOp::Subr, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(16u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(72u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(80u32))?; c.farith_value(FOp::Mul, v); }
                c.fxch(1);
                { let v = c.rdf64(c.r[4].wrapping_add(88u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Sub, 1, 0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967288u32))?);
                c.fpush(c.rdf64(338497904u32)?);
                { let v = c.rdf64(c.r[4].wrapping_add(16u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(16u32))?; c.farith_value(FOp::Mul, v); }
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(1);
                c.farith(FOp::Mul, 0, 2, false);
                { let v = c.rdf64(c.r[4].wrapping_add(64u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 3, 0, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                if c.sf != c.of { pc = 0x10737a10; continue; }
            pc = 0x10737b3b; continue;
            }
            0x10737b3b => {
                c.fpush(c.rdf64(338497904u32)?);
                { let v = c.rdf64(c.r[4].wrapping_add(64u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(64u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(338497808u32)?);
                { let v = c.rdf64(c.r[4].wrapping_add(64u32))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(278724200u32)?);
                c.farith(FOp::Mul, 0, 2, false);
                c.fcompp();
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.pf { pc = 0x10737b98; continue; }
            pc = 0x10737b64; continue;
            }
            0x10737b64 => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(68u32))?;
                { let v = c.st0(); c.wrf64(338497808u32, v)?; c.fpop(); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(64u32))?;
                { let v = c.r[3]; c.wr32(278724204u32, v)?; }
                c.r[3] = c.rd32(c.r[4].wrapping_add(32u32))?;
                { let v = c.r[0]; c.wr32(278724200u32, v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(136u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(140u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(144u32), v)?; }
                pc = 0x10737b9a; continue;
            }
            0x10737b98 => {
                c.fst_st(0, true);
            pc = 0x10737b9a; continue;
            }
            0x10737b9a => {
                c.r[1] = c.incdec(c.r[1], 32, false);
                c.r[6] = c.alu(AluOp::Sub, c.r[6], 24u32, 32);
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                if c.sf != c.of { pc = 0x10737c1e; continue; }
            pc = 0x10737ba2; continue;
            }
            0x10737ba2 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.alu(AluOp::Sub, c.rd32(c.r[0])?, 3u32, 32);
                if c.sf != c.of { pc = 0x10737c1e; continue; }
            pc = 0x10737baa; continue;
            }
            0x10737baa => {
                c.fpush(c.rdf64(338497864u32)?);
                { let v = 3u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.rdf64(c.r[6].wrapping_add(4294967288u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497864u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338497872u32)?);
                { let v = c.rdf64(c.r[6])?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497872u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338497880u32)?);
                { let v = c.rdf64(c.r[6].wrapping_add(8u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497880u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338497928u32)?);
                c.farith(FOp::Add, 0, 0, false);
                { let v = c.rdf64(276702320u32)?; c.farith_value(FOp::Subr, v); }
                { let v = c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497936u32))?; c.farith_value(FOp::Sub, v); }
                { let v = c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497936u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497904u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(338497928u32)?);
                { let v = c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497936u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497928u32, v)?; c.fpop(); }
                pc = 0x10737990; continue;
            }
            0x10737c1e => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[2] = c.incdec(c.r[2], 32, false);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], 24u32, 32);
                c.alu(AluOp::Sub, c.r[2], c.r[3], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.sf != c.of { pc = 0x10737d1f; continue; }
            pc = 0x10737c32; continue;
            }
            0x10737c32 => {
                c.fpush(c.rdf64(338497840u32)?);
                { let v = 2u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.rdf64(c.r[0].wrapping_add(4294967288u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497840u32, v)?; c.fpop(); }
                c.r[1] = c.rd32(338497840u32)?;
                c.fpush(c.rdf64(338497848u32)?);
                c.r[6] = c.rd32(338497844u32)?;
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Add, v); }
                { let v = c.r[1]; c.wr32(338497864u32, v)?; }
                { let v = c.r[6]; c.wr32(338497868u32, v)?; }
                { let v = c.st0(); c.wrf64(338497848u32, v)?; c.fpop(); }
                c.r[1] = c.rd32(338497848u32)?;
                c.fpush(c.rdf64(338497856u32)?);
                c.r[6] = c.rd32(338497852u32)?;
                { let v = c.rdf64(c.r[0].wrapping_add(8u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.r[1]; c.wr32(338497872u32, v)?; }
                { let v = c.r[6]; c.wr32(338497876u32, v)?; }
                c.r[6] = c.rd32(c.r[4].wrapping_add(48u32))?;
                { let v = c.st0(); c.wrf64(338497856u32, v)?; c.fpop(); }
                c.r[1] = c.rd32(338497860u32)?;
                c.fpush(c.rdf64(338497920u32)?);
                c.r[0] = c.rd32(338497856u32)?;
                c.farith(FOp::Add, 0, 0, false);
                { let v = c.r[1]; c.wr32(338497884u32, v)?; }
                { let v = c.r[0]; c.wr32(338497880u32, v)?; }
                { let v = c.rdf64(276812232u32)?; c.farith_value(FOp::Subr, v); }
                { let v = c.rdf64(c.r[2].wrapping_mul(8).wrapping_add(338497936u32))?; c.farith_value(FOp::Sub, v); }
                { let v = c.rdf64(c.r[2].wrapping_mul(8).wrapping_add(338497936u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497896u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497896u32, v)?; c.fpop(); }
                c.r[1] = c.rd32(338497900u32)?;
                c.fpush(c.rdf64(338497920u32)?);
                c.r[0] = c.rd32(338497896u32)?;
                { let v = c.rdf64(c.r[2].wrapping_mul(8).wrapping_add(338497936u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.r[1]; c.wr32(338497908u32, v)?; }
                { let v = c.r[0]; c.wr32(338497904u32, v)?; }
                { let v = c.st0(); c.wrf64(338497920u32, v)?; c.fpop(); }
                c.r[1] = c.rd32(338497924u32)?;
                c.r[0] = c.rd32(338497920u32)?;
                { let v = c.r[1]; c.wr32(338497932u32, v)?; }
                c.r[1] = c.rd32(c.r[5].wrapping_add(12u32))?;
                { let v = c.r[0]; c.wr32(338497928u32, v)?; }
                pc = 0x10737990; continue;
            }
            0x10737d1f => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[3] = c.incdec(c.r[3], 32, false);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], 24u32, 32);
                c.alu(AluOp::Sub, c.r[3], 1u32, 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                if c.sf != c.of { pc = 0x10737e6d; continue; }
            pc = 0x10737d38; continue;
            }
            0x10737d38 => {
                c.fpush(c.rdf64(338497816u32)?);
                c.r[6] = c.rd32(c.r[4].wrapping_add(48u32))?;
                { let v = c.rdf64(c.r[0].wrapping_add(4294967288u32))?; c.farith_value(FOp::Add, v); }
                { let v = 1u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.st0(); c.wrf64(338497816u32, v)?; c.fpop(); }
                c.r[2] = c.rd32(338497816u32)?;
                c.fpush(c.rdf64(338497824u32)?);
                c.r[1] = c.rd32(338497820u32)?;
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Add, v); }
                { let v = c.r[2]; c.wr32(338497840u32, v)?; }
                { let v = c.r[2]; c.wr32(338497864u32, v)?; }
                { let v = c.r[1]; c.wr32(338497844u32, v)?; }
                { let v = c.st0(); c.wrf64(338497824u32, v)?; c.fpop(); }
                c.r[2] = c.rd32(338497824u32)?;
                c.fpush(c.rdf64(338497832u32)?);
                { let v = c.r[1]; c.wr32(338497868u32, v)?; }
                { let v = c.rdf64(c.r[0].wrapping_add(8u32))?; c.farith_value(FOp::Add, v); }
                c.r[1] = c.rd32(338497828u32)?;
                { let v = c.r[2]; c.wr32(338497848u32, v)?; }
                { let v = c.r[2]; c.wr32(338497872u32, v)?; }
                { let v = c.st0(); c.wrf64(338497832u32, v)?; c.fpop(); }
                c.r[2] = c.rd32(338497832u32)?;
                c.fpush(c.rdf64(338497912u32)?);
                c.r[0] = c.rd32(338497836u32)?;
                c.farith(FOp::Add, 0, 0, false);
                { let v = c.r[1]; c.wr32(338497852u32, v)?; }
                { let v = c.r[1]; c.wr32(338497876u32, v)?; }
                c.r[1] = c.r[2];
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Subr, v); }
                { let v = c.r[2]; c.wr32(338497856u32, v)?; }
                c.r[2] = c.r[0];
                { let v = c.r[0]; c.wr32(338497860u32, v)?; }
                { let v = c.rdf64(c.r[3].wrapping_mul(8).wrapping_add(338497936u32))?; c.farith_value(FOp::Sub, v); }
                { let v = c.r[1]; c.wr32(338497880u32, v)?; }
                { let v = c.r[2]; c.wr32(338497884u32, v)?; }
                { let v = c.rdf64(c.r[3].wrapping_mul(8).wrapping_add(338497936u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497888u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(338497888u32, v)?; c.fpop(); }
                c.r[0] = c.rd32(338497888u32)?;
                c.fpush(c.rdf64(338497912u32)?);
                c.r[1] = c.rd32(338497892u32)?;
                { let v = c.rdf64(c.r[3].wrapping_mul(8).wrapping_add(338497936u32))?; c.farith_value(FOp::Add, v); }
                c.r[2] = c.r[0];
                { let v = c.r[0]; c.wr32(338497896u32, v)?; }
                c.r[0] = c.r[1];
                { let v = c.r[1]; c.wr32(338497900u32, v)?; }
                { let v = c.st0(); c.wrf64(338497912u32, v)?; c.fpop(); }
                c.r[1] = c.rd32(338497912u32)?;
                { let v = c.r[2]; c.wr32(338497904u32, v)?; }
                c.r[2] = c.rd32(338497916u32)?;
                { let v = c.r[0]; c.wr32(338497908u32, v)?; }
                c.r[0] = c.r[1];
                { let v = c.r[1]; c.wr32(338497920u32, v)?; }
                c.r[1] = c.r[2];
                { let v = c.r[1]; c.wr32(338497932u32, v)?; }
                c.r[1] = c.rd32(c.r[5].wrapping_add(12u32))?;
                { let v = c.r[2]; c.wr32(338497924u32, v)?; }
                { let v = c.r[0]; c.wr32(338497928u32, v)?; }
                pc = 0x10737980; continue;
            }
            0x10737e6d => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(140u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(144u32))?;
                c.r[1] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[6], c.r[0], 32);
                { let v = if !c.zf { 1u32 } else { 0u32 }; c.set8(2, 0, v); }
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = if !c.zf { 1u32 } else { 0u32 }; c.set8(3, 0, v); }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(148u32), v)?; }
                c.r[2] = c.alu(AluOp::Add, c.r[2], c.r[3], 32);
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(136u32))?, c.r[6], 32);
                { let v = if !c.zf { 1u32 } else { 0u32 }; c.set8(3, 0, v); }
                c.r[2] = c.alu(AluOp::Add, c.r[2], c.r[3], 32);
                c.alu(AluOp::Sub, c.r[2], 1u32, 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                if !c.zf { pc = 0x10737ed6; continue; }
            pc = 0x10737eac; continue;
            }
            0x10737eac => {
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                if c.zf { pc = 0x10737ed6; continue; }
            pc = 0x10737eb0; continue;
            }
            0x10737eb0 => {
                c.alu(AluOp::Sub, c.r[6], c.r[0], 32);
                if !c.zf { pc = 0x10737ebd; continue; }
            pc = 0x10737eb4; continue;
            }
            0x10737eb4 => {
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(144u32), v)?; }
                pc = 0x10737ed6; continue;
            }
            0x10737ebd => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(136u32))?;
                c.alu(AluOp::Sub, c.r[0], c.r[6], 32);
                if !c.zf { pc = 0x10737ed6; continue; }
            pc = 0x10737ec8; continue;
            }
            0x10737ec8 => {
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(148u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(144u32), v)?; }
            pc = 0x10737ed6; continue;
            }
            0x10737ed6 => {
                c.fpush(c.rdf64(276379832u32)?);
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.fpush(c.rdf64(276379832u32)?);
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = c.st0(); c.wrf64(338497928u32, v)?;  }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                c.fpush(c.rdf64(276379832u32)?);
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                { let v = c.st0(); c.wrf64(338497880u32, v)?;  }
                c.fpush(c.rdf64(276379832u32)?);
                { let v = c.st0(); c.wrf64(338497872u32, v)?;  }
                c.fpush(c.rdf64(276379832u32)?);
                { let v = c.st0(); c.wrf64(338497864u32, v)?;  }
                pc = 0x10737f20; continue;
            }
            0x10737f20 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(c.r[6].wrapping_mul(4)).wrapping_add(136u32))?;
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                if c.sf == c.of { pc = 0x10737f70; continue; }
            pc = 0x10737f2b; continue;
            }
            0x10737f2b => {
                c.fpush(c.rd32(c.r[4].wrapping_add(48u32))? as i32 as f64);
                c.r[0] = c.r[1].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[0] = c.r[0].wrapping_mul(8).wrapping_add(338498072u32);
                c.r[4] = c.r[4];
            pc = 0x10737f40; continue;
            }
            0x10737f40 => {
                c.fld_st(0);
                c.r[1] = c.incdec(c.r[1], 32, true);
                { let v = c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338497928u32))?; c.farith_value(FOp::Mul, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 24u32, 32);
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                c.fld_st(0);
                c.farith(FOp::Add, 6, 0, true);
                c.farith(FOp::Mul, 0, 1, false);
                c.farith(FOp::Add, 6, 0, true);
                c.fld_st(0);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967264u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 2, 0, true);
                c.fld_st(0);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967272u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 3, 0, true);
                c.fld_st(0);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967280u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 4, 0, true);
                if c.sf != c.of { pc = 0x10737f40; continue; }
            pc = 0x10737f6e; continue;
            }
            0x10737f6e => {
                c.fst_st(0, true);
            pc = 0x10737f70; continue;
            }
            0x10737f70 => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::Sub, c.r[2], c.rd32(c.r[5].wrapping_add(12u32))?, 32);
                { let v = if !c.zf { 1u32 } else { 0u32 }; c.set8(0, 0, v); }
                c.r[3] = c.alu(AluOp::Add, c.r[3], c.r[0], 32);
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.alu(AluOp::Sub, c.r[6], 4u32, 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                if c.sf != c.of { pc = 0x10737f20; continue; }
            pc = 0x10737f84; continue;
            }
            0x10737f84 => {
                { let v = c.st0(); c.wrf64(338497864u32, v)?;  }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.fxch(3);
                c.r[2] = c.rd32(c.r[5].wrapping_add(24u32))?;
                { let v = c.st0(); c.wrf64(338497928u32, v)?;  }
                c.r[6] = c.rd32(c.r[5].wrapping_add(20u32))?;
                c.fst_st(3, true);
                c.r[1] = c.r[2].wrapping_add(24u32);
                { let v = c.st0(); c.wrf64(338497872u32, v)?; c.fpop(); }
                c.r[6] = c.alu(AluOp::Sub, c.r[6], c.r[2], 32);
                c.r[2] = c.rd32(c.r[5].wrapping_add(28u32))?;
                { let v = c.st0(); c.wrf64(338497880u32, v)?; c.fpop(); }
                c.r[0] = 338497864u32;
                c.r[2] = c.alu(AluOp::Sub, c.r[2], c.r[0], 32);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                c.farith(FOp::Subr, 0, 2, false);
                { let v = c.st0(); c.wrf64(338497904u32, v)?; c.fpop(); }
                c.fld_st(0);
                c.fchs();
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(80u32), v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[4].wrapping_add(16u32))? as i32 as f64);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(104u32), v)?;  }
                c.farith(FOp::Sub, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(32u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[4].wrapping_add(80u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(80u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(96u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(32u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(80u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(120u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(32u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(32u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(112u32), v)?;  }
                { let v = c.rdf64(c.r[4].wrapping_add(96u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(88u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(104u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(104u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(88u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(88u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(276379832u32)?);
                c.fpush(c.rdf64(276379816u32)?);
                { let v = c.rdf64(c.r[4].wrapping_add(88u32))?; c.farith_value(FOp::Div, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(104u32), v)?; c.fpop(); }
                pc = 0x10738050; continue;
            }
            0x10738050 => {
                c.fpush(c.rdf64(338497904u32)?);
                { let v = c.rdf64(c.r[7].wrapping_add(c.r[0]))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(338497904u32)?);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[1]))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(80u32))?);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(72u32), v)?;  }
                c.fpush(c.rdf64(c.r[4].wrapping_add(32u32))?);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(48u32), v)?; c.fpop(); }
                c.fld_st(2);
                c.farith(FOp::Sub, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(56u32), v)?; c.fpop(); }
                c.fld_st(1);
                c.farith(FOp::Sub, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(64u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fxch(1);
                { let v = c.rdf64(c.r[4].wrapping_add(48u32))?; c.farith_value(FOp::Sub, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(96u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(128u32), v)?; c.fpop(); }
                { let v = c.rdf64(c.r[4].wrapping_add(48u32))?; c.farith_value(FOp::Sub, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(96u32))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(64u32))?);
                c.fabs();
                { let v = c.rdf64(c.r[4].wrapping_add(64u32))?; c.farith_value(FOp::Subr, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(56u32))?);
                c.fabs();
                { let v = c.rdf64(c.r[4].wrapping_add(56u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(120u32))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(56u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(88u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(56u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(64u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(88u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(64u32), v)?; c.fpop(); }
                c.fld_st(1);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fpush(c.rdf64(c.r[4].wrapping_add(128u32))?);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                { let v = c.rdf64(c.r[4].wrapping_add(128u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 2, 0, true);
                c.fxch(1);
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Mul, v); }
                c.fxch(1);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[4].wrapping_add(120u32))?);
                c.farith(FOp::Mul, 0, 1, false);
                c.fxch(1);
                { let v = c.rdf64(c.r[4].wrapping_add(112u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(48u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(64u32))?);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                { let v = c.rdf64(c.r[4].wrapping_add(64u32))?; c.farith_value(FOp::Subr, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(56u32))?);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                { let v = c.rdf64(c.r[4].wrapping_add(56u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Mul, v); }
                c.fxch(1);
                c.fst_st(0, true);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.fpush(c.rdf64(c.r[4].wrapping_add(48u32))?);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 8u32, 32);
                c.alu(AluOp::Sub, c.r[0], 338497888u32, 32);
                c.farith(FOp::Subr, 0, 1, false);
                c.fpush(c.rdf64(c.r[4].wrapping_add(48u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(80u32))?; c.farith_value(FOp::Mul, v); }
                c.fld_st(2);
                { let v = c.rdf64(c.r[4].wrapping_add(32u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Sub, 1, 0, true);
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967288u32))?);
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Mul, v); }
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(6);
                c.farith(FOp::Mul, 0, 2, false);
                c.farith(FOp::Mul, 0, 2, false);
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 5, 0, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                { let v = c.rdf64(c.r[4].wrapping_add(104u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(c.r[4].wrapping_add(72u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Div, v); }
                { let v = c.st0(); c.wrf64(c.r[1].wrapping_add(4294967264u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(48u32))?);
                { let v = c.rdf64(c.r[4].wrapping_add(104u32))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[4].wrapping_add(32u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Div, v); }
                { let v = c.st0(); c.wrf64(c.r[1].wrapping_add(4294967288u32), v)?; c.fpop(); }
                c.r[3] = c.rd32(c.r[0].wrapping_add(4294967288u32))?;
                { let v = c.r[3]; c.wr32(c.r[2].wrapping_add(c.r[0]).wrapping_add(4294967288u32), v)?; }
                c.r[3] = c.rd32(c.r[0].wrapping_add(4294967292u32))?;
                { let v = c.r[3]; c.wr32(c.r[2].wrapping_add(c.r[0]).wrapping_add(4294967292u32), v)?; }
                if c.sf != c.of { pc = 0x10738050; continue; }
            pc = 0x107381b8; continue;
            }
            0x107381b8 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[0] = 1u32;
                c.alu(AluOp::Sub, c.r[1], c.r[0], 32);
                if c.zf { pc = 0x107381c9; continue; }
            pc = 0x107381c5; continue;
            }
            0x107381c5 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
            pc = 0x107381c9; continue;
            }
            0x107381c9 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.fxch(1);
                { let v = c.r[0]; c.wr32(c.r[1], v)?; }
                { let v = c.rdf64(338497904u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                c.farith(FOp::Divr, 0, 1, false);
                { let v = c.pop()?; c.r[3] = v; }
                c.fxch(1);
                c.fst_st(0, true);
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_107381f0(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x107381f0;
    loop {
        match pc {
            0x107381f0 => {
                c.push(c.r[5])?;
                c.r[5] = c.r[4];
                c.r[4] = c.alu(AluOp::And, c.r[4], 4294967288u32, 32);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 244u32, 32);
                c.r[1] = c.rd32(c.r[5].wrapping_add(24u32))?;
                { let v = c.neg(((c.r[0] >> 0) & 0xff), 8); c.set8(0, 0, v); }
                c.push(c.r[3])?;
                c.push(c.r[6])?;
                c.push(c.r[7])?;
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                { let v = 278724176u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[0] = c.alu(AluOp::Sbb, c.r[0], c.r[0], 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 6u32, 32);
                { let v = c.r[0]; c.wr32(278724180u32, v)?; }
                c.r[0] = c.r[4].wrapping_add(80u32);
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                pc = 0x10738230; continue;
            }
            0x10738230 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[7] = c.rd32(c.r[6])?;
                c.r[1] = 8u32;
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[7], 32);
                c.r[7] = 1u32;
                c.r[7] = c.shift(ShiftOp::Shl, c.r[7], ((c.r[1] >> 0) & 0xff), 32);
                c.r[1] = 256u32;
                c.r[6] = c.r[0].wrapping_add(32u32);
                c.r[3] = c.r[0];
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[7], 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 16u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = 2u32; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                pc = 0x10738270; continue;
            }
            0x10738270 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.fpush(c.rdf64(c.r[1].wrapping_add(c.r[2]))?);
                c.fcom_value(c.rdf64(276379832u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if !c.pf { pc = 0x1073829b; continue; }
            pc = 0x10738284; continue;
            }
            0x10738284 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.fpush(c.rdf64(c.r[0])?);
                { let v = c.rdf64(c.r[1].wrapping_add(c.r[2]))?; c.farith_value(FOp::Div, v); }
                { let v = c.rdf64(276956504u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(338504984u32, v)?; c.fpop(); }
                pc = 0x107382af; continue;
            }
            0x1073829b => {
                { let v = 0u32; c.wr32(338504984u32, v)?; }
                { let v = 0u32; c.wr32(338504988u32, v)?; }
            pc = 0x107382af; continue;
            }
            0x107382af => {
                c.r[1] = c.rd32(338504988u32)?;
                c.r[2] = c.rd32(338504984u32)?;
                c.push(c.r[1])?;
                c.push(c.r[2])?;
                c.push(0x107382c2)?; dispatch(c, 0x10741158)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                c.push(0x107382ca)?; dispatch(c, 0x10740cb4)?;
                c.r[2] = c.r[0];
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                if c.sf == c.of { pc = 0x107382d4; continue; }
            pc = 0x107382d0; continue;
            }
            0x107382d0 => {
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                pc = 0x107382e0; continue;
            }
            0x107382d4 => {
                c.alu(AluOp::Sub, c.r[2], 256u32, 32);
                if c.sf != c.of { pc = 0x107382e0; continue; }
            pc = 0x107382dc; continue;
            }
            0x107382dc => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(36u32))?;
            pc = 0x107382e0; continue;
            }
            0x107382e0 => {
                c.r[2] = c.alu(AluOp::And, c.r[2], c.rd32(c.r[4].wrapping_add(36u32))?, 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[1] = c.rd32(c.r[0])?;
                c.r[0] = c.r[2];
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                c.fpush(c.rd32(c.r[4].wrapping_add(44u32))? as i32 as f64);
                c.fcom_value(c.rdf64(338504984u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.zf { pc = 0x1073830d; continue; }
            pc = 0x10738305; continue;
            }
            0x10738305 => {
                c.r[0] = c.r[2];
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[7], 32);
                if c.sf { pc = 0x1073830d; continue; }
            pc = 0x1073830b; continue;
            }
            0x1073830b => {
                c.r[2] = c.r[0];
            pc = 0x1073830d; continue;
            }
            0x1073830d => {
                c.r[0] = c.r[2];
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[3], v)?; }
                { let v = c.r[0]; c.wr32(c.r[6].wrapping_add(4294967248u32), v)?; }
                c.r[0] = c.r[2].wrapping_add(c.r[7]);
                c.alu(AluOp::Sub, c.r[0], 256u32, 32);
                if c.sf == c.of { pc = 0x10738324; continue; }
            pc = 0x10738322; continue;
            }
            0x10738322 => {
                c.r[2] = c.r[0];
            pc = 0x10738324; continue;
            }
            0x10738324 => {
                c.r[0] = c.r[2];
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.r[1] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 24u32, 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 4u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], c.r[0], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(20u32))?;
                { let v = c.r[2]; c.wr32(c.r[6].wrapping_add(4294967292u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[0], v)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], 12u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(12u32))?;
                c.r[3] = c.alu(AluOp::Add, c.r[3], 20u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, false);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                if !c.zf { pc = 0x10738270; continue; }
            pc = 0x1073835e; continue;
            }
            0x1073835e => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 64u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 8u32, 32);
                c.alu(AluOp::Sub, c.r[1], 278724188u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.sf != c.of { pc = 0x10738230; continue; }
            pc = 0x10738387; continue;
            }
            0x10738387 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.alu(AluOp::Sub, c.r[0], 3u32, 32);
                if !c.zf { pc = 0x107386a2; continue; }
            pc = 0x10738393; continue;
            }
            0x10738393 => {
                c.r[2] = c.rd32(c.r[5].wrapping_add(24u32))?;
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.r[2] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[2] = c.alu(AluOp::Add, c.r[2], 48u32, 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                c.r[3] = c.r[4].wrapping_add(68u32);
                c.r[1] = 338504608u32;
                c.r[0] = 338498456u32;
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                c.r[3] = c.r[3];
            pc = 0x107383c0; continue;
            }
            0x107383c0 => {
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[0] = c.r[3].wrapping_add(4294967292u32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = 4u32; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                pc = 0x107383e0; continue;
            }
            0x107383e0 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[2] = c.rd32(c.r[6])?;
                c.r[0] = c.rd32(c.r[6].wrapping_add(4u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[2], 32);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.r[2] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], 1u32, 32);
                { let v = c.r[0]; c.wr32(c.r[6].wrapping_add(8u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.fpush(c.rd32(c.r[3].wrapping_add(4294967292u32))? as i32 as f64);
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[2].wrapping_add(4294967288u32), v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[3])? as i32 as f64);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[2], v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[3].wrapping_add(4u32))? as i32 as f64);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Mul, v); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[2].wrapping_add(8u32), v)?; c.fpop(); }
                c.r[2] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.alu(AluOp::Sub, c.r[2], 4u32, 32);
                if c.sf != c.of { pc = 0x1073854f; continue; }
            pc = 0x10738435; continue;
            }
            0x10738435 => {
                c.r[6] = c.rd32(c.r[5].wrapping_add(20u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(12u32))?;
                c.r[3] = 3u32;
                c.r[6] = c.alu(AluOp::Add, c.r[6], 16u32, 32);
            pc = 0x10738444; continue;
            }
            0x10738444 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967248u32))?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.fld_st(0);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.farith(FOp::Mul, 0, 1, false);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967280u32))?; c.farith_value(FOp::Mul, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967216u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967248u32))?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967280u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967224u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967248u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967280u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967232u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967240u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967248u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967256u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2])?);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967264u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2])?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967272u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2])?);
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967280u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(24u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(8u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967288u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(24u32))?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 4u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.fld_st(0);
                c.r[7] = c.alu(AluOp::Add, c.r[7], 4u32, 32);
                { let v = c.rdf64(c.r[6].wrapping_add(8u32))?; c.farith_value(FOp::Mul, v); }
                c.r[2] = c.alu(AluOp::Add, c.r[2], 96u32, 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 32u32, 32);
                c.alu(AluOp::Sub, c.r[3], c.rd32(c.r[5].wrapping_add(12u32))?, 32);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967280u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967224u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967272u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967288u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                if c.sf != c.of { pc = 0x10738444; continue; }
            pc = 0x1073854b; continue;
            }
            0x1073854b => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(36u32))?;
            pc = 0x1073854f; continue;
            }
            0x1073854f => {
                c.alu(AluOp::Sub, c.r[7], c.rd32(c.r[5].wrapping_add(12u32))?, 32);
                if c.sf == c.of { pc = 0x107385b1; continue; }
            pc = 0x10738554; continue;
            }
            0x10738554 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[2] = c.r[2].wrapping_add(c.r[7].wrapping_mul(2));
                c.r[6] = c.r[7];
                c.r[6] = c.alu(AluOp::Add, c.r[6], c.r[2], 32);
                c.r[2] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[2] = c.r[2].wrapping_add(c.r[6].wrapping_mul(8));
                c.r[6] = c.rd32(c.r[5].wrapping_add(20u32))?;
            pc = 0x10738568; continue;
            }
            0x10738568 => {
                c.fpush(c.rdf64(c.r[2])?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.fld_st(0);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 24u32, 32);
                c.alu(AluOp::Sub, c.r[7], c.rd32(c.r[5].wrapping_add(12u32))?, 32);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967272u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967280u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967288u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                if c.sf != c.of { pc = 0x10738568; continue; }
            pc = 0x107385b1; continue;
            }
            0x107385b1 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[2] = c.alu(AluOp::Add, c.r[2], 16u32, 32);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 16u32, 32);
                c.r[7] = c.alu(AluOp::Add, c.r[7], 32u32, 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 512u32, 32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 32u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, false);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                if !c.zf { pc = 0x107383e0; continue; }
            pc = 0x107385ee; continue;
            }
            0x107385ee => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                c.r[0] = c.r[6];
                c.r[6] = c.rd32(c.r[4].wrapping_add(12u32))?;
                c.r[2] = 8u32;
                c.r[7] = c.alu(AluOp::Add, c.r[7], c.r[2], 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[0], 338504600u32, 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                if c.sf != c.of { pc = 0x107383c0; continue; }
            pc = 0x10738621; continue;
            }
            0x10738621 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(24u32))?;
                c.fpush(c.rdf64(c.r[0].wrapping_add(16u32))?);
                c.r[3] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.fpush(c.rdf64(c.r[0].wrapping_add(8u32))?);
                c.r[6] = c.alu(AluOp::Or, c.r[6], 4294967295u32, 32);
                c.fpush(c.rdf64(c.r[0])?);
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                c.fld_st(0);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                c.farith(FOp::Mul, 0, 1, false);
                c.r[3] = c.incdec(c.r[3], 32, false);
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(276956496u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(338498448u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                pc = 0x10738660; continue;
            }
            0x10738660 => {
                c.fpush(c.rdf64(276379832u32)?);
                c.r[0] = c.r[7];
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], 2u32, 32);
                c.r[1] = c.r[7];
                c.r[1] = c.alu(AluOp::And, c.r[1], 3u32, 32);
                c.r[0] = c.alu(AluOp::And, c.r[0], 3u32, 32);
                c.r[2] = c.r[7];
                c.r[2] = c.shift(ShiftOp::Sar, c.r[2], 4u32, 32);
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 9u32, 32);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 9u32, 32);
                c.r[2] = c.shift(ShiftOp::Shl, c.r[2], 9u32, 32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 338498456u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 338500504u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 338502552u32, 32);
                c.alu(AluOp::Sub, c.r[3], 15u32, 32);
                if !c.cf && !c.zf { pc = 0x107395f8; continue; }
            pc = 0x1073869b; continue;
            }
            0x1073869b => {
                pc = c.rd32(c.r[3].wrapping_mul(4).wrapping_add(276011336u32))?; if ![0x107395c0, 0x10739585, 0x1073954a, 0x1073950f, 0x107394d4, 0x10739487, 0x10739431, 0x107393db, 0x10739385, 0x1073932f, 0x107392d9, 0x10739283, 0x1073922d, 0x107391d7, 0x10739181, 0x1073912b, 0x107390b2, 0x10739062, 0x10739012, 0x10738fc2, 0x10738f4e, 0x10738eda, 0x10738e66, 0x10738df2, 0x10738d7e, 0x10738d0a, 0x10738c96, 0x10738c22, 0x10738bae, 0x10738b3a, 0x10738ac6, 0x10738a52].contains(&pc) { return Err(Fault("jmp", pc)); } continue;
            }
            0x107386a2 => {
                c.alu(AluOp::Sub, c.r[0], 4u32, 32);
                if !c.zf { pc = 0x10739621; continue; }
            pc = 0x107386ab; continue;
            }
            0x107386ab => {
                c.r[2] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[1] = c.rd32(c.r[5].wrapping_add(24u32))?;
                c.r[2] = c.alu(AluOp::Add, c.r[2], 48u32, 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                c.r[3] = c.r[4].wrapping_add(68u32);
                c.r[6] = 338504608u32;
                c.r[0] = 338498456u32;
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
            pc = 0x107386d2; continue;
            }
            0x107386d2 => {
                c.r[1] = c.r[6];
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                { let v = 4u32; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.r[4] = c.r[4];
            pc = 0x107386f0; continue;
            }
            0x107386f0 => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[0] = c.rd32(c.r[7].wrapping_add(4294967292u32))?;
                c.r[2] = c.rd32(c.r[7])?;
                c.r[2] = c.r[2].wrapping_add(c.r[0].wrapping_mul(2)).wrapping_add(1u32);
                c.r[0] = 1431655766u32;
                { let (lo, hi) = c.imul_wide(c.r[0], c.r[2]); c.r[0] = lo; c.r[2] = hi; }
                c.r[0] = c.r[2];
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 31u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], c.r[0], 32);
                c.r[0] = c.rd32(c.r[7].wrapping_add(4294967292u32))?;
                { let v = c.r[2]; c.wr32(c.r[7].wrapping_add(4u32), v)?; }
                c.r[2] = c.rd32(c.r[7])?;
                c.r[2] = c.r[0].wrapping_add(c.r[2].wrapping_mul(2)).wrapping_add(1u32);
                c.r[0] = 1431655766u32;
                { let (lo, hi) = c.imul_wide(c.r[0], c.r[2]); c.r[0] = lo; c.r[2] = hi; }
                c.r[0] = c.r[2];
                c.r[0] = c.shift(ShiftOp::Shr, c.r[0], 31u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], c.r[0], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(20u32))?;
                { let v = c.r[2]; c.wr32(c.r[7].wrapping_add(8u32), v)?; }
                c.r[2] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.fpush(c.rd32(c.r[3].wrapping_add(4294967292u32))? as i32 as f64);
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                c.alu(AluOp::Sub, c.r[2], 4u32, 32);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_add(4294967288u32), v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[3])? as i32 as f64);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[6], v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[3].wrapping_add(4u32))? as i32 as f64);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_add(8u32), v)?; c.fpop(); }
                c.fpush(c.rd32(c.r[3].wrapping_add(8u32))? as i32 as f64);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Mul, v); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                { let v = c.rdf64(276714464u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_add(16u32), v)?; c.fpop(); }
                if c.sf != c.of { pc = 0x107388e6; continue; }
            pc = 0x10738774; continue;
            }
            0x10738774 => {
                c.r[6] = c.rd32(c.r[5].wrapping_add(20u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(12u32))?;
                c.r[3] = 3u32;
                c.r[6] = c.alu(AluOp::Add, c.r[6], 16u32, 32);
            pc = 0x10738783; continue;
            }
            0x10738783 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967248u32))?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.fld_st(0);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.farith(FOp::Mul, 0, 1, false);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967280u32))?; c.farith_value(FOp::Mul, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967216u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967248u32))?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967280u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967224u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967248u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967280u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967232u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967248u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(16u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967280u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967240u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967248u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967256u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967264u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(16u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967272u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2])?);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967280u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2])?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967288u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2])?);
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.fld_st(0);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.farith(FOp::Mul, 0, 1, false);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 4u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                { let v = c.rdf64(c.r[6])?; c.farith_value(FOp::Mul, v); }
                c.r[7] = c.alu(AluOp::Add, c.r[7], 4u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 96u32, 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 32u32, 32);
                c.alu(AluOp::Sub, c.r[3], c.rd32(c.r[5].wrapping_add(12u32))?, 32);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967248u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967200u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(16u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967264u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967256u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967224u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967272u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967264u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967224u32))?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967272u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967272u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967224u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967272u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967280u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967224u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(16u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967272u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967288u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                if c.sf != c.of { pc = 0x10738783; continue; }
            pc = 0x107388de; continue;
            }
            0x107388de => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(28u32))?;
            pc = 0x107388e6; continue;
            }
            0x107388e6 => {
                c.alu(AluOp::Sub, c.r[7], c.rd32(c.r[5].wrapping_add(12u32))?, 32);
                if c.sf == c.of { pc = 0x10738967; continue; }
            pc = 0x107388eb; continue;
            }
            0x107388eb => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[2] = c.r[2].wrapping_add(c.r[7].wrapping_mul(2));
                c.r[6] = c.r[7];
                c.r[6] = c.alu(AluOp::Add, c.r[6], c.r[2], 32);
                c.r[2] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[2] = c.r[2].wrapping_add(c.r[6].wrapping_mul(8));
                c.r[4] = c.r[4];
            pc = 0x10738900; continue;
            }
            0x10738900 => {
                c.fpush(c.rdf64(c.r[2])?);
                c.r[6] = c.rd32(c.r[5].wrapping_add(20u32))?;
                { let v = c.rdf64(c.r[1].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                c.fld_st(0);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 8u32, 32);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)))?; c.farith_value(FOp::Mul, v); }
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 24u32, 32);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967264u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967272u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967280u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(16u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(4294967288u32))?; c.farith_value(FOp::Mul, v); }
                c.r[6] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.alu(AluOp::Sub, c.r[7], c.r[6], 32);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967288u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                if c.sf != c.of { pc = 0x10738900; continue; }
            pc = 0x10738963; continue;
            }
            0x10738963 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(28u32))?;
            pc = 0x10738967; continue;
            }
            0x10738967 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.alu(AluOp::Add, c.r[2], 16u32, 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[3] = c.alu(AluOp::Add, c.r[3], 16u32, 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 32u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 512u32, 32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 32u32, 32);
                c.r[2] = c.incdec(c.r[2], 32, false);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                if !c.zf { pc = 0x107386f0; continue; }
            pc = 0x1073899f; continue;
            }
            0x1073899f => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(12u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                c.r[1] = 8u32;
                c.r[7] = c.alu(AluOp::Add, c.r[7], c.r[1], 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], c.r[1], 32);
                c.alu(AluOp::Sub, c.r[0], 338504600u32, 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                if c.sf != c.of { pc = 0x107386d2; continue; }
            pc = 0x107389cc; continue;
            }
            0x107389cc => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(24u32))?;
                c.fpush(c.rdf64(c.r[0].wrapping_add(16u32))?);
                c.r[3] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.fpush(c.rdf64(c.r[0].wrapping_add(8u32))?);
                c.r[6] = c.alu(AluOp::Or, c.r[6], 4294967295u32, 32);
                c.fpush(c.rdf64(c.r[0])?);
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                c.fld_st(0);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                c.farith(FOp::Mul, 0, 1, false);
                c.r[3] = c.incdec(c.r[3], 32, false);
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(276956496u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(338498448u32, v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                pc = 0x10738a10; continue;
            }
            0x10738a10 => {
                c.fpush(c.rdf64(276379832u32)?);
                c.r[0] = c.r[7];
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], 2u32, 32);
                c.r[1] = c.r[7];
                c.r[1] = c.alu(AluOp::And, c.r[1], 3u32, 32);
                c.r[0] = c.alu(AluOp::And, c.r[0], 3u32, 32);
                c.r[2] = c.r[7];
                c.r[2] = c.shift(ShiftOp::Sar, c.r[2], 4u32, 32);
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 9u32, 32);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 9u32, 32);
                c.r[2] = c.shift(ShiftOp::Shl, c.r[2], 9u32, 32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 338498456u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 338500504u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 338502552u32, 32);
                c.alu(AluOp::Sub, c.r[3], 15u32, 32);
                if !c.cf && !c.zf { pc = 0x107390ff; continue; }
            pc = 0x10738a4b; continue;
            }
            0x10738a4b => {
                pc = c.rd32(c.r[3].wrapping_mul(4).wrapping_add(276011400u32))?; if ![0x107390b2, 0x10739062, 0x10739012, 0x10738fc2, 0x10738f4e, 0x10738eda, 0x10738e66, 0x10738df2, 0x10738d7e, 0x10738d0a, 0x10738c96, 0x10738c22, 0x10738bae, 0x10738b3a, 0x10738ac6, 0x10738a52].contains(&pc) { return Err(Fault("jmp", pc)); } continue;
            }
            0x10738a52 => {
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(480u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(480u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(480u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(488u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(488u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(488u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(496u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(496u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(496u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(504u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(504u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(504u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.fst_st(1, true);
            pc = 0x10738ac6; continue;
            }
            0x10738ac6 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(448u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(448u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(448u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(456u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(456u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(456u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(464u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(464u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(464u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(472u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(472u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(472u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10738b3a; continue;
            }
            0x10738b3a => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(416u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(416u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(416u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(424u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(424u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(424u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(432u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(432u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(432u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(440u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(440u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(440u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10738bae; continue;
            }
            0x10738bae => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(384u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(384u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(384u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(392u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(392u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(392u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(400u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(400u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(400u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(408u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(408u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(408u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10738c22; continue;
            }
            0x10738c22 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(352u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(352u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(352u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(360u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(360u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(360u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(368u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(368u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(368u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(376u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(376u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(376u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10738c96; continue;
            }
            0x10738c96 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(320u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(320u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(320u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(328u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(328u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(328u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(336u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(336u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(336u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(344u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(344u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(344u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10738d0a; continue;
            }
            0x10738d0a => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(288u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(288u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(288u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(296u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(296u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(296u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(304u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(304u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(304u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(312u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(312u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(312u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10738d7e; continue;
            }
            0x10738d7e => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(256u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(256u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(256u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(264u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(264u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(264u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(272u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(272u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(272u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(280u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(280u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(280u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10738df2; continue;
            }
            0x10738df2 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(224u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(224u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(224u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(232u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(232u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(232u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(240u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(240u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(240u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(248u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(248u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(248u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10738e66; continue;
            }
            0x10738e66 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(192u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(192u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(192u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(200u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(200u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(200u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(208u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(208u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(208u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(216u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(216u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(216u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10738eda; continue;
            }
            0x10738eda => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(160u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(160u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(160u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(168u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(168u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(168u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(176u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(176u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(176u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(184u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(184u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(184u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10738f4e; continue;
            }
            0x10738f4e => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(128u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(128u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(128u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(136u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(136u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(136u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(144u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(144u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(144u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(152u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(152u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(152u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10738fc2; continue;
            }
            0x10738fc2 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(96u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(96u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(96u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(104u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(104u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(104u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(112u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(112u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(112u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(120u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(120u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(120u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10739012; continue;
            }
            0x10739012 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(64u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(64u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(64u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(72u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(72u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(72u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(80u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(80u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(80u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(88u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(88u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(88u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10739062; continue;
            }
            0x10739062 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(32u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(32u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(32u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(40u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(40u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(40u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(48u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(48u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(48u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(56u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(56u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(56u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x107390b2; continue;
            }
            0x107390b2 => {
                c.fpush(c.rdf64(c.r[2])?);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(8u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(8u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(16u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(16u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(16u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(24u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(24u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(24u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x107390ff; continue;
            }
            0x107390ff => {
                c.fcom_value(c.rdf64(338498448u32)?, false);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x10739116; continue; }
            pc = 0x1073910c; continue;
            }
            0x1073910c => {
                { let v = c.st0(); c.wrf64(338498448u32, v)?; c.fpop(); }
                c.r[6] = c.r[7];
                pc = 0x10739118; continue;
            }
            0x10739116 => {
                c.fst_st(0, true);
            pc = 0x10739118; continue;
            }
            0x10739118 => {
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.alu(AluOp::Sub, c.r[7], 64u32, 32);
                if c.sf != c.of { pc = 0x10738a10; continue; }
            pc = 0x10739122; continue;
            }
            0x10739122 => {
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                pc = 0x10739625; continue;
            }
            0x1073912b => {
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(360u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(360u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(360u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(368u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(368u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(368u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(376u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(376u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(376u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.fst_st(1, true);
            pc = 0x10739181; continue;
            }
            0x10739181 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(336u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(336u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(336u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(344u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(344u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(344u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(352u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(352u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(352u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x107391d7; continue;
            }
            0x107391d7 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(312u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(312u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(312u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(320u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(320u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(320u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(328u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(328u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(328u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x1073922d; continue;
            }
            0x1073922d => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(288u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(288u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(288u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(296u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(296u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(296u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(304u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(304u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(304u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10739283; continue;
            }
            0x10739283 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(264u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(264u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(264u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(272u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(272u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(272u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(280u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(280u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(280u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x107392d9; continue;
            }
            0x107392d9 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(240u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(240u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(240u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(248u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(248u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(248u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(256u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(256u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(256u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x1073932f; continue;
            }
            0x1073932f => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(216u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(216u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(216u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(224u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(224u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(224u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(232u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(232u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(232u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10739385; continue;
            }
            0x10739385 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(192u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(192u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(192u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(200u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(200u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(200u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(208u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(208u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(208u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x107393db; continue;
            }
            0x107393db => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(168u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(168u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(168u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(176u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(176u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(176u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(184u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(184u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(184u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10739431; continue;
            }
            0x10739431 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(144u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(144u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(144u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(152u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(152u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(152u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(160u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(160u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(160u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10739487; continue;
            }
            0x10739487 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(120u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(120u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(120u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(128u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(128u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(128u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(136u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(136u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(136u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x107394d4; continue;
            }
            0x107394d4 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(96u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(96u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(96u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(104u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(104u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(104u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(112u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(112u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(112u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x1073950f; continue;
            }
            0x1073950f => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(72u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(72u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(72u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(80u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(80u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(80u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(88u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(88u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(88u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x1073954a; continue;
            }
            0x1073954a => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(48u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(48u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(48u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(56u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(56u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(56u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(64u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(64u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(64u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x10739585; continue;
            }
            0x10739585 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(24u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(24u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(24u32))?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(32u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(32u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(32u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(40u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(40u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(40u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x107395c0; continue;
            }
            0x107395c0 => {
                c.fpush(c.rdf64(c.r[2])?);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Add, v); }
                c.fpush(c.rdf64(c.r[2].wrapping_add(8u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(8u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(8u32))?; c.farith_value(FOp::Add, v); }
                c.fld_st(0);
                c.farith(FOp::Add, 0, 2, false);
                c.fxch(2);
                c.farith(FOp::Sub, 0, 1, false);
                c.fabs();
                c.farith(FOp::Sub, 2, 0, true);
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[2].wrapping_add(16u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(16u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.rdf64(c.r[1].wrapping_add(16u32))?; c.farith_value(FOp::Add, v); }
                c.farith(FOp::Add, 0, 0, false);
                c.fld_st(1);
                c.farith(FOp::Add, 0, 1, false);
                c.fxch(1);
                c.farith(FOp::Sub, 0, 2, false);
                c.fabs();
                c.farith(FOp::Sub, 1, 0, true);
                c.farith(FOp::Add, 2, 0, true);
                c.fst_st(0, true);
            pc = 0x107395f8; continue;
            }
            0x107395f8 => {
                c.fcom_value(c.rdf64(338498448u32)?, false);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x1073960f; continue; }
            pc = 0x10739605; continue;
            }
            0x10739605 => {
                { let v = c.st0(); c.wrf64(338498448u32, v)?; c.fpop(); }
                c.r[6] = c.r[7];
                pc = 0x10739611; continue;
            }
            0x1073960f => {
                c.fst_st(0, true);
            pc = 0x10739611; continue;
            }
            0x10739611 => {
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.alu(AluOp::Sub, c.r[7], 64u32, 32);
                if c.sf != c.of { pc = 0x10738660; continue; }
            pc = 0x1073961b; continue;
            }
            0x1073961b => {
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                pc = 0x10739625; continue;
            }
            0x10739621 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(12u32))?;
            pc = 0x10739625; continue;
            }
            0x10739625 => {
                c.r[0] = c.r[6];
                c.r[0] = c.alu(AluOp::And, c.r[0], 3u32, 32);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 5u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 338504600u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                c.r[0] = c.r[6];
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], 4u32, 32);
                c.r[3] = c.r[6];
                c.r[3] = c.shift(ShiftOp::Sar, c.r[3], 2u32, 32);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 5u32, 32);
                c.r[3] = c.alu(AluOp::And, c.r[3], 3u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 338504856u32, 32);
                c.r[3] = c.shift(ShiftOp::Shl, c.r[3], 5u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[0] = c.rd32(c.r[5].wrapping_add(28u32))?;
                c.r[3] = c.alu(AluOp::Add, c.r[3], 338504728u32, 32);
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], 278724176u32, 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(60u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                pc = 0x10739670; continue;
            }
            0x1073966c => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(12u32))?;
            pc = 0x10739670; continue;
            }
            0x10739670 => {
                c.r[0] = c.r[6];
                c.r[1] = c.r[7].wrapping_add(c.r[7]);
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.r[2] = c.r[7].wrapping_mul(4);
                c.r[6] = c.rd32(c.r[2].wrapping_add(278724176u32))?;
                c.r[1] = 8u32;
                c.r[0] = c.alu(AluOp::And, c.r[0], 3u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[2], 32);
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[6], 32);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 4u32, 32);
                c.r[6] = c.rd32(c.r[4].wrapping_add(c.r[0]).wrapping_add(64u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[0]).wrapping_add(68u32))?;
                c.r[6] = c.shift(ShiftOp::Sar, c.r[6], ((c.r[1] >> 0) & 0xff), 32);
                c.r[1] = c.r[6];
                c.r[6] = c.rd32(c.r[4].wrapping_add(44u32))?;
                { let v = c.r[1]; c.wr32(c.r[6].wrapping_add(c.r[2]).wrapping_add(278724176u32), v)?; }
                c.r[1] = 8u32;
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.rd32(c.r[2].wrapping_add(278724176u32))?, 32);
                c.r[0] = c.shift(ShiftOp::Sar, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.alu(AluOp::Sub, c.r[7], 3u32, 32);
                { let v = c.r[0]; c.wr32(c.r[6].wrapping_add(c.r[2]).wrapping_add(278724188u32), v)?; }
                if c.sf != c.of { pc = 0x1073966c; continue; }
            pc = 0x107396c4; continue;
            }
            0x107396c4 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.fpush(c.rdf64(276379832u32)?);
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10739940; continue; }
            pc = 0x107396dd; continue;
            }
            0x107396dd => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(20u32))?;
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.r[1] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 8u32, 32);
            pc = 0x107396ea; continue;
            }
            0x107396ea => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(24u32))?;
                c.fpush(c.rdf64(c.r[0].wrapping_add(16u32))?);
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                c.fpush(c.rdf64(c.r[0].wrapping_add(8u32))?);
                { let v = 4294967295u32; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                c.fpush(c.rdf64(c.r[0])?);
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.alu(AluOp::Sub, c.r[0], 4u32, 32);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                c.farith(FOp::Add, 0, 0, false);
                c.fst_st(3, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                if c.sf != c.of { pc = 0x107398a1; continue; }
            pc = 0x10739723; continue;
            }
            0x10739723 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[3], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[3], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.r[3].wrapping_add(8u32);
                c.r[3] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[6] = c.alu(AluOp::Add, c.r[6], 16u32, 32);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[3], 32);
                { let v = 3u32; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
            pc = 0x10739757; continue;
            }
            0x10739757 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.fpush(c.rdf64(c.r[1].wrapping_add(8u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(c.r[7].wrapping_mul(8)))?; c.farith_value(FOp::Sub, v); }
                c.r[3] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.fpush(c.rdf64(c.r[1])?);
                { let v = c.rdf64(c.r[2].wrapping_add(4294967288u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(4294967288u32))?);
                { let v = c.rdf64(c.r[6].wrapping_add(4294967280u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(c.r[3])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(48u32), v)?;  }
                c.fst_st(3, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fcom(1, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x1073979f; continue; }
            pc = 0x10739795; continue;
            }
            0x10739795 => {
                c.fst_st(0, true);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                c.fpush(c.rdf64(c.r[4].wrapping_add(48u32))?);
            pc = 0x1073979f; continue;
            }
            0x1073979f => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.fpush(c.rdf64(c.r[1].wrapping_add(8u32))?);
                { let v = c.rdf64(c.r[2].wrapping_add(c.r[0]))?; c.farith_value(FOp::Sub, v); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.fpush(c.rdf64(c.r[1])?);
                { let v = c.rdf64(c.r[2])?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(4294967288u32))?);
                { let v = c.rdf64(c.r[2].wrapping_add(c.r[0]))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(1);
                c.farith(FOp::Mul, 0, 2, false);
                c.fld_st(1);
                c.farith(FOp::Mul, 0, 2, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(c.r[3])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(48u32), v)?;  }
                c.fst_st(3, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fcom(1, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x107397ed; continue; }
            pc = 0x107397dc; continue;
            }
            0x107397dc => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[4].wrapping_add(48u32))?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 4294967294u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
            pc = 0x107397ed; continue;
            }
            0x107397ed => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.fpush(c.rdf64(c.r[1].wrapping_add(8u32))?);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[0]))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[1])?);
                { let v = c.rdf64(c.r[2].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(4294967288u32))?);
                { let v = c.rdf64(c.r[6])?; c.farith_value(FOp::Sub, v); }
                c.fld_st(1);
                c.farith(FOp::Mul, 0, 2, false);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(1);
                c.farith(FOp::Mul, 0, 2, false);
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(c.r[3])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(48u32), v)?;  }
                c.fst_st(3, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fcom(1, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x10739835; continue; }
            pc = 0x10739826; continue;
            }
            0x10739826 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[4].wrapping_add(48u32))?);
                c.r[0] = c.incdec(c.r[0], 32, false);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
            pc = 0x10739835; continue;
            }
            0x10739835 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.fpush(c.rdf64(c.r[1].wrapping_add(8u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(c.r[7].wrapping_mul(8)).wrapping_add(24u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[1])?);
                { let v = c.rdf64(c.r[2].wrapping_add(16u32))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(4294967288u32))?);
                { let v = c.rdf64(c.r[6].wrapping_add(8u32))?; c.farith_value(FOp::Sub, v); }
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(c.r[3])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(48u32), v)?;  }
                c.fst_st(3, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fcom(1, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x1073987e; continue; }
            pc = 0x10739870; continue;
            }
            0x10739870 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.fst_st(0, true);
                c.fpush(c.rdf64(c.r[4].wrapping_add(48u32))?);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
            pc = 0x1073987e; continue;
            }
            0x1073987e => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[3] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 4u32, 32);
                c.r[7] = c.alu(AluOp::Add, c.r[7], 4u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 32u32, 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 32u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[3], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                if c.sf != c.of { pc = 0x10739757; continue; }
            pc = 0x1073989d; continue;
            }
            0x1073989d => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(60u32))?;
            pc = 0x107398a1; continue;
            }
            0x107398a1 => {
                c.alu(AluOp::Sub, c.r[7], c.rd32(c.r[5].wrapping_add(8u32))?, 32);
                if c.sf == c.of { pc = 0x10739912; continue; }
            pc = 0x107398a6; continue;
            }
            0x107398a6 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[6] = c.alu(AluOp::Sub, c.r[6], c.r[3], 32);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[3], 32);
                c.r[2] = c.r[3].wrapping_add(c.r[7].wrapping_mul(8));
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.r[4] = c.r[4];
            pc = 0x107398c0; continue;
            }
            0x107398c0 => {
                c.fpush(c.rdf64(c.r[1].wrapping_add(8u32))?);
                c.r[0] = c.rd32(c.r[4].wrapping_add(36u32))?;
                { let v = c.rdf64(c.r[2].wrapping_add(c.r[6]))?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[1])?);
                { let v = c.rdf64(c.r[2])?; c.farith_value(FOp::Sub, v); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(4294967288u32))?);
                { let v = c.rdf64(c.r[2].wrapping_add(c.r[0]))?; c.farith_value(FOp::Sub, v); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                c.fld_st(3);
                c.farith(FOp::Mul, 0, 4, false);
                c.farith(FOp::Add, 1, 0, true);
                c.fld_st(2);
                c.farith(FOp::Mul, 0, 3, false);
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(48u32), v)?;  }
                c.fst_st(3, true);
                c.fst_st(0, true);
                c.fst_st(0, true);
                c.fcom(1, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x10739907; continue; }
            pc = 0x107398fd; continue;
            }
            0x107398fd => {
                c.fst_st(0, true);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                c.fpush(c.rdf64(c.r[4].wrapping_add(48u32))?);
            pc = 0x10739907; continue;
            }
            0x10739907 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 8u32, 32);
                c.alu(AluOp::Sub, c.r[7], c.r[0], 32);
                if c.sf != c.of { pc = 0x107398c0; continue; }
            pc = 0x10739912; continue;
            }
            0x10739912 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.farith(FOp::Add, 1, 0, true);
                c.r[2] = c.rd32(c.r[5].wrapping_add(32u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(12u32))?;
                { let v = c.r[6]; c.wr32(c.r[2].wrapping_add(c.r[0].wrapping_mul(4)), v)?; }
                c.r[6] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 8u32, 32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 24u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.sf != c.of { pc = 0x107396ea; continue; }
            pc = 0x10739940; continue;
            }
            0x10739940 => {
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[3] = v; }
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_107399d0(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x107399d0;
    loop {
        match pc {
            0x107399d0 => {
                c.push(c.r[5])?;
                c.r[5] = c.r[4];
                c.r[4] = c.alu(AluOp::And, c.r[4], 4294967288u32, 32);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 84u32, 32);
                { let v = c.rd8(c.r[5].wrapping_add(36u32))? as u32; c.set8(0, 0, v); }
                { let v = c.neg(((c.r[0] >> 0) & 0xff), 8); c.set8(0, 0, v); }
                c.push(c.r[3])?;
                c.push(c.r[6])?;
                c.push(c.r[7])?;
                c.r[0] = c.alu(AluOp::Sbb, c.r[0], c.r[0], 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 6u32, 32);
                { let v = c.r[0]; c.wr32(278724180u32, v)?; }
                c.r[0] = c.rd32(338511944u32)?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if !c.zf { pc = 0x10739b6b; continue; }
            pc = 0x107399f8; continue;
            }
            0x107399f8 => {
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                { let v = 1u32; c.wr32(338511944u32, v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                pc = 0x10739a10; continue;
            }
            0x10739a10 => {
                c.r[7] = c.rd32(c.r[2].wrapping_mul(4).wrapping_add(278724176u32))?;
                c.r[0] = 1u32;
                c.r[1] = c.r[7];
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.r[3] = c.r[0];
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.sf != c.of { pc = 0x10739b5d; continue; }
            pc = 0x10739a37; continue;
            }
            0x10739a37 => {
                pc = 0x10739a40; continue;
            }
            0x10739a40 => {
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.sf != c.of { pc = 0x10739b4e; continue; }
            pc = 0x10739a4c; continue;
            }
            0x10739a4c => {
                c.r[6] = 8u32;
                c.r[6] = c.alu(AluOp::Sub, c.r[6], c.r[7], 32);
                c.r[2] = c.r[3];
                c.r[1] = c.r[6];
                c.r[2] = c.shift(ShiftOp::Shl, c.r[2], ((c.r[1] >> 0) & 0xff), 32);
                c.r[1] = c.r[7].wrapping_add(c.r[7]).wrapping_add(4294967288u32);
                c.r[3] = c.shift(ShiftOp::Sar, c.r[3], ((c.r[1] >> 0) & 0xff), 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[3], 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                pc = 0x10739a77; continue;
            }
            0x10739a6b => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
            pc = 0x10739a77; continue;
            }
            0x10739a77 => {
                c.r[1] = c.r[7].wrapping_add(c.r[7]).wrapping_add(4294967288u32);
                c.r[3] = c.r[0];
                c.r[3] = c.shift(ShiftOp::Sar, c.r[3], ((c.r[1] >> 0) & 0xff), 32);
                c.r[1] = c.r[6];
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], ((c.r[1] >> 0) & 0xff), 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[7] = 278724208u32;
                c.r[3] = c.alu(AluOp::Or, c.r[3], c.r[0], 32);
                pc = 0x10739a98; continue;
            }
            0x10739a94 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
            pc = 0x10739a98; continue;
            }
            0x10739a98 => {
                c.r[0] = c.rd32(c.r[7].wrapping_add(4u32))?;
                c.r[1] = c.rd32(c.r[7])?;
                c.r[0] = c.imul(c.r[0], c.r[3]);
                c.r[1] = c.imul(c.r[1], c.r[2]);
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[1], 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.rd32(c.r[7].wrapping_add(8u32))?, 32);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                { let (q, r) = c.idiv(c.rd32(c.r[7].wrapping_add(12u32))?)?; c.r[0] = q; c.r[2] = r; }
                c.r[2] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[1] = c.r[2].wrapping_add(c.r[0].wrapping_mul(2));
                c.r[2] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[1], 32);
                c.r[1] = c.r[2].wrapping_add(c.r[0].wrapping_mul(2));
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[1], 32);
                c.r[6] = c.r[0];
                { let v = c.rd8(c.r[6].wrapping_add(c.r[6].wrapping_mul(2)).wrapping_add(338505032u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10739af5; continue; }
            pc = 0x10739acb; continue;
            }
            0x10739acb => {
                c.r[2] = c.rd8(c.r[6].wrapping_add(c.r[6].wrapping_mul(2)).wrapping_add(338505033u32))? as u32;
                c.r[0] = c.rd8(c.r[6].wrapping_add(c.r[6].wrapping_mul(2)).wrapping_add(338505034u32))? as u32;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[1] = c.r[0];
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[2], 32);
                c.r[1] = c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[0] = c.r[3];
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[2], 32);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[1], c.r[0], 32);
                if c.zf || c.sf != c.of { pc = 0x10739b1b; continue; }
            pc = 0x10739af5; continue;
            }
            0x10739af5 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(44u32))?;
                { let v = c.rd8(c.r[4].wrapping_add(24u32))? as u32; c.set8(2, 0, v); }
                { let v = c.rd8(c.r[4].wrapping_add(16u32))? as u32; c.set8(0, 0, v); }
                { let v = c.shift(ShiftOp::Shl, ((c.r[2] >> 0) & 0xff), ((c.r[1] >> 0) & 0xff), 8); c.set8(2, 0, v); }
                { let v = c.shift(ShiftOp::Shl, ((c.r[0] >> 0) & 0xff), ((c.r[1] >> 0) & 0xff), 8); c.set8(0, 0, v); }
                { let v = 1u32; c.wr8(c.r[6].wrapping_add(c.r[6].wrapping_mul(2)).wrapping_add(338505032u32), v as u8)?; }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[6].wrapping_add(c.r[6].wrapping_mul(2)).wrapping_add(338505033u32), v as u8)?; }
                { let v = ((c.r[0] >> 0) & 0xff); c.wr8(c.r[6].wrapping_add(c.r[6].wrapping_mul(2)).wrapping_add(338505034u32), v as u8)?; }
            pc = 0x10739b1b; continue;
            }
            0x10739b1b => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.r[7] = c.alu(AluOp::Add, c.r[7], 16u32, 32);
                c.alu(AluOp::Sub, c.r[7], 278724256u32, 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                if c.sf != c.of { pc = 0x10739a94; continue; }
            pc = 0x10739b33; continue;
            }
            0x10739b33 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[0] = c.incdec(c.r[0], 32, false);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if !c.sf { pc = 0x10739a6b; continue; }
            pc = 0x10739b46; continue;
            }
            0x10739b46 => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(48u32))?;
            pc = 0x10739b4e; continue;
            }
            0x10739b4e => {
                c.r[3] = c.incdec(c.r[3], 32, false);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if !c.sf { pc = 0x10739a40; continue; }
            pc = 0x10739b59; continue;
            }
            0x10739b59 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(36u32))?;
            pc = 0x10739b5d; continue;
            }
            0x10739b5d => {
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.alu(AluOp::Sub, c.r[2], 3u32, 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                if c.sf != c.of { pc = 0x10739a10; continue; }
            pc = 0x10739b6b; continue;
            }
            0x10739b6b => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[2] = c.rd32(c.r[1])?;
                c.r[2] = c.incdec(c.r[2], 32, false);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x10739e8c; continue; }
            pc = 0x10739b81; continue;
            }
            0x10739b81 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(20u32))?;
                c.r[7] = c.rd32(c.r[5].wrapping_add(24u32))?;
                c.r[7] = c.alu(AluOp::Sub, c.r[7], c.r[0], 32);
                c.r[0] = c.r[4].wrapping_add(60u32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                c.r[0] = 338505008u32;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                pc = 0x10739ba0; continue;
            }
            0x10739b9c => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(36u32))?;
            pc = 0x10739ba0; continue;
            }
            0x10739ba0 => {
                c.r[6] = c.rd32(c.r[5].wrapping_add(20u32))?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = c.r[1]; c.wr32(c.r[0], v)?; }
                { let v = c.r[1]; c.wr32(c.r[0].wrapping_add(4u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                c.r[1] = c.rd32(c.r[4].wrapping_add(44u32))?;
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
            pc = 0x10739bb6; continue;
            }
            0x10739bb6 => {
                c.fpush(c.rdf64(c.r[6])?);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 8u32, 32);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7]))?; c.farith_value(FOp::Div, v); }
                { let v = c.rdf64(276956504u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[4], v)?; c.fpop(); }
                c.push(0x10739bd2)?; dispatch(c, 0x10741158)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 8u32, 32);
                c.push(0x10739bda)?; dispatch(c, 0x10740cb4)?;
                c.r[3] = c.r[0];
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.sf == c.of { pc = 0x10739bee; continue; }
            pc = 0x10739be4; continue;
            }
            0x10739be4 => {
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                pc = 0x10739bfe; continue;
            }
            0x10739bee => {
                c.alu(AluOp::Sub, c.r[3], 256u32, 32);
                if c.sf != c.of { pc = 0x10739c02; continue; }
            pc = 0x10739bf6; continue;
            }
            0x10739bf6 => {
                { let v = 255u32; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
            pc = 0x10739bfe; continue;
            }
            0x10739bfe => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(28u32))?;
            pc = 0x10739c02; continue;
            }
            0x10739c02 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[1] = c.r[2].wrapping_add(c.r[3].wrapping_mul(2));
                c.r[2] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[0] = c.r[3];
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[1], 32);
                c.r[1] = c.r[2].wrapping_add(c.r[0].wrapping_mul(2));
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[1], 32);
                { let v = c.rd8(c.r[0].wrapping_add(c.r[0].wrapping_mul(2)).wrapping_add(338505032u32))? as u32; c.set8(2, 0, v); }
                c.r[0] = c.r[4].wrapping_add(52u32);
                { let v = 1u32; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                { let v = ((c.r[2] >> 0) & 0xff); c.wr8(c.r[4].wrapping_add(15u32), v as u8)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                c.r[1] = 338504992u32;
                pc = 0x10739c40; continue;
            }
            0x10739c38 => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[4] = c.r[4];
            pc = 0x10739c40; continue;
            }
            0x10739c40 => {
                { let v = c.rd8(c.r[4].wrapping_add(15u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                if c.zf { pc = 0x10739c6c; continue; }
            pc = 0x10739c4c; continue;
            }
            0x10739c4c => {
                c.fpush(c.rdf64(c.r[6])?);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7]))?; c.farith_value(FOp::Div, v); }
                { let v = c.rdf64(276956504u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.rd32(c.r[4].wrapping_add(28u32))? as i32 as f64; c.farith_value(FOp::Sub, v); }
                { let v = c.rd32(c.r[4].wrapping_add(32u32))? as i32 as f64; c.farith_value(FOp::Mul, v); }
                c.fcom_value(c.rdf64(276379832u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.zf { pc = 0x10739ccf; continue; }
            pc = 0x10739c6c; continue;
            }
            0x10739c6c => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[3] = c.alu(AluOp::Add, c.r[3], c.r[2], 32);
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                if c.sf == c.of { pc = 0x10739c7e; continue; }
            pc = 0x10739c7a; continue;
            }
            0x10739c7a => {
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                pc = 0x10739c8b; continue;
            }
            0x10739c7e => {
                c.alu(AluOp::Sub, c.r[3], 256u32, 32);
                if c.sf != c.of { pc = 0x10739c8f; continue; }
            pc = 0x10739c86; continue;
            }
            0x10739c86 => {
                c.r[3] = 255u32;
            pc = 0x10739c8b; continue;
            }
            0x10739c8b => {
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
            pc = 0x10739c8f; continue;
            }
            0x10739c8f => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[2] = c.r[0].wrapping_add(c.r[3].wrapping_mul(2));
                c.r[0] = c.r[3];
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[2], 32);
                c.r[2] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.r[2].wrapping_add(c.r[0].wrapping_mul(2));
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[2], 32);
                { let v = c.rd8(c.r[0].wrapping_add(c.r[0].wrapping_mul(2)).wrapping_add(338505032u32))? as u32; c.set8(2, 0, v); }
                c.alu(AluOp::And, ((c.r[2] >> 0) & 0xff), ((c.r[2] >> 0) & 0xff), 8);
                c.r[0] = c.r[0].wrapping_add(c.r[0].wrapping_mul(2)).wrapping_add(338505032u32);
                if !c.zf { pc = 0x10739ccf; continue; }
            pc = 0x10739cb5; continue;
            }
            0x10739cb5 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[2] = c.imul(c.r[2], 27u32);
                c.r[4] = c.r[4];
            pc = 0x10739cc0; continue;
            }
            0x10739cc0 => {
                c.r[3] = c.alu(AluOp::Add, c.r[3], c.rd32(c.r[4].wrapping_add(32u32))?, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[2], 32);
                c.alu(AluOp::Sub, c.rd8(c.r[0])? as u32, 0u32, 8);
                if c.zf { pc = 0x10739cc0; continue; }
            pc = 0x10739ccb; continue;
            }
            0x10739ccb => {
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
            pc = 0x10739ccf; continue;
            }
            0x10739ccf => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[0], 4u32, 32);
                { let v = c.r[2]; c.wr32(c.r[1], v)?; }
                { let v = c.r[2]; c.wr32(c.r[1].wrapping_add(4u32), v)?; }
                if c.sf != c.of { pc = 0x10739d6a; continue; }
            pc = 0x10739ce2; continue;
            }
            0x10739ce2 => {
                c.fpush(c.rd32(c.r[4].wrapping_add(20u32))? as i32 as f64);
                c.r[3] = 3u32;
                c.r[0] = c.r[6].wrapping_add(48u32);
            pc = 0x10739cee; continue;
            }
            0x10739cee => {
                c.fld_st(0);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 4u32, 32);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7]))?; c.farith_value(FOp::Mul, v); }
                c.r[2] = c.alu(AluOp::Add, c.r[2], 4u32, 32);
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967248u32))?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 96u32, 32);
                c.alu(AluOp::Sub, c.r[3], c.rd32(c.r[5].wrapping_add(16u32))?, 32);
                { let v = c.rdf64(276956504u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Sub, 1, 0, true);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[1], v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7]))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967176u32))?);
                { let v = c.rdf64(276956504u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Sub, 1, 0, true);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[1], v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7]))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967200u32))?);
                { let v = c.rdf64(276956504u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Sub, 1, 0, true);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[1], v)?; c.fpop(); }
                c.fst_st(0, true);
                c.fld_st(0);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7]))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967224u32))?);
                { let v = c.rdf64(276956504u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Sub, 1, 0, true);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[1], v)?; c.fpop(); }
                c.fst_st(0, true);
                if c.sf != c.of { pc = 0x10739cee; continue; }
            pc = 0x10739d64; continue;
            }
            0x10739d64 => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.fst_st(0, true);
            pc = 0x10739d6a; continue;
            }
            0x10739d6a => {
                c.alu(AluOp::Sub, c.r[2], c.rd32(c.r[5].wrapping_add(16u32))?, 32);
                if c.sf == c.of { pc = 0x10739db6; continue; }
            pc = 0x10739d6f; continue;
            }
            0x10739d6f => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.fpush(c.rd32(c.r[4].wrapping_add(20u32))? as i32 as f64);
                c.r[0] = c.r[0].wrapping_add(c.r[2].wrapping_mul(2));
                c.r[3] = c.r[2];
                c.r[3] = c.alu(AluOp::Add, c.r[3], c.r[0], 32);
                c.r[0] = c.rd32(c.r[5].wrapping_add(20u32))?;
                c.r[0] = c.r[0].wrapping_add(c.r[3].wrapping_mul(8));
                c.r[3] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.r[3] = c.alu(AluOp::Sub, c.r[3], c.r[2], 32);
                c.r[4] = c.r[4];
            pc = 0x10739d90; continue;
            }
            0x10739d90 => {
                c.fld_st(0);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 24u32, 32);
                c.r[3] = c.incdec(c.r[3], 32, false);
                { let v = c.rdf64(c.r[6].wrapping_add(c.r[7]))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(4294967272u32))?);
                { let v = c.rdf64(276956504u32)?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Sub, 1, 0, true);
                c.fld_st(0);
                c.farith(FOp::Mul, 0, 1, false);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[1], v)?; c.fpop(); }
                c.fst_st(0, true);
                if !c.zf { pc = 0x10739d90; continue; }
            pc = 0x10739db0; continue;
            }
            0x10739db0 => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.fst_st(0, true);
            pc = 0x10739db6; continue;
            }
            0x10739db6 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                { let v = c.r[3]; c.wr32(c.r[0], v)?; }
                c.r[1] = c.alu(AluOp::Add, c.r[1], 8u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 4u32, 32);
                c.r[2] = c.neg(c.r[2], 32);
                c.alu(AluOp::Sub, c.r[1], 338505008u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                if c.sf != c.of { pc = 0x10739c38; continue; }
            pc = 0x10739ddc; continue;
            }
            0x10739ddc => {
                c.fpush(c.rdf64(338504992u32)?);
                c.fcom_value(c.rdf64(338505000u32)?, true);
                c.fpush(c.rdf64(338504992u32)?);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x10739e03; continue; }
            pc = 0x10739df5; continue;
            }
            0x10739df5 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(36u32))?;
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[0], v)?; c.fpop(); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(52u32))?;
                pc = 0x10739e39; continue;
            }
            0x10739e03 => {
                c.fcom_value(c.rdf64(338505000u32)?, true);
                c.fpush(c.rdf64(338505000u32)?);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                c.r[0] = c.rd32(c.r[4].wrapping_add(36u32))?;
                { let v = c.rdf64(c.r[0])?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[0], v)?; c.fpop(); }
                if !c.zf { pc = 0x10739e2a; continue; }
            pc = 0x10739e1e; continue;
            }
            0x10739e1e => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(40u32))?;
                { let v = c.r[2]; c.wr32(c.r[0], v)?; }
                pc = 0x10739e3f; continue;
            }
            0x10739e2a => {
                c.alu(AluOp::And, c.rd8(c.r[4].wrapping_add(28u32))? as u32, 1u32, 8);
                c.r[0] = c.rd32(c.r[4].wrapping_add(52u32))?;
                if !c.zf { pc = 0x10739e39; continue; }
            pc = 0x10739e35; continue;
            }
            0x10739e35 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(56u32))?;
            pc = 0x10739e39; continue;
            }
            0x10739e39 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(40u32))?;
                { let v = c.r[0]; c.wr32(c.r[1], v)?; }
            pc = 0x10739e3f; continue;
            }
            0x10739e3f => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 12u32, 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 8u32, 32);
                c.alu(AluOp::Sub, c.r[0], 3u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                if c.sf != c.of { pc = 0x10739bb6; continue; }
            pc = 0x10739e5f; continue;
            }
            0x10739e5f => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[3] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 8u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                c.r[1] = c.rd32(c.r[2])?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[3] = c.alu(AluOp::Add, c.r[3], 4u32, 32);
                c.r[1] = c.incdec(c.r[1], 32, false);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                { let v = c.r[3]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                if c.sf != c.of { pc = 0x10739b9c; continue; }
            pc = 0x10739e8c; continue;
            }
            0x10739e8c => {
                c.r[2] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[0] = c.rd32(c.r[2])?;
                c.alu(AluOp::Sub, c.r[0], 4u32, 32);
                if !c.zf { pc = 0x10739ef2; continue; }
            pc = 0x10739e96; continue;
            }
            0x10739e96 => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.fpush(c.rdf64(338505008u32)?);
                { let v = c.rd8(c.r[0].wrapping_add(420u32))? as u32; c.set8(1, 0, v); }
                c.alu(AluOp::And, ((c.r[1] >> 0) & 0xff), ((c.r[1] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10739ec0; continue; }
            pc = 0x10739ea9; continue;
            }
            0x10739ea9 => {
                c.fcom_value(c.rdf64(338505024u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.pf { pc = 0x10739ee8; continue; }
            pc = 0x10739eb6; continue;
            }
            0x10739eb6 => {
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                pc = 0x10739f1a; continue;
            }
            0x10739ec0 => {
                c.fcom_value(c.rdf64(338505016u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.pf { pc = 0x10739ed5; continue; }
            pc = 0x10739ecd; continue;
            }
            0x10739ecd => {
                c.fpush(c.rdf64(338505008u32)?);
                pc = 0x10739ea9; continue;
            }
            0x10739ed5 => {
                c.fpush(c.rdf64(338505016u32)?);
                c.fcom_value(c.rdf64(338505024u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.pf { pc = 0x10739f12; continue; }
            pc = 0x10739ee8; continue;
            }
            0x10739ee8 => {
                { let v = 2u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                pc = 0x10739f1a; continue;
            }
            0x10739ef2 => {
                c.alu(AluOp::Sub, c.r[0], 3u32, 32);
                if !c.zf { pc = 0x10739f1a; continue; }
            pc = 0x10739ef7; continue;
            }
            0x10739ef7 => {
                c.fpush(c.rdf64(338505008u32)?);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.fcom_value(c.rdf64(338505016u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if !c.pf { pc = 0x10739f1a; continue; }
            pc = 0x10739f12; continue;
            }
            0x10739f12 => {
                { let v = 1u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
            pc = 0x10739f1a; continue;
            }
            0x10739f1a => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[6] = c.rd32(c.r[5].wrapping_add(28u32))?;
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.r[7] = c.r[4].wrapping_add(c.r[7].wrapping_mul(4)).wrapping_add(60u32);
                c.r[6] = c.alu(AluOp::Sub, c.r[6], 278724176u32, 32);
                c.r[1] = c.r[1];
            pc = 0x10739f30; continue;
            }
            0x10739f30 => {
                c.r[1] = c.rd32(c.r[7])?;
                c.r[2] = c.r[0].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[1] = c.alu(AluOp::Add, c.r[1], c.r[2], 32);
                c.r[2] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.r[2].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[1] = c.alu(AluOp::Add, c.r[1], c.r[2], 32);
                c.r[2] = c.r[1];
                c.r[3] = c.rd8(c.r[2].wrapping_add(c.r[2].wrapping_mul(2)).wrapping_add(338505033u32))? as u32;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = 8u32; c.set8(1, 0, v); }
                { let v = c.alu(AluOp::Sub, ((c.r[1] >> 0) & 0xff), c.rd8(c.r[0].wrapping_mul(4).wrapping_add(278724176u32))? as u32, 8); c.set8(1, 0, v); }
                c.r[7] = c.alu(AluOp::Add, c.r[7], 12u32, 32);
                c.r[3] = c.shift(ShiftOp::Shr, c.r[3], ((c.r[1] >> 0) & 0xff), 32);
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = 8u32; c.set8(1, 0, v); }
                { let v = c.r[3]; c.wr32(c.r[6].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(278724176u32), v)?; }
                { let v = c.rd8(c.r[0].wrapping_mul(4).wrapping_add(278724176u32))? as u32; c.set8(3, 0, v); }
                c.r[2] = c.rd8(c.r[2].wrapping_add(c.r[2].wrapping_mul(2)).wrapping_add(338505034u32))? as u32;
                { let v = c.alu(AluOp::Sub, ((c.r[1] >> 0) & 0xff), ((c.r[3] >> 0) & 0xff), 8); c.set8(1, 0, v); }
                c.r[2] = c.shift(ShiftOp::Shr, c.r[2], ((c.r[1] >> 0) & 0xff), 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], 3u32, 32);
                { let v = c.r[2]; c.wr32(c.r[6].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(278724184u32), v)?; }
                if c.sf != c.of { pc = 0x10739f30; continue; }
            pc = 0x10739f85; continue;
            }
            0x10739f85 => {
                { let v = c.rd8(c.r[5].wrapping_add(36u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x10739f9f; continue; }
            pc = 0x10739f8c; continue;
            }
            0x10739f8c => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(28u32))?;
                c.r[2] = c.rd32(c.r[0].wrapping_add(4u32))?;
                c.r[1] = c.rd32(c.r[0].wrapping_add(16u32))?;
                c.r[2] = c.shift(ShiftOp::Shl, c.r[2], 1u32, 32);
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 1u32, 32);
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(4u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[0].wrapping_add(16u32), v)?; }
            pc = 0x10739f9f; continue;
            }
            0x10739f9f => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                c.r[2] = c.rd32(c.r[4].wrapping_add(16u32))?;
                if c.zf || c.sf != c.of { pc = 0x10739fb2; continue; }
            pc = 0x10739faa; continue;
            }
            0x10739faa => {
                c.r[7] = c.rd32(c.r[5].wrapping_add(32u32))?;
                c.r[0] = c.r[2].wrapping_add(1u32);
                c.rep_stos(4)?;
            pc = 0x10739fb2; continue;
            }
            0x10739fb2 => {
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                if c.zf { pc = 0x10739fbe; continue; }
            pc = 0x10739fb6; continue;
            }
            0x10739fb6 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[0] = c.r[2].wrapping_add(2u32);
                { let v = c.r[0]; c.wr32(c.r[1], v)?; }
            pc = 0x10739fbe; continue;
            }
            0x10739fbe => {
                c.fpush(c.rdf64(c.r[2].wrapping_mul(8).wrapping_add(338505008u32))?);
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[3] = v; }
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10739fd0(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10739fd0;
    loop {
        match pc {
            0x10739fd0 => {
                c.push(c.r[5])?;
                c.r[5] = c.r[4];
                c.r[4] = c.alu(AluOp::And, c.r[4], 4294967288u32, 32);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 208u32, 32);
                c.push(c.r[6])?;
                c.push(c.r[7])?;
                c.r[7] = c.rd32(c.r[3])?;
                c.r[6] = c.r[0];
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.r[1] = c.r[4].wrapping_add(88u32);
                c.push(c.r[1])?;
                c.r[1] = c.rd32(c.r[5].wrapping_add(24u32))?;
                c.alu(AluOp::Sub, c.r[7], 4u32, 32);
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(0, 0, v); }
                c.r[2] = c.r[4].wrapping_add(44u32);
                c.push(c.r[2])?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(20u32))?;
                c.push(c.r[1])?;
                c.r[1] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.push(c.r[2])?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.push(c.r[1])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
                c.push(c.r[2])?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 3u32, 32);
                c.push(c.r[0])?;
                { let v = c.rd8(c.r[5].wrapping_add(36u32))? as u32; c.set8(0, 0, v); }
                c.r[2] = c.r[6];
                c.push(0x1073a016)?; dispatch(c, 0x107381f0)?;
                { let v = c.st0(); c.wrf64(338512000u32, v)?; c.fpop(); }
                c.r[1] = c.rd32(c.r[3])?;
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4294967294u32, 32);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], 2u32, 32);
                { let v = c.r[1]; c.wr32(c.r[3], v)?; }
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.r[4] = c.alu(AluOp::Add, c.r[4], 28u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                c.r[7] = 1u32;
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[5].wrapping_add(8u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073a1c9; continue; }
            pc = 0x1073a046; continue;
            }
            0x1073a046 => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
            pc = 0x1073a050; continue;
            }
            0x1073a050 => {
                c.r[1] = c.rd32(c.r[3])?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], c.r[0], 32);
                c.r[2] = c.rd32(c.r[1].wrapping_mul(4).wrapping_add(278725024u32))?;
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073a1a3; continue; }
            pc = 0x1073a06b; continue;
            }
            0x1073a06b => {
                pc = 0x1073a070; continue;
            }
            0x1073a070 => {
                c.r[2] = c.alu(AluOp::Xor, c.r[2], c.r[2], 32);
                c.r[0] = 338511960u32;
                pc = 0x1073a080; continue;
            }
            0x1073a080 => {
                c.r[1] = c.rd32(c.r[3])?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], c.rd32(c.r[4].wrapping_add(12u32))?, 32);
                c.r[7] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[1] = c.r[1].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[1] = c.r[7].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[7] = c.rd32(c.r[4].wrapping_add(12u32))?;
                c.r[1] = c.r[2].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 4u32, 32);
                c.fpush(c.rdf64(c.r[1].wrapping_add(278724256u32))?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 24u32, 32);
                { let v = c.rdf64(c.r[6])?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(278724264u32))?);
                { let v = c.rdf64(c.r[6].wrapping_add(24u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967264u32), v)?; c.fpop(); }
                c.r[1] = c.rd32(c.r[3])?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], c.r[7], 32);
                c.r[7] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[1] = c.r[1].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[1] = c.r[7].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[7] = c.rd32(c.r[4].wrapping_add(12u32))?;
                c.r[1] = c.r[2].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 4u32, 32);
                c.fpush(c.rdf64(c.r[1].wrapping_add(278724264u32))?);
                { let v = c.rdf64(c.r[6].wrapping_add(32u32))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(278724256u32))?);
                { let v = c.rdf64(c.r[6].wrapping_add(8u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967272u32), v)?; c.fpop(); }
                c.r[1] = c.rd32(c.r[3])?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], c.r[7], 32);
                c.r[7] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[1] = c.r[1].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[1] = c.r[7].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[1] = c.r[2].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 4u32, 32);
                c.fpush(c.rdf64(c.r[1].wrapping_add(278724264u32))?);
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.alu(AluOp::Sub, c.r[0], 338512008u32, 32);
                { let v = c.rdf64(c.r[6].wrapping_add(40u32))?; c.farith_value(FOp::Mul, v); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(278724256u32))?);
                { let v = c.rdf64(c.r[6].wrapping_add(16u32))?; c.farith_value(FOp::Mul, v); }
                c.farith(FOp::Add, 1, 0, true);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967280u32), v)?; c.fpop(); }
                if c.sf != c.of { pc = 0x1073a080; continue; }
            pc = 0x1073a119; continue;
            }
            0x1073a119 => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[2] = c.r[7];
                c.r[2] = c.shift(ShiftOp::Shl, c.r[2], 6u32, 32);
                c.r[1] = c.r[4].wrapping_add(c.r[2]).wrapping_add(88u32);
                c.push(c.r[1])?;
                c.r[2] = c.r[7].wrapping_add(c.r[7].wrapping_mul(2));
                c.r[1] = c.r[4].wrapping_add(c.r[2].wrapping_mul(8)).wrapping_add(44u32);
                c.r[2] = c.rd32(c.r[5].wrapping_add(24u32))?;
                c.push(c.r[1])?;
                c.r[1] = c.rd32(c.r[5].wrapping_add(20u32))?;
                c.push(c.r[2])?;
                c.r[2] = c.rd32(c.r[5].wrapping_add(16u32))?;
                c.push(c.r[1])?;
                c.r[1] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.push(c.r[2])?;
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(c.r[7].wrapping_mul(4)).wrapping_add(52u32), v)?; }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 3u32, 32);
                c.push(c.r[1])?;
                c.push(c.r[0])?;
                { let v = c.rd8(c.r[5].wrapping_add(36u32))? as u32; c.set8(0, 0, v); }
                c.r[2] = 338511952u32;
                c.push(0x1073a158)?; dispatch(c, 0x107381f0)?;
                c.r[0] = c.r[7].wrapping_mul(8);
                c.r[2] = 338512008u32;
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(338512000u32), v)?;  }
                c.r[2] = c.alu(AluOp::Sub, c.r[2], c.r[0], 32);
                c.fcom_value(c.rdf64(c.r[2])?, true);
                c.r[4] = c.alu(AluOp::Add, c.r[4], 28u32, 32);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x1073a185; continue; }
            pc = 0x1073a178; continue;
            }
            0x1073a178 => {
                c.r[0] = 1u32;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[7], 32);
                c.r[7] = c.r[0];
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
            pc = 0x1073a185; continue;
            }
            0x1073a185 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.rd32(c.r[3])?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[1] = c.incdec(c.r[1], 32, true);
                c.r[2] = c.alu(AluOp::Add, c.r[2], c.r[0], 32);
                c.alu(AluOp::Sub, c.r[1], c.rd32(c.r[2].wrapping_mul(4).wrapping_add(278725024u32))?, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.sf != c.of { pc = 0x1073a070; continue; }
            pc = 0x1073a1a3; continue;
            }
            0x1073a1a3 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(12u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[2] = c.alu(AluOp::Add, c.r[2], 2u32, 32);
                c.r[1] = c.incdec(c.r[1], 32, true);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(12u32), v)?; }
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 3u32, 32);
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.sf != c.of { pc = 0x1073a050; continue; }
            pc = 0x1073a1c9; continue;
            }
            0x1073a1c9 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(12u32))?;
                c.r[2] = 1u32;
                c.r[2] = c.alu(AluOp::Sub, c.r[2], c.r[7], 32);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                if c.zf || c.sf != c.of { pc = 0x1073a1e5; continue; }
            pc = 0x1073a1d7; continue;
            }
            0x1073a1d7 => {
                c.r[7] = c.rd32(c.r[5].wrapping_add(32u32))?;
                c.r[0] = c.r[2];
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 6u32, 32);
                c.r[6] = c.r[4].wrapping_add(c.r[0]).wrapping_add(88u32);
                c.rep_movs(4)?;
            pc = 0x1073a1e5; continue;
            }
            0x1073a1e5 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(28u32))?;
                c.r[0] = c.r[2].wrapping_add(c.r[2].wrapping_mul(2));
                c.r[1] = c.alu(AluOp::Add, c.r[1], 8u32, 32);
                c.r[0] = c.r[4].wrapping_add(c.r[0].wrapping_mul(8)).wrapping_add(44u32);
                c.r[6] = 2u32;
            pc = 0x1073a1f7; continue;
            }
            0x1073a1f7 => {
                c.r[7] = c.rd32(c.r[0].wrapping_add(4294967292u32))?;
                { let v = c.r[7]; c.wr32(c.r[1].wrapping_add(4294967288u32), v)?; }
                c.r[7] = c.rd32(c.r[0])?;
                { let v = c.r[7]; c.wr32(c.r[1].wrapping_add(4294967292u32), v)?; }
                c.r[7] = c.rd32(c.r[0].wrapping_add(4u32))?;
                { let v = c.r[7]; c.wr32(c.r[1], v)?; }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 12u32, 32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 12u32, 32);
                c.r[6] = c.incdec(c.r[6], 32, false);
                if !c.zf { pc = 0x1073a1f7; continue; }
            pc = 0x1073a210; continue;
            }
            0x1073a210 => {
                { let v = c.rd8(c.r[5].wrapping_add(36u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x1073a22f; continue; }
            pc = 0x1073a217; continue;
            }
            0x1073a217 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(28u32))?;
                c.r[0] = c.r[2].wrapping_add(c.r[2].wrapping_mul(2));
                c.r[6] = c.rd32(c.r[4].wrapping_add(c.r[0].wrapping_mul(8)).wrapping_add(44u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(c.r[0].wrapping_mul(8)).wrapping_add(56u32))?;
                c.r[6] = c.shift(ShiftOp::Shl, c.r[6], 1u32, 32);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 1u32, 32);
                { let v = c.r[6]; c.wr32(c.r[1].wrapping_add(4u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[1].wrapping_add(16u32), v)?; }
            pc = 0x1073a22f; continue;
            }
            0x1073a22f => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(c.r[2].wrapping_mul(4)).wrapping_add(32u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 3u32, 32);
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.r[1]; c.wr32(c.r[3], v)?; }
                c.fpush(c.rdf64(c.r[2].wrapping_mul(8).wrapping_add(338512000u32))?);
                { let v = c.pop()?; c.r[6] = v; }
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_1073a250(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x1073a250;
    loop {
        match pc {
            0x1073a250 => {
                c.push(c.r[5])?;
                c.r[5] = c.r[4];
                c.r[4] = c.alu(AluOp::And, c.r[4], 4294967288u32, 32);
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 1284u32, 32);
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                c.push(c.r[3])?;
                c.push(c.r[6])?;
                c.push(c.r[7])?;
                c.r[3] = 1u32;
                if c.zf { pc = 0x1073aff7; continue; }
            pc = 0x1073a26f; continue;
            }
            0x1073a26f => {
                c.r[0] = c.rd32(c.r[1])?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(32u32), v)?; }
                if c.zf { pc = 0x1073aff7; continue; }
            pc = 0x1073a27d; continue;
            }
            0x1073a27d => {
                c.fpush(c.rd32(c.r[4].wrapping_add(32u32))? as i32 as f64);
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Divr, v); }
                { let v = c.st0(); c.wrf64(c.r[4].wrapping_add(64u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(392u32))?);
                c.fcom_value(c.rdf64(276379832u32)?, true);
                c.r[0] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(68u32))?;
                { let v = c.r[0]; c.wr32(338513912u32, v)?; }
                { let v = c.r[2]; c.wr32(338513916u32, v)?; }
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x1073a2b9; continue; }
            pc = 0x1073a2b1; continue;
            }
            0x1073a2b1 => {
                c.fpush(c.rdf64(276379832u32)?);
                pc = 0x1073a2bf; continue;
            }
            0x1073a2b9 => {
                c.fpush(c.rdf64(c.r[1].wrapping_add(392u32))?);
            pc = 0x1073a2bf; continue;
            }
            0x1073a2bf => {
                { let v = c.st0(); c.wrf64(338513888u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(400u32))?);
                c.fcom_value(c.rdf64(276379832u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x1073a2e0; continue; }
            pc = 0x1073a2d8; continue;
            }
            0x1073a2d8 => {
                c.fpush(c.rdf64(276379832u32)?);
                pc = 0x1073a2e6; continue;
            }
            0x1073a2e0 => {
                c.fpush(c.rdf64(c.r[1].wrapping_add(400u32))?);
            pc = 0x1073a2e6; continue;
            }
            0x1073a2e6 => {
                { let v = c.st0(); c.wrf64(338513896u32, v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(408u32))?);
                c.fcom_value(c.rdf64(276379832u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if c.pf { pc = 0x1073a307; continue; }
            pc = 0x1073a2ff; continue;
            }
            0x1073a2ff => {
                c.fpush(c.rdf64(276379832u32)?);
                pc = 0x1073a30d; continue;
            }
            0x1073a307 => {
                c.fpush(c.rdf64(c.r[1].wrapping_add(408u32))?);
            pc = 0x1073a30d; continue;
            }
            0x1073a30d => {
                { let v = c.st0(); c.wrf64(338513904u32, v)?; c.fpop(); }
                { let v = c.rd8(c.r[1].wrapping_add(420u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x1073a32e; continue; }
            pc = 0x1073a31d; continue;
            }
            0x1073a31d => {
                c.r[0] = 4u32;
                { let v = c.r[0]; c.wr32(c.r[1].wrapping_add(424u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[1].wrapping_add(416u32), v)?; }
            pc = 0x1073a32e; continue;
            }
            0x1073a32e => {
                c.r[0] = 338513520u32;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[7] = c.r[1].wrapping_add(8u32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                pc = 0x1073a350; continue;
            }
            0x1073a350 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                { let v = c.r[1]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338513480u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[0].wrapping_mul(8).wrapping_add(338513484u32), v)?; }
                c.alu(AluOp::Sub, c.rd32(c.r[4].wrapping_add(32u32))?, 4u32, 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                if c.sf != c.of { pc = 0x1073a491; continue; }
            pc = 0x1073a373; continue;
            }
            0x1073a373 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[6] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[2] = c.r[7].wrapping_add(c.r[0]);
                c.r[0] = 338513544u32;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[6], 32);
                { let v = 3u32; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                c.r[1] = c.r[7].wrapping_add(24u32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(56u32), v)?; }
            pc = 0x1073a393; continue;
            }
            0x1073a393 => {
                c.fpush(c.rdf64(c.r[1].wrapping_add(4294967272u32))?);
                c.fcom_value(c.rdf64(c.r[7])?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a3a6; continue; }
            pc = 0x1073a39f; continue;
            }
            0x1073a39f => {
                c.r[0] = 1u32;
                pc = 0x1073a3a8; continue;
            }
            0x1073a3a6 => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
            pc = 0x1073a3a8; continue;
            }
            0x1073a3a8 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.fpush(c.rdf64(c.r[6].wrapping_mul(8).wrapping_add(338513888u32))?);
                c.r[3] = c.alu(AluOp::And, c.r[3], c.r[0], 32);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967272u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[2].wrapping_add(4294967272u32), v)?;  }
                { let v = c.rdf64(c.r[6].wrapping_mul(8).wrapping_add(338513480u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_mul(8).wrapping_add(338513480u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[1])?);
                c.fcom_value(c.rdf64(c.r[7])?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a3db; continue; }
            pc = 0x1073a3d4; continue;
            }
            0x1073a3d4 => {
                c.r[0] = 1u32;
                pc = 0x1073a3dd; continue;
            }
            0x1073a3db => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
            pc = 0x1073a3dd; continue;
            }
            0x1073a3dd => {
                c.fpush(c.rdf64(c.r[6].wrapping_mul(8).wrapping_add(338513888u32))?);
                c.r[3] = c.alu(AluOp::And, c.r[3], c.r[0], 32);
                { let v = c.rdf64(c.r[1])?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[2], v)?;  }
                { let v = c.rdf64(c.r[6].wrapping_mul(8).wrapping_add(338513480u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_mul(8).wrapping_add(338513480u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(24u32))?);
                c.fcom_value(c.rdf64(c.r[7])?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a40b; continue; }
            pc = 0x1073a404; continue;
            }
            0x1073a404 => {
                c.r[0] = 1u32;
                pc = 0x1073a40d; continue;
            }
            0x1073a40b => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
            pc = 0x1073a40d; continue;
            }
            0x1073a40d => {
                c.fpush(c.rdf64(c.r[6].wrapping_mul(8).wrapping_add(338513888u32))?);
                c.r[3] = c.alu(AluOp::And, c.r[3], c.r[0], 32);
                { let v = c.rdf64(c.r[1].wrapping_add(24u32))?; c.farith_value(FOp::Mul, v); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.fld_st(0);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(c.r[1]), v)?; c.fpop(); }
                { let v = c.rdf64(c.r[6].wrapping_mul(8).wrapping_add(338513480u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_mul(8).wrapping_add(338513480u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(48u32))?);
                c.fcom_value(c.rdf64(c.r[7])?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a443; continue; }
            pc = 0x1073a43c; continue;
            }
            0x1073a43c => {
                c.r[0] = 1u32;
                pc = 0x1073a445; continue;
            }
            0x1073a443 => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
            pc = 0x1073a445; continue;
            }
            0x1073a445 => {
                c.fpush(c.rdf64(c.r[6].wrapping_mul(8).wrapping_add(338513888u32))?);
                c.r[3] = c.alu(AluOp::And, c.r[3], c.r[0], 32);
                { let v = c.rdf64(c.r[1].wrapping_add(48u32))?; c.farith_value(FOp::Mul, v); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 96u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 96u32, 32);
                c.fld_st(0);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(c.r[1]).wrapping_add(4294967200u32), v)?; c.fpop(); }
                c.r[0] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 4u32, 32);
                { let v = c.rdf64(c.r[6].wrapping_mul(8).wrapping_add(338513480u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(40u32))?;
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_mul(8).wrapping_add(338513480u32), v)?; c.fpop(); }
                c.r[6] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 4u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[6], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                if c.sf != c.of { pc = 0x1073a393; continue; }
            pc = 0x1073a491; continue;
            }
            0x1073a491 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.alu(AluOp::Sub, c.r[1], c.rd32(c.r[4].wrapping_add(32u32))?, 32);
                if c.sf == c.of { pc = 0x1073a4fd; continue; }
            pc = 0x1073a49b; continue;
            }
            0x1073a49b => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[0] = c.r[1];
                c.r[1] = c.r[2].wrapping_add(c.r[0].wrapping_mul(2));
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[1], 32);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 3u32, 32);
                c.r[1] = c.r[0].wrapping_add(c.r[2]).wrapping_add(8u32);
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[6] = c.r[0].wrapping_add(338513504u32);
                c.r[2] = c.alu(AluOp::Sub, c.r[2], c.rd32(c.r[4].wrapping_add(20u32))?, 32);
                c.r[7] = c.r[7];
            pc = 0x1073a4c0; continue;
            }
            0x1073a4c0 => {
                c.fpush(c.rdf64(c.r[1])?);
                c.fcom_value(c.rdf64(c.r[7])?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a4d2; continue; }
            pc = 0x1073a4cb; continue;
            }
            0x1073a4cb => {
                c.r[0] = 1u32;
                pc = 0x1073a4d4; continue;
            }
            0x1073a4d2 => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
            pc = 0x1073a4d4; continue;
            }
            0x1073a4d4 => {
                c.fpush(c.rdf64(c.r[1])?);
                c.r[3] = c.alu(AluOp::And, c.r[3], c.r[0], 32);
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                { let v = c.rdf64(c.r[0].wrapping_mul(8).wrapping_add(338513888u32))?; c.farith_value(FOp::Mul, v); }
                c.r[1] = c.alu(AluOp::Add, c.r[1], 24u32, 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 24u32, 32);
                c.r[2] = c.incdec(c.r[2], 32, false);
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_add(4294967272u32), v)?;  }
                { let v = c.rdf64(c.r[0].wrapping_mul(8).wrapping_add(338513480u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_mul(8).wrapping_add(338513480u32), v)?; c.fpop(); }
                if !c.zf { pc = 0x1073a4c0; continue; }
            pc = 0x1073a4fd; continue;
            }
            0x1073a4fd => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[7] = c.alu(AluOp::Add, c.r[7], 8u32, 32);
                c.alu(AluOp::Sub, c.r[0], 3u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.sf != c.of { pc = 0x1073a350; continue; }
            pc = 0x1073a512; continue;
            }
            0x1073a512 => {
                c.r[6] = c.rd32(c.r[5].wrapping_add(8u32))?;
                { let v = 1u32; c.wr8(c.r[4].wrapping_add(76u32), v as u8)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[6] = c.alu(AluOp::Add, c.r[6], 16u32, 32);
                pc = 0x1073a530; continue;
            }
            0x1073a530 => {
                c.fpush(c.rdf64(c.r[6].wrapping_add(4294967288u32))?);
                { let v = c.rdf64(276956504u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Add, v); }
                c.push(0x1073a544)?; dispatch(c, 0x10740cb4)?;
                c.fpush(c.rdf64(c.r[6])?);
                { let v = c.rdf64(276956504u32)?; c.farith_value(FOp::Mul, v); }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Add, v); }
                c.push(0x1073a55b)?; dispatch(c, 0x10740cb4)?;
                c.fpush(c.rdf64(c.r[6].wrapping_add(8u32))?);
                { let v = c.rdf64(276956504u32)?; c.farith_value(FOp::Mul, v); }
                c.r[7] = c.r[0];
                { let v = c.rdf64(276674888u32)?; c.farith_value(FOp::Add, v); }
                c.push(0x1073a571)?; dispatch(c, 0x10740cb4)?;
                c.r[1] = c.r[0];
                c.r[0] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[7], 32);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[2], 32);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[0], 2u32, 32);
                if !c.zf && c.sf == c.of { pc = 0x1073a5ba; continue; }
            pc = 0x1073a583; continue;
            }
            0x1073a583 => {
                c.r[0] = c.r[7];
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[2], 32);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[0], 2u32, 32);
                if !c.zf && c.sf == c.of { pc = 0x1073a5ba; continue; }
            pc = 0x1073a591; continue;
            }
            0x1073a591 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[0] = c.r[1];
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.r[2] = if (c.r[0] as i32) < 0 { 0xffff_ffff } else { 0 };
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[2], 32);
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                c.alu(AluOp::Sub, c.r[0], 2u32, 32);
                if !c.zf && c.sf == c.of { pc = 0x1073a5ba; continue; }
            pc = 0x1073a5a3; continue;
            }
            0x1073a5a3 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 24u32, 32);
                c.alu(AluOp::Sub, c.r[0], 16u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                if c.sf != c.of { pc = 0x1073a530; continue; }
            pc = 0x1073a5b8; continue;
            }
            0x1073a5b8 => {
                pc = 0x1073a5bf; continue;
            }
            0x1073a5ba => {
                { let v = 0u32; c.wr8(c.r[4].wrapping_add(76u32), v as u8)?; }
            pc = 0x1073a5bf; continue;
            }
            0x1073a5bf => {
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if c.zf { pc = 0x1073a613; continue; }
            pc = 0x1073a5c3; continue;
            }
            0x1073a5c3 => {
                c.r[6] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[1] = c.rd32(c.r[6].wrapping_add(416u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(76u32))?;
                c.push(c.r[2])?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[0] = c.r[6].wrapping_add(452u32);
                c.push(c.r[0])?;
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(72u32), v)?; }
                c.r[1] = c.r[6].wrapping_add(428u32);
                c.push(c.r[1])?;
                c.push(338513888u32)?;
                c.push(338513504u32)?;
                c.push(c.r[2])?;
                c.r[0] = c.r[4].wrapping_add(88u32);
                c.push(c.r[0])?;
                c.push(c.r[6])?;
                c.push(0x1073a5fd)?; dispatch(c, 0x107399d0)?;
                c.fst_st(0, true);
                c.r[1] = c.rd32(c.r[4].wrapping_add(96u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 32u32, 32);
                { let v = c.r[1]; c.wr32(c.r[6].wrapping_add(424u32), v)?; }
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[3] = v; }
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            0x1073a613 => {
                c.r[3] = c.alu(AluOp::Xor, c.r[3], c.r[3], 32);
                pc = 0x1073a620; continue;
            }
            0x1073a620 => {
                c.fpush(c.rdf64(c.r[4].wrapping_add(64u32))?);
                c.r[0] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[6] = c.r[3].wrapping_mul(8);
                { let v = c.rdf64(c.r[6].wrapping_add(338513480u32))?; c.farith_value(FOp::Mul, v); }
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::Sub, c.r[0], 4u32, 32);
                c.fld_st(0);
                c.fld_st(1);
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_add(338513480u32), v)?; c.fpop(); }
                if c.sf != c.of { pc = 0x1073a69a; continue; }
            pc = 0x1073a646; continue;
            }
            0x1073a646 => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.fld_st(0);
                c.r[2] = 3u32;
                c.r[0] = c.r[6];
            pc = 0x1073a653; continue;
            }
            0x1073a653 => {
                c.fpush(c.rdf64(c.r[0].wrapping_add(338513504u32))?);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 4u32, 32);
                c.farith(FOp::Sub, 0, 1, false);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 96u32, 32);
                c.alu(AluOp::Sub, c.r[2], c.r[7], 32);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(338513000u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(338513432u32))?);
                c.farith(FOp::Sub, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(338513024u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(338513456u32))?);
                c.farith(FOp::Sub, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(338513048u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(338513480u32))?);
                c.farith(FOp::Sub, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(338513072u32), v)?; c.fpop(); }
                if c.sf != c.of { pc = 0x1073a653; continue; }
            pc = 0x1073a698; continue;
            }
            0x1073a698 => {
                c.fst_st(0, true);
            pc = 0x1073a69a; continue;
            }
            0x1073a69a => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.alu(AluOp::Sub, c.r[1], c.r[2], 32);
                if c.sf == c.of { pc = 0x1073a6c4; continue; }
            pc = 0x1073a6a2; continue;
            }
            0x1073a6a2 => {
                c.r[7] = c.r[3].wrapping_add(c.r[1].wrapping_mul(2));
                c.r[0] = c.r[1];
                c.r[0] = c.alu(AluOp::Add, c.r[0], c.r[7], 32);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 3u32, 32);
                c.r[2] = c.alu(AluOp::Sub, c.r[2], c.r[1], 32);
                c.r[7] = c.r[7];
            pc = 0x1073a6b0; continue;
            }
            0x1073a6b0 => {
                c.fpush(c.rdf64(c.r[0].wrapping_add(338513504u32))?);
                c.r[0] = c.alu(AluOp::Add, c.r[0], 24u32, 32);
                c.r[2] = c.incdec(c.r[2], 32, false);
                c.farith(FOp::Sub, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(338513072u32), v)?; c.fpop(); }
                if !c.zf { pc = 0x1073a6b0; continue; }
            pc = 0x1073a6c4; continue;
            }
            0x1073a6c4 => {
                c.fst_st(0, true);
                c.r[3] = c.incdec(c.r[3], 32, true);
                c.alu(AluOp::Sub, c.r[3], 3u32, 32);
                c.fld_st(0);
                c.fchs();
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_add(338513048u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[6].wrapping_add(338513888u32))?);
                c.farith(FOp::Sub, 0, 1, false);
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_add(338513072u32), v)?; c.fpop(); }
                c.fst_st(0, true);
                if c.sf != c.of { pc = 0x1073a620; continue; }
            pc = 0x1073a6ea; continue;
            }
            0x1073a6ea => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.r[7] = c.alu(AluOp::Xor, c.r[7], c.r[7], 32);
                c.alu(AluOp::Sub, c.r[0], c.r[1], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
                if c.zf || c.sf != c.of { pc = 0x1073a96c; continue; }
            pc = 0x1073a702; continue;
            }
            0x1073a702 => {
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.alu(AluOp::Sub, c.r[7], 4u32, 32);
                if c.sf != c.of { pc = 0x1073a7f3; continue; }
            pc = 0x1073a70d; continue;
            }
            0x1073a70d => {
                c.r[3] = 3u32;
                c.r[2] = 338512672u32;
            pc = 0x1073a717; continue;
            }
            0x1073a717 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967288u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513504u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a74a; continue; }
            pc = 0x1073a727; continue;
            }
            0x1073a727 => {
                c.fpush(c.rdf64(c.r[2])?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513512u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a74a; continue; }
            pc = 0x1073a736; continue;
            }
            0x1073a736 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(8u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513520u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if !c.pf { pc = 0x1073a845; continue; }
            pc = 0x1073a74a; continue;
            }
            0x1073a74a => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(16u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513504u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a77e; continue; }
            pc = 0x1073a75a; continue;
            }
            0x1073a75a => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(24u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513512u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a77e; continue; }
            pc = 0x1073a76a; continue;
            }
            0x1073a76a => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(32u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513520u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if !c.pf { pc = 0x1073a83a; continue; }
            pc = 0x1073a77e; continue;
            }
            0x1073a77e => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(40u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513504u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a7b2; continue; }
            pc = 0x1073a78e; continue;
            }
            0x1073a78e => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(48u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513512u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a7b2; continue; }
            pc = 0x1073a79e; continue;
            }
            0x1073a79e => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(56u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513520u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if !c.pf { pc = 0x1073a83d; continue; }
            pc = 0x1073a7b2; continue;
            }
            0x1073a7b2 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(64u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513504u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a7e2; continue; }
            pc = 0x1073a7c2; continue;
            }
            0x1073a7c2 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(72u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513512u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a7e2; continue; }
            pc = 0x1073a7d2; continue;
            }
            0x1073a7d2 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(80u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513520u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if !c.pf { pc = 0x1073a842; continue; }
            pc = 0x1073a7e2; continue;
            }
            0x1073a7e2 => {
                c.r[3] = c.alu(AluOp::Add, c.r[3], 4u32, 32);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 4u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 96u32, 32);
                c.alu(AluOp::Sub, c.r[3], c.r[7], 32);
                if c.sf != c.of { pc = 0x1073a717; continue; }
            pc = 0x1073a7f3; continue;
            }
            0x1073a7f3 => {
                c.alu(AluOp::Sub, c.r[6], c.r[7], 32);
                if c.sf == c.of { pc = 0x1073a847; continue; }
            pc = 0x1073a7f7; continue;
            }
            0x1073a7f7 => {
                c.r[2] = c.r[6].wrapping_add(c.r[6].wrapping_mul(2));
                c.r[2] = c.r[2].wrapping_mul(8).wrapping_add(338512672u32);
            pc = 0x1073a801; continue;
            }
            0x1073a801 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(4294967288u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513504u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a830; continue; }
            pc = 0x1073a811; continue;
            }
            0x1073a811 => {
                c.fpush(c.rdf64(c.r[2])?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513512u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if c.pf { pc = 0x1073a830; continue; }
            pc = 0x1073a820; continue;
            }
            0x1073a820 => {
                c.fpush(c.rdf64(c.r[2].wrapping_add(8u32))?);
                c.fcom_value(c.rdf64(c.r[1].wrapping_add(338513520u32))?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 68u32, 8);
                if !c.pf { pc = 0x1073a845; continue; }
            pc = 0x1073a830; continue;
            }
            0x1073a830 => {
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 24u32, 32);
                c.alu(AluOp::Sub, c.r[6], c.r[7], 32);
                if c.sf != c.of { pc = 0x1073a801; continue; }
            pc = 0x1073a838; continue;
            }
            0x1073a838 => {
                pc = 0x1073a845; continue;
            }
            0x1073a83a => {
                c.r[6] = c.incdec(c.r[6], 32, true);
                pc = 0x1073a845; continue;
            }
            0x1073a83d => {
                c.r[6] = c.alu(AluOp::Add, c.r[6], 2u32, 32);
                pc = 0x1073a845; continue;
            }
            0x1073a842 => {
                c.r[6] = c.alu(AluOp::Add, c.r[6], 3u32, 32);
            pc = 0x1073a845; continue;
            }
            0x1073a845 => {
                c.alu(AluOp::Sub, c.r[6], c.r[7], 32);
            pc = 0x1073a847; continue;
            }
            0x1073a847 => {
                c.r[0] = c.r[6].wrapping_add(c.r[6].wrapping_mul(2));
                if !c.zf { pc = 0x1073a8fc; continue; }
            pc = 0x1073a850; continue;
            }
            0x1073a850 => {
                c.r[2] = c.rd32(c.r[1].wrapping_add(338513504u32))?;
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 3u32, 32);
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(338512664u32), v)?; }
                c.r[2] = c.rd32(c.r[1].wrapping_add(338513508u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(338512668u32), v)?; }
                c.r[2] = c.rd32(c.r[1].wrapping_add(338513512u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(338512672u32), v)?; }
                c.r[2] = c.rd32(c.r[1].wrapping_add(338513516u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(338512676u32), v)?; }
                c.r[2] = c.rd32(c.r[1].wrapping_add(338513520u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(338512680u32), v)?; }
                c.r[2] = c.rd32(c.r[1].wrapping_add(338513524u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(338512684u32), v)?; }
                c.r[2] = c.rd32(c.r[1].wrapping_add(338513096u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(338512280u32), v)?; }
                c.r[2] = c.rd32(c.r[1].wrapping_add(338513100u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(338512284u32), v)?; }
                c.r[2] = c.rd32(c.r[1].wrapping_add(338513104u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(338512288u32), v)?; }
                c.r[2] = c.rd32(c.r[1].wrapping_add(338513108u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(338512292u32), v)?; }
                c.r[2] = c.rd32(c.r[1].wrapping_add(338513112u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(338512296u32), v)?; }
                c.r[2] = c.rd32(c.r[1].wrapping_add(338513116u32))?;
                { let v = c.r[2]; c.wr32(c.r[0].wrapping_add(338512300u32), v)?; }
                { let v = 0u32; c.wr32(c.r[6].wrapping_mul(8).wrapping_add(338512152u32), v)?; }
                { let v = 1072693248u32; c.wr32(c.r[6].wrapping_mul(8).wrapping_add(338512156u32), v)?; }
                c.r[7] = c.incdec(c.r[7], 32, true);
                pc = 0x1073a949; continue;
            }
            0x1073a8fc => {
                c.fpush(c.rdf64(c.r[1].wrapping_add(338513096u32))?);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 3u32, 32);
                { let v = c.rdf64(c.r[0].wrapping_add(338512280u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(338512280u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[1].wrapping_add(338513104u32))?);
                { let v = c.rdf64(c.r[0].wrapping_add(338512288u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(338512288u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[0].wrapping_add(338512296u32))?);
                { let v = c.rdf64(c.r[1].wrapping_add(338513112u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(338512296u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(c.r[6].wrapping_mul(8).wrapping_add(338512152u32))?);
                { let v = c.rdf64(276379816u32)?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[6].wrapping_mul(8).wrapping_add(338512152u32), v)?; c.fpop(); }
            pc = 0x1073a949; continue;
            }
            0x1073a949 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(208u32), v)?; }
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 24u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                if c.sf != c.of { pc = 0x1073a702; continue; }
            pc = 0x1073a968; continue;
            }
            0x1073a968 => {
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(24u32), v)?; }
            pc = 0x1073a96c; continue;
            }
            0x1073a96c => {
                c.r[1] = c.alu(AluOp::Xor, c.r[1], c.r[1], 32);
                c.alu(AluOp::And, c.r[7], c.r[7], 32);
                if c.zf || c.sf != c.of { pc = 0x1073a9c0; continue; }
            pc = 0x1073a972; continue;
            }
            0x1073a972 => {
                c.r[0] = 338512288u32;
                pc = 0x1073a980; continue;
            }
            0x1073a980 => {
                c.fpush(c.rdf64(338513912u32)?);
                c.r[1] = c.incdec(c.r[1], 32, true);
                { let v = c.rdf64(c.r[1].wrapping_mul(8).wrapping_add(338512144u32))?; c.farith_value(FOp::Mul, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 24u32, 32);
                c.alu(AluOp::Sub, c.r[1], c.r[7], 32);
                { let v = c.st0(); c.wrf64(c.r[1].wrapping_mul(8).wrapping_add(338512144u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(338513912u32)?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967264u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967264u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(338513912u32)?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967272u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967272u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(338513912u32)?);
                { let v = c.rdf64(c.r[0].wrapping_add(4294967280u32))?; c.farith_value(FOp::Mul, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967280u32), v)?; c.fpop(); }
                if c.sf != c.of { pc = 0x1073a980; continue; }
            pc = 0x1073a9c0; continue;
            }
            0x1073a9c0 => {
                c.alu(AluOp::Sub, c.r[7], 2u32, 32);
                if !c.zf { pc = 0x1073aa4c; continue; }
            pc = 0x1073a9c9; continue;
            }
            0x1073a9c9 => {
                c.r[6] = c.rd32(c.r[5].wrapping_add(8u32))?;
                { let v = c.rd8(c.r[6].wrapping_add(420u32))? as u32; c.set8(0, 0, v); }
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if !c.zf { pc = 0x1073aa4c; continue; }
            pc = 0x1073a9d6; continue;
            }
            0x1073a9d6 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(76u32))?;
                c.push(c.r[0])?;
                c.r[0] = c.rd32(c.r[6].wrapping_add(416u32))?;
                c.r[1] = c.r[4].wrapping_add(84u32);
                c.push(c.r[1])?;
                c.r[2] = c.r[6].wrapping_add(428u32);
                c.push(c.r[2])?;
                c.push(338513888u32)?;
                c.push(338512152u32)?;
                c.push(338512664u32)?;
                c.push(c.r[7])?;
                c.r[3] = c.r[6].wrapping_add(424u32);
                c.push(c.r[0])?;
                c.r[0] = 338512664u32;
                { let v = c.r[7]; c.wr32(c.r[3], v)?; }
                c.push(0x1073aa10)?; dispatch(c, 0x10739fd0)?;
                c.fst_st(0, true);
                c.r[2] = c.rd32(c.r[4].wrapping_add(64u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 32u32, 32);
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                if c.zf || c.sf != c.of { pc = 0x1073aff7; continue; }
            pc = 0x1073aa23; continue;
            }
            0x1073aa23 => {
                c.r[1] = c.r[6].wrapping_add(452u32);
                c.r[4] = c.r[4];
            pc = 0x1073aa30; continue;
            }
            0x1073aa30 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(208u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(c.r[6].wrapping_mul(4)).wrapping_add(80u32))?;
                { let v = c.r[6]; c.wr32(c.r[1], v)?; }
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                if c.sf != c.of { pc = 0x1073aa30; continue; }
            pc = 0x1073aa45; continue;
            }
            0x1073aa45 => {
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[3] = v; }
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            0x1073aa4c => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
                c.r[3] = 338512128u32;
                c.r[0] = 338513096u32;
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
                c.push(0x1073aa65)?; dispatch(c, 0x10736430)?;
                c.push(c.r[6])?;
                c.r[1] = c.r[4].wrapping_add(276u32);
                c.push(c.r[1])?;
                c.push(c.r[7])?;
                c.r[0] = c.r[3];
                c.r[2] = 338512664u32;
                c.push(0x1073aa7b)?; dispatch(c, 0x10736c70)?;
                c.r[2] = c.r[4].wrapping_add(92u32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                c.r[3] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[0] = c.r[4].wrapping_add(76u32);
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
                { let v = c.r[6]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                { let v = 338512032u32; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(56u32), v)?; }
                { let v = 16u32; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                pc = 0x1073aac0; continue;
            }
            0x1073aaa8 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(36u32))?;
                { let v = 16u32; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                pc = 0x1073aac0; continue;
            }
            0x1073aab6 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[3] = c.r[3];
            pc = 0x1073aac0; continue;
            }
            0x1073aac0 => {
                c.r[1] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[2] = c.rd32(c.r[1].wrapping_add(416u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[2] = c.incdec(c.r[2], 32, false);
                { let v = c.r[2]; c.wr32(c.r[0], v)?; }
                c.r[2] = c.r[7].wrapping_add(c.r[1]).wrapping_add(4294967295u32);
                c.alu(AluOp::Sub, c.r[2], 16u32, 32);
                c.push(338512128u32)?;
                if c.zf { pc = 0x1073ab1c; continue; }
            pc = 0x1073aae6; continue;
            }
            0x1073aae6 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.push(c.r[1])?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.push(338512280u32)?;
                c.r[2] = c.r[6];
                c.push(c.r[1])?;
                c.r[2] = c.shift(ShiftOp::Shl, c.r[2], 6u32, 32);
                c.push(c.r[0])?;
                c.r[2] = c.r[4].wrapping_add(c.r[2]).wrapping_add(292u32);
                c.r[0] = 338512152u32;
                c.push(0x1073ab0c)?; dispatch(c, 0x10737000)?;
                { let v = c.st0(); c.wrf64(c.r[7].wrapping_mul(8).wrapping_add(338512016u32), v)?; c.fpop(); }
                c.r[4] = c.alu(AluOp::Add, c.r[4], 20u32, 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                pc = 0x1073ab2e; continue;
            }
            0x1073ab1c => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.push(338512664u32)?;
                c.push(c.r[2])?;
                c.push(0x1073ab2b)?; dispatch(c, 0x10736a20)?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
            pc = 0x1073ab2e; continue;
            }
            0x1073ab2e => {
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.r[0] = c.r[6];
                c.r[0] = c.alu(AluOp::And, c.r[0], 2147483663u32, 32);
                if !c.sf { pc = 0x1073ab3d; continue; }
            pc = 0x1073ab38; continue;
            }
            0x1073ab38 => {
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.r[0] = c.alu(AluOp::Or, c.r[0], 4294967280u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
            pc = 0x1073ab3d; continue;
            }
            0x1073ab3d => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(52u32))?;
                c.r[1] = 0u32;
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(1, 0, v); }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(60u32), v)?; }
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[1], 32);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 6u32, 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
                c.r[2] = c.r[4].wrapping_add(c.r[0]).wrapping_add(272u32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                c.r[4] = c.r[4];
            pc = 0x1073ab70; continue;
            }
            0x1073ab70 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(60u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.push(c.r[0])?;
                c.push(c.r[1])?;
                c.push(c.r[2])?;
                c.r[0] = 338512128u32;
                c.r[2] = 338512664u32;
                c.push(0x1073ab8e)?; dispatch(c, 0x10736c70)?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
                c.r[6] = c.incdec(c.r[6], 32, false);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                if c.sf != c.of { pc = 0x1073abc5; continue; }
            pc = 0x1073ab9a; continue;
            }
            0x1073ab9a => {
                c.r[0] = c.r[6];
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 6u32, 32);
                c.r[7] = c.r[4].wrapping_add(c.r[0]).wrapping_add(272u32);
            pc = 0x1073aba6; continue;
            }
            0x1073aba6 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[3] = c.r[7];
                c.push(0x1073abb5)?; dispatch(c, 0x10736fa0)?;
                c.r[3] = c.r[0];
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if !c.zf { pc = 0x1073ac10; continue; }
            pc = 0x1073abbb; continue;
            }
            0x1073abbb => {
                c.r[6] = c.incdec(c.r[6], 32, false);
                c.r[7] = c.alu(AluOp::Sub, c.r[7], 64u32, 32);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                if c.sf == c.of { pc = 0x1073aba6; continue; }
            pc = 0x1073abc3; continue;
            }
            0x1073abc3 => {
                pc = 0x1073abc9; continue;
            }
            0x1073abc5 => {
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if !c.zf { pc = 0x1073ac10; continue; }
            pc = 0x1073abc9; continue;
            }
            0x1073abc9 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(52u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x1073ac2a; continue; }
            pc = 0x1073abd1; continue;
            }
            0x1073abd1 => {
                c.r[7] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.alu(AluOp::Sub, c.r[7], 16u32, 32);
                if c.sf == c.of { pc = 0x1073ac2a; continue; }
            pc = 0x1073abdb; continue;
            }
            0x1073abdb => {
                c.r[1] = c.r[7];
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 6u32, 32);
                c.r[6] = c.r[4].wrapping_add(c.r[1]).wrapping_add(272u32);
                pc = 0x1073abf0; continue;
            }
            0x1073abf0 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[3] = c.r[6];
                c.push(0x1073abff)?; dispatch(c, 0x10736fa0)?;
                c.r[3] = c.r[0];
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if !c.zf { pc = 0x1073ac10; continue; }
            pc = 0x1073ac05; continue;
            }
            0x1073ac05 => {
                c.r[7] = c.incdec(c.r[7], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 64u32, 32);
                c.alu(AluOp::Sub, c.r[7], 16u32, 32);
                if c.sf != c.of { pc = 0x1073abf0; continue; }
            pc = 0x1073ac0e; continue;
            }
            0x1073ac0e => {
                pc = 0x1073ac2a; continue;
            }
            0x1073ac10 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x1073ac2a; continue; }
            pc = 0x1073ac18; continue;
            }
            0x1073ac18 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(60u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], 2u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(60u32), v)?; }
                if c.sf != c.of { pc = 0x1073ab70; continue; }
            pc = 0x1073ac2a; continue;
            }
            0x1073ac2a => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[2] = c.r[0];
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                if c.zf { pc = 0x1073ac41; continue; }
            pc = 0x1073ac39; continue;
            }
            0x1073ac39 => {
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if c.zf { pc = 0x1073aab6; continue; }
            pc = 0x1073ac41; continue;
            }
            0x1073ac41 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf || c.sf != c.of { pc = 0x1073ac5f; continue; }
            pc = 0x1073ac49; continue;
            }
            0x1073ac49 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[6] = c.shift(ShiftOp::Shl, c.r[6], 6u32, 32);
                c.r[6] = c.r[4].wrapping_add(c.r[6]).wrapping_add(272u32);
                c.r[1] = c.r[0];
                c.rep_movs(4)?;
            pc = 0x1073ac5f; continue;
            }
            0x1073ac5f => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[7] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(20u32), v)?; }
                c.r[0] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[0] = c.alu(AluOp::Add, c.r[0], 48u32, 32);
                c.r[7] = c.alu(AluOp::Add, c.r[7], 4u32, 32);
                c.r[2] = c.alu(AluOp::Add, c.r[2], 64u32, 32);
                c.alu(AluOp::Sub, c.r[0], 338512128u32, 32);
                { let v = c.r[7]; c.wr32(c.r[4].wrapping_add(56u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.sf != c.of { pc = 0x1073aaa8; continue; }
            pc = 0x1073ac94; continue;
            }
            0x1073ac94 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(44u32))?;
                c.alu(AluOp::Sub, c.r[2], 1u32, 32);
                if !c.zf { pc = 0x1073acc0; continue; }
            pc = 0x1073ac9d; continue;
            }
            0x1073ac9d => {
                c.fpush(c.rdf64(338512016u32)?);
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
                c.fcom_value(c.rdf64(338512024u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if !c.pf { pc = 0x1073acbc; continue; }
            pc = 0x1073acb8; continue;
            }
            0x1073acb8 => {
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(44u32), v)?; }
            pc = 0x1073acbc; continue;
            }
            0x1073acbc => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(44u32))?;
            pc = 0x1073acc0; continue;
            }
            0x1073acc0 => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.r[6] = c.alu(AluOp::Xor, c.r[6], c.r[6], 32);
            pc = 0x1073acc4; continue;
            }
            0x1073acc4 => {
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if !c.zf { pc = 0x1073ad4b; continue; }
            pc = 0x1073accc; continue;
            }
            0x1073accc => {
                c.r[0] = c.r[6].wrapping_add(c.r[2].wrapping_mul(2));
                c.r[1] = c.r[0].wrapping_add(c.r[0].wrapping_mul(2));
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 3u32, 32);
                c.fpush(c.rdf64(c.r[1].wrapping_add(338512032u32))?);
                c.fcom_value(c.rdf64(338513048u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if !c.pf { pc = 0x1073ad4b; continue; }
            pc = 0x1073ace8; continue;
            }
            0x1073ace8 => {
                c.fpush(c.rdf64(c.r[1].wrapping_add(338512032u32))?);
                c.fcom_value(c.rdf64(338513072u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.zf { pc = 0x1073ad4b; continue; }
            pc = 0x1073acfb; continue;
            }
            0x1073acfb => {
                c.fpush(c.rdf64(c.r[1].wrapping_add(338512040u32))?);
                c.fcom_value(c.rdf64(338513056u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if !c.pf { pc = 0x1073ad4b; continue; }
            pc = 0x1073ad0e; continue;
            }
            0x1073ad0e => {
                c.fpush(c.rdf64(c.r[1].wrapping_add(338512040u32))?);
                c.fcom_value(c.rdf64(338513080u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.zf { pc = 0x1073ad4b; continue; }
            pc = 0x1073ad21; continue;
            }
            0x1073ad21 => {
                c.fpush(c.rdf64(c.r[1].wrapping_add(338512048u32))?);
                c.fcom_value(c.rdf64(338513064u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 5u32, 8);
                if !c.pf { pc = 0x1073ad4b; continue; }
            pc = 0x1073ad34; continue;
            }
            0x1073ad34 => {
                c.fpush(c.rdf64(c.r[1].wrapping_add(338512048u32))?);
                c.fcom_value(c.rdf64(338513088u32)?, true);
                c.set16(0, c.fsw as u32);
                c.alu(AluOp::And, ((c.r[0] >> 8) & 0xff), 65u32, 8);
                if c.zf { pc = 0x1073ad4b; continue; }
            pc = 0x1073ad47; continue;
            }
            0x1073ad47 => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                pc = 0x1073ad50; continue;
            }
            0x1073ad4b => {
                c.r[0] = 1u32;
            pc = 0x1073ad50; continue;
            }
            0x1073ad50 => {
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.alu(AluOp::Sub, c.r[6], 2u32, 32);
                if c.sf != c.of { pc = 0x1073acc4; continue; }
            pc = 0x1073ad5a; continue;
            }
            0x1073ad5a => {
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x1073af14; continue; }
            pc = 0x1073ad62; continue;
            }
            0x1073ad62 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf || c.sf != c.of { pc = 0x1073ad80; continue; }
            pc = 0x1073ad6a; continue;
            }
            0x1073ad6a => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[0] = c.r[2];
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 6u32, 32);
                c.r[6] = c.r[4].wrapping_add(c.r[0]).wrapping_add(80u32);
                c.r[7] = c.r[4].wrapping_add(272u32);
                c.rep_movs(4)?;
            pc = 0x1073ad80; continue;
            }
            0x1073ad80 => {
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.r[1] = c.r[2].wrapping_add(c.r[2].wrapping_mul(2));
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 4u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
                c.r[0] = c.r[1].wrapping_add(338512032u32);
                c.r[1] = c.r[4].wrapping_add(c.r[2].wrapping_mul(4)).wrapping_add(64u32);
                { let v = 8u32; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(56u32), v)?; }
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(40u32), v)?; }
                c.r[3] = c.r[3];
            pc = 0x1073adb0; continue;
            }
            0x1073adb0 => {
                c.r[2] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[1] = c.rd32(c.r[2].wrapping_add(416u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(40u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(56u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[1] = c.incdec(c.r[1], 32, false);
                { let v = c.r[1]; c.wr32(c.r[0], v)?; }
                c.push(338512128u32)?;
                c.push(c.r[2])?;
                c.r[1] = c.r[6];
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 6u32, 32);
                c.push(338513048u32)?;
                c.r[2] = c.r[4].wrapping_add(c.r[1]).wrapping_add(284u32);
                c.r[1] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.push(338512280u32)?;
                c.push(c.r[1])?;
                c.push(c.r[0])?;
                c.r[0] = 338512152u32;
                c.push(0x1073adf4)?; dispatch(c, 0x10737760)?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(68u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 24u32, 32);
                { let v = c.st0(); c.wrf64(c.r[2].wrapping_mul(8).wrapping_add(338512016u32), v)?; c.fpop(); }
                c.r[6] = c.incdec(c.r[6], 32, true);
                c.r[0] = c.r[6];
                c.r[0] = c.alu(AluOp::And, c.r[0], 2147483663u32, 32);
                if !c.sf { pc = 0x1073ae11; continue; }
            pc = 0x1073ae0c; continue;
            }
            0x1073ae0c => {
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.r[0] = c.alu(AluOp::Or, c.r[0], 4294967280u32, 32);
                c.r[0] = c.incdec(c.r[0], 32, true);
            pc = 0x1073ae11; continue;
            }
            0x1073ae11 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(52u32))?;
                c.r[1] = 0u32;
                { let v = if c.zf { 1u32 } else { 0u32 }; c.set8(1, 0, v); }
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(36u32), v)?; }
                { let v = 0u32; c.wr32(c.r[4].wrapping_add(60u32), v)?; }
                c.r[2] = c.alu(AluOp::Or, c.r[2], c.r[1], 32);
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 6u32, 32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(52u32), v)?; }
                c.r[7] = c.r[4].wrapping_add(c.r[0]).wrapping_add(272u32);
                c.r[4] = c.r[4];
            pc = 0x1073ae40; continue;
            }
            0x1073ae40 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(60u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.push(c.r[2])?;
                c.push(c.r[7])?;
                c.push(c.r[0])?;
                c.r[0] = 338512128u32;
                c.r[2] = 338512664u32;
                c.push(0x1073ae5a)?; dispatch(c, 0x10736c70)?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[4] = c.alu(AluOp::Add, c.r[4], 12u32, 32);
                c.r[6] = c.incdec(c.r[6], 32, false);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                if c.sf != c.of { pc = 0x1073ae9d; continue; }
            pc = 0x1073ae66; continue;
            }
            0x1073ae66 => {
                c.r[1] = c.r[6];
                c.r[1] = c.shift(ShiftOp::Shl, c.r[1], 6u32, 32);
                c.r[2] = c.r[4].wrapping_add(c.r[1]).wrapping_add(272u32);
                { let v = c.r[2]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
            pc = 0x1073ae76; continue;
            }
            0x1073ae76 => {
                c.r[3] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[0] = c.r[7];
                c.push(0x1073ae85)?; dispatch(c, 0x10736fa0)?;
                c.r[3] = c.r[0];
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if !c.zf { pc = 0x1073aee7; continue; }
            pc = 0x1073ae8b; continue;
            }
            0x1073ae8b => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(28u32))?;
                c.r[6] = c.incdec(c.r[6], 32, false);
                c.r[1] = c.alu(AluOp::Sub, c.r[1], 64u32, 32);
                c.alu(AluOp::And, c.r[6], c.r[6], 32);
                { let v = c.r[1]; c.wr32(c.r[4].wrapping_add(28u32), v)?; }
                if c.sf == c.of { pc = 0x1073ae76; continue; }
            pc = 0x1073ae9b; continue;
            }
            0x1073ae9b => {
                pc = 0x1073aea1; continue;
            }
            0x1073ae9d => {
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if !c.zf { pc = 0x1073aee7; continue; }
            pc = 0x1073aea1; continue;
            }
            0x1073aea1 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(52u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x1073aef9; continue; }
            pc = 0x1073aea9; continue;
            }
            0x1073aea9 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(36u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], 16u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.sf == c.of { pc = 0x1073aef9; continue; }
            pc = 0x1073aeb7; continue;
            }
            0x1073aeb7 => {
                c.r[1] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[0] = c.shift(ShiftOp::Shl, c.r[0], 6u32, 32);
                c.r[6] = c.r[4].wrapping_add(c.r[0]).wrapping_add(272u32);
            pc = 0x1073aec5; continue;
            }
            0x1073aec5 => {
                c.r[0] = c.r[7];
                c.r[3] = c.r[6];
                c.push(0x1073aece)?; dispatch(c, 0x10736fa0)?;
                c.r[3] = c.r[0];
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if !c.zf { pc = 0x1073aee7; continue; }
            pc = 0x1073aed4; continue;
            }
            0x1073aed4 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[6] = c.alu(AluOp::Add, c.r[6], 64u32, 32);
                c.alu(AluOp::Sub, c.r[0], 16u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(16u32), v)?; }
                if c.sf != c.of { pc = 0x1073aec5; continue; }
            pc = 0x1073aee5; continue;
            }
            0x1073aee5 => {
                pc = 0x1073aef9; continue;
            }
            0x1073aee7 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(60u32))?;
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.alu(AluOp::Sub, c.r[0], 2u32, 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(60u32), v)?; }
                if c.sf != c.of { pc = 0x1073ae40; continue; }
            pc = 0x1073aef9; continue;
            }
            0x1073aef9 => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(48u32))?;
                c.r[1] = c.r[0];
                c.r[0] = c.incdec(c.r[0], 32, false);
                c.alu(AluOp::And, c.r[1], c.r[1], 32);
                { let v = c.r[0]; c.wr32(c.r[4].wrapping_add(48u32), v)?; }
                if c.zf { pc = 0x1073af10; continue; }
            pc = 0x1073af08; continue;
            }
            0x1073af08 => {
                c.alu(AluOp::And, c.r[3], c.r[3], 32);
                if c.zf { pc = 0x1073adb0; continue; }
            pc = 0x1073af10; continue;
            }
            0x1073af10 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(44u32))?;
            pc = 0x1073af14; continue;
            }
            0x1073af14 => {
                c.r[6] = c.r[2].wrapping_add(c.r[2].wrapping_mul(2));
                c.r[6] = c.shift(ShiftOp::Shl, c.r[6], 4u32, 32);
                c.r[0] = 338512088u32;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[6], 32);
                c.r[1] = c.r[6].wrapping_add(338512040u32);
                c.r[7] = 2u32;
                c.r[4] = c.r[4];
            pc = 0x1073af30; continue;
            }
            0x1073af30 => {
                c.fpush(c.rdf64(338513480u32)?);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 24u32, 32);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967264u32))?; c.farith_value(FOp::Add, v); }
                c.r[0] = c.alu(AluOp::Add, c.r[0], 24u32, 32);
                c.r[7] = c.incdec(c.r[7], 32, false);
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967264u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(338513488u32)?);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967272u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967272u32), v)?; c.fpop(); }
                c.fpush(c.rdf64(338513496u32)?);
                { let v = c.rdf64(c.r[1].wrapping_add(4294967280u32))?; c.farith_value(FOp::Add, v); }
                { let v = c.st0(); c.wrf64(c.r[0].wrapping_add(4294967280u32), v)?; c.fpop(); }
                if !c.zf { pc = 0x1073af30; continue; }
            pc = 0x1073af5d; continue;
            }
            0x1073af5d => {
                c.r[0] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.r[2] = c.rd32(c.r[4].wrapping_add(c.r[2].wrapping_mul(4)).wrapping_add(64u32))?;
                { let v = c.rd8(c.r[0].wrapping_add(420u32))? as u32; c.set8(1, 0, v); }
                c.r[2] = c.incdec(c.r[2], 32, true);
                c.alu(AluOp::And, ((c.r[1] >> 0) & 0xff), ((c.r[1] >> 0) & 0xff), 8);
                c.r[3] = c.r[0].wrapping_add(424u32);
                { let v = c.r[2]; c.wr32(c.r[3], v)?; }
                if c.zf { pc = 0x1073af7d; continue; }
            pc = 0x1073af77; continue;
            }
            0x1073af77 => {
                { let v = 4u32; c.wr32(c.r[3], v)?; }
            pc = 0x1073af7d; continue;
            }
            0x1073af7d => {
                c.r[0] = c.rd32(c.r[4].wrapping_add(76u32))?;
                c.r[7] = c.rd32(c.r[5].wrapping_add(8u32))?;
                c.push(c.r[0])?;
                c.r[1] = c.r[4].wrapping_add(84u32);
                c.push(c.r[1])?;
                c.r[1] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[2] = c.r[7].wrapping_add(428u32);
                c.push(c.r[2])?;
                c.r[2] = c.rd32(c.r[7].wrapping_add(416u32))?;
                c.push(338513888u32)?;
                c.push(338512152u32)?;
                c.push(338512664u32)?;
                c.push(c.r[1])?;
                c.r[0] = 338512080u32;
                c.r[0] = c.alu(AluOp::Sub, c.r[0], c.r[6], 32);
                c.push(c.r[2])?;
                c.push(0x1073afb8)?; dispatch(c, 0x10739fd0)?;
                c.fst_st(0, true);
                { let v = c.rd8(c.r[7].wrapping_add(420u32))? as u32; c.set8(0, 0, v); }
                c.r[4] = c.alu(AluOp::Add, c.r[4], 32u32, 32);
                c.alu(AluOp::And, ((c.r[0] >> 0) & 0xff), ((c.r[0] >> 0) & 0xff), 8);
                if c.zf { pc = 0x1073afd2; continue; }
            pc = 0x1073afc7; continue;
            }
            0x1073afc7 => {
                c.alu(AluOp::Sub, c.rd32(c.r[3])?, 4u32, 32);
                if c.zf { pc = 0x1073afd2; continue; }
            pc = 0x1073afcc; continue;
            }
            0x1073afcc => {
                { let v = 4u32; c.wr32(c.r[3], v)?; }
            pc = 0x1073afd2; continue;
            }
            0x1073afd2 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(32u32))?;
                c.r[0] = c.alu(AluOp::Xor, c.r[0], c.r[0], 32);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                if c.zf || c.sf != c.of { pc = 0x1073aff7; continue; }
            pc = 0x1073afdc; continue;
            }
            0x1073afdc => {
                c.r[1] = c.r[7].wrapping_add(452u32);
            pc = 0x1073afe2; continue;
            }
            0x1073afe2 => {
                c.r[6] = c.rd32(c.r[4].wrapping_add(c.r[0].wrapping_mul(4)).wrapping_add(208u32))?;
                c.r[6] = c.rd32(c.r[4].wrapping_add(c.r[6].wrapping_mul(4)).wrapping_add(80u32))?;
                { let v = c.r[6]; c.wr32(c.r[1], v)?; }
                c.r[0] = c.incdec(c.r[0], 32, true);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 4u32, 32);
                c.alu(AluOp::Sub, c.r[0], c.r[2], 32);
                if c.sf != c.of { pc = 0x1073afe2; continue; }
            pc = 0x1073aff7; continue;
            }
            0x1073aff7 => {
                { let v = c.pop()?; c.r[7] = v; }
                { let v = c.pop()?; c.r[6] = v; }
                { let v = c.pop()?; c.r[3] = v; }
                c.r[4] = c.r[5];
                { let v = c.pop()?; c.r[5] = v; }
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn f_10740cb4(c: &mut Cpu) -> R<()> {
    let mut pc: u32 = 0x10740cb4;
    loop {
        match pc {
            0x10740cb4 => {
                c.push(c.r[5])?;
                c.r[5] = c.r[4];
                c.r[4] = c.alu(AluOp::Sub, c.r[4], 32u32, 32);
                c.r[4] = c.alu(AluOp::And, c.r[4], 4294967280u32, 32);
                c.fld_st(0);
                { let v = c.st0(); c.wrf32(c.r[4].wrapping_add(24u32), v as f32)?;  }
                { let v = c.fround(c.st0()) as i64; c.wr64(c.r[4].wrapping_add(16u32), v as u64)?; c.fpop(); }
                c.fpush(c.rd64(c.r[4].wrapping_add(16u32))? as i64 as f64);
                c.r[2] = c.rd32(c.r[4].wrapping_add(24u32))?;
                c.r[0] = c.rd32(c.r[4].wrapping_add(16u32))?;
                c.alu(AluOp::And, c.r[0], c.r[0], 32);
                if c.zf { pc = 0x10740d13; continue; }
            pc = 0x10740cd7; continue;
            }
            0x10740cd7 => {
                c.farith(FOp::Sub, 1, 0, true);
                c.alu(AluOp::And, c.r[2], c.r[2], 32);
                if !c.sf { pc = 0x10740cfb; continue; }
            pc = 0x10740cdd; continue;
            }
            0x10740cdd => {
                { let v = c.st0(); c.wrf32(c.r[4], v as f32)?; c.fpop(); }
                c.r[1] = c.rd32(c.r[4])?;
                c.r[1] = c.alu(AluOp::Xor, c.r[1], 2147483648u32, 32);
                c.r[1] = c.alu(AluOp::Add, c.r[1], 2147483647u32, 32);
                c.r[0] = c.alu(AluOp::Adc, c.r[0], 0u32, 32);
                c.r[2] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[2] = c.alu(AluOp::Adc, c.r[2], 0u32, 32);
                pc = 0x10740d27; continue;
            }
            0x10740cfb => {
                { let v = c.st0(); c.wrf32(c.r[4], v as f32)?; c.fpop(); }
                c.r[1] = c.rd32(c.r[4])?;
                c.r[1] = c.alu(AluOp::Add, c.r[1], 2147483647u32, 32);
                c.r[0] = c.alu(AluOp::Sbb, c.r[0], 0u32, 32);
                c.r[2] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.r[2] = c.alu(AluOp::Sbb, c.r[2], 0u32, 32);
                pc = 0x10740d27; continue;
            }
            0x10740d13 => {
                c.r[2] = c.rd32(c.r[4].wrapping_add(20u32))?;
                c.alu(AluOp::And, c.r[2], 2147483647u32, 32);
                if !c.zf { pc = 0x10740cd7; continue; }
            pc = 0x10740d1f; continue;
            }
            0x10740d1f => {
                { let v = c.st0(); c.wrf32(c.r[4].wrapping_add(24u32), v as f32)?; c.fpop(); }
                { let v = c.st0(); c.wrf32(c.r[4].wrapping_add(24u32), v as f32)?; c.fpop(); }
            pc = 0x10740d27; continue;
            }
            0x10740d27 => {
                c.r[4] = c.r[5]; c.r[5] = c.pop()?;
                c.r[4] = c.r[4].wrapping_add(4); return Ok(());
            }
            _ => return Err(Fault("pc", pc)),
        }
    }
}

pub fn dispatch(c: &mut Cpu, target: u32) -> R<()> {
    match target {
        0x1072c270 => f_1072c270(c),
        0x1072c4f0 => f_1072c4f0(c),
        0x1072dfa0 => f_1072dfa0(c),
        0x1072e050 => f_1072e050(c),
        0x1072e070 => f_1072e070(c),
        0x1072e140 => f_1072e140(c),
        0x1072fe80 => f_1072fe80(c),
        0x10731690 => f_10731690(c),
        0x10731710 => f_10731710(c),
        0x10735b80 => f_10735b80(c),
        0x10735c00 => f_10735c00(c),
        0x10735c30 => f_10735c30(c),
        0x10735c60 => f_10735c60(c),
        0x10736430 => f_10736430(c),
        0x10736a20 => f_10736a20(c),
        0x10736c70 => f_10736c70(c),
        0x10736fa0 => f_10736fa0(c),
        0x10737000 => f_10737000(c),
        0x10737760 => f_10737760(c),
        0x107381f0 => f_107381f0(c),
        0x107399d0 => f_107399d0(c),
        0x10739fd0 => f_10739fd0(c),
        0x1073a250 => f_1073a250(c),
        0x10740cb4 => f_10740cb4(c),
        0x10302860 => host_alloc(c, 0x10302860),
        0x10303283 => host_free(c, 0x10303283),
        0x10306203 => host_free(c, 0x10306203),
        0x10324810 => host_free(c, 0x10324810),
        0x10324840 => host_free(c, 0x10324840),
        0x1032f690 => host_alloc(c, 0x1032f690),
        0x10735b60 => host_fault(c, 0x10735b60),
        0x10740e54 => host_free(c, 0x10740e54),
        0x10740e60 => host_malloc(c, 0x10740e60),
        0x1074113a => host_calloc(c, 0x1074113a),
        0x10741158 => host_floor(c, 0x10741158),
        _ => Err(Fault("call", target)),
    }
}

pub const RODATA: &[(u32, &[u8])] = &[
    (0x107315dc, &[212, 255, 114, 16, 149, 255, 114, 16, 170, 255, 114, 16, 191, 255, 114, 16, 230, 255, 114, 16, 230, 255, 114, 16, 230, 255, 114, 16, 240, 255, 114, 16, 251, 255, 114, 16, 230, 255, 114, 16, 10, 0, 115, 16, 240, 255, 114, 16, 240, 255, 114, 16, 92, 1, 115, 16, 249, 0, 115, 16, 26, 1, 115, 16, 59, 1, 115, 16, 133, 1, 115, 16, 24, 2, 115, 16, 34, 3, 115, 16, 161, 3, 115, 16, 222, 3, 115, 16, 163, 2, 115, 16, 63, 4, 115, 16, 161, 3, 115, 16, 161, 3, 115, 16, 70, 7, 115, 16, 91, 7, 115, 16, 112, 7, 115, 16, 135, 7, 115, 16, 195, 8, 115, 16, 85, 8, 115, 16, 145, 8, 115, 16, 170, 8, 115, 16, 231, 8, 115, 16, 119, 9, 115, 16, 138, 10, 115, 16, 10, 11, 115, 16, 71, 11, 115, 16, 7, 10, 115, 16, 150, 11, 115, 16]),
    (0x10731610, &[92, 1, 115, 16, 249, 0, 115, 16, 26, 1, 115, 16, 59, 1, 115, 16, 133, 1, 115, 16, 24, 2, 115, 16, 34, 3, 115, 16, 161, 3, 115, 16, 222, 3, 115, 16, 163, 2, 115, 16, 63, 4, 115, 16, 161, 3, 115, 16, 161, 3, 115, 16, 70, 7, 115, 16, 91, 7, 115, 16, 112, 7, 115, 16, 135, 7, 115, 16, 195, 8, 115, 16, 85, 8, 115, 16, 145, 8, 115, 16, 170, 8, 115, 16, 231, 8, 115, 16, 119, 9, 115, 16, 138, 10, 115, 16, 10, 11, 115, 16, 71, 11, 115, 16, 7, 10, 115, 16, 150, 11, 115, 16]),
    (0x10731644, &[70, 7, 115, 16, 91, 7, 115, 16, 112, 7, 115, 16, 135, 7, 115, 16, 195, 8, 115, 16, 85, 8, 115, 16, 145, 8, 115, 16, 170, 8, 115, 16, 231, 8, 115, 16, 119, 9, 115, 16, 138, 10, 115, 16, 10, 11, 115, 16, 71, 11, 115, 16, 7, 10, 115, 16, 150, 11, 115, 16]),
    (0x10731654, &[195, 8, 115, 16, 85, 8, 115, 16, 145, 8, 115, 16, 170, 8, 115, 16, 231, 8, 115, 16, 119, 9, 115, 16, 138, 10, 115, 16, 10, 11, 115, 16, 71, 11, 115, 16, 7, 10, 115, 16, 150, 11, 115, 16]),
    (0x10735f7c, &[174, 93, 115, 16, 45, 93, 115, 16, 88, 93, 115, 16, 214, 93, 115, 16, 131, 93, 115, 16]),
    (0x10739948, &[192, 149, 115, 16, 133, 149, 115, 16, 74, 149, 115, 16, 15, 149, 115, 16, 212, 148, 115, 16, 135, 148, 115, 16, 49, 148, 115, 16, 219, 147, 115, 16, 133, 147, 115, 16, 47, 147, 115, 16, 217, 146, 115, 16, 131, 146, 115, 16, 45, 146, 115, 16, 215, 145, 115, 16, 129, 145, 115, 16, 43, 145, 115, 16, 178, 144, 115, 16, 98, 144, 115, 16, 18, 144, 115, 16, 194, 143, 115, 16, 78, 143, 115, 16, 218, 142, 115, 16, 102, 142, 115, 16, 242, 141, 115, 16, 126, 141, 115, 16, 10, 141, 115, 16, 150, 140, 115, 16, 34, 140, 115, 16, 174, 139, 115, 16, 58, 139, 115, 16, 198, 138, 115, 16, 82, 138, 115, 16]),
    (0x10739988, &[178, 144, 115, 16, 98, 144, 115, 16, 18, 144, 115, 16, 194, 143, 115, 16, 78, 143, 115, 16, 218, 142, 115, 16, 102, 142, 115, 16, 242, 141, 115, 16, 126, 141, 115, 16, 10, 141, 115, 16, 150, 140, 115, 16, 34, 140, 115, 16, 174, 139, 115, 16, 58, 139, 115, 16, 198, 138, 115, 16, 82, 138, 115, 16]),
    (0x10791558, &[0, 0, 128, 63]),
    (0x107938a8, &[0, 0, 0, 0, 0, 0, 240, 63]),
    (0x107938b8, &[0, 0, 0, 0, 0, 0, 0, 0]),
    (0x10798638, &[0, 0, 0, 63]),
    (0x10798ae4, &[0, 0, 112, 65]),
    (0x10799fac, &[10, 215, 35, 60]),
    (0x107c4f18, &[0, 0, 0, 0, 0, 0, 240, 65]),
    (0x107da478, &[0, 0, 128, 79]),
    (0x107db948, &[0, 0, 0, 0, 0, 0, 224, 63]),
    (0x107dc348, &[0, 0, 255, 66]),
    (0x107e2470, &[0, 0, 0, 0, 0, 0, 20, 64]),
    (0x107e53e0, &[16, 16, 16, 16, 16, 16, 112, 63]),
    (0x107fd1c8, &[0, 0, 0, 0, 0, 0, 8, 64]),
    (0x10809f60, &[0, 0, 0, 0, 0, 0, 12, 64]),
    (0x108203c8, &[17, 234, 45, 129, 153, 151, 113, 61]),
    (0x108203d8, &[16, 16, 16, 16, 16, 16, 128, 63]),
    (0x10820484, &[129, 128, 0, 60]),
    (0x10820550, &[0, 0, 0, 0, 0, 0, 64, 64]),
    (0x10820558, &[0, 0, 0, 0, 0, 224, 111, 64]),
    (0x109cfa70, &[128, 0, 0, 0]),
    (0x109cfe50, &[5, 0, 0, 0, 6, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 240, 63, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 240, 191, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 191, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 192, 0, 0, 0, 0, 0, 0, 8, 64, 0, 0, 0, 0, 0, 0, 8, 64, 0, 0, 0, 0, 0, 0, 0, 192, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 240, 191, 0, 0, 0, 0, 0, 0, 240, 191, 0, 0, 0, 0, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 248, 63, 0, 0, 0, 0, 0, 0, 224, 191, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 224, 191, 0, 0, 0, 0, 0, 0, 248, 63, 0, 0, 0, 0, 0, 0, 248, 63, 0, 0, 0, 0, 0, 0, 224, 191, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 240, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 224, 191, 0, 0, 0, 0, 0, 0, 248, 63, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 240, 5, 130, 16, 0, 0, 0, 0, 46, 63, 65, 86, 116, 121, 112, 101, 95, 105, 110, 102, 111, 64, 64, 0, 240, 5, 130, 16, 0, 0, 0, 0, 46, 63, 65, 86, 111, 117, 116, 95, 111, 102, 95, 114, 97, 110, 103, 101, 64, 115, 116, 100, 64, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    (0x109cfe54, &[6, 0, 0, 0]),
    (0x109cfe60, &[0, 0, 0, 0]),
    (0x109cfe60, &[0, 0, 0, 0, 0, 0, 240, 63]),
    (0x109cfe64, &[0, 0, 240, 63]),
    (0x109cfe68, &[0, 0, 0, 0]),
    (0x109cfe68, &[0, 0, 0, 0, 0, 0, 240, 63]),
    (0x109cfe6c, &[0, 0, 240, 63]),
    (0x109d01a0, &[2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 240, 5, 130, 16, 0, 0, 0, 0, 46, 63, 65, 86, 116, 121, 112, 101, 95, 105, 110, 102, 111, 64, 64, 0, 240, 5, 130, 16, 0, 0, 0, 0, 46, 63, 65, 86, 111, 117, 116, 95, 111, 102, 95, 114, 97, 110, 103, 101, 64, 115, 116, 100, 64, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
];
pub const GLOBALS: (u32, u32) = (0x142d0000, 0x142d6000);
