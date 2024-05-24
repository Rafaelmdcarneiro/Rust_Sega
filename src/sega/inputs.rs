use super::clocks;
use sdl2::event; // Keycode
use sdl2::keyboard; // Keycode

#[derive(Clone, Copy)]
pub struct Joystick {
    port1_value: u8,
    port2_value: u8,
    last_y: u8,
    lg1x: u8,
    lg1y: u8,
    lg2x: u8,
    lg2y: u8,
    x: u8,
}

impl Joystick {
    const PORT1_J1UP_BIT: u8 = (1 << 0);
    const PORT1_J1DOWN_BIT: u8 = (1 << 1);
    const PORT1_J1LEFT_BIT: u8 = (1 << 2);
    const PORT1_J1RIGHT_BIT: u8 = (1 << 3);
    const PORT1_J1FIREA_BIT: u8 = (1 << 4);
    const PORT1_J1FIREB_BIT: u8 = (1 << 5);
    const PORT1_J2UP_BIT: u8 = (1 << 6);
    const PORT1_J2DOWN_BIT: u8 = (1 << 7);
    const PORT2_J2LEFT_BIT: u8 = (1 << 0);
    const PORT2_J2RIGHT_BIT: u8 = (1 << 1);
    const PORT2_J2FIREA_BIT: u8 = (1 << 2);
    const PORT2_J2FIREB_BIT: u8 = (1 << 3);
    const PORT2_RESET_BIT: u8 = (1 << 4);
    const PORT2_UNUSED_BIT: u8 = (1 << 5);
    const PORT2_LG1_BIT: u8 = (1 << 6);
    const PORT2_LG2_BIT: u8 = (1 << 7);

    pub fn new() -> Self {
        Self {
            port1_value: 0xFF,
            port2_value: 0xFF,
            last_y: 0,
            lg1x: 0,
            lg1y: 0,
            lg2x: 0,
            lg2y: 0,
            x: 0,
        }
    }

    pub fn set_bit(initial: u8, mask: u8, value: bool) -> u8 {
        if value {
            initial | mask
        } else {
            initial & !mask
        }
    }

    pub fn set_y_pos(&mut self, y: u8) {
        if (y == self.lg2y) && (y != self.last_y) {
            self.lg2(false);
        } else if (y == self.lg1y) && (y != self.last_y) {
            self.lg1(false);
        } else {
            self.lg1(true);
            self.lg2(true);
        }

        self.last_y = y;
    }

    pub fn get_xp_pos(&self, vcounter: u8) -> u8 {
        self.x
    }
    pub fn read_port1(&self) -> u8 {
        self.port1_value
    }
    pub fn read_port2(&self) -> u8 {
        self.port2_value
    }

    pub fn j1_up(&mut self, value: bool) {
        self.port1_value = Joystick::set_bit(self.port1_value, Joystick::PORT1_J1UP_BIT, value);
    }
    pub fn j1_down(&mut self, value: bool) {
        self.port1_value = Joystick::set_bit(self.port1_value, Joystick::PORT1_J1DOWN_BIT, value);
    }
    pub fn j1_left(&mut self, value: bool) {
        self.port1_value = Joystick::set_bit(self.port1_value, Joystick::PORT1_J1LEFT_BIT, value);
    }
    pub fn j1_right(&mut self, value: bool) {
        self.port1_value = Joystick::set_bit(self.port1_value, Joystick::PORT1_J1RIGHT_BIT, value);
    }
    pub fn j1_fire_a(&mut self, value: bool) {
        self.port1_value = Joystick::set_bit(self.port1_value, Joystick::PORT1_J1FIREA_BIT, value);
    }
    pub fn j1_fire_b(&mut self, value: bool) {
        self.port1_value = Joystick::set_bit(self.port1_value, Joystick::PORT1_J1FIREB_BIT, value);
    }
    pub fn j2_up(&mut self, value: bool) {
        self.port1_value = Joystick::set_bit(self.port1_value, Joystick::PORT1_J2UP_BIT, value);
    }
    pub fn j2_down(&mut self, value: bool) {
        self.port1_value = Joystick::set_bit(self.port1_value, Joystick::PORT1_J2DOWN_BIT, value);
    }
    pub fn j2_left(&mut self, value: bool) {
        self.port2_value = Joystick::set_bit(self.port2_value, Joystick::PORT2_J2LEFT_BIT, value);
    }
    pub fn j2_right(&mut self, value: bool) {
        self.port2_value = Joystick::set_bit(self.port2_value, Joystick::PORT2_J2RIGHT_BIT, value);
    }
    pub fn j2_fire_a(&mut self, value: bool) {
        self.port2_value = Joystick::set_bit(self.port2_value, Joystick::PORT2_J2FIREA_BIT, value);
    }
    pub fn j2_fire_b(&mut self, value: bool) {
        self.port2_value = Joystick::set_bit(self.port2_value, Joystick::PORT2_J2FIREB_BIT, value);
    }
    pub fn reset(&mut self, value: bool) {
        self.port2_value = Joystick::set_bit(self.port2_value, Joystick::PORT2_RESET_BIT, value);
    }

