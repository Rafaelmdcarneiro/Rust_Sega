// This module is intended to hold all of the 'extended' instructions.
// Basically all instructions that require 2 op-codes to decode
//
// 0xCB
// 0xDD
// 0xFD
// 0xED
//
// The main reason to separate, is to make it easier to adjust PC offset/timing settings later.
// Initially, setting each one to do it's own offset/increments.

use super::super::clocks;
use super::super::memory::memory;
use super::super::ports;
use super::instruction_set;
use super::pc_state;
use super::status_flags;

/*************************************************************************************/
/* Utility functions                                                                 */
/*************************************************************************************/

fn get_i8_displacement_as_u8<M, R16>(memory: &mut M, pc_reg: &R16) -> u16
where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
{
    // Note the '+2' assumes the 'dddddddd' is at a specific offset from the current pc.
    memory.read(pc_reg.get()) as i8 as u16
}

// 'pc_reg' and 'i16_reg' need the same trait, but can be different types.
fn get_i_d_address<M, R16>(memory: &mut M, pc_reg: &R16, i16_reg: &R16) -> u16
where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
{
    i16_reg
        .get()
        .wrapping_add(get_i8_displacement_as_u8(memory, pc_reg))
}

/*************************************************************************************/
/* Extended Load Instructions                                                        */
/*************************************************************************************/

// LD (IX+d), r; LD (IY+d),
// op code:  0xDD, 0b01110rrr, 0bdddddddd
// op code:  0xFD, 0b01110rrr, 0bdddddddd
// pub fn ld_iy_d_r
// pub fn ld_ix_d_r
pub fn ld_i_d_r<M, R16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    r: u8,
    pc_reg: &mut R16,
    i16_reg: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
{
    let address = get_i_d_address(memory, pc_reg, i16_reg);
    memory.write(address, r);
    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(19);
}

// LD I, nn
// LD IX, nn; LD IY, nn
pub fn ld_i_nn<M, R16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    i16_reg: &mut R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
{
    i16_reg.set(memory.read16(pc_reg.get()));
    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(14);
}

// LD I, (nn)
// LD IX, (nn); LD IY, (nn)
// was ld_i__nn
pub fn ld_i_mem_nn<M, R16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    i16_reg: &mut R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
{
    i16_reg.set(memory.read16(memory.read16(pc_reg.get())));
    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(20);
}

// LD (nn), HL (Extended)
// same as ld_nn_hl, but part of the extended group?
// pub fn ld_nn_hl_extended
// pub fn ld_nn_hl
// pub fn ld_nn_I
pub fn ld_mem_nn_reg16<M, R16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    reg16: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
{
    memory.write(memory.read16(pc_reg.get()), reg16.get_low());
    memory.write(memory.read16(pc_reg.get()) + 1, reg16.get_high());

    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(20);
}

// LD dd, (nn)
// 0b00dd0001 -> BC 00, DE 01, HL 10, SP 11
// 0bnnnnnnnn
// 0bnnnnnnnn
pub fn ld_dd_mem_nn<M, F: FnMut(&mut pc_state::PcState, u16)>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    mut reg16: F,
    pc_state: &mut pc_state::PcState,
) where
    M: memory::MemoryRW,
{
    reg16(pc_state, memory.read16(memory.read16(pc_state.get_pc())));

    pc_state.increment_pc(2);
    clock.increment(20);
}
//
//  LD SP, IX
//  LD SP, IY
//  Load index register into SP
pub fn ld_sp_i<R16>(clock: &mut clocks::Clock, pc_reg: &mut R16, sp_reg: &mut R16, i16_reg: &R16)
where
    R16: pc_state::Reg16RW,
{
    sp_reg.set(i16_reg.get());

    clock.increment(10);
}

// LD (IX+d), n; LD (IY+d), n
pub fn ld_i_d_n<M, R16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    i16_reg: &mut R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
{
    let tmp16 = get_i_d_address(memory, pc_reg, i16_reg);
    memory.write(tmp16, memory.read(pc_reg.get() + 1));
    pc_state::PcState::increment_reg(pc_reg, 2);
    clock.increment(19);
}

// LD r, (IY+d); LD r, (IY+d);
// 0xDD, 0b01rrr110
// 0xFD, 0b01rrr110
pub fn ld_r_i_d<M, F: FnMut(&mut pc_state::PcState, u8)>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_state: &mut pc_state::PcState,
    i16_value: u16,
    mut dst_fn: F,
) where
    M: memory::MemoryRW,
{
    let address = i16_value.wrapping_add(get_i8_displacement_as_u8(memory, &pc_state.pc_reg));
    dst_fn(pc_state, memory.read(address));
    pc_state.increment_pc(1);
    clock.increment(19);
}

// LD A, R
pub fn ld_a_r(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) {
    // Treat 'r' as relatively random (just connect to cycles) in the lower 7 bits.  Keep the highest bit.
    pc_state.set_r(((clock.cycles >> 2) & 0x7F) as u8 | (pc_state.get_r() & 0x80));
    pc_state.set_a(pc_state.get_r());
    let mut f_status = pc_state.get_f();
    status_flags::accumulator_flags(&mut f_status, pc_state.get_a(), pc_state.get_iff2());
    pc_state.set_f(f_status);

    clock.increment(9);
}

