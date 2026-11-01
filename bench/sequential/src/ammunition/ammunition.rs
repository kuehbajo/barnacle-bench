/*

  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license (GPL 2) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

  Version V 2.0

  Name: ammunition

  Author: Vladimir Makarov <vmakarov@gcc.gnu.org>

  Function: Tests reusable packages bits and arith
    bits: Work with bit strings (copying, moving, setting, testing, comparison).
    arith: Implementing host machine-independently arbitrary precision integer
      numbers arithmetic.  The implementation of the package functions are not
      sufficiently efficient in order to use for run-time.  The package
      functions are oriented to implement constant-folding in compilers,
      cross-compilers.

  Source: DINO programming language repository
          https://github.com/dino-lang/dino/

  Changes: no major functional changes

  License: GPL 2 and LGPL 2

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

mod arithm;
mod bits;
mod general;

use arithm::{
    ammunition_unsigned_integer_from_string, ammunition_unsigned_integer_to_string,
    ammunition_integer_from_string, ammunition_integer_to_string,
    ammunition_add_unsigned_integer, ammunition_add_integer,
    ammunition_subtract_unsigned_integer, ammunition_subtract_integer,
    ammunition_multiply_unsigned_integer, ammunition_multiply_integer,
    ammunition_divide_unsigned_integer, ammunition_divide_integer,
    ammunition_unsigned_integer_remainder,
    ammunition_unsigned_integer_shift_right, ammunition_integer_shift_right,
    ammunition_unsigned_integer_shift_left, ammunition_integer_shift_left,
    ammunition_eq_integer, ammunition_eq_unsigned_integer,
    ammunition_ne_unsigned_integer, ammunition_ne_integer,
    ammunition_gt_unsigned_integer, ammunition_gt_integer,
    ammunition_lt_unsigned_integer, ammunition_lt_integer,
    ammunition_ge_unsigned_integer, ammunition_ge_integer,
    ammunition_le_unsigned_integer, ammunition_le_integer,
    ammunition_change_unsigned_integer_size, ammunition_change_integer_size,
    ammunition_unsigned_integer_or, ammunition_integer_or,
    ammunition_unsigned_integer_and, ammunition_integer_and,
    ammunition_unsigned_integer_not, ammunition_integer_not,
};

use bits::{
    CHAR_BIT, INT_MIN, INT_MAX, UINT_MAX,
    bit, set_bit, Compare,
    ammunition_is_zero_bit_string,
    ammunition_bit_string_set,
    ammunition_bit_string_comparison,
    ammunition_bit_string_move,
    ammunition_bit_string_copy,
};

use general::{
    ammunition_strcmp, ammunition_atoi,
    ammunition_sprintf_u, ammunition_sprintf_d,
};

/*
##################
    ammunition_string.h
##################
*/

/*
  Core functions
*/

fn ammunition_reset_str_bits(str: &mut [u8; 8], s: &mut [u8; 8]) {
    // loopbound min 8 max 8
    for (e1, e2) in str.iter_mut().zip(s.iter_mut()) {
        *e1 = 0;
        *e2 = 0;
    }
}

fn ammunition_reset_str_arithm(
    str: &mut [u8; 20],
    s: &mut [u8; 20],
    d: &mut [u8; 4],
    e: &mut [u8; 4],
    g: &mut [u8; 6],
) {
    // loopbound min 20 max 20
    for i in 0..20 {
        str[ i ] = 0;
        s[ i ] = 0;
    }
    // loopbound min 4 max 4
    for i in 0..4 {
        d[ i ] = 0;
        e[ i ] = 0;
    }
    // loopbound min 6 max 6
    for el in g.iter_mut() {
        *el = 0;
    }
}

