use super::cartridge;

///  Map the current 'pc' address to an 'absolute' address.  The
/// structure of the 'absolute' address is somewhat arbitrary, but  the
/// idea is to be fairly sensible with the layout to make the mapping
/// sensible and efficient.
///
/// segments:
///
/// cartridge - ROM - 0x4000 * 64 (0x3F)
/// cartridge - RAM - 0x4000 * 2
/// system    - RAM - 0x2000
///
///
/// ROM - 0x000000 - 0x0FFFFF
/// ROM - 0x100000 - 0x1FFFFF
/// RAM   0x200000 - 0x207FFF
/// RAM   0x208000 - 0x20A000
///
/// -> Total = 3F 0x42
///
/// mapped memory:
///     0x0000 - 0x03FF                     -> ROM (bank 0) (0x0000 - 0x03FF)
///     0x0400 - 0x3FFF + (0xFFFD(0-0x3F))          -> ROM (bank x) (0x0400 - 0x4000)
///     0x4000 - 0x7FFF + (0xFFFE(0-0x3F))          -> ROM (bank x) (0x0000 - 0x4000)
///     0x8000 - 0xBFFF + (0xFFFF(0-0x3F) + 0xFFFC(0x0C)) -> ROM (bank x) (0x0000 - 0x4000) or RAM (bank x)
/// - 11x0
///     0xC000 - 0xDFFF                     -> System RAM   (0x0000 - 0x2000)
///     0xE000 - 0xFFFF                     -> System RAM   (0x0000 - 0x2000) (mirror)
///
///
///     0x0000 - 0x03FF                     -> ROM (bank 0) (0x0000 - 0x03FF)
///     0x0400 - 0x3FFF + (0xFFFD)          -> ROM (bank x) (0x0400 - 0x4000)
///
///     0x4000 - 0x7FFF + (0xFFFE)          -> ROM (bank x) (0x0000 - 0x4000)
///     0x8000 - 0xBFFF + (0xFFFF + 0xFFFC) -> ROM (bank x) (0x0000 - 0x4000) or RAM (bank x)
///
///     0xC000 - 0xDFFF                     -> System RAM   (0x0000 - 0x2000)
///     0xE000 - 0xFFFF                     -> System RAM   (0x0000 - 0x2000) (mirror)
///
///
///     absolute = page[(address >> 13)] | address & 0x1FFF
///
///
///     | 6-bit - page 0| 6-bit - page 1| 6-bit - page 2 | 1-bit - ram/rom select | 1-bit - ram select | 3 - sub-page address | 13 - address
///
///     all bits -> 6 + 6 + 6 + 18
///

pub struct MemoryAbsoluteConstants {}

pub struct MemoryBase {}

pub struct MemoryAbsolute {
    page_2: u8,
    ram_select: u8,

    upper_mappings: Vec<AbsoluteAddressType>,

    // Complete memory map
    memory_map: Vec<u8>,
}

impl MemoryBase {
    const MEMREGISTERS: AddressType = 0xFFFC;
    const ADDRESS_MASK: AddressType = 0xFFFF;

    const RAM_SELECT_REGISTER: AddressType = 0xFFFC;
    const PAGE0_BANK_SELECT_REGISTER: AddressType = 0xFFFD;
    const PAGE1_BANK_SELECT_REGISTER: AddressType = 0xFFFE;
    const PAGE2_BANK_SELECT_REGISTER: AddressType = 0xFFFF;

    const MAPCARTRAM: u8 = 0x08;
    const PAGEOFRAM: u8 = 0x04;

    // Memory map offsets
    const PAGE0: u16 = 0x400; // 0 to Page0 offset always holds bank 0
    const PAGE1: u16 = 0x4000;
    const RAM_OFFSET: u16 = 0xC000;

    const LOWERMASK: AddressType = 0x03FFF;

    const BANK_SIZE: BankSizeType = 0x4000;
}

type BankSizeType = u16;
type NumBanksType = u8;
pub type AddressType = u16;
type AbsoluteAddressType = u32;

//const fn max<T: ~const PartialOrd + Copy>(a: T, b: T) -> T {
//    [a, b][(a < b) as usize]
//}
const fn max(a: AbsoluteAddressType, b: AbsoluteAddressType) -> AbsoluteAddressType {
    [a, b][(a < b) as usize]
}

