#![macro_use]
use super::super::Core;
use cpu::{CFLAG_SET, ZFLAG_SET, XFLAG_SET, NFLAG_SET, ZFLAG_CLEAR, VFLAG_CLEAR, CFLAG_CLEAR, XFLAG_CLEAR, NFLAG_CLEAR};
use std::num::Wrapping;

macro_rules! ir_dx {
    ($e:ident) => (($e.ir >> 9 & 7) as usize);
}
macro_rules! ir_dy {
    ($e:ident) => (($e.ir & 7) as usize);
}
macro_rules! ir_ax {
    ($e:ident) => (8+($e.ir >> 9 & 7) as usize);
}
macro_rules! ir_ay {
    ($e:ident) => (8+($e.ir & 7) as usize);
}
macro_rules! dx {
    ($e:ident) => ($e.dar[ir_dx!($e)]);
}
macro_rules! dy {
    ($e:ident) => ($e.dar[ir_dy!($e)]);
}
macro_rules! ax {
    ($e:ident) => ($e.dar[ir_ax!($e)]);
}
macro_rules! ay {
    ($e:ident) => ($e.dar[ir_ay!($e)]);
}
macro_rules! sp {
    ($e:ident) => ($e.dar[15]);
}
macro_rules! mask_out_above_8 {
    ($e:expr) => ($e & 0xff)
}
macro_rules! mask_out_below_8 {
    ($e:expr) => ($e & !0xff)
}
macro_rules! mask_out_above_16 {
    ($e:expr) => ($e & 0xffff)
}
macro_rules! mask_out_below_16 {
    ($e:expr) => ($e & !0xffff)
}
macro_rules! mask_out_above_32 {
    ($e:expr) => ($e & 0xffffffff)
}
macro_rules! low_nibble {
    ($e:expr) => ($e & 0x0f);
}
macro_rules! high_nibble {
    ($e:expr) => ($e & 0xf0);
}
macro_rules! true_is_1 {
    ($e:expr) => (if $e {1} else {0})
}
macro_rules! false_is_1 {
    ($e:expr) => (if $e {0} else {1})
}
macro_rules! not1 {
    ($e:expr) => (true_is_1!($e == 0))
}
macro_rules! msb_8_set {
    ($e:expr) => (($e & 0x80) > 0)
}
macro_rules! msb_16_set {
    ($e:expr) => (($e & 0x8000) > 0)
}
macro_rules! msb_32_set {
    ($e:expr) => (($e & 0x80000000) > 0)
}
// All instructions are ported from https://github.com/kstenerud/Musashi
pub fn abcd_8(core: &mut Core, dst: u32, src: u32) -> u32 {
    // unsigned int res = ((src) & 0x0f) + ((dst) & 0x0f) + ((m68ki_cpu.x_flag>>8)&1);
    let mut res = low_nibble!(src) + low_nibble!(dst) + core.x_flag_as_1();

    // m68ki_cpu.v_flag = ~res;
    core.v_flag = !res;

    // if(res > 9)
    //  res += 6;
    if res > 9 {
        res += 6;
    }
    // res += ((src) & 0xf0) + ((dst) & 0xf0);
    res += high_nibble!(src) + high_nibble!(dst);
    // m68ki_cpu.x_flag = m68ki_cpu.c_flag = (res > 0x99) << 8;
    core.c_flag = true_is_1!(res > 0x99) << 8;
    core.x_flag = core.c_flag;

    if core.c_flag > 0 {
        res = (Wrapping(res) - Wrapping(0xa0)).0;
    }

    // m68ki_cpu.v_flag &= res;
    // m68ki_cpu.n_flag = (res);
    core.v_flag &= res;
    core.n_flag = res;

    // res = ((res) & 0xff);
    // m68ki_cpu.not_z_flag |= res;
    res = mask_out_above_8!(res);
    core.not_z_flag |= res;
    res
}

pub fn add_8(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_8!(dst);
    let src = mask_out_above_8!(src);

    let res = dst + src;
    // m68ki_cpu.n_flag = (res);
    core.n_flag = res;
    // m68ki_cpu.v_flag = ((src^res) & (dst^res));
    core.v_flag = (src ^ res) & (dst ^ res);
    // m68ki_cpu.x_flag = m68ki_cpu.c_flag = (res);
    core.c_flag = res;
    core.x_flag = res;
    // m68ki_cpu.not_z_flag = ((res) & 0xff);
    let res8 = mask_out_above_8!(res);
    core.not_z_flag = res8;
    res8
}
pub fn add_16(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_16!(dst);
    let src = mask_out_above_16!(src);
    let res = dst + src;

    // m68ki_cpu.n_flag = ((res)>>8);
    let res_hi = res >> 8;
    core.n_flag = res_hi;
    // m68ki_cpu.v_flag = (((src^res) & (dst^res))>>8);
    core.v_flag = ((src ^ res) & (dst ^ res)) >> 8;
    // m68ki_cpu.x_flag = m68ki_cpu.c_flag = ((res)>>8);
    core.c_flag = res_hi;
    core.x_flag = res_hi;
    // m68ki_cpu.not_z_flag = ((res) & 0xffff);
    let res16 = mask_out_above_16!(res);
    core.not_z_flag = res16;

    res16
}
pub fn add_32(core: &mut Core, dst: u32, src: u32) -> u32 {
    let res: u64 = (dst as u64) + (src as u64);

    let res_hi = (res >> 24) as u32;
    core.n_flag = res_hi;
    // m68ki_cpu.v_flag = (((src^res) & (dst^res))>>24);
    core.v_flag = (((src as u64 ^ res) & (dst as u64 ^ res)) >> 24) as u32;
     // m68ki_cpu.x_flag = m68ki_cpu.c_flag = (((src & dst) | (~res & (src | dst)))>>23);
    core.c_flag = res_hi;
    core.x_flag = res_hi;

    let res32 = res as u32;

    core.not_z_flag = res32;

    res32
}

