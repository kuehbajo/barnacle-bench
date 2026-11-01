/*
##################
    ammunition_limits.h
##################
*/

pub const CHAR_BIT: u8 = 8;
pub const UCHAR_MAX: u8 = 255;
pub const UINT_MAX: u32 = INT_MAX as u32 * 2 + 1;
pub const INT_MAX: i32 = 2147483647;
pub const INT_MIN: i32 = -INT_MAX-1;

/*
##################
    bits.h
##################
*/

/*

  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license (GPL 2) as the original main file.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

  FILE NAME:   bits.h

  TITLE:       Include file of package for work with bits

  DESCRIPTION:
    This is header file contains macros and the ANSI C prototype
    definitions for the package for work with bits and bit strings
    and C++ class for work with bits and bit strings.  A bit is
    given by address (start address) of byte from which counting
    bits starts and its displacement which is any non negative
    number of bit from the start address.  The most significant bit
    of the start address byte has number 0.  The bit string is
    given by its first bit and its length in bits.

*/

/* This macro value returns bit vlaue (0 or 1) with given bit
   displacement (0, 1, ...).  The macro has side effects!  Value of
   `bit_displacement' must be nonegative and can be greater than
   CHAR_BIT. */

#[inline]
pub fn bit(start_byte: &[u8], bit_displacement: usize) -> u8 {
    let byte_i = bit_displacement / CHAR_BIT as usize;
    let bit_i = CHAR_BIT - 1 - bit_displacement as u8 % CHAR_BIT;
    (start_byte[byte_i] >> bit_i) & 1
}

/* This macro value sets up new value (must be `0' or `1') of a given
   bit (bit displacement starts with 0).  The macro has side effects!
   Value of `bit_displacement' must be nonegative and can be greater
   than CHAR_BIT. */

#[inline]
pub fn set_bit(start_byte: &mut [u8], bit_displacement: usize, bit: u8) {
    let byte_i = bit_displacement / CHAR_BIT as usize;
    let bit_i = CHAR_BIT - 1 - bit_displacement as u8 % CHAR_BIT;
    let and_val = !(1 << bit_i);
    let or_val = bit << bit_i;
    start_byte[byte_i] = (start_byte[byte_i] & and_val) | or_val;
}

/*
##################
    bits.c
##################
*/
                            
/*

  FILE NAME:   bits.c

  TITLE:       Package for work with bits

  DESCRIPTION: This file implements functions of the package for work
      with bit strings.  A bit is given by address (start address) of
      byte from which counting bits starts and its displacement which
      is any non negative number of bit from the start address.  The
      most significant bit of the start address byte has number 0.
      The bit string is given by its first bit and its length in
      bits.

*/

/* This function determines that given bit string contains only zero
   bits.  The function retruns TRUE if all bits of given bit string
   are zero or `bit_length' <= 0.  Value of `bit_displacement' must be
   non-negative and can be greater than CHAR_BIT. */

pub fn ammunition_is_zero_bit_string(
    start_byte: &mut [u8],
    bit_displacement: usize,
    bit_length: usize,
) -> bool {
    let mut current_byte_i: usize = 0;
    let mut bit_length = bit_length as isize;
    if bit_length <= 0 {
        return true;
    }
    current_byte_i += bit_displacement / CHAR_BIT as usize;
    let bit_displacement = bit_displacement as isize % CHAR_BIT as isize;
    if bit_length < CHAR_BIT as isize - bit_displacement {
        return ((start_byte[current_byte_i] << bit_displacement)
            & (UCHAR_MAX << (CHAR_BIT - bit_length as u8))
            & UCHAR_MAX) == 0;
    } else if bit_displacement != 0 {
        if ((start_byte[current_byte_i] << bit_displacement) & UCHAR_MAX) != 0 {
            return false;
        }
        current_byte_i += 1;
        bit_length -= CHAR_BIT as isize - bit_displacement;
    }
    // loopbound min 0 max 7
    while bit_length >= CHAR_BIT as isize {
        if start_byte[current_byte_i] != 0 {
            return false;
        }
        current_byte_i += 1;
        bit_length -= CHAR_BIT as isize;
    }
    if bit_length > 0 && ( start_byte[current_byte_i] >> (CHAR_BIT - bit_length as u8)) != 0 {
        return false;
    }
    true
}

/* This function sets up new value of all bits of given bit string.
   This function is analog of standard C function `memset'.  Value of
   `bit_displacement' must be non-negative and can be greater than
   CHAR_BIT. */