impl MemoryAbsoluteConstants {
    const ABSOLUTE_PAGE_0_ROM_OFFSET: AbsoluteAddressType = 0x000000;
    const ABSOLUTE_PAGE_X_ROM_OFFSET: AbsoluteAddressType = 0x100000;
    const ABSOLUTE_CART_RAM_OFFSET: AbsoluteAddressType = 0x200000;
    const ABSOLUTE_SYS_RAM_OFFSET: AbsoluteAddressType = 0x208000;
    const ABSOLUTE_SEGMENT_SIZE: AbsoluteAddressType = 0x2000;
}

impl MemoryAbsolute {
    pub fn new() -> Self {
        // TODO: Improve auto increment the offsets
        let mut index = 0;
        fn new_segment(index: &mut u32) -> u32 {
            *index += 1;
            MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET
                + (MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE * *index)
        }

        Self {
            upper_mappings: vec![
                MemoryAbsoluteConstants::ABSOLUTE_PAGE_0_ROM_OFFSET,
                new_segment(&mut index),
                new_segment(&mut index),
                new_segment(&mut index),
                new_segment(&mut index),
                new_segment(&mut index),
                MemoryAbsoluteConstants::ABSOLUTE_SYS_RAM_OFFSET,
                MemoryAbsoluteConstants::ABSOLUTE_SYS_RAM_OFFSET,
            ],

            // Complete memory map
            memory_map: vec![
                0;
                max(
                    MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET
                        + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE,
                    MemoryAbsoluteConstants::ABSOLUTE_SYS_RAM_OFFSET
                        + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE
                ) as usize
            ],
            page_2: 0,
            ram_select: 0,
        }
    }

    pub fn get_absolute_address(&self, address: AddressType) -> AbsoluteAddressType {
        self.upper_mappings[(address >> 13) as usize] | (address & 0x1FFF) as AbsoluteAddressType
    }

    pub fn read(&self, address: AddressType) -> u8 {
        self.memory_map[(self.upper_mappings[(address >> 13) as usize]
            | (address & 0x1FFF) as AbsoluteAddressType) as usize]
    }

    pub fn reset(&mut self, cartridge_name: &str) {
        let mut cartridge = cartridge::Cartridge::new(cartridge_name);
        match cartridge.load() {
            Ok(()) => {
                println!("Ok");
            }
            _ => {
                println!("Error loading cartridge.");
            }
        }

        self.initialise_read(cartridge);
    }

    pub fn write(&mut self, address: AddressType, data: u8) {
        // TODO: Should check inputs, see which instructions/condition can overflow
        self.private_write(address, data)
    }

    fn initialise_read(&mut self, cartridge: cartridge::Cartridge) {
        // Un-optimised address translation, uses paging registers.

        self.populate_absolute_memory_map(cartridge);
        self.write(0xFFFC, 0);
        self.write(0xFFFD, 0);
        self.write(0xFFFE, 1);
        self.write(0xFFFF, 2);
    }

    fn populate_absolute_memory_map(&mut self, mut cartridge: cartridge::Cartridge) {
        for bank in 0..cartridge.num_banks as NumBanksType {
            // Page '0'/'1' lookup
            for address in 0..MemoryBase::PAGE0 as BankSizeType {
                let bank_address = address & MemoryBase::LOWERMASK; // BANK_MASK
                let bank_offset = (bank_address as AbsoluteAddressType)
                    + (bank as AbsoluteAddressType)
                        * (MemoryBase::BANK_SIZE as AbsoluteAddressType);
                self.memory_map[(MemoryAbsoluteConstants::ABSOLUTE_PAGE_0_ROM_OFFSET + bank_offset)
                    as usize] = cartridge.read(0, bank_address);
                self.memory_map[(MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET + bank_offset)
                    as usize] = cartridge.read(bank, bank_address);
            }

            for address in MemoryBase::PAGE0..MemoryBase::PAGE1 as BankSizeType {
                let bank_address = address & MemoryBase::LOWERMASK; // BANK_MASK
                let bank_offset = (bank_address as AbsoluteAddressType)
                    + (bank as AbsoluteAddressType)
                        * (MemoryBase::BANK_SIZE as AbsoluteAddressType);
                self.memory_map[(MemoryAbsoluteConstants::ABSOLUTE_PAGE_0_ROM_OFFSET + bank_offset)
                    as usize] = cartridge.read(bank, bank_address);
                self.memory_map[(MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET + bank_offset)
                    as usize] = cartridge.read(bank, bank_address);
            }
        }
    }