pub fn addx_8(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_8!(dst);
    let src = mask_out_above_8!(src);

    let res = dst + src + core.x_flag_as_1();

    core.n_flag = res;
    core.v_flag = (src ^ res) & (dst ^ res);
    core.c_flag = res;
    core.x_flag = res;

    let res8 = mask_out_above_8!(res);
    core.not_z_flag |= res8;
    res8
}
pub fn addx_16(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_16!(dst);
    let src = mask_out_above_16!(src);
    let res = dst + src + core.x_flag_as_1();

    let res_hi = res >> 8;
    core.n_flag = res_hi;
    core.v_flag = ((src ^ res) & (dst ^ res)) >> 8;
    core.c_flag = res_hi;
    core.x_flag = res_hi;

    let res16 = mask_out_above_16!(res);
    core.not_z_flag |= res16;
    res16
}
pub fn addx_32(core: &mut Core, dst: u32, src: u32) -> u32 {
    let res: u64 = (dst as u64) + (src as u64) + core.x_flag_as_1() as u64;

    let res_hi = (res >> 24) as u32;
    core.n_flag = res_hi;
    core.v_flag = (((src as u64 ^ res) & (dst as u64 ^ res)) >> 24) as u32;
    core.c_flag = res_hi;
    core.x_flag = res_hi;

    let res32 = res as u32;
    core.not_z_flag |= res32;
    res32
}

pub fn and_8(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_8!(dst);
    let src = mask_out_above_8!(src);
    let res = dst & src;

    core.not_z_flag = res;
    core.n_flag = res;
    core.c_flag = 0;
    core.v_flag = 0;

    res
}
pub fn and_16(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_16!(dst);
    let src = mask_out_above_16!(src);
    let res = dst & src;

    let res_hi = res >> 8;
    core.not_z_flag = res;
    core.n_flag = res_hi;
    core.c_flag = 0;
    core.v_flag = 0;

    res
}
pub fn and_32(core: &mut Core, dst: u32, src: u32) -> u32 {
    let res = dst & src;

    let res_hi = res >> 24;
    core.not_z_flag = res;
    core.n_flag = res_hi;
    core.c_flag = 0;
    core.v_flag = 0;

    res
}

pub fn asr_8(core: &mut Core, dst: u32, shift: u32) -> u32 {
    let src = mask_out_above_8!(dst);
    let res = src.wrapping_shr(shift);

    if shift != 0 {
        if shift < 8 {
            let res = if msb_8_set!(src) {
                res | SHIFT_8_TABLE[shift as usize]
            } else {
                res
            };
            core.n_flag = res;
            core.not_z_flag = res;
            core.v_flag = VFLAG_CLEAR;
            core.c_flag = src.wrapping_shl(9-shift);
            core.x_flag = core.c_flag;
            res
        } else {
            if msb_8_set!(src) {
                core.c_flag = CFLAG_SET;
                core.x_flag = XFLAG_SET;
                core.n_flag = NFLAG_SET;
                core.not_z_flag = ZFLAG_CLEAR;
                core.v_flag = VFLAG_CLEAR;
                0xff
            } else {
                core.c_flag = CFLAG_CLEAR;
                core.x_flag = XFLAG_CLEAR;
                core.n_flag = NFLAG_CLEAR;
                core.not_z_flag = ZFLAG_SET;
                core.v_flag = VFLAG_CLEAR;
                0x00
            }
        }
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        res
    }
}

pub fn asr_16(core: &mut Core, dst: u32, shift: u32) -> u32 {
    let src = mask_out_above_16!(dst);
    let res = src.wrapping_shr(shift);
    if shift != 0 {
        if shift < 16 {
            let res = if msb_16_set!(src) {
                res | SHIFT_16_TABLE[shift as usize]
            } else {
                res
            };
            core.n_flag = res >> 8;
            core.not_z_flag = res;
            core.v_flag = VFLAG_CLEAR;
            core.c_flag = src.wrapping_shr(shift - 1) << 8;
            core.x_flag = core.c_flag;
            res
        } else {
            if msb_16_set!(src) {
                core.c_flag = CFLAG_SET;
                core.x_flag = XFLAG_SET;
                core.n_flag = NFLAG_SET;
                core.not_z_flag = ZFLAG_CLEAR;
                core.v_flag = VFLAG_CLEAR;
                0xffff
            } else {
                core.c_flag = CFLAG_CLEAR;
                core.x_flag = XFLAG_CLEAR;
                core.n_flag = NFLAG_CLEAR;
                core.not_z_flag = ZFLAG_SET;
                core.v_flag = VFLAG_CLEAR;
                0x0000
            }
        }
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src >> 8;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        res
    }
}