pub fn ammunition_bit_string_set(
    start_byte: &mut [u8],
    bit_displacement: usize,
    bit: u8,
    bit_length: usize,
) {
    let mut current_byte_i = 0;
    let mut bit_length = bit_length as isize;

    if bit_length <= 0 { return; }
    let bit = bit != 0; /* 1 or 0 */
    let filling_byte = if bit { UCHAR_MAX } else { 0 };
    current_byte_i += bit_displacement / CHAR_BIT as usize;
    let bit_displacement = bit_displacement as isize % CHAR_BIT as isize;
    if bit_displacement != 0 {
        let mut mask = UCHAR_MAX << (CHAR_BIT - bit_displacement as u8);
        if bit_length < (CHAR_BIT as isize - bit_displacement) {
            mask |= UCHAR_MAX >> ( bit_displacement + bit_length );
        }
        start_byte[current_byte_i] = (start_byte[current_byte_i] & mask) | (filling_byte & !mask);
        current_byte_i += 1;
        bit_length -= CHAR_BIT as isize - bit_displacement;
    }
    // loopbound min 0 max 8
    while bit_length >= CHAR_BIT as isize {
        start_byte[current_byte_i] = filling_byte;
        current_byte_i += 1;
        bit_length -= CHAR_BIT as isize;
    }
    if bit_length > 0 {
        let lhs = start_byte[current_byte_i] & !(UCHAR_MAX << (CHAR_BIT - bit_length as u8));
        let rhs = filling_byte & (UCHAR_MAX << (CHAR_BIT - bit_length as u8));
        start_byte[current_byte_i] = lhs | rhs;
    }
}

/* This function copys a bit string to another bit string.  This
   function is analog of standard C function `memcpy'.  Values of
   `to_bit_displacement' and `from_bit_displacement' must be
   non-negative and can be greater than CHAR_BIT.  The bit string must
   be non-overlapped. */

pub fn ammunition_bit_string_copy(
    to: &mut [u8],
    to_bit_displacement: usize,
    from: &[u8],
    from_bit_displacement: usize,
    bit_length: usize,
) {
    let mut current_to_byte_i = 0;
    let mut current_from_byte_i = 0;
    let mut byte;
    let mut bit_length = bit_length as isize;

    if bit_length <= 0 { return; }
    current_to_byte_i += to_bit_displacement / CHAR_BIT as usize;
    let to_bit_displacement = to_bit_displacement as isize % CHAR_BIT as isize;
    current_from_byte_i += from_bit_displacement / CHAR_BIT as usize;
    let from_bit_displacement = from_bit_displacement as isize % CHAR_BIT as isize;
    // loopbound min 0 max 7
    loop {
        let lhs = ((from[current_from_byte_i] as u16) << from_bit_displacement) & UCHAR_MAX as u16;
        let rhs_cond = (from_bit_displacement != 0) && (bit_length > (CHAR_BIT as isize - from_bit_displacement));
        let rhs =  if rhs_cond { (from[current_from_byte_i + 1] as u16) >> (CHAR_BIT - from_bit_displacement as u8) } else { 0 };
        byte = lhs | rhs;
        if bit_length <= CHAR_BIT as isize { break; }
        /* Shift is correct when to_bit_displacement == 0 because its
        value is less than word bit size. */
        let lhs = (to[current_to_byte_i] as u16) & ( (UCHAR_MAX as u16) << (CHAR_BIT - to_bit_displacement as u8));
        let rhs = byte >> to_bit_displacement;
        to[current_to_byte_i] = (lhs | rhs) as u8;
        if to_bit_displacement != 0 {
            let lhs = (to[current_to_byte_i + 1] as u16) & ((UCHAR_MAX as u16) >> to_bit_displacement);
            let rhs = byte << (CHAR_BIT - to_bit_displacement as u8);
            to[current_to_byte_i + 1] = (lhs | rhs) as u8;
        }
        bit_length -= CHAR_BIT as isize;
        current_from_byte_i += 1;
        current_to_byte_i += 1;
    }
    /* Shift is correct when to_bit_displacement == 0 because its
        value is less than word bit size. */
    let lhs = (UCHAR_MAX as u16) << (CHAR_BIT - to_bit_displacement as u8);
    let rhs = (UCHAR_MAX as u16) >> (to_bit_displacement + bit_length);
    let mask = lhs | rhs;
    to[current_to_byte_i] = ((to[current_to_byte_i] as u16 & mask) as u16 | ((byte >> to_bit_displacement) & !mask)) as u8;
    bit_length -= CHAR_BIT as isize - to_bit_displacement;
    if bit_length > 0 {
        let lhs = (to[current_to_byte_i + 1] as u16) & ((UCHAR_MAX as u16) >> bit_length);
        let rhs = (byte << (CHAR_BIT - to_bit_displacement as u8)) & ((UCHAR_MAX as u16) << (CHAR_BIT - bit_length as u8));
        to[current_to_byte_i + 1] = (lhs | rhs) as u8;
    }
}

