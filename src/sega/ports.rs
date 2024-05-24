use super::audio::sound;
use super::clocks;
use super::inputs;

struct NullPort {}

impl NullPort {
    fn new() -> Self {
        Self {}
    }
}

pub trait Port {
    fn write(&mut self, clock: &clocks::Clock, value: u8);
    fn read(&mut self, clock: &clocks::Clock) -> u8;
}

pub trait Device {
    fn poll_interrupts(&mut self, raw_display: &mut Vec<u8>, clock: &clocks::Clock) -> bool;
    fn port_write(&mut self, clock: &clocks::Clock, port_address: u8, value: u8);
    fn port_read(&mut self, clock: &clocks::Clock, port_address: u8) -> Option<u8>;
    fn export(&mut self, raw_display: &mut Vec<u8>) -> bool;
}

impl Port for NullPort {
    fn write(&mut self, _clock: &clocks::Clock, value: u8) {
        println!("null write value = {}", value);
    }

    fn read(&mut self, _clock: &clocks::Clock) -> u8 {
        0
    }
}

pub struct Ports {
    ports: Vec<Box<dyn Port>>,
    devices: Vec<Box<dyn Device>>,
    pub joysticks: inputs::Joystick,
    pub audio: sound::Sound,
}

impl Ports {
    const MAXPORTS: u16 = 256;
    pub fn new() -> Self {
        let mut new_ports: Vec<Box<dyn Port>> = Vec::new();
        for _i in 0..Ports::MAXPORTS {
            let new_port = NullPort::new();
            new_ports.push(Box::new(new_port));
        }
        Self {
            ports: new_ports,
            devices: Vec::new(),
            joysticks: inputs::Joystick::new(),
            audio: sound::Sound::new(),
        }
    }

    pub fn add_device(&mut self, device: Box<dyn Device>) {
        self.devices.push(device);
    }

    pub fn add_port(&mut self, port_address: u8, port: Box<dyn Port>) {
        self.ports[port_address as usize] = port;
    }

    pub fn port_read(&mut self, clock: &clocks::Clock, port_address: u8) -> u8 {
        for i in 0..self.devices.len() {
            if let Some(value) = self.devices[i].port_read(clock, port_address) {
                return value;
            };
        }

        if let Some(value) = self.joysticks.port_read(clock, port_address) {
            return value;
        };

        0
    }

    pub fn port_write(&mut self, clock: &clocks::Clock, port_address: u8, value: u8) {
        for i in 0..self.devices.len() {
            // TODO: Replace with something useful.
            self.devices[i].port_write(clock, port_address, value);
        }

        if port_address & 0xC0 == 0x40 {
            // 7E + 7F plus all of the pirror ports.
            self.audio.write_port(value);
        }
    }

    pub fn export(&mut self, raw_display: &mut Vec<u8>) -> bool {
        let mut result = false;
        for i in 0..self.devices.len() {
            result |= self.devices[i].export(raw_display);
        }
        result
    }

    pub fn poll_interrupts(&mut self, raw_display: &mut Vec<u8>, clock: &clocks::Clock) -> bool {
        let mut interrupt = false;
        for i in 0..self.devices.len() {
            interrupt |= self.devices[i].poll_interrupts(raw_display, clock);
        }

        interrupt
    }
}