pub fn asr_32(core: &mut Core, dst: u32, shift: u32) -> u32 {
    let src = dst;
    let res = src.wrapping_shr(shift);
    if shift != 0 {
        if shift < 32 {
            let res = if msb_32_set!(src) {
                res | SHIFT_32_TABLE[shift as usize]
            } else {
                res
            };
            core.n_flag = res >> 24;
            core.not_z_flag = res;
            core.v_flag = VFLAG_CLEAR;
            core.c_flag = src.wrapping_shr(shift - 1) << 8;
            core.x_flag = core.c_flag;
            res
        } else {
            if msb_32_set!(src) {
                core.c_flag = CFLAG_SET;
                core.x_flag = XFLAG_SET;
                core.n_flag = NFLAG_SET;
                core.not_z_flag = ZFLAG_CLEAR;
                core.v_flag = VFLAG_CLEAR;
                0xffffffff
            } else {
                core.c_flag = CFLAG_CLEAR;
                core.x_flag = XFLAG_CLEAR;
                core.n_flag = NFLAG_CLEAR;
                core.not_z_flag = ZFLAG_SET;
                core.v_flag = VFLAG_CLEAR;
                0x00000000
            }
        }
    } else {
        core.n_flag = src >> 24;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        core.c_flag = CFLAG_CLEAR;
        res
    }
}

pub fn asl_8(core: &mut Core, dst: u32, shift: u32) -> u32 {
    let src = mask_out_above_8!(dst);
    let res = mask_out_above_8!(src.wrapping_shl(shift));

    if shift != 0 {
        if shift < 8 {
            core.n_flag = res;
            core.not_z_flag = res;
            core.c_flag = src.wrapping_shl(shift);
            core.x_flag = core.c_flag;
            let src = src & SHIFT_8_TABLE[shift as usize + 1];
            core.v_flag = false_is_1!(src == 0 || src == SHIFT_8_TABLE[shift as usize + 1]) << 7;
            res
        } else {
            core.c_flag = (if shift == 8 {src & 1} else {0}) << 8;
            core.x_flag = core.c_flag;
            core.n_flag = NFLAG_CLEAR;
            core.not_z_flag = ZFLAG_SET;
            core.v_flag = false_is_1!(src == 0) << 7;
            0x00
        }
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        res
    }
}

pub fn asl_16(core: &mut Core, dst: u32, shift: u32) -> u32 {
    let src = mask_out_above_16!(dst);
    let res = mask_out_above_16!(src.wrapping_shl(shift));
    if shift != 0 {
        if shift < 16 {
            core.n_flag = res >> 8;
            core.not_z_flag = res;
            core.c_flag = src.wrapping_shl(shift) >> 8;
            core.x_flag = core.c_flag;
            let src = src & SHIFT_16_TABLE[shift as usize + 1];
            core.v_flag = false_is_1!(src == 0 || src == SHIFT_16_TABLE[shift as usize + 1]) << 7;
            res
        } else {
            core.c_flag = (if shift == 16 {src & 1} else {0}) << 8;
            core.x_flag = core.c_flag;
            core.n_flag = NFLAG_CLEAR;
            core.not_z_flag = ZFLAG_SET;
            core.v_flag = false_is_1!(src == 0) << 7;
            0x0000
        }
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src >> 8;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        res
    }
}

pub fn asl_32(core: &mut Core, dst: u32, shift: u32) -> u32 {
    let src = dst;
    let res = src.wrapping_shl(shift);
    if shift != 0 {
        if shift < 32 {
            core.n_flag = res >> 24;
            core.not_z_flag = res;
            core.c_flag = src.wrapping_shr(32 - shift) << 8;
            core.x_flag = core.c_flag;
            let src = src & SHIFT_32_TABLE[shift as usize + 1];
            core.v_flag = false_is_1!(src == 0 || src == SHIFT_32_TABLE[shift as usize + 1]) << 7;
            res
        } else {
            core.c_flag = (if shift == 32 {src & 1} else {0}) << 8;
            core.x_flag = core.c_flag;
            core.n_flag = NFLAG_CLEAR;
            core.not_z_flag = ZFLAG_SET;
            core.v_flag = false_is_1!(src == 0) << 7;
            0x00000000
        }
    } else {
        core.n_flag = src >> 24;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        core.c_flag = CFLAG_CLEAR;
        res
    }
}

static SHIFT_8_TABLE:  [u32; 65] = [
 0x00, 0x80, 0xc0, 0xe0, 0xf0, 0xf8, 0xfc, 0xfe, 0xff, 0xff, 0xff, 0xff,
 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
 0xff, 0xff, 0xff, 0xff, 0xff
];

static SHIFT_16_TABLE: [u32; 65] = [
 0x0000, 0x8000, 0xc000, 0xe000, 0xf000, 0xf800, 0xfc00, 0xfe00, 0xff00,
 0xff80, 0xffc0, 0xffe0, 0xfff0, 0xfff8, 0xfffc, 0xfffe, 0xffff, 0xffff,
 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff, 0xffff,
 0xffff, 0xffff
];

static SHIFT_32_TABLE: [u32; 65] = [
 0x00000000, 0x80000000, 0xc0000000, 0xe0000000, 0xf0000000, 0xf8000000,
 0xfc000000, 0xfe000000, 0xff000000, 0xff800000, 0xffc00000, 0xffe00000,
 0xfff00000, 0xfff80000, 0xfffc0000, 0xfffe0000, 0xffff0000, 0xffff8000,
 0xffffc000, 0xffffe000, 0xfffff000, 0xfffff800, 0xfffffc00, 0xfffffe00,
 0xffffff00, 0xffffff80, 0xffffffc0, 0xffffffe0, 0xfffffff0, 0xfffffff8,
 0xfffffffc, 0xfffffffe, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff,
 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff,
 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff,
 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff,
 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff,
 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff, 0xffffffff
];