/* This function copys a bit string to another bit string.  Copying
   starts with the last bits of the bit strings.  This function is
   used by function `bit_string_move'.  Values of
   `to_bit_displacement' and `from_bit_displacement' must be
   non-negative and can be greater than CHAR_BIT.  The bit string must
   be non-overlapped. */

fn ammunition_reverse_bit_string_copy(
    to: &mut [u8],
    to_bit_displacement: usize,
    from: &[u8],
    from_bit_displacement: usize,
    bit_length: usize,
) {
    let mut current_to_byte_i = 0;
    let mut current_from_byte_i = 0;
    let mut byte;
    let mut bit_length = bit_length as isize;

    if bit_length <= 0 { return; }
    let mut to_bit_displacement = to_bit_displacement as isize + bit_length - 1;
    current_to_byte_i += to_bit_displacement as usize / CHAR_BIT as usize; /* last byte */
    to_bit_displacement %= CHAR_BIT as isize; /* last bit */
    let mut from_bit_displacement = from_bit_displacement as isize + bit_length - 1;
    current_from_byte_i += from_bit_displacement as usize / CHAR_BIT as usize; /* last byte */
    from_bit_displacement %= CHAR_BIT as isize; /* last bit */
    // loopbound min 0 max 7
    loop {
        /* Shift is correct when to_bit_displacement == 0 because its
        value is less than word bit size. */
        let lhs = (from[current_from_byte_i] as u16) >> (CHAR_BIT - 1 - from_bit_displacement as u8);
        let rhs_cond = (from_bit_displacement != CHAR_BIT as isize - 1) && (bit_length > from_bit_displacement + 1);
        let rhs = if rhs_cond { (from[current_from_byte_i - 1] as u16) << (from_bit_displacement + 1) } else { 0 } & UCHAR_MAX as u16;
        byte = lhs | rhs;
        if bit_length <= CHAR_BIT as isize { break; }
        /* Shift is correct when to_bit_displacement == 0 because its
        value is less than word bit size. */
        let lhs = (to[current_to_byte_i] as u16) & ((UCHAR_MAX as u16) >> (to_bit_displacement + 1));
        let rhs = byte << (CHAR_BIT - 1 - to_bit_displacement as u8);
        to[current_to_byte_i] = (lhs | rhs) as u8;
        if to_bit_displacement != CHAR_BIT as isize - 1 {
            let lhs = (to[current_to_byte_i - 1] as u16) & ((UCHAR_MAX as u16) << (CHAR_BIT - 1 - to_bit_displacement as u8));
            let rhs = byte >> (to_bit_displacement + 1);
            to[current_to_byte_i - 1] = (lhs | rhs) as u8;
        }
        bit_length -= CHAR_BIT as isize;
        current_from_byte_i -= 1;
        current_to_byte_i -= 1;
    }
    /* Shift is correct when to_bit_displacement == 0 because its
        value is less than word bit size. */
    let lhs = (UCHAR_MAX as u16) >> (to_bit_displacement + 1);
    let rhs = (UCHAR_MAX as u16) << (CHAR_BIT - 1 - to_bit_displacement as u8 + bit_length as u8);
    let mask = lhs | rhs;
    let lhs = to[current_to_byte_i] as u16 & mask;
    let rhs = (byte << (CHAR_BIT - 1 - to_bit_displacement as u8)) & !mask;
    to[current_to_byte_i] = (lhs | rhs) as u8;
    bit_length -= to_bit_displacement + 1;
    if bit_length > 0 {
        let lhs = (to[current_to_byte_i - 1] as u16) & ((UCHAR_MAX as u16) << bit_length);
        let rhs = byte >> (to_bit_displacement + 1) & ( (UCHAR_MAX as u16) >> (CHAR_BIT - bit_length as u8));
        to[current_to_byte_i - 1] = (lhs | rhs) as u8;
    }
}