// LD A, I
pub fn ld_a_i(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) {
    pc_state.set_a(pc_state.get_i());
    let mut f_status = pc_state.get_f();
    status_flags::accumulator_flags(&mut f_status, pc_state.get_a(), pc_state.get_iff2());
    pc_state.set_f(f_status);

    clock.increment(9);
}

// LD R, A
pub fn ld_r_a(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) {
    pc_state.set_r(pc_state.get_a());
    clock.increment(9);
}

// LD I, A
pub fn ld_i_a(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) {
    pc_state.set_i(pc_state.get_a());
    clock.increment(9);
}

// POP I
pub fn pop_i<M, R16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    sp_reg: &mut R16,
    i16_reg: &mut R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
{
    i16_reg.set_low(memory.read(sp_reg.get()));
    pc_state::PcState::increment_reg(sp_reg, 1);
    i16_reg.set_high(memory.read(sp_reg.get()));
    pc_state::PcState::increment_reg(sp_reg, 1);
    clock.increment(14);
}

// PUSH I
pub fn push_i<M, R16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    sp_reg: &mut R16,
    i16_reg: &mut R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
{
    pc_state::PcState::increment_reg(sp_reg, -1);
    memory.write(sp_reg.get(), i16_reg.get_high());

    pc_state::PcState::increment_reg(sp_reg, -1);
    memory.write(sp_reg.get(), i16_reg.get_low());

    clock.increment(15);
}

// LDDR
pub fn lddr<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState)
where
    M: memory::MemoryRW,
{
    memory.write(pc_state.de_reg.get(), memory.read(pc_state.hl_reg.get()));
    pc_state::PcState::increment_reg(&mut pc_state.de_reg, -1);
    pc_state::PcState::increment_reg(&mut pc_state.hl_reg, -1);
    pc_state::PcState::increment_reg(&mut pc_state.bc_reg, -1);
    if pc_state.bc_reg.get() == 0 {
        let mut f_status = pc_state.get_f();
        f_status.set_h(0);
        f_status.set_n(0);
        f_status.set_pv(0);
        pc_state.set_f(f_status);

        clock.increment(16);
    } else {
        pc_state::PcState::increment_reg(&mut pc_state.pc_reg, -2);
        // This branch is longer because the PC is actually 'decremented' by two
        clock.increment(21);
    }
}

// LDIR
pub fn ldir<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState)
where
    M: memory::MemoryRW,
{
    pc_state::PcState::increment_reg(&mut pc_state.bc_reg, -1);
    memory.write(pc_state.de_reg.get(), memory.read(pc_state.hl_reg.get()));
    pc_state::PcState::increment_reg(&mut pc_state.de_reg, 1);
    pc_state::PcState::increment_reg(&mut pc_state.hl_reg, 1);

    let mut f_status = pc_state.get_f();
    f_status.set_h(0);
    f_status.set_pv(0);
    if pc_state.bc_reg.get() == 0 {
        f_status.set_n(0);
        clock.increment(16);
    } else {
        // This branch is longer because the PC is actually 'decremented' by two
        f_status.set_n(1); // Not sure.
        pc_state::PcState::increment_reg(&mut pc_state.pc_reg, -2);
        clock.increment(21);
    }
    pc_state.set_f(f_status);
}

// OTIR
// Flags match emulator, not z80 document
pub fn otir<M>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_state: &mut pc_state::PcState,
    ports: &mut ports::Ports,
) where
    M: memory::MemoryRW,
{
    pc_state.set_b(pc_state.get_b().wrapping_sub(1));
    ports.port_write(clock, pc_state.get_c(), memory.read(pc_state.hl_reg.get()));
    pc_state::PcState::increment_reg(&mut pc_state.hl_reg, 1);

    let mut f_status = pc_state.get_f();
    // ??? status.set_c(0)
    f_status.set_s(0); // Unknown
    f_status.set_h(0); // Unknown
    f_status.set_pv(0); // Unknown
    f_status.set_n(1);
    if pc_state.get_b() == 0 {
        f_status.set_z(1);
        clock.increment(16);
    } else {
        f_status.set_z(0);
        pc_state.increment_pc(-2);
        clock.increment(21);
    }

    pc_state.set_f(f_status);
}

// EX SP I
pub fn ex_sp_i<M, R16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    sp_reg: &mut R16,
    i16_reg: &mut R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
{
    let mut tmp8 = memory.read(sp_reg.get());
    memory.write(sp_reg.get(), i16_reg.get_low());
    i16_reg.set_low(tmp8);

    tmp8 = memory.read(sp_reg.get() + 1);
    memory.write(sp_reg.get() + 1, i16_reg.get_high());
    i16_reg.set_high(tmp8);

    clock.increment(23);
}