pub fn cmp_8(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_8!(dst);
    let src = mask_out_above_8!(src);

    let res = (Wrapping(dst) - Wrapping(src)).0;

    core.n_flag = res;
    core.v_flag = (src ^ dst) & (res ^ dst);
    core.c_flag = res;

    let res8 = mask_out_above_8!(res);
    core.not_z_flag = res8;
    res8
}
pub fn cmp_16(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_16!(dst);
    let src = mask_out_above_16!(src);
    let res = (Wrapping(dst) - Wrapping(src)).0;

    let res_hi = res >> 8;
    core.n_flag = res_hi;
    core.v_flag = ((src ^ dst) & (res ^ dst)) >> 8;
    core.c_flag = res_hi;

    let res16 = mask_out_above_16!(res);
    core.not_z_flag = res16;
    res16
}
pub fn cmp_32(core: &mut Core, dst: u32, src: u32) -> u32 {
    let res = (Wrapping(dst as u64) - Wrapping(src as u64)).0;

    let res_hi = (res >> 24) as u32;
    core.n_flag = res_hi;
    core.v_flag = (((src as u64 ^ dst as u64) & (res ^ dst as u64)) >> 24) as u32;
    core.c_flag = res_hi;

    let res32 = res as u32;
    core.not_z_flag = res32;
    res32
}

// Put common implementation of DBcc here
// Put common implementation of DIVS here
pub fn divs_16(core: &mut Core, dst: u32, src: i16) {
    if dst == 0x80000000 && src == -1 {
        core.n_flag = 0;
        core.v_flag = 0;
        core.c_flag = 0;
        core.not_z_flag = 0;
        dx!(core) = 0;
        return;
    }
    let quotient: i32 = (dst as i32) / (src as i32);
    let remainder: i32 = (dst as i32) % (src as i32);
    if quotient == quotient as i16 as i32 {
        core.not_z_flag = quotient as u32;
        core.n_flag = quotient as u32 >> 8;
        core.v_flag = 0;
        core.c_flag = 0;
        dx!(core) = ((remainder as u32) << 16) | mask_out_above_16!(quotient as u32);
    } else {
        core.v_flag = 0x80;
    }
}

// Put common implementation of DIVU here
pub fn divu_16(core: &mut Core, dst: u32, src: u16) {
    let quotient: u32 = dst / (src as u32);
    let remainder: u32 = dst % (src as u32);
    if quotient < 0x10000 {
        core.not_z_flag = quotient;
        core.n_flag = quotient >> 8;
        core.v_flag = 0;
        core.c_flag = 0;
        dx!(core) = (remainder << 16) | mask_out_above_16!(quotient);
    } else {
        core.v_flag = 0x80;
    }
}

// Put common implementation of EOR here
pub fn eor_8(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_8!(dst);
    let src = mask_out_above_8!(src);
    let res = dst ^ src;

    core.not_z_flag = res;
    core.n_flag = res;
    core.c_flag = 0;
    core.v_flag = 0;

    res
}
pub fn eor_16(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_16!(dst);
    let src = mask_out_above_16!(src);
    let res = dst ^ src;

    let res_hi = res >> 8;
    core.not_z_flag = res;
    core.n_flag = res_hi;
    core.c_flag = 0;
    core.v_flag = 0;

    res
}
pub fn eor_32(core: &mut Core, dst: u32, src: u32) -> u32 {
    let res = dst ^ src;

    let res_hi = res >> 24;
    core.not_z_flag = res;
    core.n_flag = res_hi;
    core.c_flag = 0;
    core.v_flag = 0;

    res
}

// No common implementation of EXG needed
// No common implementation of EXT needed
// No common implementation of ILLEGAL needed
// No common implementation of JMP needed
// No common implementation of JSR needed
// No common implementation of LEA needed
// No common implementation of LINK needed

// Put common implementation of LSL, LSR here
pub fn lsr_8(core: &mut Core, dst: u32, shift: u32) -> u32 {
    let src = mask_out_above_8!(dst);
    let res = src.wrapping_shr(shift);

    if shift != 0 {
        if shift <= 8 {
            core.n_flag = NFLAG_CLEAR;
            core.not_z_flag = res;
            core.v_flag = VFLAG_CLEAR;
            core.c_flag = src.wrapping_shl(9-shift);
            core.x_flag = core.c_flag;
            res
        } else {
            core.c_flag = CFLAG_CLEAR;
            core.x_flag = XFLAG_CLEAR;
            core.n_flag = NFLAG_CLEAR;
            core.not_z_flag = ZFLAG_SET;
            core.v_flag = VFLAG_CLEAR;
            0x00
        }
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        res
    }
}

pub fn lsr_16(core: &mut Core, dst: u32, shift: u32) -> u32 {
    let src = mask_out_above_16!(dst);
    let res = src.wrapping_shr(shift);
    if shift != 0 {
        if shift <= 16 {
            core.n_flag = NFLAG_CLEAR;
            core.not_z_flag = res;
            core.v_flag = VFLAG_CLEAR;
            core.c_flag = src.wrapping_shr(shift - 1) << 8;
            core.x_flag = core.c_flag;
            res
        } else {
            core.c_flag = CFLAG_CLEAR;
            core.x_flag = XFLAG_CLEAR;
            core.n_flag = NFLAG_CLEAR;
            core.not_z_flag = ZFLAG_SET;
            core.v_flag = VFLAG_CLEAR;
            0x0000
        }
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src >> 8;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        res
    }
}

