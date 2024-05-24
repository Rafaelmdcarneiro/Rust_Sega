use super::pc_state;

pub fn signed_char_to_int(v: i8) -> i16 {
    v as i16
}

// Calculate the parity.  Check if the the number of bits is odd or even.
// Even number of '1's -> return 'true' (P=1)
// Odd number of '1's -> return 'false' (P=0)
pub fn calculate_parity(a: u8) -> bool {
    // Do parity half at a time
    let mut h = (a >> 4) ^ (a & 0xF);
    h = (h >> 2) ^ (h & 0x3);
    (h >> 1) == (h & 0x1) // Even parity
}

macro_rules! zero_flag {
    ($a:expr) => {
        if ($a == 0) {
            1
        } else {
            0
        }
    };
}

macro_rules! sign_flag {
    ($a:expr, $type:ty) => {
        (($a >> (8 * std::mem::size_of::<$type>() - 1)) & 0x1) as u8
    };
}

// An Overflow can't occur if a and b have different sign bits
// If they're the same, an overflow occurred if the sign of the result changed.
// Basically, tread both arguments as signed numbers
// if (((a & 0x8000) ^ (b & 0x8000)) == 0x0000) && // arguments same sign
//    (((a & 0x8000) ^ (r & 0x8000)) == 0x8000)
macro_rules! add_overflow_flag {
    ($a:expr, $b:expr, $r:expr, $type:ty) => {
        (((($a >> (8 * std::mem::size_of::<$type>() - 1)) & 0x1)
            ^ (($b >> (8 * std::mem::size_of::<$type>() - 1)) & 0x1))
            == 0
            && ((($a >> (8 * std::mem::size_of::<$type>() - 1)) & 0x1)
                ^ (($r >> (8 * std::mem::size_of::<$type>() - 1)) & 0x1)
                == 0x1)) as u8
    };
}

macro_rules! sub_overflow_flag {
    ($a:expr, $b:expr, $r:expr, $type:ty) => {
        add_overflow_flag!($a, !$b, $r, $type)
    };
}

// Calculate 'carry' flags, given two unsigned integers and the 'mask' to check for overflow.
// eg  u8: 'max=0xFF' is full carry, 'max=0xF' is 'half carry
//    u16: 'max=0xFFFF' is full carry, 'max=0xFFF' is 'half carry
macro_rules! calculate_ucarry {
    ($a:expr, $b:expr, $max:expr) => {
        ($max & $a) > ($max & !$b)
    };
    ($a:expr, $b:expr, $c:expr, $max:expr) => {
        if $c {
            ($max & $a) >= ($max & !$b)
        } else {
            ($max & $a) > ($max & !$b)
        }
    };
}

macro_rules! calculate_borrow_carry {
    ($a:expr, $b:expr, $c:expr, $mask:expr) => {
        calculate_ucarry!(!$a, $b, $c, $mask)
    };
}

// self.pc_state.Add two 8 bit ints plus the carry bit, and set flags accordingly
pub fn u8_carry(a: u8, b: u8, c: bool, f_status: &mut pc_state::PcStatusFlagFields) -> u8 {
    let r = a.wrapping_add(b).wrapping_add(u8::from(c));

    f_status.set_pv(add_overflow_flag!(a, b, r, u8));
    f_status.set_h(calculate_ucarry!(a, b, c, 0xF) as u8);
    f_status.set_c(calculate_ucarry!(a, b, c, u8::MAX) as u8);

    zero_and_sign_flags(f_status, r);

    r
}
pub fn i8_carry(a: u8, b: u8, c: bool, f_status: &mut pc_state::PcStatusFlagFields) -> u8 {
    let r = a.wrapping_sub(b).wrapping_sub(c as u8);

    f_status.set_h(calculate_borrow_carry!(a, b, c, 0xF) as u8);
    // overflow
    f_status.set_pv(sub_overflow_flag!(a, b, r, u8));
    f_status.set_n(1);
    f_status.set_c(calculate_borrow_carry!(a, b, c, 0xFF) as u8);
    f_status.set_s(sign_flag!(r, u8));
    f_status.set_z(zero_flag!(r));

    r
}