///////////////////////////////////////////////////////////////////////
//  BIT instructions
///////////////////////////////////////////////////////////////////////

// BIT b, r
pub fn bit_b_r<R16, F16>(
    clock: &mut clocks::Clock,
    bit_pos: u8,
    r: u8,
    pc_reg: &mut R16,
    af_reg: &mut F16,
) where
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    let mut f_status = af_reg.get_flags();
    status_flags::set_bit_test_flags(r, bit_pos, &mut f_status);
    af_reg.set_flags(&f_status);
    clock.increment(8);
}

// BIT b, (HL)
pub fn bit_b_mem<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    bit_pos: u8,
    pc_reg: &mut R16,
    af_reg: &mut F16,
    addr_reg: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    let mut f_status = af_reg.get_flags();
    status_flags::set_bit_test_flags(memory.read(addr_reg.get()), bit_pos, &mut f_status);
    af_reg.set_flags(&f_status);
    clock.increment(12);
}

pub fn set_b_r<F: FnMut(&mut pc_state::PcState, u8)>(
    clock: &mut clocks::Clock,
    bit_pos: u8,
    pc_state: &mut pc_state::PcState,
    mut dst_fn: F,
    original_value: u8,
) {
    dst_fn(pc_state, original_value | (0x1 << bit_pos));
    clock.increment(8);
}

// SET b, (HL)
pub fn set_b_mem<M, R16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    bit_pos: u8,
    pc_reg: &mut R16,
    addr_reg: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
{
    memory.write(
        addr_reg.get(),
        memory.read(addr_reg.get()) | (0x1 << bit_pos),
    );

    clock.increment(12);
}

// RES b, r
pub fn res_b_r<F: FnMut(&mut pc_state::PcState, u8)>(
    clock: &mut clocks::Clock,
    bit_pos: u8,
    pc_state: &mut pc_state::PcState,
    mut dst_fn: F,
    original_value: u8,
) {
    dst_fn(pc_state, original_value & !(0x1 << bit_pos));
    clock.increment(8);
}

// RES b, (HL)
pub fn res_b_mem<M, R16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    bit_pos: u8,
    pc_reg: &mut R16,
    addr_reg: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
{
    memory.write(
        addr_reg.get(),
        memory.read(addr_reg.get()) & !(0x1 << bit_pos),
    );

    clock.increment(12);
}

// BIT b, (IY+d),  BIT b, (IX+d) (if mem at pc + 3 -> 0b01XXXXXX)
// RES b, (IY+d),  RES b, (IX+d) (if mem at pc + 3 -> 0b10XXXXXX)
// SET b, (IY+d),  SET b, (IX+d) (if mem at pc + 3 -> 0b11XXXXXX)
// (if mem at pc + 3 -> 0b11XXXXXX) -> ERROR
pub fn bit_res_set_b_i_d<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    af_reg: &mut F16,
    i16_reg: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    let tmp16 = get_i_d_address(memory, pc_reg, i16_reg);
    let test_value = memory.read(tmp16);
    let op_details = memory.read(pc_reg.get() + 1);
    let bit_pos = (op_details >> 3) & 0x7;

    match op_details >> 6 {
        0b01 => {
            /* BIT b */
            let mut f_status = af_reg.get_flags();
            status_flags::set_bit_test_flags(test_value, bit_pos, &mut f_status);
            af_reg.set_flags(&f_status);
            clock.increment(20);
        }
        0b10 => {
            /* RES b */
            memory.write(tmp16, test_value & !(0x1 << bit_pos));
            clock.increment(23);
        }
        0b11 => {
            /* SET b */
            memory.write(tmp16, test_value | (0x1 << bit_pos));
            clock.increment(23);
        }
        _ => {
            panic!("Unsupported byte value! {}", op_details);
        }
    }

    pc_state::PcState::increment_reg(pc_reg, 2);
}

///////////////////////////////////////////////////////////////////////
//  Jump instructions
///////////////////////////////////////////////////////////////////////

//  JP (IX), JP (IY)
// Load PC with IX, IY, to jump to that location.
pub fn jp_i<R16>(clock: &mut clocks::Clock, pc_reg: &mut R16, i16_reg: &R16)
where
    R16: pc_state::Reg16RW,
{
    pc_reg.set(i16_reg.get());
    clock.increment(8);
}

// CP n
// Compare accumulator with 'n' to set status flags (but don't change accumulator)
//pub fn cp_i_d<M>(clock: &mut clocks::Clock, memory: &mut M, i16_value: u16, pc_state: &mut pc_state::PcState) -> () where M: memory::MemoryRW {
pub fn cp_i_d<M, R16, AF>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    i16_reg: &mut R16,
    af_reg: &mut AF,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    AF: pc_state::FlagReg + pc_state::AfRegister,
{
    // This function sets the 'pc_state.f'
    let address = get_i_d_address(memory, pc_reg, i16_reg);
    instruction_set::cp_flags(af_reg.get_a(), memory.read(address), af_reg);

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(19);
}