pub fn lsr_32(core: &mut Core, dst: u32, shift: u32) -> u32 {
    let src = dst;
    let res = src.wrapping_shr(shift);
    if shift != 0 {
        if shift < 32 {
            core.n_flag = NFLAG_CLEAR;
            core.not_z_flag = res;
            core.v_flag = VFLAG_CLEAR;
            core.c_flag = src.wrapping_shr(shift - 1) << 8;
            core.x_flag = core.c_flag;
            res
        } else {
            core.c_flag = if shift == 32 {((src) & 0x80000000)>>23} else {0};
            core.x_flag = core.c_flag;
            core.n_flag = NFLAG_CLEAR;
            core.not_z_flag = ZFLAG_SET;
            core.v_flag = VFLAG_CLEAR;
            0x00000000
        }
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src >> 24;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        res
    }
}

pub fn lsl_8(core: &mut Core, dst: u32, shift: u32) -> u32 {
    let src = mask_out_above_8!(dst);
    let res = mask_out_above_8!(src.wrapping_shl(shift));

    if shift != 0 {
        if shift <= 8 {
            core.n_flag = res;
            core.not_z_flag = res;
            core.c_flag = src.wrapping_shl(shift);
            core.x_flag = core.c_flag;
            core.v_flag = VFLAG_CLEAR;
            res
        } else {
            core.c_flag = CFLAG_CLEAR;
            core.x_flag = XFLAG_CLEAR;
            core.n_flag = NFLAG_CLEAR;
            core.not_z_flag = ZFLAG_SET;
            core.v_flag = VFLAG_CLEAR;
            0x00
        }
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        res
    }
}

pub fn lsl_16(core: &mut Core, dst: u32, shift: u32) -> u32 {
    let src = mask_out_above_16!(dst);
    let res = mask_out_above_16!(src.wrapping_shl(shift));
    if shift != 0 {
        if shift <= 16 {
            core.n_flag = res >> 8;
            core.not_z_flag = res;
            core.c_flag = src.wrapping_shl(shift) >> 8;
            core.x_flag = core.c_flag;
            core.v_flag = VFLAG_CLEAR;
            res
        } else {
            core.c_flag = CFLAG_CLEAR;
            core.x_flag = XFLAG_CLEAR;
            core.n_flag = NFLAG_CLEAR;
            core.not_z_flag = ZFLAG_SET;
            core.v_flag = VFLAG_CLEAR;
            0x0000
        }
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src >> 8;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        res
    }
}

pub fn lsl_32(core: &mut Core, dst: u32, shift: u32) -> u32 {
    let src = dst;
    let res = src.wrapping_shl(shift);
    if shift != 0 {
        if shift < 32 {
            core.n_flag = res >> 24;
            core.not_z_flag = res;
            core.c_flag = src.wrapping_shr(32 - shift) << 8;
            core.x_flag = core.c_flag;
            core.v_flag = VFLAG_CLEAR;
            res
        } else {
            core.c_flag = (if shift == 32 {src & 1} else {0}) << 8;
            core.x_flag = core.c_flag;
            core.n_flag = NFLAG_CLEAR;
            core.not_z_flag = ZFLAG_SET;
            core.v_flag = VFLAG_CLEAR;
            0x00000000
        }
    } else {
        core.n_flag = src >> 24;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        core.c_flag = CFLAG_CLEAR;
        res
    }
}

// Put common implementation of MOVE here
pub fn move_flags(core: &mut Core, src: u32, shift: u32) -> u32 {
    core.n_flag = src >> shift;
    core.not_z_flag = src;
    core.v_flag = 0;
    core.c_flag = 0;
    src
}

// Put common implementation of MOVEA here
// Put common implementation of MOVE to CCR here
// Put common implementation of MOVE from SR here
// Put common implementation of MOVE to SR here
// Put common implementation of MOVE USP here
// Put common implementation of MOVEM here
// Put common implementation of MOVEP here
// Put common implementation of MOVEQ here
// Put common implementation of MULS here
pub fn muls_16(core: &mut Core, dst: i16, src: i16) -> u32 {
    let res = (dst as i32).wrapping_mul(src as i32) as u32;
    core.not_z_flag = res;
    core.n_flag = res >> 24;
    core.v_flag = 0;
    core.c_flag = 0;
    res
}
// Put common implementation of MULU here
pub fn mulu_16(core: &mut Core, dst: u16, src: u16) -> u32 {
    let res = (dst as u32).wrapping_mul(src as u32) as u32;
    core.not_z_flag = res;
    core.n_flag = res >> 24;
    core.v_flag = 0;
    core.c_flag = 0;
    res
}
// Put common implementation of NBCD here
pub fn nbcd(core: &mut Core, dst: u32) -> Option<u32> {
    let mut res = mask_out_above_8!((0x9a as u32).wrapping_sub(dst).wrapping_sub(core.x_flag_as_1()));
    let answer = if res != 0x9a {
        core.v_flag = !res;
        if (res & 0x0f) == 0xa {
            res = (res & 0xf0) + 0x10;
        }

        res &= 0xff;
        core.v_flag &= res;

        core.not_z_flag |= res;
        core.c_flag = CFLAG_SET;
        core.x_flag = XFLAG_SET;
        Some(res)
    }
    else
    {
        core.v_flag = 0;
        core.c_flag = 0;
        core.x_flag = 0;
        None
    };
    core.n_flag = res;
    answer
}
// Put common implementation of NEG here
// Put common implementation of NEGX here
// Put common implementation of NOP here
// Put common implementation of NOT here
pub fn not_8(core: &mut Core, dst: u32) -> u32 {
    let res = mask_out_above_8!(!dst);

    core.not_z_flag = res;
    core.n_flag = res;
    core.c_flag = 0;
    core.v_flag = 0;

    res
}
pub fn not_16(core: &mut Core, dst: u32) -> u32 {
    let res = mask_out_above_16!(!dst);

    let res_hi = res >> 8;
    core.not_z_flag = res;
    core.n_flag = res_hi;
    core.c_flag = 0;
    core.v_flag = 0;

    res
}
pub fn not_32(core: &mut Core, dst: u32) -> u32 {
    let res = !dst;

    let res_hi = res >> 24;
    core.not_z_flag = res;
    core.n_flag = res_hi;
    core.c_flag = 0;
    core.v_flag = 0;

    res
}