fn ammunition_bits_test() -> i32 {
    let mut str: [u8; 8] = [0; 8];
    let mut str1: [u8; 8] = [0; 8];

    let mut result = 0;

    /* Test 1 */
    ammunition_reset_str_bits(&mut str, &mut str1);

    // loopbound min 64 max 64
    for i in 0..(str.len() * CHAR_BIT as usize) {
        if bit(&str, i) == 1 {
            result = 1;
        }
    }

    // loopbound min 64 max 64
    for i in 0..(str.len() * CHAR_BIT as usize) {
        set_bit(&mut str, i, 1);
        // loopbound min 64 max 64
        for j in 0..(str.len() * CHAR_BIT as usize) {
            if j <= i {
                if bit(&str, j) == 0 {
                    result = 1;
                }
            } else if bit(&str, j) == 1 {
                result = 1;
            }
        }
    }

    /* Test 2 */
    ammunition_reset_str_bits(&mut str, &mut str1);

    // loopbound min 64 max 64
    for i in 0..(str.len() * CHAR_BIT as usize) {
        let length = (str.len() * CHAR_BIT as usize - i) / 2 + 1;
        let cond = !ammunition_is_zero_bit_string(&mut str, i, length);
        if cond {
            result = 1;
        }
    }
    ammunition_bit_string_set(&mut str, 13, 1, 35);
    // loopbound min 13 max 13
    for i in 0..13 {
        if !ammunition_is_zero_bit_string(&mut str, i, 13 - i) {
            result = 1;
        }
    } 
    // loopbound min 35 max 35
    for i in 13..48 {
        if ammunition_is_zero_bit_string(&mut str, i, 48 - i) {
            result = 1;
        }
    }
      
    // loopbound min 16 max 16
    for i in 48..(str.len() * CHAR_BIT as usize) {
        let length = str.len() * CHAR_BIT as usize - i;
        let cond = !ammunition_is_zero_bit_string(&mut str, i, length);
        if cond {
            result = 1;
        }
    }

    /* Test 3 */
    ammunition_reset_str_bits(&mut str, &mut str1);

    // loopbound min 42 max 42
    let mut i = 0;
    while (i + i / 2 + 1) < (str.len() * CHAR_BIT as usize) {
        ammunition_bit_string_set(&mut str, i, 1, i / 2 + 1);
        if !ammunition_is_zero_bit_string(&mut str, 0, i - 1) {
            result = 1;
        }
        if ammunition_is_zero_bit_string(&mut str, i, i / 2 + 1) {
            result = 1;
        }
        let length = str.len() * CHAR_BIT as usize - (i + i / 2 + 1);
        let cond = !ammunition_is_zero_bit_string(&mut str, i + i / 2 + 1, length);
        if cond {
            result = 1;
        }
        let length = str.len() * CHAR_BIT as usize;
        ammunition_bit_string_set(&mut str, 0, 0, length);
        i += 1;
    }

    /* Test 4 */
    ammunition_reset_str_bits(&mut str, &mut str1);

    ammunition_bit_string_set(&mut str, 2, 1, 43);
    ammunition_bit_string_set(&mut str1, 2, 1, 40);
    // loopbound min 42 max 42
    for i in 0..42 {
        let cond = ammunition_bit_string_comparison(&str, i, &str1, i, 42 - i) != Compare::EQ;
        if cond {
            result = 1;
        }
    }
        
    // loopbound min 43 max 43
    for i in 0..43 {
        let cond = ammunition_bit_string_comparison(&str, i, &str1, i, str.len() * CHAR_BIT as usize - i) <= Compare::EQ;
        if cond {
            result = 1;
        }
    }
    // loopbound min 43 max 43
    for i in 0..43 {
        let cond = ammunition_bit_string_comparison(&str1, i, &str, i, str.len() * CHAR_BIT as usize - i) >= Compare::EQ;
        if cond {
            result = 1;
        }
    }   

    /* Test 5 */
    ammunition_reset_str_bits(&mut str, &mut str1);

    ammunition_bit_string_set(&mut str, 2, 1, 43);
    // loopbound min 59 max 59
    let mut i = 0;
    while (i + 5) < (str.len() * CHAR_BIT as usize) {
        ammunition_bit_string_copy(&mut str1, i + 5, &str, i, str.len() * CHAR_BIT as usize - i - 5);
        let cond = ammunition_bit_string_comparison(&str1, i + 5, &str, i, str.len() * CHAR_BIT as usize - i - 5) != Compare::EQ;
        if cond {
            result = 1;
        }
        i += 1;
    }

    /* Test 6 */
    ammunition_reset_str_bits(&mut str, &mut str1);

    ammunition_bit_string_set(&mut str, 2, 1, 43);
    ammunition_bit_string_set(&mut str1, 2, 1, 43);
    // loopbound min 59 max 59
    let mut i = 0; 
    while (i + 5) < (str.len() * CHAR_BIT as usize) {
        let length = str.len() * CHAR_BIT as usize;
        ammunition_bit_string_set(&mut str, 0, 0, length);
        ammunition_bit_string_set(&mut str, 2, 1, 43);
        let str_cpy = str;
        let length = str.len() * CHAR_BIT as usize - i - 5;
        ammunition_bit_string_move(&mut str, i + 5, &str_cpy, i, length);
        let cond = ammunition_bit_string_comparison(&str, i + 5, &str1, i, str.len() * CHAR_BIT as usize - i - 5) != Compare::EQ;
        if cond {
            result = 1;
        }
        i += 1;
    }

    /* Test 7 */
    ammunition_reset_str_bits(&mut str, &mut str1);

    ammunition_bit_string_set(&mut str, 2, 1, 43);
    ammunition_bit_string_set(&mut str1, 2, 1, 43);
    // loopbound min 59 max 59
    let mut i = 0;
    while (i + 5) < (str.len() * CHAR_BIT as usize) {
        let length = str.len() * CHAR_BIT as usize;
        ammunition_bit_string_set(&mut str, 0, 0, length);
        ammunition_bit_string_set(&mut str, 2, 1, 43);
        let str_cpy = str;
        let length = str.len() * CHAR_BIT as usize - i - 5;
        ammunition_bit_string_move(&mut str, i, &str_cpy, i + 5, length);
        let cond = ammunition_bit_string_comparison(&str, i, &str1, i + 5, str.len() * CHAR_BIT as usize - i - 5) != Compare::EQ;
        if cond {
            result = 1;
        }
        i += 1;
    }
    result
}