// CPI
// Compare accumulator with contents of memory address HL, increment HL
pub fn cpi<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState)
where
    M: memory::MemoryRW,
{
    // This function sets the 'pc_state.f'
    instruction_set::cp_flags(
        pc_state.get_a(),
        memory.read(pc_state.hl_reg.get()),
        &mut pc_state.af_reg,
    );

    pc_state::PcState::increment_reg(&mut pc_state.hl_reg, 1);
    pc_state::PcState::increment_reg(&mut pc_state.bc_reg, -1);
    let mut f_status = pc_state.get_f();
    if pc_state.bc_reg.get() == 0 {
        f_status.set_pv(1);
    } else {
        f_status.set_pv(0);
    }

    pc_state.set_f(f_status);

    clock.increment(16);
}

// CPD
// Compare accumulator with contents of memory address HL, decrement HL
pub fn cpd<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState)
where
    M: memory::MemoryRW,
{
    // This function sets the 'pc_state.f'
    instruction_set::cp_flags(
        pc_state.get_a(),
        memory.read(pc_state.hl_reg.get()),
        &mut pc_state.af_reg,
    );

    pc_state::PcState::increment_reg(&mut pc_state.hl_reg, -1);
    pc_state::PcState::increment_reg(&mut pc_state.bc_reg, -1);
    let mut f_status = pc_state.get_f();
    if pc_state.bc_reg.get() == 0 {
        f_status.set_pv(1);
    } else {
        f_status.set_pv(0);
    }

    pc_state.set_f(f_status);

    clock.increment(16);
}

// LDI
// Load, increment HL, DE, decrement BC.
pub fn ldi<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState)
where
    M: memory::MemoryRW,
{
    memory.write(pc_state.de_reg.get(), memory.read(pc_state.hl_reg.get()));

    pc_state::PcState::increment_reg(&mut pc_state.hl_reg, 1);
    pc_state::PcState::increment_reg(&mut pc_state.de_reg, 1);
    pc_state::PcState::increment_reg(&mut pc_state.bc_reg, -1);
    let mut f_status = pc_state.get_f();
    if pc_state.bc_reg.get() == 0 {
        f_status.set_pv(1);
    } else {
        f_status.set_pv(0);
    }
    f_status.set_h(0);
    f_status.set_n(0);

    pc_state.set_f(f_status);

    clock.increment(16);
}

// LDD
// Load, decrement HL, DE, decrement BC.
pub fn ldd<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState)
where
    M: memory::MemoryRW,
{
    memory.write(pc_state.de_reg.get(), memory.read(pc_state.hl_reg.get()));

    pc_state::PcState::increment_reg(&mut pc_state.hl_reg, -1);
    pc_state::PcState::increment_reg(&mut pc_state.de_reg, -1);
    pc_state::PcState::increment_reg(&mut pc_state.bc_reg, -1);
    let mut f_status = pc_state.get_f();
    if pc_state.bc_reg.get() == 0 {
        f_status.set_pv(1);
    } else {
        f_status.set_pv(0);
    }
    f_status.set_h(0);
    f_status.set_n(0);

    pc_state.set_f(f_status);

    clock.increment(16);
}

// CPIR
// Compare and repeat,  A with the contents of memory in HL, increment HL, decrement BC.
pub fn cpir<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState)
where
    M: memory::MemoryRW,
{
    // This function sets the 'pc_state.f'
    let original_carry = pc_state.get_f().get_c();
    pc_state::PcState::increment_reg(&mut pc_state.bc_reg, -1);
    instruction_set::cp_flags(
        pc_state.get_a(),
        memory.read(pc_state.hl_reg.get()),
        &mut pc_state.af_reg,
    );
    pc_state::PcState::increment_reg(&mut pc_state.hl_reg, 1);
    let mut f_status = pc_state.get_f();
    f_status.set_c(original_carry);
    if (pc_state.bc_reg.get() == 0) || f_status.get_z() == 1 {
        f_status.set_pv(0);
        clock.increment(16);
    } else {
        f_status.set_pv(1);
        pc_state.increment_pc(-2);
        clock.increment(21);
    }

    pc_state.set_f(f_status);
}

// CPDR
// Compare and repeat,  A with the contents of memory in HL, decrement HL, decrement BC.
pub fn cpdr<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState)
where
    M: memory::MemoryRW,
{
    // This function sets the 'pc_state.f'
    let original_carry = pc_state.get_f().get_c();
    pc_state::PcState::increment_reg(&mut pc_state.bc_reg, -1);
    instruction_set::cp_flags(
        pc_state.get_a(),
        memory.read(pc_state.hl_reg.get()),
        &mut pc_state.af_reg,
    );
    pc_state::PcState::increment_reg(&mut pc_state.hl_reg, -1);
    let mut f_status = pc_state.get_f();
    f_status.set_c(original_carry);
    if (pc_state.bc_reg.get() == 0) || f_status.get_z() == 1 {
        f_status.set_pv(0);
        clock.increment(16);
    } else {
        f_status.set_pv(1);
        pc_state.increment_pc(-2);
        clock.increment(21);
    }

    pc_state.set_f(f_status);
}