// Put common implementation of OR here
pub fn or_8(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_8!(dst);
    let src = mask_out_above_8!(src);
    let res = dst | src;

    core.not_z_flag = res;
    core.n_flag = res;
    core.c_flag = 0;
    core.v_flag = 0;

    res
}
pub fn or_16(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_16!(dst);
    let src = mask_out_above_16!(src);
    let res = dst | src;

    let res_hi = res >> 8;
    core.not_z_flag = res;
    core.n_flag = res_hi;
    core.c_flag = 0;
    core.v_flag = 0;

    res
}
pub fn or_32(core: &mut Core, dst: u32, src: u32) -> u32 {
    let res = dst | src;

    let res_hi = res >> 24;
    core.not_z_flag = res;
    core.n_flag = res_hi;
    core.c_flag = 0;
    core.v_flag = 0;

    res
}

// Put common implementation of ORI here
// Put common implementation of ORI to CCR here
// Put common implementation of ORI to SR here
// Put common implementation of PEA here
// Put common implementation of RESET here
// Put common implementation of ROL, ROR here
pub fn ror_8(core: &mut Core, dst: u32, orig_shift: u32) -> u32 {
    let src = mask_out_above_8!(dst);

    if orig_shift != 0 {
        let shift = orig_shift & 7;
        let res = (src as u8).rotate_right(shift) as u32;
        core.n_flag = res;
        core.not_z_flag = res;
        core.v_flag = VFLAG_CLEAR;
        core.c_flag = src.wrapping_shl(8-(shift.wrapping_sub(1) & 7));
        res
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        src
    }
}

pub fn ror_16(core: &mut Core, dst: u32, orig_shift: u32) -> u32 {
    let src = mask_out_above_16!(dst);

    if orig_shift != 0 {
        let shift = orig_shift & 15;
        let res = (src as u16).rotate_right(shift) as u32;
        core.n_flag = res >> 8;
        core.not_z_flag = res;
        core.v_flag = VFLAG_CLEAR;
        core.c_flag = src.wrapping_shr(shift.wrapping_sub(1) & 15) << 8;
        res
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src >> 8;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        src
    }
}

pub fn ror_32(core: &mut Core, dst: u32, orig_shift: u32) -> u32 {
    let src = dst;
    if orig_shift != 0 {
        let shift = orig_shift & 31;
        let res = src.rotate_right(shift);
        core.n_flag = res >> 24;
        core.not_z_flag = res;
        core.v_flag = VFLAG_CLEAR;
        core.c_flag = src.wrapping_shr(shift.wrapping_sub(1) & 31) << 8;
        res
    } else {
        core.n_flag = src >> 24;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        core.c_flag = CFLAG_CLEAR;
        src
    }
}

pub fn rol_8(core: &mut Core, dst: u32, orig_shift: u32) -> u32 {
    let src = mask_out_above_8!(dst);

    if orig_shift != 0 {
        let shift = orig_shift & 7;
        if shift != 0 {
            let res = (src as u8).rotate_left(shift) as u32;
            core.n_flag = res;
            core.not_z_flag = res;
            core.c_flag = src.wrapping_shl(shift);
            core.v_flag = VFLAG_CLEAR;
            res
        } else {
            core.c_flag = (src & 1) << 8;
            core.n_flag = src;
            core.not_z_flag = src;
            core.v_flag = VFLAG_CLEAR;
            src
        }
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        src
    }
}

pub fn rol_16(core: &mut Core, dst: u32, orig_shift: u32) -> u32 {
    let src = mask_out_above_16!(dst);
    if orig_shift != 0 {
        let shift = orig_shift & 15;
        if shift != 0 {
            let res = (src as u16).rotate_left(shift) as u32;
            core.n_flag = res >> 8;
            core.not_z_flag = res;
            core.c_flag = src.wrapping_shl(shift) >> 8;
            core.v_flag = VFLAG_CLEAR;
            res
        } else {
            core.c_flag = (src & 1) << 8;
            core.n_flag = src >> 8;
            core.not_z_flag = src;
            core.v_flag = VFLAG_CLEAR;
            src
        }
    } else {
        core.c_flag = CFLAG_CLEAR;
        core.n_flag = src >> 8;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        src
    }
}

pub fn rol_32(core: &mut Core, dst: u32, orig_shift: u32) -> u32 {
    let src = dst;
    if orig_shift != 0 {
        let shift = orig_shift & 31;
        let res = src.rotate_left(shift);
        core.n_flag = res >> 24;
        core.not_z_flag = res;
        core.c_flag = src.wrapping_shr(32 - shift) << 8;
        core.v_flag = VFLAG_CLEAR;
        res
    } else {
        core.n_flag = src >> 24;
        core.not_z_flag = src;
        core.v_flag = VFLAG_CLEAR;
        core.c_flag = CFLAG_CLEAR;
        src
    }
}

