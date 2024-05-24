use super::clocks;
use super::cpu::pc_state;
use super::memory::memory;

pub struct Interruptor {
    pub next_interrupt: u32,
}

pub trait Interrupt {
    fn interrupt<M>(pc_state: &mut pc_state::PcState, memory: &mut M)
    where
        M: memory::MemoryRW;
}

pub trait PollForInterrupt {
    fn poll_interrupts(&mut self, clock: &clocks::Clock) -> bool;
}

impl Interruptor {}

impl Interruptor {
    pub fn new() -> Self {
        Self { next_interrupt: 0 }
    }
    // TODO: Add the actual interruptor trait/implementation (previously VDU).
    pub fn set_cycle(&mut self, _cycles: clocks::ClockType) {
        // TODO: Do something
    }
}