// calculate the 'sub c' flags (although carry isn't used), this matches a
// previous implementation (to make comparisons easier).
// TODO: Once other issues are sorted out, revisit setting of these flags.
pub fn i8_no_carry(a: u8, b: u8, f_status: &mut pc_state::PcStatusFlagFields) -> u8 {
    i8_carry(a, b, false, f_status)
}

pub fn u16_carry(a: u16, b: u16, c: bool, f_status: &mut pc_state::PcStatusFlagFields) -> u16 {
    // Perform a u16-bit add with carry, setting the flags (except N, which is
    // left to add/sub)
    let r = a.wrapping_add(b).wrapping_add(u16::from(c));

    f_status.set_s(sign_flag!(r, u16));
    f_status.set_z(zero_flag!(r));
    f_status.set_pv(add_overflow_flag!(a, b, r, u16));
    f_status.set_h(calculate_ucarry!(a, b, c, 0xFFF) as u8);
    f_status.set_c(calculate_ucarry!(a, b, c, 0xFFFF) as u8);

    r
}

pub fn u16_no_carry(a: u16, b: u16, f_status: &mut pc_state::PcStatusFlagFields) -> u16 {
    // Perform a u16-bit add with carry, setting the flags (except N, which is
    // left to add/sub)
    let r = a.wrapping_add(b);

    f_status.set_h(calculate_ucarry!(a, b, false, 0xFFF) as u8);
    f_status.set_c(calculate_ucarry!(a, b, false, 0xFFFF) as u8);

    r
}

pub fn i16_carry(a: u16, b: u16, c: bool, f_status: &mut pc_state::PcStatusFlagFields) -> u16 {
    let r = a.wrapping_sub(b).wrapping_sub(c as u16);

    f_status.set_h(calculate_borrow_carry!(a, b, c, 0xFFF) as u8);
    // overflow
    f_status.set_pv(sub_overflow_flag!(a, b, r, u16));
    f_status.set_n(1);
    f_status.set_c(calculate_borrow_carry!(a, b, c, 0xFFFF) as u8);
    f_status.set_s(sign_flag!(r, u16));
    f_status.set_z(zero_flag!(r));

    r
}

pub fn calculate_dec_flags(status: &mut pc_state::PcStatusFlagFields, new_value: u8) {
    status.set_n(1);

    if (new_value & 0xF) == 0xF {
        // Half borrow
        status.set_h(1);
    } else {
        status.set_h(0);
    }

    if new_value == 0x7F {
        // Was 80
        status.set_pv(1);
    } else {
        status.set_pv(0);
    }

    status.set_n(1);
    zero_and_sign_flags(status, new_value);
}

pub fn calculate_inc_flags(status: &mut pc_state::PcStatusFlagFields, new_value: u8) {
    if (new_value & 0xF) == 0x0 {
        // Half borrow
        status.set_h(1);
    } else {
        status.set_h(0);
    }

    if new_value == 0x80 {
        // Was 0x7F
        status.set_pv(1);
    } else {
        status.set_pv(0);
    }

    status.set_n(0);
    zero_and_sign_flags(status, new_value);
}

pub fn accumulator_flags(status: &mut pc_state::PcStatusFlagFields, accumulator: u8, iff2: bool) {
    // Used by LD A, I; LD A, R
    status.set_n(0);
    status.set_h(0);
    status.set_pv(iff2 as u8);
    zero_and_sign_flags(status, accumulator)
}

pub fn and_flags(status: &mut pc_state::PcStatusFlagFields, value: u8) {
    // Used by AND s
    status.set_c(0);
    status.set_n(0);
    status.set_h(1);
    status.set_pv(calculate_parity(value) as u8); // Documented as 'set on overflow', not sure what it should be here
    zero_and_sign_flags(status, value)
}