fn ammunition_arithm_test(overflow_bit: &mut bool) -> i32 {
    let mut result: i32 = 0;

    /* Test 1 */
    let mut str: [u8; 20] = [0; 20];
    let mut s: [u8; 20] = [0; 20];
    let mut d: [u8; 4] = [0; 4];
    let mut e: [u8; 4] = [0; 4];
    let mut g: [u8; 6] = [0; 6];

    ammunition_integer_from_string(4, b"-2147483649", &mut d, overflow_bit);
    if !*overflow_bit {
        return 1;
    }
    ammunition_sprintf_d(&mut str, INT_MIN);
    ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
    if *overflow_bit {
        result = 1;
    }
    ammunition_integer_to_string(4, &d, &mut s);
    if ammunition_strcmp(&s, &str) != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"2147483648", &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_sprintf_d(&mut str, INT_MAX);
    ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
    if *overflow_bit {
        result = 1;
    }
    ammunition_integer_to_string(4, &d, &mut s);
    if ammunition_strcmp(&s, &str) != 0 {
        result = 1;
    }

    // loopbound min 4000 max 4000
    for i in (-2000)..2000 {
        ammunition_sprintf_d(&mut str, i);
        ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
        if *overflow_bit {
            result = 1;
        }
        ammunition_integer_to_string(4, &d, &mut s);
        if ammunition_strcmp(&s, &str) != 0 {
            result = 1;
        }
    }

    /* Test 2 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"4294967296", &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_sprintf_u(&mut str, UINT_MAX);
    ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
    if *overflow_bit {
        result = 1;
    }
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if ammunition_strcmp(&s, &str) != 0 {
        result = 1;
    }
    // loopbound min 4000 max 4000
    for i in 0..4000 {
      ammunition_sprintf_u(&mut str, i);
      ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
      if *overflow_bit {
          result = 1;
      }
      ammunition_unsigned_integer_to_string(4, &d, &mut s);
      if ammunition_strcmp(&s, &str) != 0 {
          result = 1;
      }
    }

    /* Test 3 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_sprintf_d(&mut str, INT_MAX);
    ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1", &mut e, overflow_bit);
    let d_copy = d;
    ammunition_add_integer( 4, &d_copy, &e, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_sprintf_d(&mut str, INT_MAX - 4);
    ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_sprintf_d(&mut str, 4);
    ammunition_integer_from_string(4, &str, &mut e, overflow_bit);
    let d_copy = d;
    ammunition_add_integer( 4, &d_copy, &e, &mut d, overflow_bit);
    if *overflow_bit {
        result = 1;
    }
    ammunition_integer_to_string(4, &d, &mut s);
    ammunition_sprintf_d(&mut str, INT_MAX);
    if ammunition_strcmp(&s, &str) != 0 {
        result = 1;
    }
    // loopbound min 4000 max 4000
    for i in (-2000)..2000 {
      ammunition_sprintf_d(&mut str, i);
      ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
      ammunition_sprintf_d(&mut str, i + 1);
      ammunition_integer_from_string(4, &str, &mut e, overflow_bit);
      let d_copy = d;
      ammunition_add_integer( 4, &d_copy, &e, &mut d, overflow_bit);
      if *overflow_bit {
          result = 1;
      }
     ammunition_integer_to_string(4, &d, &mut s);
      if ammunition_atoi(&s) != i + i + 1 {
          result = 1;
      }
    }

    /* Test 4 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_sprintf_u(&mut str, UINT_MAX);
    ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1", &mut e, overflow_bit);
    let d_copy = d;
    ammunition_add_unsigned_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_sprintf_u(&mut str, UINT_MAX - 4);
    ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_sprintf_u(&mut str, 4);
    ammunition_unsigned_integer_from_string(4, &str, &mut e, overflow_bit);
    let d_copy = d;
    ammunition_add_unsigned_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if *overflow_bit {
        result = 1;
    }
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    ammunition_sprintf_u(&mut str, UINT_MAX);
    if ammunition_strcmp(&s, &str) != 0 {
        result = 1;
    }
    // loopbound min 4000 max 4000
    for i in 0..4000 {
      ammunition_sprintf_u(&mut str, i);
      ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
      ammunition_sprintf_u(&mut str, i + 1);
      ammunition_unsigned_integer_from_string(4, &str, &mut e, overflow_bit);
      let d_copy = d;
      ammunition_add_unsigned_integer(4, &d_copy, &e, &mut d, overflow_bit);
      if *overflow_bit {
          result = 1;
      }
      ammunition_unsigned_integer_to_string(4, &d, &mut s);
      if ammunition_atoi(&s) != (i + i + 1) as i32 {
          result = 1;
      }
    }

    /* Test 5 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_sprintf_d(&mut str, INT_MIN);
    ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1", &mut e, overflow_bit);
    let d_copy = d;
    ammunition_subtract_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_sprintf_d(&mut str, INT_MIN + 4);
    ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_sprintf_d(&mut str, 4);
    ammunition_integer_from_string(4, &str, &mut e, overflow_bit);
    let d_copy = d;
    ammunition_subtract_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if *overflow_bit {
        result = 1;

    }
    ammunition_integer_to_string(4, &d, &mut s);
    ammunition_sprintf_d(&mut str, INT_MIN);
    if ammunition_strcmp(&s, &str) != 0 {
        result = 1;
    }
    // loopbound min 4000 max 4000
    for i in (-2000)..2000 {
      ammunition_sprintf_d(&mut str, i);
      ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
      ammunition_sprintf_d(&mut str, 10 - i);
      ammunition_integer_from_string(4, &str, &mut e, overflow_bit);
      let d_copy = d;
      ammunition_subtract_integer(4, &d_copy, &e, &mut d, overflow_bit);
      if *overflow_bit {
          result = 1;
      }
      ammunition_integer_to_string(4, &d, &mut s);
      if ammunition_atoi(&s) != i + i - 10 {
          result = 1;
      }
    }

    /* Test 6 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);
    ammunition_sprintf_u(&mut str, UINT_MAX - 2);
    ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_sprintf_u(&mut str, UINT_MAX - 1);
    ammunition_unsigned_integer_from_string(4, &str, &mut e, overflow_bit);
    let d_copy = d;
    ammunition_subtract_unsigned_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_sprintf_u(&mut str, UINT_MAX);
    ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
    let d_copy = d;
    ammunition_subtract_unsigned_integer(4, &d_copy, &d_copy, &mut d, overflow_bit);
    if *overflow_bit {
        result = 1;
    }
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if ammunition_strcmp(&s, b"0") != 0 {
        result = 1;
    }
    // loopbound min 4000 max 4000
    for i in 0..4000 {
      ammunition_sprintf_u(&mut str, i);
      ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
      ammunition_sprintf_u(&mut str, i / 2);
      ammunition_unsigned_integer_from_string(4, &str, &mut e, overflow_bit);
      let d_copy = d;
      ammunition_subtract_unsigned_integer(4, &d_copy, &e, &mut d, overflow_bit);
      if *overflow_bit {
          result = 1;
      }
      ammunition_unsigned_integer_to_string(4, &d, &mut s);
      if ammunition_atoi(&s) != (i - i / 2) as i32 {
          result = 1;
      }
    }

    /* Test 7 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_sprintf_d(&mut str, INT_MAX / 2 + 1);
    ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"2", &mut e, overflow_bit);
    let d_copy = d;
    ammunition_multiply_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_sprintf_d(&mut str, INT_MIN / 2 - 1);
    ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"2", &mut e, overflow_bit);
    let d_copy = d;
    ammunition_multiply_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_sprintf_d(&mut str, INT_MAX / 3);
    ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_sprintf_d(&mut str, 3);
    ammunition_integer_from_string(4, &str, &mut e, overflow_bit);
    let d_copy = d;
    ammunition_multiply_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if *overflow_bit {
        result = 1;
    }
    ammunition_integer_to_string(4, &d, &mut s);
    ammunition_sprintf_d(&mut str, ( INT_MAX / 3) * 3);
    if ammunition_strcmp(&s, &str) != 0 {
        result = 1;
    }
    ammunition_sprintf_d(&mut str, INT_MIN / 2);
    ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_sprintf_d(&mut str, 2);
    ammunition_integer_from_string(4, &str, &mut e, overflow_bit);
    let d_copy = d;
    ammunition_multiply_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if *overflow_bit {
        result = 1;
    }
    ammunition_integer_to_string(4, &d, &mut s);
    ammunition_sprintf_d(&mut str, ( INT_MIN / 2) * 2);
    if ammunition_strcmp(&s, &str) != 0 {
        result = 1;
    }
    // loopbound min 4000 max 4000
    for i in (-2000)..2000 {
      ammunition_sprintf_d(&mut str, i);
      ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
      ammunition_sprintf_d(&mut str, i + 1000);
      ammunition_integer_from_string(4, &str, &mut e, overflow_bit);
      let d_copy = d;
      ammunition_multiply_integer(4, &d_copy, &e, &mut d, overflow_bit);
      if *overflow_bit {
          result = 1;
      }
     ammunition_integer_to_string(4, &d, &mut s);
      if ammunition_atoi(&s) != i * ( i + 1000) {
          result = 1;
      }
    }

    /* Test 8 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_sprintf_u(&mut str, UINT_MAX / 5 + 1);
    ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_sprintf_u(&mut str, 5);
    ammunition_unsigned_integer_from_string(4, &str, &mut e, overflow_bit);
    let d_copy = d;
    ammunition_multiply_unsigned_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_sprintf_u(&mut str, UINT_MAX / 2);
    ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
    ammunition_sprintf_u(&mut str, 2);
    ammunition_unsigned_integer_from_string(4, &str, &mut e, overflow_bit);
    let d_copy = d;
    ammunition_multiply_unsigned_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if *overflow_bit {
        result = 1;
    }
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    ammunition_sprintf_u(&mut str, ( UINT_MAX / 2) * 2);
    if ammunition_strcmp(&s, &str) != 0 {
        result = 1;
    }
    // loopbound min 4000 max 4000
    for i in 0..4000 {
      ammunition_sprintf_u(&mut str, i);
      ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
      ammunition_sprintf_u(&mut str, i / 2);
      ammunition_unsigned_integer_from_string(4, &str, &mut e, overflow_bit);
      let d_copy = d;
      ammunition_multiply_unsigned_integer(4, &d_copy, &e, &mut d, overflow_bit);
      if *overflow_bit {
          result = 1;
      }
      ammunition_unsigned_integer_to_string(4, &d, &mut s);
      if ammunition_atoi(&s) != (i * ( i / 2)) as i32 {
          result = 1;
      }
    }

    /* Test 9 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"10", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"0", &mut e, overflow_bit);
    let d_copy = d;
    ammunition_divide_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    // loopbound min 4000 max 4000
    for i in (-2000)..2000 {
      ammunition_sprintf_d(&mut str, i);
      ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
      ammunition_sprintf_d(&mut str, if i < 0 { - i / 20 + 1 } else { - i / 20 - 1 });
      ammunition_integer_from_string(4, &str, &mut e, overflow_bit);
      let d_copy = d;
      ammunition_divide_integer(4, &d_copy, &e, &mut d, overflow_bit);
      if *overflow_bit {
          result = 1;
      }
     ammunition_integer_to_string(4, &d, &mut s);
      if ammunition_atoi(&s) != i / (if i < 0 { - i / 20 + 1 } else { - i / 20 - 1 }) {
          result = 1;
      }
      ammunition_sprintf_d(&mut str, i);
      ammunition_integer_from_string(4, &str, &mut d, overflow_bit);
      let e_copy = e;
      ammunition_divide_integer(4, &d, &e_copy, &mut e, overflow_bit);
      if *overflow_bit {
          result = 1;
      }
     ammunition_integer_to_string(4, &e, &mut s);
      if ammunition_atoi(&s) != i / (if i < 0 { - i / 20 + 1 } else { - i / 20 - 1 }) {
          result = 1;
      }
    }

    /* Test 10 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"10", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"0", &mut e, overflow_bit);
    let d_copy = d;
    ammunition_divide_unsigned_integer(4, &d_copy, &e, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    // loopbound min 4000 max 4000
    for i in 0..4000 {
      ammunition_sprintf_u(&mut str, i);
      ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
      ammunition_sprintf_u(&mut str, i / 20 + 1);
      ammunition_unsigned_integer_from_string(4, &str, &mut e, overflow_bit);
      let d_copy = d;
      ammunition_divide_unsigned_integer(4, &d_copy, &e, &mut d, overflow_bit);
      if *overflow_bit {
          result = 1;
      }
      ammunition_unsigned_integer_to_string(4, &d, &mut s);
      if ammunition_atoi(&s) != (i / ( i / 20 + 1)) as i32 {
          result = 1;
      }
    }

    /* Test 11 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"10", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"0", &mut e, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_remainder(4, &d_copy, &e, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    // loopbound min 4000 max 4000
    for i in 0..4000 {
      ammunition_sprintf_u(&mut str, i);
      ammunition_unsigned_integer_from_string(4, &str, &mut d, overflow_bit);
      ammunition_sprintf_u(&mut str, i / 20 + 1);
      ammunition_unsigned_integer_from_string(4, &str, &mut e, overflow_bit);
      let d_copy = d;
      ammunition_unsigned_integer_remainder(4, &d_copy, &e, &mut d, overflow_bit);
      if *overflow_bit {
          result = 1;
      }
      ammunition_unsigned_integer_to_string(4, &d, &mut s);
      if ammunition_atoi(&s) != (i % ( i / 20 + 1)) as i32 {
          result = 1;
      }
    }

    /* Test 12 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_shift_right(4, &d_copy, 0, &mut d, overflow_bit);
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"1348") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_shift_right(4, &d_copy, 32, &mut d, overflow_bit);
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if !*overflow_bit || ammunition_strcmp(&s, b"0") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_shift_right(4, &d_copy, 8, &mut d, overflow_bit);
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if !*overflow_bit || ammunition_strcmp(&s, b"5") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"134890", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_shift_right(4, &d_copy, 13, &mut d, overflow_bit);
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if !*overflow_bit || ammunition_strcmp(&s, b"16") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"134890", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_shift_left(4, &d_copy, -13, &mut d, overflow_bit);
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if !*overflow_bit || ammunition_strcmp(&s, b"16") != 0 {
        result = 1;
    }

    /* Test 13 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_right(4, &d_copy, 0, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"1348") != 0 {
        result = 1;
    }

    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_right(4, &d_copy, 32, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if !*overflow_bit || ammunition_strcmp(&s, b"0") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_right(4, &d_copy, 8, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if !*overflow_bit || ammunition_strcmp(&s, b"5") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"134890", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_right(4, &d_copy, 13, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if !*overflow_bit || ammunition_strcmp(&s, b"16") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"134890", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_left( 4, &d_copy, -13, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if !*overflow_bit || ammunition_strcmp(&s, b"16") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_right(4, &d_copy, 0, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"-1348") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_right(4, &d_copy, 32, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if !*overflow_bit || ammunition_strcmp(&s, b"-1") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_right(4, &d_copy, 8, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if !*overflow_bit || ammunition_strcmp(&s, b"-6") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-134890", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_right(4, &d_copy, 13, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if !*overflow_bit || ammunition_strcmp(&s, b"-17") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-134890", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_left( 4, &d_copy, -13, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if !*overflow_bit || ammunition_strcmp(&s, b"-17") != 0 {
        result = 1;
    }

    /* Test 14 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_shift_left(4, &d_copy, 0, &mut d, overflow_bit);
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"1348") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_shift_left(4, &d_copy, 22, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_shift_left(4, &d_copy, 8, &mut d, overflow_bit);
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"345088") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"134890", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_shift_left(4, &d_copy, 13, &mut d, overflow_bit);
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"1105018880") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"134890", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_shift_right(4, &d_copy, -13, &mut d, overflow_bit);
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"1105018880") != 0 {
        result = 1;
    }

    /* Test 15 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_left( 4, &d_copy, 0, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"1348") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_left( 4, &d_copy, 21, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_left( 4, &d_copy, 8, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"345088") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"134890", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_left( 4, &d_copy, 13, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"1105018880") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"134890", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_right(4, &d_copy, -13, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"1105018880") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_left( 4, &d_copy, 0, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"-1348") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_left( 4, &d_copy, 21, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_left( 4, &d_copy, 8, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"-345088") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-134890", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_left( 4, &d_copy, 13, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"-1105018880") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-134890", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_shift_right(4, &d_copy, -13, &mut d, overflow_bit);
    ammunition_integer_to_string(4, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"-1105018880") != 0 {
        result = 1;
    }

    /* Test 16 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"-10", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_eq_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"-1348", &mut e, overflow_bit);
    if !ammunition_eq_integer( 4, &d, &mut e) {
        result = 1;
    }

    /* Test 17 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"10", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_eq_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_eq_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }

    /* Test 18 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"-10", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_ne_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"-1348", &mut e, overflow_bit);
    if ammunition_ne_integer( 4, &d, &mut e) {
        result = 1;
    }

    /* Test 19 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"10", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_ne_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_ne_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }

    /* Test 20 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"-10", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_gt_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"-1348", &mut e, overflow_bit);
    if ammunition_gt_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1000000", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"-1348", &mut e, overflow_bit);
    if ammunition_gt_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1000000", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_gt_integer( 4, &d, &mut e) {
        result = 1;
    }

    /* Test 21 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"10", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_gt_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_gt_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1000000", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_gt_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }

    /* Test 22 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"-10", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_lt_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"-1348", &mut e, overflow_bit);
    if ammunition_lt_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1000000", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"-1348", &mut e, overflow_bit);
    if !ammunition_lt_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1000000", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_lt_integer( 4, &d, &mut e) {
        result = 1;
    }

    /* Test 23 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"10", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_lt_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_lt_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1000000", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_lt_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }

    /* Test 24 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"-10", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_ge_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"-1348", &mut e, overflow_bit);
    if !ammunition_ge_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1000000", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"-1348", &mut e, overflow_bit);
    if ammunition_ge_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1000000", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_ge_integer( 4, &d, &mut e) {
        result = 1;
    }

    /* Test 25 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"10", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_ge_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_ge_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1000000", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_ge_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }

    /* Test 26 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"-10", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_le_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"-1348", &mut e, overflow_bit);
    if !ammunition_le_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1000000", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"-1348", &mut e, overflow_bit);
    if !ammunition_le_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1000000", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_le_integer( 4, &d, &mut e) {
        result = 1;
    }

    /* Test 27 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"10", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_le_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if !ammunition_le_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1000000", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    if ammunition_le_unsigned_integer( 4, &d, &mut e) {
        result = 1;
    }

    /* Test 28 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"70000", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_change_unsigned_integer_size( 4, &d_copy, 2, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"30000", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_change_unsigned_integer_size( 4, &d_copy, 2, &mut d, overflow_bit);
    ammunition_integer_to_string(2, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"30000") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"11230000", &mut g, overflow_bit);
    let g_copy = g;
    ammunition_change_unsigned_integer_size( 4, &g_copy, 6, &mut g, overflow_bit);
    ammunition_integer_to_string(6, &g, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"11230000") != 0 {
        result = 1;
    }

    /* Test 29 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"40000", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_change_integer_size( 4, &d_copy, 2, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-33000", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_change_integer_size( 4, &d_copy, 2, &mut d, overflow_bit);
    if !*overflow_bit {
        result = 1;
    }
    ammunition_integer_from_string(4, b"30000", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_change_integer_size( 4, &d_copy, 2, &mut d, overflow_bit);
    ammunition_integer_to_string(2, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"30000") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-30000", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_change_integer_size( 4, &d_copy, 2, &mut d, overflow_bit);
    ammunition_integer_to_string(2, &d, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"-30000") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"11230000", &mut g, overflow_bit);
    let g_copy = g;
    ammunition_change_integer_size( 4, &g_copy, 6, &mut g, overflow_bit);
    ammunition_integer_to_string(6, &g, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"11230000") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-11230000", &mut g, overflow_bit);
    let g_copy = g;
    ammunition_change_integer_size( 4, &g_copy, 6, &mut g, overflow_bit);
    ammunition_integer_to_string(6, &g, &mut s);
    if *overflow_bit || ammunition_strcmp(&s, b"-11230000") != 0 {
        result = 1;
    }

    /* Test 30 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_unsigned_integer_or( 4, &d, &e_copy, &mut e);
    ammunition_unsigned_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"1348") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"0", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_unsigned_integer_or( 4, &d, &e_copy, &mut e);
    ammunition_unsigned_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"1348") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"4294967295", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_unsigned_integer_or( 4, &d, &e_copy, &mut e);
    ammunition_unsigned_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"4294967295") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"96", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_unsigned_integer_or( 4, &d, &e_copy, &mut e);
    ammunition_unsigned_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"1380") != 0 {
        result = 1;
    }

    /* Test 31 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1348", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_integer_or( 4, &d, &e_copy, &mut e);
    ammunition_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"1348") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"0", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_integer_or( 4, &d, &e_copy, &mut e);
    ammunition_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"1348") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"-1", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_integer_or( 4, &d, &e_copy, &mut e);
    ammunition_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"-1") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"96", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_integer_or( 4, &d, &e_copy, &mut e);
    ammunition_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"1380") != 0 {
        result = 1;
    }

    /* Test 32 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"1348", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_unsigned_integer_and( 4, &d, &e_copy, &mut e);
    ammunition_unsigned_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"1348") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"0", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_unsigned_integer_and( 4, &d, &e_copy, &mut e);
    ammunition_unsigned_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"0") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"4294967295", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_unsigned_integer_and( 4, &d, &e_copy, &mut e);
    ammunition_unsigned_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"1348") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_unsigned_integer_from_string(4, b"96", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_unsigned_integer_and( 4, &d, &e_copy, &mut e);
    ammunition_unsigned_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"64") != 0 {
        result = 1;
    }

    /* Test 33 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"1348", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_integer_and( 4, &d, &e_copy, &mut e);
    ammunition_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"1348") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"0", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_integer_and( 4, &d, &e_copy, &mut e);
    ammunition_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"0") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"-1", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_integer_and( 4, &d, &e_copy, &mut e);
    ammunition_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"1348") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    ammunition_integer_from_string(4, b"96", &mut e, overflow_bit);
    let e_copy = e;
    ammunition_integer_and( 4, &d, &e_copy, &mut e);
    ammunition_integer_to_string(4, &e, &mut s);
    if ammunition_strcmp(&s, b"64") != 0 {
        result = 1;
    }

    /* Test 34 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_unsigned_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_not( 4, &d_copy, &mut d);
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if ammunition_strcmp(&s, b"4294965947") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"0", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_not( 4, &d_copy, &mut d);
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if ammunition_strcmp(&s, b"4294967295") != 0 {
        result = 1;
    }
    ammunition_unsigned_integer_from_string(4, b"4294967295", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_unsigned_integer_not( 4, &d_copy, &mut d);
    ammunition_unsigned_integer_to_string(4, &d, &mut s);
    if ammunition_strcmp(&s, b"0") != 0 {
        result = 1;
    }

    /* Test 35 */
    ammunition_reset_str_arithm(&mut str, &mut s, &mut d, &mut e, &mut g);

    ammunition_integer_from_string(4, b"1348", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_not(4, &d_copy, &mut d);
    ammunition_integer_to_string(4, &d, &mut s);
    if ammunition_strcmp(&s, b"-1349") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"0", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_not(4, &d_copy, &mut d);
    ammunition_integer_to_string(4, &d, &mut s);
    if ammunition_strcmp(&s, b"-1") != 0 {
        result = 1;
    }
    ammunition_integer_from_string(4, b"-1", &mut d, overflow_bit);
    let d_copy = d;
    ammunition_integer_not(4, &d_copy, &mut d);
    ammunition_integer_to_string(4, &d, &mut s);
    if ammunition_strcmp(&s, b"0") != 0 {
        result = 1;
    }

    result
}


/*
  Initialization- and return-value-related functions
*/

fn ammunition_init(result: &mut i32) {
    *result = 0;
}

fn ammunition_return(result: &mut i32) -> bool {
    *result == 0
}

/*
  Main functions
*/

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn ammunition_main(
    result: &mut i32,
    overflow_bit: &mut bool,
) {
    *result |= ammunition_bits_test();
    *result |= ammunition_arithm_test(overflow_bit);
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn ammunition_log(
    result: &mut i32,
    overflow_bit: &mut bool,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        ammunition_init(black_box(result));
        let start = black_box(DWT::cycle_count());
        ammunition_main(
            black_box(result), black_box(overflow_bit),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("ammunition - elapsed: {}", cycles);
    });

    black_box(ammunition_return(black_box(result)))
}

// overflow interrupt -> overflow in software handlen

#[cfg(any(unix, windows, not(feature = "log")))]
fn ammunition_std(
    result: &mut i32,
    overflow_bit: &mut bool,
) -> bool {
    ammunition_init(black_box(result));
    ammunition_main(
        black_box(result), black_box(overflow_bit),
    );
    black_box(ammunition_return(black_box(result)))
}

fn ammunition() -> bool {
    static mut RESULT: i32 = 0;
    static mut OVERFLOW_BIT: bool = false;
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let result = unsafe { &mut *{ &raw mut RESULT } };
    let overflow_bit = unsafe { &mut *{ &raw mut OVERFLOW_BIT } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(ammunition_log(
            black_box(result), black_box(overflow_bit),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(ammunition_std(
            black_box(result), black_box(overflow_bit),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(ammunition());
    
    #[cfg(feature = "log")]
    {
        rprintln!("ammunition: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(ammunition());

    #[cfg(feature = "log")]
    std::println!("ammunition: {}", success);

    std::process::exit(!success as i32);
}