// RTI
// Fself.pc_state.IXME, should check, since there is only one
// interupting device, this is the same as normal ret
// RETI
pub fn reti<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState)
where
    M: memory::MemoryRW,
{
    pc_state.set_pc_low(memory.read(pc_state.sp_reg.get()));
    pc_state.increment_sp(1);
    pc_state.set_pc_high(memory.read(pc_state.sp_reg.get()));
    pc_state.increment_sp(1);

    clock.increment(14);
}

////////////////////////////////////////////////////
// 16-bit arithmetic Group
////////////////////////////////////////////////////

// ADD HL, ss
pub fn add16<R16, F16>(
    clock: &mut clocks::Clock,
    src_value: u16,
    pc_reg: &mut R16,
    dst_reg: &mut R16,
    af_reg: &mut F16,
) where
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    let mut f_status = af_reg.get_flags();
    dst_reg.set(status_flags::u16_no_carry(
        dst_reg.get(),
        src_value,
        &mut f_status,
    ));
    f_status.set_n(0);
    af_reg.set_flags(&f_status);

    clock.increment(15);
}

pub fn add16c<F16>(a: u16, b: u16, c: bool, af_reg: &mut F16) -> u16
where
    F16: pc_state::FlagReg,
{
    let mut f_status = af_reg.get_flags();
    let result = status_flags::u16_carry(a, b, c, &mut f_status);
    f_status.set_n(0);
    af_reg.set_flags(&f_status);

    result
}

pub fn sub16c<F16>(a: u16, b: u16, c: bool, af_reg: &mut F16) -> u16
where
    F16: pc_state::FlagReg,
{
    let mut f_status = af_reg.get_flags();
    let result = status_flags::i16_carry(a, b, c, &mut f_status);

    af_reg.set_flags(&f_status);

    result
}

// INC I
pub fn inc_16<R16>(clock: &mut clocks::Clock, pc_reg: &mut R16, reg16: &mut R16)
where
    R16: pc_state::Reg16RW,
{
    reg16.set(reg16.get().wrapping_add(1));
    clock.increment(10);
}

// DEC I
pub fn dec_16<R16>(clock: &mut clocks::Clock, pc_reg: &mut R16, reg16: &mut R16)
where
    R16: pc_state::Reg16RW,
{
    reg16.set(reg16.get().wrapping_sub(1));
    clock.increment(10);
}

// INC (IX+d), INC (IY+d),
pub fn inc_i_d<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    af_reg: &mut F16,
    i16_reg: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    let address = get_i_d_address(memory, pc_reg, i16_reg);
    let new_value = memory.read(address).wrapping_add(1);

    memory.write(address, new_value);

    let mut f_value = af_reg.get_flags();
    status_flags::calculate_inc_flags(&mut f_value, new_value);
    af_reg.set_flags(&f_value);

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(23);
}

// DEC (IX+d), DEC (IY+d),
pub fn dec_i_d<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    af_reg: &mut F16,
    i16_reg: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    let address = get_i_d_address(memory, pc_reg, i16_reg);
    let new_value = memory.read(address).wrapping_sub(1);

    memory.write(address, new_value);

    let mut f_value = af_reg.get_flags();
    status_flags::calculate_dec_flags(&mut f_value, new_value);
    af_reg.set_flags(&f_value);

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(23);
}

// ADC (IX+d),
// ADC (IY+d),
pub fn adc_i_d<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    i16_reg: &R16,
    af_reg: &mut F16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg + pc_state::AfRegister,
{
    let address = get_i_d_address(memory, pc_reg, i16_reg);

    let new_value = instruction_set::add8c(
        af_reg.get_a(),
        memory.read(address),
        af_reg.get_flags().get_c() == 1,
        af_reg,
    );
    af_reg.set_a(new_value);

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(19);
}

// SUB (IX+d),
// SUB (IY+d),
pub fn sub_i_d<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    i16_reg: &R16,
    af_reg: &mut F16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg + pc_state::AfRegister,
{
    let address = get_i_d_address(memory, pc_reg, i16_reg);
    let new_value = instruction_set::sub8(af_reg.get_a(), memory.read(address), af_reg);
    af_reg.set_a(new_value);

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(19);
}

// AND (IX+d),
// AND (IY+d),
pub fn and_i_d<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    i16_reg: &R16,
    af_reg: &mut F16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg + pc_state::AfRegister,
{
    let address = get_i_d_address(memory, pc_reg, i16_reg);
    let new_value = af_reg.get_a() & memory.read(address);
    let mut f_status = af_reg.get_flags();
    status_flags::and_flags(&mut f_status, new_value);
    af_reg.set_flags(&f_status);
    af_reg.set_a(new_value);

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(19);
}

// XOR (IX+d)
// XOR (IY+d)
pub fn xor_i_d<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    i16_reg: &R16,
    af_reg: &mut F16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg + pc_state::AfRegister,
{
    let address = get_i_d_address(memory, pc_reg, i16_reg);
    let new_value = af_reg.get_a() ^ memory.read(address);
    let mut f_status = af_reg.get_flags();
    status_flags::xor_flags(&mut f_status, new_value);
    af_reg.set_flags(&f_status);
    af_reg.set_a(new_value);

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(19);
}