/* This function copys a bit string to another bit string with the aid
   of functions `bit_string_copy' and `reverse_bit_string_copy'.  This
   function is analog of standard C function `memmove'.  Values of
   `to_bit_displacement' and `from_bit_displacement' must be
   non-negative and can be greater than CHAR_BIT.  The bit string can
   be overlapped. */

pub fn ammunition_bit_string_move(
    to: &mut [u8],
    to_bit_displacement: usize,
    from: &[u8],
    from_bit_displacement: usize,
    bit_length: usize,
) {
    let mut current_to_byte_i = 0;
    let mut current_from_byte_i = 0;

    if bit_length <= 0 { return; }
    current_to_byte_i += to_bit_displacement / CHAR_BIT as usize;
    let to_bit_displacement = to_bit_displacement % CHAR_BIT as usize;
    current_from_byte_i += from_bit_displacement / CHAR_BIT as usize;
    let from_bit_displacement = from_bit_displacement % CHAR_BIT as usize;
    let lhs = current_from_byte_i > current_to_byte_i;
    let rhs = (current_from_byte_i == current_to_byte_i) && (from_bit_displacement > to_bit_displacement);
    if lhs || rhs {
        ammunition_bit_string_copy(&mut to[current_to_byte_i..], to_bit_displacement,
            &from[current_from_byte_i..], from_bit_displacement, bit_length);
    } else {
        ammunition_reverse_bit_string_copy(&mut to[current_to_byte_i..], to_bit_displacement,
            &from[current_from_byte_i..], from_bit_displacement, bit_length);
    }
}

/* This function compares bit strings.  This function is analog of
   standard C function `memcmp'.  The function returns 0 if the bit
   strings are equal, 1 if the first bit string is greater than the
   second, -1 if the first bit string is less than the first.  Values
   of `bit_displacement1' and `bit_displacement2' must be non-negative
   and can be greater than CHAR_BIT. */

#[derive(PartialEq, PartialOrd)]
pub enum Compare { LT = -1, EQ = 0, GT = 1 }

pub fn ammunition_bit_string_comparison(
    str1: &[u8],
    bit_displacement1: usize,
    str2: &[u8],
    bit_displacement2: usize,
    bit_length: usize,
) -> Compare {
    let mut current_byte1_i = 0;
    let mut current_byte2_i = 0;
    let mut byte1: u8;
    let mut byte2: u8;
    let mut bit_length = bit_length;

    if bit_length <= 0 { return Compare::EQ; }
    current_byte1_i += bit_displacement1 / CHAR_BIT as usize;
    let bit_displacement1 = bit_displacement1 % CHAR_BIT as usize;
    current_byte2_i += bit_displacement2 / CHAR_BIT as usize;
    let bit_displacement2 = bit_displacement2 % CHAR_BIT as usize;
    // loopbound min 0 max 7
    loop {
        let lhs = (str1[current_byte1_i] << bit_displacement1) & UCHAR_MAX;
        /* Shift is correct when to_bit_displacement == 0
            because its value is less than word bit size. */
        let rhs_cond = (bit_displacement1 != 0) && (bit_length > CHAR_BIT as usize - bit_displacement1);
        let rhs = if rhs_cond { str1[current_byte1_i + 1] >> (CHAR_BIT as usize - bit_displacement1) } else { 0 };
        byte1 = lhs | rhs;

        let lhs = (str2[current_byte2_i] << bit_displacement2) & UCHAR_MAX;
        let rhs_cond = (bit_displacement2 != 0) && (bit_length > CHAR_BIT as usize - bit_displacement2);
        let rhs = if rhs_cond { str2[current_byte2_i + 1] >> (CHAR_BIT as usize - bit_displacement2) } else { 0 };
        byte2 = lhs | rhs;
        if bit_length <= CHAR_BIT as usize { break; }
        if byte1 > byte2 {
            return Compare::GT;
        }
        else {
            if byte1 < byte2 {
                return Compare::LT;
            }
        }
        bit_length -= CHAR_BIT as usize;
        current_byte2_i += 1;
        current_byte1_i += 1;
    }
    /* Shift is correct when to_bit_displacement == 0 because its value
        is less than word bit size. */
    let mask = UCHAR_MAX << (CHAR_BIT - bit_length as u8);
    if (byte1 & mask) > (byte2 & mask) {
        return Compare::GT;
    } else if (byte1 & mask) < (byte2 & mask) {
        return Compare::LT;
    } else {
        return Compare::EQ;
    }
}