pub fn xor_flags(status: &mut pc_state::PcStatusFlagFields, value: u8) {
    // Used by AND s
    status.set_c(0);
    status.set_n(0);
    status.set_h(0);
    status.set_pv(calculate_parity(value) as u8); // Documented as set on even for xor
    zero_and_sign_flags(status, value)
}

// The 'new' value and carry
pub fn set_rotate_accumulator_flags(carry: bool, status: &mut pc_state::PcStatusFlagFields) {
    status.set_c(carry as u8);
    status.set_h(0);
    status.set_n(0);
}

// The 'new' value and carry.  The flags set for rotating accumulator vs registers differ.
pub fn set_shift_register_flags(value: u8, carry: bool, status: &mut pc_state::PcStatusFlagFields) {
    status.set_c(carry as u8);
    status.set_n(0);
    status.set_h(0);
    status.set_pv(calculate_parity(value) as u8); // Documented as set on even for xor
    zero_and_sign_flags(status, value)
}

pub fn rotate_decimal_flags(status: &mut pc_state::PcStatusFlagFields, value: u8) {
    // Carry not affected
    status.set_n(0);
    status.set_h(0);
    status.set_pv(calculate_parity(value) as u8);
    zero_and_sign_flags(status, value)
}

pub fn zero_and_sign_flags(status: &mut pc_state::PcStatusFlagFields, value: u8) {
    // Utility function, to set the zero and sign flags
    status.set_s(sign_flag!(value, u8));
    status.set_z(zero_flag!(value));
}

pub fn or_flags(status: &mut pc_state::PcStatusFlagFields, value: u8) {
    xor_flags(status, value);
}

// Add two 8 bit ints plus the carry bit, and set flags accordingly
pub fn set_bit_test_flags(r: u8, bit_pos: u8, f_status: &mut pc_state::PcStatusFlagFields) {
    let bit = (r >> (bit_pos & 7)) & 0x1;
    f_status.set_z(bit ^ 0x1);
    f_status.set_pv(calculate_parity(bit) as u8); // Documented as 'unknown', not sure if/where this is needed.
    f_status.set_h(1);
    f_status.set_n(0);
    f_status.set_s(0);
}

#[cfg(test)]
mod tests {
    use crate::sega::cpu::pc_state;
    use crate::sega::cpu::status_flags;

    #[test]
    fn test_carry_flag() {
        // For 'add'
        let test_values = [
            ((0xFF, 0x01, false), (0x00, true)),
            ((0xFF, 0x01, true), (0x01, true)),
            ((0xFF, 0x00, false), (0xFF, false)),
            ((0xFF, 0x00, true), (0x00, true)),
            ((0x7F, 0x7F, true), (0xFF, false)),
            ((0x0F, 0x00, true), (0x10, false)),
            ((0x0F, 0x00, false), (0x0F, false)),
        ];

        for ((a, b, initial_carry), (expected_value, expected_carry)) in test_values {
            let mut f_status = pc_state::PcStatusFlagFields(0);
            f_status.set_c(0);
            assert_eq!(
                status_flags::u8_carry(a, b, initial_carry, &mut f_status),
                expected_value
            );
            assert_eq!(f_status.get_c(), expected_carry as u8);

            // Order shouldn't matter.
            f_status.set_c(0);
            assert_eq!(
                status_flags::u8_carry(b, a, initial_carry, &mut f_status),
                expected_value
            );
            assert_eq!(f_status.get_c(), expected_carry as u8);

            assert_eq!(calculate_ucarry!(a, b, initial_carry, 0xFF), expected_carry);
            assert_eq!(calculate_ucarry!(b, a, initial_carry, 0xFF), expected_carry);
        }
    }