// OR (IX+d),
// OR (IY+d),
pub fn or_i_d<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    i16_reg: &R16,
    af_reg: &mut F16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg + pc_state::AfRegister,
{
    let address = get_i_d_address(memory, pc_reg, i16_reg);
    let new_value = af_reg.get_a() | memory.read(address);
    let mut f_status = af_reg.get_flags();
    status_flags::or_flags(&mut f_status, new_value);
    af_reg.set_flags(&f_status);
    af_reg.set_a(new_value);

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(19);
}

// ADD A (IX+d)
// ADD A (IY+d)
pub fn add_i_d<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    i16_reg: &R16,
    af_reg: &mut F16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg + pc_state::AfRegister,
{
    let address = get_i_d_address(memory, pc_reg, i16_reg);

    let new_value = instruction_set::add8(af_reg.get_a(), memory.read(address), af_reg);
    af_reg.set_a(new_value);

    pc_state::PcState::increment_reg(pc_reg, 1);
    clock.increment(19);
}

////////////////////////////////////////////////////
// Rotate and shift group
////////////////////////////////////////////////////

// RL r
// Rotate Left
fn common_rotate_shift<F: FnMut(&mut pc_state::PcState, u8), Rot: Fn(u8, bool) -> (u8, bool)>(
    shift_rot_fn: Rot,
    clock: &mut clocks::Clock,
    pc_state: &mut pc_state::PcState,
    mut dst_fn: F,
    src: u8,
) {
    let mut f_value = pc_state.get_f();
    let (new_value, carry) = shift_rot_fn(src, f_value.get_c() == 1);

    dst_fn(pc_state, new_value);
    status_flags::set_shift_register_flags(new_value, carry, &mut f_value);
    pc_state.set_f(f_value);

    clock.increment(8);
}

// RRC r
// Rotate Right with carry
pub fn rrc_r<F: FnMut(&mut pc_state::PcState, u8)>(
    clock: &mut clocks::Clock,
    pc_state: &mut pc_state::PcState,
    dst_fn: F,
    src: u8,
) {
    // Create closure for unused argument
    common_rotate_shift(
        |input, _carry| instruction_set::rotate_right_carry(input),
        clock,
        pc_state,
        dst_fn,
        src,
    );
}

// RR r
// Rotate Right
pub fn rr_r<F: FnMut(&mut pc_state::PcState, u8)>(
    clock: &mut clocks::Clock,
    pc_state: &mut pc_state::PcState,
    dst_fn: F,
    src: u8,
) {
    common_rotate_shift(instruction_set::rotate_right, clock, pc_state, dst_fn, src);
}

// RLC r
// Rotate Left with carry
pub fn rlc_r<F: FnMut(&mut pc_state::PcState, u8)>(
    clock: &mut clocks::Clock,
    pc_state: &mut pc_state::PcState,
    dst_fn: F,
    src: u8,
) {
    // Create closure for unused argument
    common_rotate_shift(
        |input, _carry| instruction_set::rotate_left_carry(input),
        clock,
        pc_state,
        dst_fn,
        src,
    );
}

// RL r
// Rotate Left
pub fn rl_r<F: FnMut(&mut pc_state::PcState, u8)>(
    clock: &mut clocks::Clock,
    pc_state: &mut pc_state::PcState,
    dst_fn: F,
    src: u8,
) {
    common_rotate_shift(instruction_set::rotate_left, clock, pc_state, dst_fn, src);
}

// SLA r
// Shift Left Arithmetic
pub fn sla_r<F: FnMut(&mut pc_state::PcState, u8)>(
    clock: &mut clocks::Clock,
    pc_state: &mut pc_state::PcState,
    dst_fn: F,
    src: u8,
) {
    // Create closure for unused argument
    common_rotate_shift(
        |input, _carry| instruction_set::shift_left_arithmetic(input),
        clock,
        pc_state,
        dst_fn,
        src,
    );
}

// SLL r
// Shift Left Logical (?) undocumented, inserts a 1 in the lower bit
pub fn sll_r<F: FnMut(&mut pc_state::PcState, u8)>(
    clock: &mut clocks::Clock,
    pc_state: &mut pc_state::PcState,
    dst_fn: F,
    src: u8,
) {
    // Create closure for unused argument
    common_rotate_shift(
        |input, _carry| instruction_set::shift_left_logical(input),
        clock,
        pc_state,
        dst_fn,
        src,
    );
}

// SRA r
// Shift Right Arithmetic
pub fn sra_r<F: FnMut(&mut pc_state::PcState, u8)>(
    clock: &mut clocks::Clock,
    pc_state: &mut pc_state::PcState,
    dst_fn: F,
    src: u8,
) {
    // Create closure for unused argument
    common_rotate_shift(
        |input, _carry| instruction_set::shift_right_arithmetic(input),
        clock,
        pc_state,
        dst_fn,
        src,
    );
}