    pub fn lg1(&mut self, value: bool) {
        if !value {
            self.x = self.lg1x;
        }

        self.port2_value = Joystick::set_bit(self.port2_value, Joystick::PORT2_LG1_BIT, value);
    }
    pub fn lg2(&mut self, value: bool) {
        if !value {
            self.x = self.lg2x;
        }

        self.port2_value = Joystick::set_bit(self.port2_value, Joystick::PORT2_LG2_BIT, value);
    }

    pub fn lg1pos(&mut self, x: u8, y: u8) {
        self.lg1x = x;
        self.lg1y = y;
    }

    pub fn lg2pos(&mut self, x: u8, y: u8) {
        self.lg2x = x;
        self.lg2y = y
    }

    pub fn port_read(&mut self, _clock: &clocks::Clock, port_address: u8) -> Option<u8> {
        match port_address {
            0xDC => Some(self.read_port1()),
            0xDD => Some(self.read_port2()),

            _ => {
                None /* Unhandled, just return 0 for now */
            }
        }
    }
}

pub struct Input {}

impl Input {
    const KEY_UP: keyboard::Keycode = keyboard::Keycode::Up;
    const KEY_DOWN: keyboard::Keycode = keyboard::Keycode::Down;
    const KEY_LEFT: keyboard::Keycode = keyboard::Keycode::Left;
    const KEY_RIGHT: keyboard::Keycode = keyboard::Keycode::Right;
    const KEY_FIRE_A: keyboard::Keycode = keyboard::Keycode::Z;
    const KEY_FIRE_B: keyboard::Keycode = keyboard::Keycode::X;
    const KEY_RESET: keyboard::Keycode = keyboard::Keycode::R;
    const KEY_QUIT: keyboard::Keycode = keyboard::Keycode::Escape;

    pub fn print_keys() {
        println!("Key mappings (Joystick 1):");
        println!(
            "Up: {}, Down: {}, Left: {}, Right: {}",
            Input::KEY_UP,
            Input::KEY_DOWN,
            Input::KEY_LEFT,
            Input::KEY_RIGHT
        );
        println!(
            "Fire A: {}, Fire B: {}",
            Input::KEY_FIRE_A,
            Input::KEY_FIRE_B
        );
        println!("Reset: {}", Input::KEY_RESET);
        println!();
        println!("Quit: {}", Input::KEY_QUIT);
    }

    // Return 'true' if handled, otherwise 'false' (ie quit)
    pub fn handle_events(event: event::Event, joystick: &mut Joystick) -> bool {
        match event {
            event::Event::Quit { .. }
            | event::Event::KeyDown {
                keycode: Some(Input::KEY_QUIT),
                ..
            } => return false,

            event::Event::KeyDown {
                keycode: Some(Input::KEY_UP),
                ..
            } => {
                joystick.j1_up(false);
            }
            event::Event::KeyDown {
                keycode: Some(Input::KEY_DOWN),
                ..
            } => {
                joystick.j1_down(false);
            }
            event::Event::KeyDown {
                keycode: Some(Input::KEY_LEFT),
                ..
            } => {
                joystick.j1_left(false);
            }
            event::Event::KeyDown {
                keycode: Some(Input::KEY_RIGHT),
                ..
            } => {
                joystick.j1_right(false);
            }
            event::Event::KeyDown {
                keycode: Some(Input::KEY_FIRE_A),
                ..
            } => {
                joystick.j1_fire_a(false);
            }
            event::Event::KeyDown {
                keycode: Some(Input::KEY_FIRE_B),
                ..
            } => {
                joystick.j1_fire_b(false);
            }
            event::Event::KeyDown {
                keycode: Some(Input::KEY_RESET),
                ..
            } => {
                joystick.reset(false);
            }

            event::Event::KeyUp {
                keycode: Some(Input::KEY_UP),
                ..
            } => {
                joystick.j1_up(true);
            }
            event::Event::KeyUp {
                keycode: Some(Input::KEY_DOWN),
                ..
            } => {
                joystick.j1_down(true);
            }
            event::Event::KeyUp {
                keycode: Some(Input::KEY_LEFT),
                ..
            } => {
                joystick.j1_left(true);
            }
            event::Event::KeyUp {
                keycode: Some(Input::KEY_RIGHT),
                ..
            } => {
                joystick.j1_right(true);
            }
            event::Event::KeyUp {
                keycode: Some(Input::KEY_FIRE_A),
                ..
            } => {
                joystick.j1_fire_a(true);
            }
            event::Event::KeyUp {
                keycode: Some(Input::KEY_FIRE_B),
                ..
            } => {
                joystick.j1_fire_b(true);
            }
            event::Event::KeyUp {
                keycode: Some(Input::KEY_RESET),
                ..
            } => {
                joystick.reset(true);
            }

            _ => return true,
        }

        true
    }
}