// Put common implementation of ROXL, ROXR here
pub fn roxr_8(core: &mut Core, dst: u32, orig_shift: u32) -> u32 {
    if orig_shift != 0 {
        let shift = orig_shift % 9;
        let src = mask_out_above_8!(dst);
        let x8 = core.x_flag_as_1() << 8;
        let srcx8 = src | x8;
        let res = (srcx8 >> shift) | (srcx8 << (9-shift));
        core.x_flag = res;
        core.c_flag = core.x_flag;
        let res = mask_out_above_8!(res);
        core.n_flag = res;
        core.not_z_flag = res;
        core.v_flag = VFLAG_CLEAR;
        res
    } else {
        core.c_flag = core.x_flag;
        core.n_flag = dst;
        core.not_z_flag = mask_out_above_8!(dst);
        core.v_flag = VFLAG_CLEAR;
        dst
    }
}

pub fn roxr_16(core: &mut Core, dst: u32, orig_shift: u32) -> u32 {
    if orig_shift != 0 {
        let shift = orig_shift % 17;
        let src = mask_out_above_16!(dst);
        let x16 = core.x_flag_as_1() << 16;
        let srcx16 = src | x16;
        let res = (srcx16 >> shift) | (srcx16 << (17-shift));

        core.x_flag = res >> 8;
        core.c_flag = core.x_flag;
        let res = mask_out_above_16!(res);
        core.n_flag = res >> 8;
        core.not_z_flag = res;
        core.v_flag = VFLAG_CLEAR;
        res
    } else {
        core.c_flag = core.x_flag;
        core.n_flag = dst >> 8;
        core.not_z_flag = mask_out_above_16!(dst);
        core.v_flag = VFLAG_CLEAR;
        dst
    }
}

pub fn roxr_32(core: &mut Core, dst: u32, orig_shift: u32) -> u32 {
    let src = dst;
    let shift = orig_shift % 33;
    let res = if shift != 0 {
        let x32: u64 = (core.x_flag_as_1() as u64) << 32;
        let srcx32 = (src as u64) | x32;
        let res = (srcx32 >> shift) | (srcx32 << (33-shift));
        core.x_flag = (res >> 24) as u32;
        res as u32
    } else {
        src
    };
    core.c_flag = core.x_flag;
    core.n_flag = res >> 24;
    core.not_z_flag = res;
    core.v_flag = VFLAG_CLEAR;
    res
}

pub fn roxl_8(core: &mut Core, dst: u32, orig_shift: u32) -> u32 {
    if orig_shift != 0 {
        let shift = orig_shift % 9;
        let src = mask_out_above_8!(dst);
        let x8 = core.x_flag_as_1() << 8;
        let srcx8 = src | x8;
        let res = (srcx8 << shift) | (srcx8 >> (9-shift));
        core.x_flag = res;
        core.c_flag = core.x_flag;
        let res = mask_out_above_8!(res);
        core.n_flag = res;
        core.not_z_flag = res;
        core.v_flag = VFLAG_CLEAR;
        res
    } else {
        core.c_flag = core.x_flag;
        core.n_flag = dst;
        core.not_z_flag = mask_out_above_8!(dst);
        core.v_flag = VFLAG_CLEAR;
        dst
    }
}

pub fn roxl_16(core: &mut Core, dst: u32, orig_shift: u32) -> u32 {
    if orig_shift != 0 {
        let shift = orig_shift % 17;
        let src = mask_out_above_16!(dst);
        let x16 = core.x_flag_as_1() << 16;
        let srcx16 = src | x16;
        let res = (srcx16 << shift) | (srcx16 >> (17-shift));

        core.x_flag = res >> 8;
        core.c_flag = core.x_flag;
        let res = mask_out_above_16!(res);
        core.n_flag = res >> 8;
        core.not_z_flag = res;
        core.v_flag = VFLAG_CLEAR;
        res
    } else {
        core.c_flag = core.x_flag;
        core.n_flag = dst >> 8;
        core.not_z_flag = mask_out_above_16!(dst);
        core.v_flag = VFLAG_CLEAR;
        dst
    }
}

pub fn roxl_32(core: &mut Core, dst: u32, orig_shift: u32) -> u32 {
    let src = dst;
    let shift = orig_shift % 33;
    let res = if shift != 0 {
        let x32: u64 = (core.x_flag_as_1() as u64) << 32;
        let srcx32 = (src as u64) | x32;
        let res = (srcx32 << shift) | (srcx32 >> (33-shift));
        core.x_flag = (res >> 24) as u32;
        res as u32
    } else {
        src
    };
    core.c_flag = core.x_flag;
    core.n_flag = res >> 24;
    core.not_z_flag = res;
    core.v_flag = VFLAG_CLEAR;
    res
}

// Put common implementation of RTE here
// Put common implementation of RTR here
// Put common implementation of RTS here

pub fn sbcd_8(core: &mut Core, dst: u32, src: u32) -> u32 {
    let ln_src = low_nibble!(src);
    let hn_src = high_nibble!(src);
    let ln_dst = low_nibble!(dst);
    let hn_dst = high_nibble!(dst);

    let mut res = ln_dst.wrapping_sub(ln_src).wrapping_sub(core.x_flag_as_1());

    core.v_flag = !res;

    if res > 9 {
        res -= 6;
    }
    
    res = res.wrapping_add(hn_dst.wrapping_sub(hn_src));
    core.c_flag = true_is_1!(res > 0x99) << 8;
    core.x_flag = core.c_flag;

    if core.c_flag > 0 {
        res = res.wrapping_add(0xa0);
    }

    core.v_flag &= res;
    core.n_flag = res;

    res = mask_out_above_8!(res);
    core.not_z_flag |= res;
    res
}

