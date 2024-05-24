type BankSizeType = u16;
type NumBanksType = u8;

const BANK_SIZE: BankSizeType = 0x4000;
const MAX_BANKS: NumBanksType = 64;

#[derive(Copy, Clone)]
struct Bank {
    data: [u8; BANK_SIZE as usize],
}

pub struct Cartridge {
    filename: String,
    pub num_banks: NumBanksType,
    rom: Box<[Bank; MAX_BANKS as usize]>,
}

fn print(cartridge: &Cartridge) {
    println!("read: {}", cartridge.filename);
    println!("Num banks: {}", cartridge.num_banks);
}

impl Cartridge {
    pub fn new(filename: &str) -> Self {
        Self {
            filename: filename.to_string(),
            num_banks: 0,
            rom: Box::new(
                [Bank {
                    data: [0; BANK_SIZE as usize],
                }; MAX_BANKS as usize],
            ),
        }
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        let mut buffer = Vec::new();

        #[cfg(not(target_os = "emscripten"))]
        {
            use std::fs::File;
            use std::io::Read;

            let mut file = File::open(&self.filename)?;
            file.read_to_end(&mut buffer)?;
        }

        #[cfg(target_os = "emscripten")]
        {
            JAVASCRIPT_DATA_STORE.with(|ref_cell_data| {
                buffer = ref_cell_data.borrow().raw_cart_data.clone();
            });
        }

        self.load_banks(&mut buffer);

        print(self);

        Ok(())
    }

    fn load_banks(&mut self, source: &mut Vec<u8>) {
        self.rom = Box::new(
            [Bank {
                data: [0; BANK_SIZE as usize],
            }; MAX_BANKS as usize],
        );

        for i in 0..MAX_BANKS {
            let (bank, n) = load_bank(source);

            self.rom[i as usize] = bank;
            source.drain(0..n as usize);
            self.num_banks += 1
        }
    }

    pub fn read(&mut self, bank: NumBanksType, bank_address: BankSizeType) -> u8 {
        self.rom[bank as usize].data[bank_address as usize]
    }
}

fn load_bank(source: &mut [u8]) -> (Bank, BankSizeType) {
    let mut bank = Bank {
        data: [0; BANK_SIZE as usize],
    };

    // Try to read an entire bank.
    if source.len() >= BANK_SIZE as usize {
        bank.data = source[0..BANK_SIZE as usize].try_into().unwrap();
        (bank, BANK_SIZE as BankSizeType)
    } else {
        let length = source.len();
        if length > 0 {
            bank.data = source[0..length].try_into().unwrap();
        }
        (bank, source.len() as BankSizeType)
    }
}

struct JavaScriptData {
    pub raw_cart_data: Vec<u8>,
}
impl JavaScriptData {
    pub fn new() -> Self {
        Self {
            raw_cart_data: Vec::new(),
        }
    }
}

use std::cell::RefCell;

thread_local! {
    static JAVASCRIPT_DATA_STORE: RefCell<JavaScriptData> = RefCell::new(JavaScriptData::new());
}

pub fn is_cart_ready() -> bool {
    let mut is_ready = false;
    JAVASCRIPT_DATA_STORE.with(|ref_cell_data| {
        is_ready = !ref_cell_data.borrow().raw_cart_data.is_empty();
    });
    is_ready
}

#[no_mangle]
pub extern "C" fn display_data(raw_data_ptr: *const u8, raw_data_length: usize) {
    // TODO: Although it's possible there's another way (alternate arguments), I'll just use the unsafe option for now.
    let v = unsafe { std::slice::from_raw_parts(raw_data_ptr, raw_data_length) };
    if !v.is_empty() {
        JAVASCRIPT_DATA_STORE
            .with(|ref_cell_data| ref_cell_data.borrow_mut().raw_cart_data = v.to_vec());
    }
}

#[cfg(test)]
mod tests {
    use crate::sega::memory::cartridge::Cartridge;
    use std::mem;

    #[test]
    fn test_load_rom() {
        // Do a test load of a 'fake rom' (just randomly generated data).
        let test_rom = "fake.rom";
        let mut cartridge = Cartridge::new(test_rom);
        match cartridge.load() {
            Ok(()) => {
                println!("Ok");
            }
            _ => {
                println!("Error loading cartridge.");
            }
        }
        assert_eq!(cartridge.read(0, 0), 139);
        println!("{}", mem::size_of_val(&cartridge));
    }
}