// SRL r
// Shift Right Logical
pub fn srl_r<F: FnMut(&mut pc_state::PcState, u8)>(
    clock: &mut clocks::Clock,
    pc_state: &mut pc_state::PcState,
    dst_fn: F,
    src: u8,
) {
    // Create closure for unused argument
    common_rotate_shift(
        |input, _carry| instruction_set::shift_right_logical(input),
        clock,
        pc_state,
        dst_fn,
        src,
    );
}

// RLC (HL)
pub fn rlc_hl<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    af_reg: &mut F16,
    addr_reg: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    let src = memory.read(addr_reg.get());
    let mut f_value = af_reg.get_flags();

    let (new_value, carry) = instruction_set::rotate_left_carry(src);
    status_flags::set_shift_register_flags(new_value, carry, &mut f_value);
    af_reg.set_flags(&f_value);
    memory.write(addr_reg.get(), new_value);

    clock.increment(15);
}

// RRC (HL)
pub fn rrc_hl<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    af_reg: &mut F16,
    addr_reg: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    let src = memory.read(addr_reg.get());
    let mut f_value = af_reg.get_flags();

    let (new_value, carry) = instruction_set::rotate_right_carry(src);
    status_flags::set_shift_register_flags(new_value, carry, &mut f_value);
    af_reg.set_flags(&f_value);
    memory.write(addr_reg.get(), new_value);

    clock.increment(15);
}

// SLA (HL)
pub fn sla_hl<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    af_reg: &mut F16,
    addr_reg: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    let src = memory.read(addr_reg.get());
    let mut f_value = af_reg.get_flags();

    let (new_value, carry) = instruction_set::shift_left_arithmetic(src);
    status_flags::set_shift_register_flags(new_value, carry, &mut f_value);
    af_reg.set_flags(&f_value);
    memory.write(addr_reg.get(), new_value);

    clock.increment(15);
}

// SRA (HL)
pub fn sra_hl<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    af_reg: &mut F16,
    addr_reg: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    let src = memory.read(addr_reg.get());
    let mut f_value = af_reg.get_flags();

    let (new_value, carry) = instruction_set::shift_right_arithmetic(src);
    status_flags::set_shift_register_flags(new_value, carry, &mut f_value);
    af_reg.set_flags(&f_value);
    memory.write(addr_reg.get(), new_value);

    clock.increment(15);
}

// SRL (HL)
pub fn srl_hl<M, R16, F16>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_reg: &mut R16,
    af_reg: &mut F16,
    addr_reg: &R16,
) where
    M: memory::MemoryRW,
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    let src = memory.read(addr_reg.get());
    let mut f_value = af_reg.get_flags();

    let (new_value, carry) = instruction_set::shift_right_logical(src);
    status_flags::set_shift_register_flags(new_value, carry, &mut f_value);
    af_reg.set_flags(&f_value);
    memory.write(addr_reg.get(), new_value);

    clock.increment(15);
}

// RRD
// Rotate right decimal (basically nibble shift right).
pub fn rrd<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState)
where
    M: memory::MemoryRW,
{
    let original_a = pc_state.get_a();
    let original_hl_mem = memory.read(pc_state.hl_reg.get());

    let new_value = original_a & 0xF0 | original_hl_mem & 0xF;
    pc_state.set_a(new_value);

    // Write back to HL
    memory.write(
        pc_state.hl_reg.get(),
        (original_hl_mem >> 4) & 0xF | (original_a & 0xF) << 4,
    );

    let mut f_value = pc_state.get_f();
    status_flags::rotate_decimal_flags(&mut f_value, new_value);

    pc_state.set_f(f_value);

    clock.increment(18);
}

// RLD
// Rotate left decimal (basically nibble shift right).
pub fn rld<M>(clock: &mut clocks::Clock, memory: &mut M, pc_state: &mut pc_state::PcState)
where
    M: memory::MemoryRW,
{
    let original_a = pc_state.get_a();
    let original_hl_mem = memory.read(pc_state.hl_reg.get());

    let new_value = original_a & 0xF0 | (original_hl_mem >> 4) & 0xF;
    pc_state.set_a(new_value);

    // Write back to HL
    memory.write(
        pc_state.hl_reg.get(),
        ((original_hl_mem & 0xF) << 4) | (original_a & 0xF),
    );

    let mut f_value = pc_state.get_f();
    status_flags::rotate_decimal_flags(&mut f_value, new_value);
    pc_state.set_f(f_value);

    clock.increment(18);
}

// IN r, (C)
pub fn in_r<F: FnMut(&mut pc_state::PcState, u8)>(
    clock: &mut clocks::Clock,
    src_val: u8,
    pc_state: &mut pc_state::PcState,
    mut dst_fn: F,
    ports: &mut ports::Ports,
) {
    dst_fn(pc_state, ports.port_read(clock, src_val));
    clock.increment(12);
}