    #[test]
    fn test_icarry_flag() {
        //                  (a,b,c),            (pv, c, h, n);
        let test_values = [
            ((0x00, 0x00, false), (0, 0, 0, 1)),
            ((0x00, 0x00, true), (0, 1, 1, 1)),
            ((0x01, 0x00, true), (0, 0, 0, 1)),
            ((0x0F, 0x10, false), (0, 1, 0, 1)),
            ((0x10, 0x0F, false), (0, 0, 1, 1)),
            ((0x70, 0xAF, false), (1, 1, 1, 1)),
            ((0xAF, 0x70, false), (1, 0, 0, 1)),
            ((0xFF, 0xFF, false), (0, 0, 0, 1)),
            ((0xFF, 0xFF, true), (0, 1, 1, 1)),
        ];

        let mut f_status = pc_state::PcStatusFlagFields(0);

        for ((a, b, carry), (pv, c, h, n)) in test_values {
            status_flags::i8_carry(a, b, carry, &mut f_status);
            assert_eq!(f_status.get_pv(), pv, "a:{} b:{} c:{}", a, b, c);
            assert_eq!(f_status.get_c(), c, "a:{} b:{} c:{}", a, b, c);
            assert_eq!(f_status.get_h(), h, "a:{} b:{} c:{}", a, b, c);
            assert_eq!(f_status.get_n(), n, "a:{} b:{} c:{}", a, b, c);
        }
    }

    #[test]
    fn test_half_carry_flag() {
        // For 'add'
        let test_values = [
            ((0xFF, 0x01, false), (0x00, true)),
            ((0xFF, 0x01, true), (0x01, true)),
            ((0xFF, 0x00, false), (0xFF, false)),
            ((0xFF, 0x00, true), (0x00, true)),
            ((0x7F, 0x7F, true), (0xFF, true)),
            ((0x0F, 0x00, true), (0x10, true)),
            ((0x0F, 0x00, false), (0x0F, false)),
        ];

        for ((a, b, initial_carry), (expected_value, expected_carry)) in test_values {
            let mut f_status = pc_state::PcStatusFlagFields(0);
            f_status.set_h(0);
            assert_eq!(
                status_flags::u8_carry(a, b, initial_carry, &mut f_status),
                expected_value
            );
            assert_eq!(f_status.get_h(), expected_carry as u8);

            // Order shouldn't matter.
            f_status.set_h(0);
            assert_eq!(
                status_flags::u8_carry(b, a, initial_carry, &mut f_status),
                expected_value
            );
            assert_eq!(f_status.get_h(), expected_carry as u8);
        }
    }

    #[test]
    fn test_half_carry_u16_flag() {
        // For 'add'
        let test_values = [
            ((0xFFFF, 0x01FF, false), true),
            ((0xFFFF, 0x01FF, true), true),
            ((0xFFFF, 0x0000, false), false),
            ((0xFFFF, 0x0000, true), true),
            ((0x7FFF, 0x7FFF, true), true),
            ((0x0FFF, 0x00FF, true), true),
            ((0x0FFF, 0x00FF, false), true),
            ((0xFF00, 0xF0FF, false), false),
            ((0xFF00, 0xF0FF, true), true),
        ];

        for ((a, b, initial_carry), expected_carry) in test_values {
            assert_eq!(
                calculate_ucarry!(a, b, initial_carry, 0xFFF),
                expected_carry
            );
            assert_eq!(
                calculate_ucarry!(b, a, initial_carry, 0xFFF),
                expected_carry
            );
        }
    }

    #[test]
    fn test_parity() {
        assert_eq!(status_flags::calculate_parity(0b11001001), true);
        assert_eq!(status_flags::calculate_parity(0b00101000), true);
        assert_eq!(status_flags::calculate_parity(0b10101001), true);
        assert_eq!(status_flags::calculate_parity(0b00101001), false);
        assert_eq!(status_flags::calculate_parity(0b00000001), false);
        assert_eq!(status_flags::calculate_parity(0b10000000), false);
    }
    #[test]
    fn test_bit_set() {
        let mut f_status = pc_state::PcStatusFlagFields(0);
        status_flags::set_bit_test_flags(0x30, 5, &mut f_status);
        assert_eq!(f_status.get_z(), 0);
        status_flags::set_bit_test_flags(0x30, 3, &mut f_status);
        assert_eq!(f_status.get_z(), 1);
    }
}