    fn private_write(&mut self, address: AddressType, data: u8) {
        let address = address & MemoryBase::ADDRESS_MASK; // ADDRESS_MASK;

        if address >= MemoryBase::RAM_OFFSET && address >= MemoryBase::MEMREGISTERS {
            // Should make these conditiona,
            if address == MemoryBase::PAGE0_BANK_SELECT_REGISTER {
                self.upper_mappings[0] = MemoryAbsoluteConstants::ABSOLUTE_PAGE_0_ROM_OFFSET
                    + (MemoryBase::BANK_SIZE as AbsoluteAddressType * data as AbsoluteAddressType);
                self.upper_mappings[1] = MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET
                    + (MemoryBase::BANK_SIZE as AbsoluteAddressType * data as AbsoluteAddressType)
                    + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE;
            } else if address == MemoryBase::PAGE1_BANK_SELECT_REGISTER {
                self.upper_mappings[2] = MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET
                    + (MemoryBase::BANK_SIZE as AbsoluteAddressType * data as AbsoluteAddressType);
                self.upper_mappings[3] = MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET
                    + (MemoryBase::BANK_SIZE as AbsoluteAddressType * data as AbsoluteAddressType)
                    + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE;
            } else if (address == MemoryBase::RAM_SELECT_REGISTER)
                || (address == MemoryBase::PAGE2_BANK_SELECT_REGISTER)
            {
                if address == MemoryBase::RAM_SELECT_REGISTER {
                    self.ram_select = data;
                } else if address == MemoryBase::PAGE2_BANK_SELECT_REGISTER {
                    self.page_2 = data;
                }

                if 0 != self.ram_select & MemoryBase::MAPCARTRAM {
                    // page2_is_cartridge_ram
                    // Cart RAM select.
                    if 0 != self.ram_select & MemoryBase::PAGEOFRAM {
                        self.upper_mappings[4] = MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET
                            + (MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE * 2);
                        self.upper_mappings[5] = MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET
                            + (MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE * 3);
                    } else {
                        self.upper_mappings[4] = MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET;
                        self.upper_mappings[5] = MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET
                            + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE;
                    }
                } else {
                    self.upper_mappings[4] = MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET
                        + (MemoryBase::BANK_SIZE as AbsoluteAddressType
                            * self.page_2 as AbsoluteAddressType);
                    self.upper_mappings[5] = MemoryAbsoluteConstants::ABSOLUTE_PAGE_X_ROM_OFFSET
                        + (MemoryBase::BANK_SIZE as AbsoluteAddressType
                            * self.page_2 as AbsoluteAddressType)
                        + MemoryAbsoluteConstants::ABSOLUTE_SEGMENT_SIZE;
                }
            }
        }
        let absolute_address = self.get_absolute_address(address);
        if absolute_address
            >= std::cmp::min(
                MemoryAbsoluteConstants::ABSOLUTE_CART_RAM_OFFSET,
                MemoryAbsoluteConstants::ABSOLUTE_SYS_RAM_OFFSET,
            )
        {
            self.memory_map[absolute_address as usize] = data;
        }
    }
}

// Common macro to help export the read/write rules
#[macro_export]
macro_rules! impl_common_memoryrw {
    ($T:ident) => {
        impl $crate::sega::memory::memory::MemoryRW for $T {
            fn read(&self, address: $crate::sega::memory::memory::AddressType) -> u8 {
                self.read(address)
            }

            // Also create a 'little endian' 16-bit read.
            fn read16(&self, address: $crate::sega::memory::memory::AddressType) -> u16 {
                self.read(address) as u16 + ((self.read(address + 1) as u16) << 8)
            }

            fn write(
                &mut self,
                address: $crate::sega::memory::memory::AddressType,
                data: u8,
            ) -> () {
                self.write(address, data);
            }
        }
    };
}

pub(crate) use impl_common_memoryrw;

impl_common_memoryrw!(MemoryAbsolute);

pub trait MemoryRW {
    fn read(&self, address: AddressType) -> u8;
    fn read16(&self, address: AddressType) -> u16;
    fn write(&mut self, address: AddressType, data: u8);
}

#[cfg(test)]
mod tests {
    use crate::sega::memory::memory::MemoryAbsolute;
    use std::mem;
    #[test]
    fn test_simple_memory_check() {
        let memory = MemoryAbsolute::new();

        println!("Memory length: {}", memory.memory_map.len());
        println!("Memory size: {}", mem::size_of_val(&memory));
        println!("memory_map: {}", mem::size_of_val(&memory.memory_map));
        println!(
            "upper_mappings: {}",
            mem::size_of_val(&memory.upper_mappings)
        );
    }
}