// OUT r, (C)
pub fn out_r(
    clock: &mut clocks::Clock,
    src_val: u8,
    pc_state: &mut pc_state::PcState,
    out: u8,
    ports: &mut ports::Ports,
) {
    ports.port_write(clock, src_val, out);
    clock.increment(12);
}

// OUTI
pub fn outi<M>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_state: &mut pc_state::PcState,
    ports: &mut ports::Ports,
) where
    M: memory::MemoryRW,
{
    pc_state.set_b(pc_state.get_b().wrapping_sub(1));
    ports.port_write(clock, pc_state.get_c(), memory.read(pc_state.hl_reg.get()));
    pc_state::PcState::increment_reg(&mut pc_state.hl_reg, 1);

    let mut f_status = pc_state.get_f();
    if pc_state.get_b() == 0 {
        f_status.set_z(1);
    } else {
        f_status.set_z(0);
    }
    f_status.set_n(1);
    pc_state.set_f(f_status);

    clock.increment(16);
}

// INI
pub fn ini<M>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_state: &mut pc_state::PcState,
    ports: &mut ports::Ports,
) where
    M: memory::MemoryRW,
{
    pc_state.set_b(pc_state.get_b().wrapping_sub(1));
    memory.write(
        pc_state.hl_reg.get(),
        ports.port_read(clock, pc_state.get_c()),
    );
    pc_state::PcState::increment_reg(&mut pc_state.hl_reg, 1);

    let mut f_status = pc_state.get_f();
    if pc_state.get_b() == 0 {
        f_status.set_z(1);
    } else {
        f_status.set_z(0);
    }
    f_status.set_n(1);
    pc_state.set_f(f_status);

    clock.increment(16);
}

// OUTD
pub fn outd<M>(
    clock: &mut clocks::Clock,
    memory: &mut M,
    pc_state: &mut pc_state::PcState,
    ports: &mut ports::Ports,
) where
    M: memory::MemoryRW,
{
    pc_state.set_b(pc_state.get_b().wrapping_sub(1));
    ports.port_write(clock, pc_state.get_c(), memory.read(pc_state.hl_reg.get()));

    let mut f_status = pc_state.get_f();
    if pc_state.get_b() == 0 {
        f_status.set_z(1);
    } else {
        f_status.set_z(0);
    }
    f_status.set_n(1);
    pc_state.set_f(f_status);

    pc_state::PcState::increment_reg(&mut pc_state.hl_reg, -1);
    clock.increment(16);
}

/*************************************************************************************/
/* General purpose arithmetic and CPU control                                        */
/*************************************************************************************/

pub fn neg(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) {
    let result = instruction_set::sub8(0, pc_state.get_a(), &mut pc_state.af_reg);
    pc_state.set_a(result);

    clock.increment(8);
}

pub fn im_1(clock: &mut clocks::Clock, pc_state: &mut pc_state::PcState) {
    pc_state.set_im(1);
    clock.increment(8);
}

pub fn sbc_hl_r16<R16, F16>(
    clock: &mut clocks::Clock,
    src_value: u16,
    pc_reg: &mut R16,
    hl_reg: &mut R16,
    af_reg: &mut F16,
) where
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    hl_reg.set(sub16c(
        hl_reg.get(),
        src_value,
        af_reg.get_flags().get_c() == 1,
        af_reg,
    ));
    clock.increment(15);
}

pub fn adc_hl_r16<R16, F16>(
    clock: &mut clocks::Clock,
    src_value: u16,
    pc_reg: &mut R16,
    hl_reg: &mut R16,
    af_reg: &mut F16,
) where
    R16: pc_state::Reg16RW,
    F16: pc_state::FlagReg,
{
    hl_reg.set(add16c(
        hl_reg.get(),
        src_value,
        af_reg.get_flags().get_c() == 1,
        af_reg,
    ));
    clock.increment(15);
}

pub fn sbc_i() {
    panic!("SBC A, (IX+d), SBC A, (IY+d) not implemented");
}

#[cfg(test)]
mod tests {
    use crate::sega::cpu::extended_instruction_set;
    use crate::sega::cpu::pc_state;

    #[test]
    fn test_add_sub_functions() {
        let mut pc_state = pc_state::PcState::new();

        assert_eq!(
            extended_instruction_set::add16c(0xFFFF, 0xFFFF, true, &mut pc_state.af_reg),
            0xFFFF
        );
        assert_eq!(
            extended_instruction_set::add16c(0, 0, false, &mut pc_state.af_reg),
            0
        );
        assert_eq!(pc_state.get_f().get_z(), 1);
        assert_eq!(pc_state.get_f().get_n(), 0);

        assert_eq!(
            extended_instruction_set::add16c(0x3FFF, 0x7001, true, &mut pc_state.af_reg),
            0xB001
        );
        assert_eq!(pc_state.get_f().get_h(), 1);

        assert_eq!(
            extended_instruction_set::sub16c(0x0000, 0x000F, true, &mut pc_state.af_reg),
            0xFFF0
        );
        assert_eq!(pc_state.get_f().get_n(), 1);
    }
}