// Put common implementation of Scc here
// Put common implementation of STOP here
// Put common implementation of SUB here

pub fn sub_8(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_8!(dst);
    let src = mask_out_above_8!(src);

    let res = dst.wrapping_sub(src);
    // m68ki_cpu.n_flag = (res);
    core.n_flag = res;
    // m68ki_cpu.v_flag = ((src^res) & (dst^res));
    core.v_flag = (src ^ dst) & (res ^ dst);
    // m68ki_cpu.x_flag = m68ki_cpu.c_flag = (res);
    core.c_flag = res;
    core.x_flag = res;
    // m68ki_cpu.not_z_flag = ((res) & 0xff);
    let res8 = mask_out_above_8!(res);
    core.not_z_flag = res8;
    res8
}

pub fn sub_16(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_16!(dst);
    let src = mask_out_above_16!(src);
    let res = dst.wrapping_sub(src);

    // m68ki_cpu.n_flag = ((res)>>8);
    let res_hi = res >> 8;
    core.n_flag = res_hi;
    // m68ki_cpu.v_flag = (((src^res) & (dst^res))>>8);
    core.v_flag = ((src ^ dst) & (res ^ dst)) >> 8;
    // m68ki_cpu.x_flag = m68ki_cpu.c_flag = ((res)>>8);
    core.c_flag = res_hi;
    core.x_flag = res_hi;
    // m68ki_cpu.not_z_flag = ((res) & 0xffff);
    let res16 = mask_out_above_16!(res);
    core.not_z_flag = res16;

    res16
}

pub fn sub_32(core: &mut Core, dst: u32, src: u32) -> u32 {
    let res: u64 = (dst as u64).wrapping_sub(src as u64);

    let res_hi = (res >> 24) as u32;
    core.n_flag = res_hi;
    // m68ki_cpu.v_flag = (((src^res) & (dst^res))>>24);
    core.v_flag = (((src as u64 ^ dst as u64) & (res as u64 ^ dst as u64)) >> 24) as u32;
     // m68ki_cpu.x_flag = m68ki_cpu.c_flag = (((src & dst) | (~res & (src | dst)))>>23);
    core.c_flag = res_hi;
    core.x_flag = res_hi;

    let res32 = res as u32;

    core.not_z_flag = res32;

    res32
}

pub fn subx_8(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_8!(dst);
    let src = mask_out_above_8!(src);
    let res = dst.wrapping_sub(src).wrapping_sub(core.x_flag_as_1());

    core.n_flag = res;
    core.v_flag = (src ^ dst) & (res ^ dst);
    core.c_flag = res;
    core.x_flag = res;

    let res8 = mask_out_above_8!(res);
    core.not_z_flag |= res8;
    res8
}

pub fn subx_16(core: &mut Core, dst: u32, src: u32) -> u32 {
    let dst = mask_out_above_16!(dst);
    let src = mask_out_above_16!(src);
    let res = dst.wrapping_sub(src).wrapping_sub(core.x_flag_as_1());

    let res_hi = res >> 8;
    core.n_flag = res_hi;
    core.v_flag = ((src ^ dst) & (res ^ dst)) >> 8;
    core.c_flag = res_hi;
    core.x_flag = res_hi;

    let res16 = mask_out_above_16!(res);
    core.not_z_flag |= res16;
    res16
}

pub fn subx_32(core: &mut Core, dst: u32, src: u32) -> u32 {
    let res = (dst as u64).wrapping_sub(src as u64).wrapping_sub(core.x_flag_as_1() as u64);

    let res_hi = (res >> 24) as u32;
    core.n_flag = res_hi;
    core.v_flag = (((src as u64 ^ dst as u64) & (res as u64 ^ dst as u64)) >> 24) as u32;
    core.c_flag = res_hi;
    core.x_flag = res_hi;

    let res32 = res as u32;
    core.not_z_flag |= res32;
    res32
}


// Put common implementation of SWAP here
// Put common implementation of TAS here
// Put common implementation of TRAP here
// Put common implementation of TRAPV here
// Put common implementation of TST here
// Put common implementation of UNLK here

#[cfg(test)]
mod tests {
    use super::super::super::Core;

    #[test]
    fn low_nibble() {
        assert_eq!(0x0a, low_nibble!(0xba));
    }
    #[test]
    fn high_nibble() {
        assert_eq!(0xb0, high_nibble!(0xba));
    }
    #[test]
    fn mask_out_below_8() {
        assert_eq!(0x2bcdef00, mask_out_below_8!(0x2bcdef73));
    }
    #[test]
    fn mask_out_above_8() {
        assert_eq!(0xf1, mask_out_above_8!(0x2bcdeff1));
    }
    #[test]
    fn dx_and_dy() {
        let mut core = Core::new(0x40);
        core.dar[0] = 0x00;
        core.dar[1] = 0x11;
        core.dar[2] = 0x22;
        core.dar[3] = 0x33;
        core.dar[4] = 0x44;
        core.dar[5] = 0x55;
        core.dar[6] = 0x66;
        core.dar[7] = 0x77;

        core.ir = 0b1111_1001_1111_1010; // X=4, Y=2
        assert_eq!(0x22, dy!(core));
        assert_eq!(0x44, dx!(core));

        core.ir = 0b1111_1011_1111_1110; // X=5, Y=6
        assert_eq!(0x66, dy!(core));
        assert_eq!(0x55, dx!(core));
    }
}
