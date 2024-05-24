use super::super::clocks;
use super::super::graphics;
use super::super::interruptor;
use super::super::memory::memory;
use super::super::ports;
use super::instructions;
use super::pc_state;
use std::thread;
use std::time;

pub struct Core<M> {
    pub clock: clocks::Clock,
    pub memory: M,
    pc_state: pc_state::PcState,
    pub ports: ports::Ports,
    interruptor: interruptor::Interruptor,
    raw_display: Vec<u8>,
    start_time: time::SystemTime,
}

struct Constants {}

impl Constants {
    pub const CLOCK_HZ: u32 = 3590000; // set to Z80 clock speed for SMS
}

impl<M: memory::MemoryRW> Core<M> {
    pub const IRQIM1ADDR: u16 = 0x38;

    pub fn new(
        clock: clocks::Clock,
        memory: M,
        pc_state: pc_state::PcState,
        ports: ports::Ports,
        interruptor: interruptor::Interruptor,
    ) -> Self
    where
        M: memory::MemoryRW,
    {
        Self {
            clock,
            memory,
            pc_state,
            ports,
            interruptor,
            raw_display: vec![
                0;
                (graphics::vdp::Constants::SMS_WIDTH as usize)
                    * (graphics::vdp::Constants::SMS_HEIGHT as usize)
                    * (graphics::display::SDLUtility::bytes_per_pixel() as usize)
            ],
            start_time: time::SystemTime::now(),
        }
    }

    fn interupt(&mut self) {
        if self.pc_state.get_iff1() {
            if self.pc_state.get_im() == 1 {
                self.pc_state.increment_sp(-1);
                self.memory
                    .write(self.pc_state.get_sp(), self.pc_state.get_pc_high());
                self.pc_state.increment_sp(-1);
                self.memory
                    .write(self.pc_state.get_sp(), self.pc_state.get_pc_low());
                self.pc_state.set_pc(Core::<M>::IRQIM1ADDR);

                // Disable mask-able interrupts
                self.pc_state.set_iff1(false);
            } else {
                // TODO: Fix error messages/handling.
                println!("interupt mode not supported");
            }
        }
    }

    pub fn export(&mut self) -> bool {
        self.ports.export(&mut self.raw_display)
    }

    pub fn reset(&mut self) {
        self.pc_state = pc_state::PcState::new();
        self.start_time = time::SystemTime::now();
    }

    pub fn step(&mut self, debug: bool, realtime: bool) {
        // Start with 'expanded' version of step

        if realtime {
            let in_ms: u64 = self
                .start_time
                .elapsed()
                .expect("Error getting eplapsed")
                .as_millis() as u64;
            if 1000 * self.clock.cycles / Constants::CLOCK_HZ as u64 > in_ms {
                let required_sleep =
                    (1000 * self.clock.cycles / Constants::CLOCK_HZ as u64) - in_ms;
                thread::sleep(time::Duration::from_millis(required_sleep));
            }
        }

        self.interruptor.set_cycle(self.clock.cycles);

        let op_code = self.memory.read(self.pc_state.get_pc());

        if debug {
            print!(
                "{} {:x} {:x} ({:x} {:x}) ",
                self.clock.cycles,
                op_code,
                self.pc_state.get_pc(),
                op_code,
                self.memory.read(self.pc_state.get_pc() + 1)
            );
            println!("{}", self.pc_state);
        }
        self.pc_state.increment_pc(1);
        instructions::Instruction::execute(
            op_code,
            &mut self.clock,
            &mut self.memory,
            &mut self.pc_state,
            &mut self.ports,
            &mut self.interruptor,
        );
        if self
            .ports
            .poll_interrupts(&mut self.raw_display, &self.clock)
        {
            self.interupt();
        }
    }

    pub fn generate_display(&mut self, buffer: &mut [u8]) {
        // Function to populate the display buffer drawn to the 2D texture/canvas/window.
        buffer.clone_from_slice(self.raw_display.as_slice());
    }
}

#[test]
fn test_core_creation() {
    use super::super::graphics::vdp;

    let clock = clocks::Clock::new();
    let memory = memory::MemoryAbsolute::new();
    let pc_state = pc_state::PcState::new();
    let vdp = vdp::Vdp::new();
    let mut ports = ports::Ports::new();
    let interruptor = interruptor::Interruptor::new();
    ports.add_device(Box::new(vdp));
    let mut core = Core::new(clock, memory, pc_state, ports, interruptor);

    core.step(true, false);
    println!("{}", core.pc_state);
    core.step(true, false);
}
