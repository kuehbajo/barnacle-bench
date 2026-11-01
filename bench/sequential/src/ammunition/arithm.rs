/*

  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license as the original main file.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following information.

  FILE NAME:   arithm.c

  TITLE:       Package for arbitrary precision integer arithmetic

  DESCRIPTION: This abstract data implements arbitrary precision
      integer and unsigned integer numbers by machine independent
      way.  The implementation of the package functions are not
      sufficiently efficient in order to use for run-time.  The
      package functions are oriented to implement constant-folding in
      compilers.  This package is necessary because host machine may
      not support such arithmetic for target machine.  For example,
      VAX does not support does not support more 32-bits integer
      numbers arithmetic.  The numbers are represented by bytes in
      big endian mode, negative integer numbers are represented in
      complementary code.  All sizes are given in bytes and must be
      positive.  Results of executions of all functions can coincide
      with a operand(s).  All functions of addition, subtraction,
      multiplication, division, evaluation of remainder, shift,
      changing size and transformation of string into number fix
      overflow.  The overflow is fixed when result can not be
      represented by number of given size.

*/

use core::cmp::min;

use crate::bits::{ CHAR_BIT, UCHAR_MAX };

pub fn memset(dst: &mut [u8], val: u8, size: usize) {
  let end = min(dst.len(), size);
  // loopbound min 0 max 4
  dst[..end].fill(val);
}

pub fn memcpy(dst: &mut [u8], src: &[u8], size: usize) {
  let end = [dst.len(), src.len(), size].into_iter().min().unwrap();
  // loopbound min 2 max 6
  for (d, s) in dst[..end].iter_mut().zip(src[..end].iter()) {
    *d = *s;
  }
}

/*
##################
    arithm.h
##################
*/

/* The value of macro is suggested to be maximum length of integer operands
   The length of use integers should be not greater than this value. */

const MAX_INTEGER_OPERAND_SIZE: usize = 128;

/* The following macro value is sign of integer number (0 or 1) given
   as macro parameter. */

#[inline]
fn integer_sign(val: u8) -> u8 {
  val >> (CHAR_BIT - 1)
}

/*
##################
    arithm.c
##################
*/

/* This variable can have only two values 0 or 1.  The value `1'
   corresponds to overflow.  The variable value are modified by all
   functions of addition, subtract, multiplication, division,
   evaluation of remainder, shift, changing size and transformation of
   string into number fix overflow. */

/* The following function adds unsigned integers.  The function
   returns true if unsigned integer overflow is fixed, false otherwise.
   Result can be placed in any operand. */

fn ammunition_add_unsigned_integer_without_overflow_reaction(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
) -> bool {
    let mut carry: u8 = 0;
    // loopbound min 4 max 4
    for digit_num in (0..size).rev() {
            let sum = op1[digit_num] as u16 + op2[digit_num] as u16 + carry as u16;
            result[digit_num] = (sum & UCHAR_MAX as u16) as u8;
            carry = if sum > UCHAR_MAX as u16 { 1 } else { 0 };
    }
    carry != 0
}

/* The following function adds unsigned integers.  The function
   returns 1 if unsigned integer overflow (the first operand is less
   than the second) is fixed, 0 otherwise.  Result can be placed in
   any operand.
*/
fn ammunition_subtract_unsigned_integer_without_overflow_reaction(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
) -> bool {
    let mut carry: u8 = 0;
    // loopbound min 4 max 4
    for digit_num in (0..size).rev() {
            let sub = (op1[digit_num] as u16).wrapping_sub(op2[digit_num] as u16 + carry as u16);
            result[digit_num] = sub as u8;
            carry = if (op1[digit_num] as u16) < (op2[digit_num] as u16 + carry as u16) {
              1
            } else {
              0
            };
    }
    carry != 0
}

/* The following function makes complementary code of number.  Result
   can be placed in operand. */

fn ammunition_make_complementary_code(
    size: usize,
    operand: &[u8],
    result: &mut [u8],
) {
    let mut carry: u8 = 0;
    // loopbound min 2 max 6
    for digit_num in (0..size).rev() {
      let mut sub = (0u16)
                .wrapping_sub(operand[digit_num] as u16 + carry as u16);
      result[digit_num] = sub as u8;
      if sub != 0 {
        sub += UCHAR_MAX as u16 + 1;
        carry = 1;
      } else {
        carry = 0;
      }
    }
}

/* The following function multiplys unsigned integer by digit (byte
   size).  The function returns 1 if unsigned integer overflow is
   fixed, 0 otherwise.
*/
fn ammunition_multiply_unsigned_integer_by_digit_without_overflow_reaction(
    size: usize,
    operand: &mut [u8],
    digit: u32,
) -> bool {
    let mut carry: u32 = 0;
    // loopbound min 4 max 4
    for digit_num in (0..size).rev() {
        let sum = (operand[digit_num] as u32) * digit + carry;
        operand[digit_num] = (sum % 256) as u8;
        carry = sum / 256;
    }
    carry != 0
}

/* Originally reaction on all integer and unsigned integer overflow is
   equal to the following function.  The function does nothing. */

fn ammunition_arithmetic_overflow_reaction() {}

/* Originally reaction on all integer and unsigned integer overflow is
   equal to the following function.  The function does nothing. */

fn ammunition_arithmetic_unsigned_overflow_reaction() {}

/* This page contains functions for arbitrary precision addition. */

/* The function adds unsigned integers and fixes overflow reaction if
   it is needed.  The function makes this with the aid of function
   `add_unsigned_integer_without_overflow_reaction'.  Result can be
   placed in any operand. */

pub fn ammunition_add_unsigned_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
    *overflow_bit = ammunition_add_unsigned_integer_without_overflow_reaction(size, op1, op2, result);
    if *overflow_bit {
        ammunition_arithmetic_unsigned_overflow_reaction();
    }
}

/* The function adds integers and fixes overflow reaction if it is
   needed.  The function makes this with the aid of function
   `add_unsigned_integer_without_overflow_reaction'.  Result can be
   placed in any operand. */

pub fn ammunition_add_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
    overflow_bit: &mut bool,
) {

    let op1_sign = integer_sign(op1[0]);
    let sign_equality = integer_sign(op1[0]) == integer_sign(op2[0]);
    ammunition_add_unsigned_integer_without_overflow_reaction(size, op1, op2, result);
    *overflow_bit = sign_equality && op1_sign != integer_sign(result[0]);
    if *overflow_bit {
        ammunition_arithmetic_overflow_reaction();
    }
}

/* This page contains functions for arbitrary precision subtraction. */

/* The function subtracts unsigned integers and fixes overflow
   reaction if it is needed.  The function makes this with the aid of
   function `subtract_unsigned_integer_without_overflow_reaction'.
   Result can be placed in any operand. */

pub fn ammunition_subtract_unsigned_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
    *overflow_bit = ammunition_subtract_unsigned_integer_without_overflow_reaction(size, op1, op2, result);
    if *overflow_bit {
        ammunition_arithmetic_unsigned_overflow_reaction();
    }
}

/* The function subtracts integers and fixes overflow reaction if it
   is needed.  The function makes this with the aid of function
   `subtract_unsigned_integer_without_overflow_reaction'.  Result can
   be placed in any operand. */

pub fn ammunition_subtract_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
    let op1_sign = integer_sign(op1[0]);
    let sign_unequality = integer_sign(op1[0]) != integer_sign(op2[0]);
    ammunition_subtract_unsigned_integer_without_overflow_reaction(size, op1, op2, result);
    *overflow_bit = sign_unequality && (op1_sign != integer_sign(result[0]));
    if *overflow_bit {
        ammunition_arithmetic_overflow_reaction();
    }
}

/* This page contains functions for arbitrary precision multiplication. */

/* The following function multiplys unsigned integers.  The function
   returns 1 if unsigned integer overflow is fixed, 0 otherwise.
   Result can be placed in any operand. */

fn ammunition_multiply_unsigned_integer_without_overflow_reaction(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
) -> bool {
    let mut long_result = [0u8; 2 * MAX_INTEGER_OPERAND_SIZE];

    memset(&mut long_result[size..(size + size)], 0, size);
    // loopbound min 4 max 4
    for op2_digit_num in (0..size).rev() {
        if op2[op2_digit_num] != 0 {
            let mut carry: u16 = 0;
            // loopbound min 4 max 4
            for op1_digit_num in (0..size).rev() {
                let idx = op1_digit_num + op2_digit_num + 1;

                let partial_sum = op1[op1_digit_num] as u16
                    * op2[op2_digit_num] as u16
                    + long_result[idx] as u16
                    + carry;

                long_result[idx] = (partial_sum & UCHAR_MAX as u16) as u8;
                carry = partial_sum >> 8;
            }
            long_result[op2_digit_num] = carry as u8;
        } else {
            long_result[op2_digit_num] = 0;
        }
    }

    let mut overflow_flag = false;
    // loopbound min 0 max 4
    for result_digit_number in (0..size).rev() {
        if long_result[result_digit_number] != 0 {
            overflow_flag = true;
            break;
        }
    }
    memcpy(result, &long_result[size..], size);
    overflow_flag
}

/* The following function multiplys unsigned integers and fixes
   overflow reaction if it is needed.  The function makes this with
   the aid of function
   `multiply_unsigned_integer_without_overflow_reaction'.  Result can
   be placed in any operand. */

pub fn ammunition_multiply_unsigned_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
    *overflow_bit = ammunition_multiply_unsigned_integer_without_overflow_reaction(size, op1, op2, result);
    if *overflow_bit {
        ammunition_arithmetic_unsigned_overflow_reaction();
    }
}

/* The function multiplys integers and fixes overflow reaction if it
   is needed.  The function makes this with the aid of function
   `multiply_unsigned_integer_without_overflow_reaction'.  Result can
   be placed in any operand. */

pub fn ammunition_multiply_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
    let mut op1_complementary: [u8; MAX_INTEGER_OPERAND_SIZE] = [0; MAX_INTEGER_OPERAND_SIZE];
    let mut op2_complementary: [u8; MAX_INTEGER_OPERAND_SIZE] = [0; MAX_INTEGER_OPERAND_SIZE];
    let abs_op1: &[u8];
    let abs_op2: &[u8];

    let negative_result_flag = integer_sign(op1[0]) != integer_sign(op2[0]);
    if integer_sign(op1[0]) != 0 {
        /* May be integer overflow. But result is correct because it is unsigned. */
        ammunition_make_complementary_code(size, op1, &mut op1_complementary);
        abs_op1 = &op1_complementary;
    } else {
        abs_op1 = op1;
    }
    if integer_sign(op2[0]) != 0 {
        /* May be integer overflow.  But result is correct because it is unsigned. */
        ammunition_make_complementary_code(size, op2, &mut op2_complementary);
        abs_op2 = &op2_complementary;
    } else {
        abs_op2 = op2;
    }
    *overflow_bit = ammunition_multiply_unsigned_integer_without_overflow_reaction(size, abs_op1, abs_op2, result);
    let unsigned_result_sign = integer_sign(result[0]);
    if negative_result_flag {
      let mut result_copy: [u8; MAX_INTEGER_OPERAND_SIZE] = [0; MAX_INTEGER_OPERAND_SIZE];
      result_copy[..result.len()].copy_from_slice(result);
      let result_copy = &result_copy[..result.len()];
      ammunition_make_complementary_code(size, result_copy, result);
    }
    if unsigned_result_sign != 0
        && (!negative_result_flag || integer_sign(result[0]) != unsigned_result_sign)
    {
        /* Unsigned result can not be represented as integer. */
        *overflow_bit = true;
    }
    if *overflow_bit {
        ammunition_arithmetic_overflow_reaction();
    }
}

/* This page contains functions for arbitrary precision division. */

/* The following function divides unsigned integers.  The function
   returns 1 if unsigned integer overflow (division by zero) is fixed,
   0 otherwise.  Result can be placed in any operand.  See algorithm
   in Knuth's book. */

fn ammunition_divide_unsigned_integer_without_overflow_reaction(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
) -> bool {
    let mut scaled_op1: [u8; MAX_INTEGER_OPERAND_SIZE + 1] = [0; MAX_INTEGER_OPERAND_SIZE + 1];
    let mut normalized_op2: [u8; MAX_INTEGER_OPERAND_SIZE] = [0; MAX_INTEGER_OPERAND_SIZE];
    let mut extended_normalized_op2: [u8; MAX_INTEGER_OPERAND_SIZE + 1] = [0; MAX_INTEGER_OPERAND_SIZE + 1];

    // loopbound min 3 max 4
    let mut op2_digit_number = 0;
    while op2_digit_number < size {
        if op2[op2_digit_number] != 0 {
            break;
        }
        op2_digit_number += 1;
    }
    let first_nonzero_digit_number = op2_digit_number;
    if first_nonzero_digit_number == size {
        /* Zero divisor */
        memset(result, 0u8, size);
        return true /* TRUE */;
    } else {
        if first_nonzero_digit_number == size - 1 {
            /* Division by digit. */
            let digit = op2[first_nonzero_digit_number];
            memcpy(result, &op1, size);
            let mut remainder = 0;
            // loopbound min 4 max 4
            for digit_num in 0..size {
              let divisable: u32 = remainder as u32 * (UCHAR_MAX as u32 + 1) + result[digit_num] as u32;
              remainder = divisable % digit as u32;
              result[digit_num] = (divisable / digit as u32) as u8;
            }
            return false /* FALSE */;
        }
    }
    /* Normalization of divisor. */
    let scale = (UCHAR_MAX as u32 + 1) / (op2[op2_digit_number] + 1) as u32;
    memcpy(&mut scaled_op1[1..], &op1, size);
    scaled_op1[0] = 0;

    ammunition_multiply_unsigned_integer_by_digit_without_overflow_reaction(size + 1, &mut scaled_op1, scale);

    memcpy(&mut normalized_op2, &op2, size);

    ammunition_multiply_unsigned_integer_by_digit_without_overflow_reaction(size, &mut normalized_op2, scale);

    // loopbound min 0 max 0
    for scaled_op1_digit_num in 0..=first_nonzero_digit_number {
        /* Division of `scaled_op1[scaled_op1_digit_number]..scaled_op1[size]' by
          `normalized_op2[first_nonzero_digit_number]..normalized_op2[size-1]'
          for evaluation of one digit of quotient
          `result[size-1-first_nonzero_digit_number-scaled_op1_digit_number]'.
        */
        let mut q_approximation;
        if scaled_op1 [scaled_op1_digit_num] == normalized_op2 [first_nonzero_digit_number] {
            q_approximation = UCHAR_MAX as u64;
        } else {
            q_approximation
              = (scaled_op1[scaled_op1_digit_num] as u64 * (UCHAR_MAX as u64 + 1)
                + scaled_op1[scaled_op1_digit_num + 1] as u64)
              / normalized_op2[first_nonzero_digit_number] as u64;
        }
        // loopbound min 0 max 0
        while {
          let lhs = normalized_op2[first_nonzero_digit_number + 1] as u64 * q_approximation as u64;
          let rhs_i1 = scaled_op1[scaled_op1_digit_num] as u64 * (UCHAR_MAX as u64 + 1);
          let rhs_i2 = scaled_op1[scaled_op1_digit_num + 1] as u64;
          let rhs_i3 = q_approximation as u64 * normalized_op2[first_nonzero_digit_number] as u64;
          let rhs_i = rhs_i1 + rhs_i2 - rhs_i3;
          let rhs = rhs_i * (UCHAR_MAX as u64 + 1) + scaled_op1[scaled_op1_digit_num + 2] as u64;

          lhs > rhs
        } {
            q_approximation -= 1;
        }

        /* Multiply and subtract */
        memcpy(&mut extended_normalized_op2[1..],
            &normalized_op2[first_nonzero_digit_number..],
            size - first_nonzero_digit_number
        );
        extended_normalized_op2[0] = 0;
        ammunition_multiply_unsigned_integer_by_digit_without_overflow_reaction
        (size - first_nonzero_digit_number + 1, &mut extended_normalized_op2,
          q_approximation as u32);
        let scaled_op1_copy = scaled_op1;
        let cond = ammunition_subtract_unsigned_integer_without_overflow_reaction(
          size - first_nonzero_digit_number + 1,
          &scaled_op1_copy[scaled_op1_digit_num..], &extended_normalized_op2,
          &mut scaled_op1[scaled_op1_digit_num..]);
        if cond {
          /* Negative result.  Compensation by addition. */
          q_approximation -= 1;
          memcpy(&mut extended_normalized_op2[1..],
                              &normalized_op2[first_nonzero_digit_number..],
                              size - first_nonzero_digit_number);
          extended_normalized_op2[0] = 0;

          let scaled_op1_copy = scaled_op1;
          ammunition_add_unsigned_integer_without_overflow_reaction(
            size - first_nonzero_digit_number + 1,
            &scaled_op1_copy[scaled_op1_digit_num..], &extended_normalized_op2,
            &mut scaled_op1[scaled_op1_digit_num..]);

        }
        result[size - 1 - first_nonzero_digit_number + scaled_op1_digit_num] = q_approximation as u8;
    }
    memset(result, 0u8, size - 1 - first_nonzero_digit_number);
    return true /* TRUE */;
}

/* The function divides unsigned integers and fixes overflow reaction
   if it is needed.  The function makes this with the aid of function
   `divide_unsigned_integer_without_overflow_reaction'.  Result can be
   placed in any operand. */

pub fn ammunition_divide_unsigned_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
    *overflow_bit = ammunition_divide_unsigned_integer_without_overflow_reaction(size, op1, op2, result);
    if *overflow_bit {
        ammunition_arithmetic_unsigned_overflow_reaction();
    }
}

/* The function divides integers and fixes overflow reaction if it is
   needed.  The function makes this with the aid of function
   `divide_unsigned_integer_without_overflow_reaction'.  Result can be
   placed in any operand. */

pub fn ammunition_divide_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
    let mut op1_complementary: [u8; MAX_INTEGER_OPERAND_SIZE] = [0u8; MAX_INTEGER_OPERAND_SIZE];
    let mut op2_complementary: [u8; MAX_INTEGER_OPERAND_SIZE] = [0u8; MAX_INTEGER_OPERAND_SIZE];
    let abs_op1: &[u8];
    let abs_op2: &[u8];

    let negative_result_flag = integer_sign(op1[0]) != integer_sign(op2[0]);
    if integer_sign(op1[0]) != 0 {
        /* May be integer overflow for minimal int. But result is correct because
          it is unsigned. */
        ammunition_make_complementary_code(size, &op1, &mut op1_complementary);
        abs_op1 = &op1_complementary[..size];
    } else {
        abs_op1 = &op1[..size];
    }
    if integer_sign(op2[0]) != 0 {
        /* May be integer overflow for minimal int.  But result is correct
          because it is unsigned. */
        ammunition_make_complementary_code(size, &op2, &mut op2_complementary);
        abs_op2 = &op2_complementary[..size];
    } else {
        abs_op2 = &op2[..size];
    }
    *overflow_bit = ammunition_divide_unsigned_integer_without_overflow_reaction(size, abs_op1, abs_op2, result);

    let unsigned_result_sign = integer_sign(result[0]);
    if negative_result_flag {
        let mut result_copy: [u8; MAX_INTEGER_OPERAND_SIZE] = [0; MAX_INTEGER_OPERAND_SIZE];
        result_copy[..result.len()].copy_from_slice(result);
        let result_copy = &result_copy[..result.len()];
        ammunition_make_complementary_code(size, result_copy, result);
    }
    if unsigned_result_sign != 0
        && (!negative_result_flag || integer_sign(result[0]) != unsigned_result_sign)
    {
        /* Unsigned result can not be represented as integer. */
        *overflow_bit = true;
    }
    if *overflow_bit {
        ammunition_arithmetic_overflow_reaction();
    }
}

/* This page contains functions for arbitrary precision evaluation of
   remainder. */

/* The function evaluates remainder of division of unsigned integers
   as `op1 - (op1/op2)*op2' and fixes overflow reaction if it is
   needed.  Result can be placed in any operand. */

pub fn ammunition_unsigned_integer_remainder(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
    let mut temporary: [u8; MAX_INTEGER_OPERAND_SIZE] = [0; MAX_INTEGER_OPERAND_SIZE];

    ammunition_divide_unsigned_integer(size, op1, op2, &mut temporary, overflow_bit);
    if *overflow_bit {
        /* Reaction on zero is called from `divide_unsigned_integer'. */
        memset(result, 0u8, size);
    } else {
      let temporary_copy = temporary;
      ammunition_multiply_unsigned_integer(size, &temporary_copy, op2, &mut temporary, overflow_bit);
      ammunition_subtract_unsigned_integer(size, op1, &temporary, result, overflow_bit);
    }
}

/* This page contains functions for arbitrary precision number shifts. */

/* This function makes right shift of unsigned integer of given size
   on given number of bits.  If number of bits is negative the
   function makes shift to left actually with the aid of function
   `unsigned_integer_shift_left'.  The function fixes overflow when
   result can not be represented by number of given size, i.e. in
   other words the opposite unsigned shift (to left) results in number
   not equal to source operand.  Result can be placed in operand. */

pub fn ammunition_unsigned_integer_shift_right(
    size: usize,
    operand: &[u8],
    bits: i32,
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
    if bits < 0 {
        ammunition_unsigned_integer_shift_left(size, operand, -bits, result, overflow_bit);
    } else {    
        *overflow_bit = false;
        let byte_shift = bits / CHAR_BIT as i32;
        let bit_shift = bits % CHAR_BIT as i32;
        // loopbound min 0 max 2
        let mut byte_number: usize = if byte_shift >= size as i32 { 0 } else { size - byte_shift as usize };
        while (byte_number as usize) < size {
          if operand[byte_number as usize] != 0 {
              *overflow_bit = true;
              break;
          }
          byte_number += 1;
        }
          
        if byte_shift >= size as i32 {
            memset(result, 0u8, size);
        } else {
            memcpy(&mut result[(byte_shift as usize)..], operand, size - byte_shift as usize);
            memset(result, 0u8, byte_shift as usize);
            if bit_shift == 0 {
                return;
            }
            // loopbound min 3 max 3
            let mut carry: u32 = 0;
            for byte_number in (byte_shift as usize)..size {
                let byte = result[byte_number as usize];
                result[byte_number as usize] = carry as u8 | ((byte as u32)  >> bit_shift) as u8;
                carry = ((byte as u32) << (CHAR_BIT as i32 - bit_shift)) & UCHAR_MAX as u32;
            }
            if carry != 0 {
                *overflow_bit = true;
            }      
        }
        if *overflow_bit {
            ammunition_arithmetic_unsigned_overflow_reaction();
        }
    }
}

/* This function makes right arithmetic shift of integer of given size
   on given number of bits.  If number of bits is negative the
   function makes shift to left actually with the aid of function
   `integer_shift_left'.  The function fixes overflow when result can
   not be represented by number of given size, i.e. in other words the
   opposite shift (to left) results in number not equal to source
   operand.  Result can be placed in operand. */

pub fn ammunition_integer_shift_right(
    size: usize,
    operand: &[u8],
    bits: i32,
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
    if bits < 0 {
        ammunition_integer_shift_left(size, operand, -bits, result, overflow_bit);
    } else {
        let operand_sign = integer_sign(operand[0]);
        *overflow_bit = false;
        let byte_shift = bits / CHAR_BIT as i32;
        let bit_shift = bits % CHAR_BIT as i32;
        // loopbound min 0 max 2
        let mut byte_number: usize = if byte_shift >= size as i32 { 0 } else { size - byte_shift as usize };
        while byte_number < size {
            if operand[byte_number] != 0 {
                *overflow_bit = true;
                break;
            }
            byte_number += 1;
        } 
        if byte_shift >= size as i32 {
            let val = if operand_sign != 0 { UCHAR_MAX } else { 0 };
            memset(result, val, size);
        } else {
          memcpy(&mut result[(byte_shift as usize)..], operand, size - byte_shift as usize);
          memset(result, if operand_sign != 0 { UCHAR_MAX } else { 0 }, byte_shift as usize);
          if bit_shift == 0 {
            return;
          }
          let mut carry: u32 = (((if operand_sign != 0 { UCHAR_MAX } else { 0 }) << (CHAR_BIT as i32 - bit_shift)) & UCHAR_MAX) as u32;
          // loopbound min 3 max 3
          for byte_number in (byte_shift as usize)..size {
            let byte = result[byte_number];
            result[byte_number as usize] = carry as u8 | ((byte as u32) >> bit_shift) as u8;
            carry = ((byte as u32) << (CHAR_BIT as i32 - bit_shift)) & UCHAR_MAX as u32;
          }
          if carry != 0 {
            *overflow_bit = true;
          }
        }
        if *overflow_bit {
            ammunition_arithmetic_overflow_reaction();
        }
    }
}

/* This function makes left shift of unsigned integer of given size on
   given number of bits.  If number of bits is negative the function
   makes shift to left actually with the aid of function
   `unsigned_integer_shift_right'.  The function fixes overflow when
   result can not be represented by number of given size, i.e. i.e. in
   other words the opposite shift (to right) results in number not
   equal to source operand.  Result can be placed in operand. */

pub fn ammunition_unsigned_integer_shift_left(
    size: usize,
    operand: &[u8],
    bits: i32,
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
  if bits < 0 {
    ammunition_unsigned_integer_shift_right(size, operand, -bits, result, overflow_bit);
  } else {
    *overflow_bit = false;
    let byte_shift = bits / CHAR_BIT as i32;
    let bit_shift = bits % CHAR_BIT as i32;
    // loopbound min 0 max 2
    let upper = min(byte_shift as usize, size);
    for byte_number in 0..upper {
      if operand[byte_number] != 0 {
        *overflow_bit = true;
        break;
      }
    }
    if byte_shift >= size as i32 {
      memset(result, 0u8, size);
    } else {
      memcpy(result, &operand[(byte_shift as usize)..size], size - byte_shift as usize);
      memset(&mut result[(size - byte_shift as usize)..], 0u8, byte_shift as usize);
      if bit_shift == 0 {
        return;
      }
      let mut carry: u32 = 0;
      // loopbound min 2 max 3
      for byte_number in (0..=(size as i32 - byte_shift - 1)).rev() {
        let byte = result[byte_number as usize];
        result[byte_number as usize] = carry as u8 | ((byte as u32) << bit_shift) as u8;
        carry = (byte as u32) >> (CHAR_BIT as i32 - bit_shift);
      }
      if carry != 0 {
        *overflow_bit = true;
      }
    }
    if *overflow_bit {
      ammunition_arithmetic_unsigned_overflow_reaction();
    }
  }
}

/* This function makes left arithmetic shift of integer of given size
   on given number of bits.  If number of bits is negative the
   function makes shift to left actually with the aid of function
   `integer_shift_right'.  The function fixes overflow when result can
   not be represented by number of given size, i.e. in other words the
   opposite shift (to right) results in number not equal to source
   operand.  Result can be placed in operand. */

pub fn ammunition_integer_shift_left(
    size: usize,
    operand: &[u8],
    bits: i32,
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
  if bits < 0 {
    ammunition_integer_shift_right(size, operand, -bits, result, overflow_bit);
  } else {
    let operand_sign = integer_sign(operand[0]);
    *overflow_bit = false;
    let byte_shift = bits / CHAR_BIT as i32;
    let bit_shift = bits % CHAR_BIT as i32;
    // loopbound min 0 max 2
    let upper = min(byte_shift as usize, size);
    for byte_number in 0..upper {
      if operand[byte_number] != (if operand_sign != 0 { UCHAR_MAX } else { 0 }) {
        *overflow_bit = true;
        break;
      }
    }
    if byte_shift >= size as i32 {
      memset(result, 0u8, size);
    } else {
      memcpy(result, &operand[(byte_shift as usize)..size], size - byte_shift as usize);
      memset(&mut result[(size - byte_shift as usize)..], 0u8, byte_shift as usize);
      if bit_shift == 0 {
        return;
      }
      let mut carry: u32 = 0;
      // loopbound min 2 max 3
      for byte_number in (0..=(size - byte_shift as usize - 1)).rev() {
        let byte = result[byte_number];
        result[byte_number] = carry as u8 | ((byte as u32) << bit_shift) as u8;
        carry = (byte as u32) >> (CHAR_BIT as i32 - bit_shift);
      }
      if carry != ((if operand_sign != 0 { UCHAR_MAX } else { 0 }) >> (CHAR_BIT as i32 - bit_shift)) as u32 {
        *overflow_bit = true;
      }
    }
    if operand_sign != integer_sign(result[0]) {
      *overflow_bit = true;
    }
    if *overflow_bit {
      ammunition_arithmetic_overflow_reaction();
    }
  }
}

/* This page contains functions for bitwise operations of arbitrary
   precision numbers. */

/* This function makes bitwise `or' of two integers of given size. */

pub fn ammunition_integer_or(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
) {
  // loopbound min 4 max 4
  for byte_number in 0..size {
    result[byte_number] = op1[byte_number] | op2[byte_number];
  }
}

/* This function makes bitwise `or' of two unsigned integers of given
   size. */

pub fn ammunition_unsigned_integer_or(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
) {
    ammunition_integer_or(size, op1, op2, result);
}


/* This function makes bitwise `and' of two integers of given size. */

pub fn ammunition_integer_and(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
) {
  // loopbound min 4 max 4
  for byte_number in 0..size {
    result[byte_number] = op1[byte_number] & op2[byte_number];
  }
}

/* This function makes bitwise `and' of two unsigned integers of given
   size. */

pub fn ammunition_unsigned_integer_and(
    size: usize,
    op1: &[u8],
    op2: &[u8],
    result: &mut [u8],
) {
    ammunition_integer_and(size, op1, op2, result);
}


/* This function makes bitwise `not' of integer of given size. */

pub fn ammunition_integer_not(
    size: usize,
    operand: &[u8],
    result: &mut [u8],
) {
  // loopbound min 4 max 4
  for byte_number in 0..size {
    result[byte_number] = operand[byte_number] ^ UCHAR_MAX;
  }
}

/* This function makes bitwise `not' of unsigned integer of given
   size. */

pub fn ammunition_unsigned_integer_not(
    size: usize,
    operand: &[u8],
    result: &mut [u8],
) {
  ammunition_integer_not(size, operand, result)
}

/* This page contains functions for comparison of arbitrary precision
   numbers. */

/* This function compares two unsigned integers of given size on
   equality.  The function returns 1 if unsigned integers are equal, 0
   otherwise. */

pub fn ammunition_eq_unsigned_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
) -> bool {
    op1[..size] == op2[..size]
}

/* This function compares two integers of given size on equality.  The
   function returns 1 if integers are equal, 0 otherwise. */

pub fn ammunition_eq_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
) -> bool {
    op1[..size] == op2[..size]
}

/* This function compares two unsigned integers of given size on
   inequality.  The function returns 1 if unsigned integers are not
   equal, 0 otherwise. */

pub fn ammunition_ne_unsigned_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
) -> bool {
  op1[..size] != op2[..size]
}

/* This function compares two integers of given size on inequality.
   The function returns 1 if integers are not equal, 0 otherwise. */

pub fn ammunition_ne_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
) -> bool {
  op1[..size] != op2[..size]
}

/* This function compares two memory parts of given size on that the
   first operand is greater than the second.  The bytes are described
   as unsigned.  The function returns 1 if the first operand is
   greater than the second, - 1 if the first operand is less than the
   second, 0 otherwise. */

fn ammunition_bytes_comparison(
    op1: &[u8],
    op2: &[u8],
    mut size: usize,
) -> i32 {
  let mut str_i = 0;

  // loopbound min 1 max 4
  while size > 0 && op1[str_i] == op2[str_i] {
    str_i += 1;
    size -= 1;
  }
  if size <= 0 {
    return 0;
  } else if op1[str_i] > op2[str_i] {
    return 1;
  } else {
    return -1;
  }
}

/* This function compares two unsigned integers of given size on that
   the first operand is greater than the second.  The function returns
   1 if the first unsigned integer is greater than the second, 0
   otherwise. */

pub fn ammunition_gt_unsigned_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
) -> bool {
  ammunition_bytes_comparison(op1, op2, size) > 0
}

/* This function compares two integers of given size on that the first
   operand is greater than the second.  The function returns 1 if the
   first integer is greater than the second, 0 otherwise. */

pub fn ammunition_gt_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
) -> bool {
  if integer_sign(op1[0]) == 0 {
    if integer_sign(op2[0]) == 0 {
      return ammunition_bytes_comparison(op1, op2, size) > 0;
    } else {
      return true;
    }
  } else {
    if integer_sign(op2[0]) == 0 {
      return false;
    } else {
      return ammunition_bytes_comparison(op1, op2, size) > 0;
    }
  }
}

/* This function compares two unsigned integers of given size on that
   the first operand is less than the second.  The function returns 1
   if the first unsigned integer is less than the second, 0
   otherwise. */

pub fn ammunition_lt_unsigned_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
) -> bool {
  ammunition_bytes_comparison(op1, op2, size) < 0
}

/* This function compares two integers of given size on that the first
   operand is less than the second.  The function returns 1 if the
   first integer is less than the second, 0 otherwise. */

pub fn ammunition_lt_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
) -> bool {
  if integer_sign(op1[0]) == 0 {
    if integer_sign(op2[0]) == 0 {
      return ammunition_bytes_comparison(op1, op2, size) < 0;
    } else {
      return false;
    }
  } else {
    if integer_sign(op2[0]) == 0 {
      return true;
    } else {
      return ammunition_bytes_comparison(op1, op2, size) < 0;
    }
  }
}

/* This function compares two unsigned integers of given size on that
   the first operand is greater than or equal to the second.  The
   function returns 1 if the first unsigned integer is greater than or
   equal to the second, 0 otherwise. */

pub fn ammunition_ge_unsigned_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
) -> bool {
  ammunition_bytes_comparison(op1, op2, size) >= 0
}

/* This function compares two integers of given size on that the first
   operand is greater than or equal to the second.  The function
   returns 1 if the first integer is greater than or equal to the
   second, 0 otherwise. */

pub fn ammunition_ge_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
) -> bool {
  if integer_sign(op1[0]) == 0 {
    if integer_sign(op2[0]) == 0 {
      return ammunition_bytes_comparison(op1, op2, size) >= 0;
    } else {
      return true;
    }
  } else {
    if integer_sign(op2[0]) == 0 {
      return false;
    } else {
      return ammunition_bytes_comparison(op1, op2, size) >= 0;
    }
  }
}

/* This function compares two unsigned integers of given size on that
   the first operand is less than or equal to the second.  The
   function returns 1 if the first unsigned integer is less than or
   equal to the second, 0 otherwise. */

pub fn ammunition_le_unsigned_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
) -> bool {
  ammunition_bytes_comparison(op1, op2, size) <= 0
}

/* This function compares two integers of given size on that the first
   operand is less than or equal to the second.  The function returns
   1 if the first integer is less than or equal to the second, 0
   otherwise. */

pub fn ammunition_le_integer(
    size: usize,
    op1: &[u8],
    op2: &[u8],
) -> bool {
  if integer_sign(op1[0]) == 0 {
    if integer_sign(op2[0]) == 0 {
      return ammunition_bytes_comparison(op1, op2, size) <= 0;
    } else {
      return false; /*FALSE*/
    }
  } else {
    if integer_sign(op2[0]) == 0 {
      return true; /* TRUE */
    } else {
      return ammunition_bytes_comparison(op1, op2, size) <= 0;
    }
  }
}

/* This page contains functions for changing size of arbitrary
   precision numbers. */

/* The function changes size of unsigned integer.  The function fixes
   overflow when result can not be represented by number of given
   size.  Result can be placed in operand. */

pub fn ammunition_change_unsigned_integer_size(
    operand_size: usize,
    operand: &[u8],
    result_size: usize,
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
  *overflow_bit = false;
  if operand_size <= result_size {
    memcpy(&mut result[(result_size - operand_size)..], &operand, operand_size);
    memset(result, 0u8, result_size - operand_size);
  } else {
    // loopbound min 1 max 2
    for operand_digit_number in 0..(operand_size - result_size) {
      if operand[operand_digit_number] != 0 {
        *overflow_bit = true;
        break;
      }
    }
    memcpy(result, &operand[(operand_size - result_size)..], result_size);
  }
  if *overflow_bit {
    ammunition_arithmetic_unsigned_overflow_reaction();
  }
}

/* The function changes size of integer.  The function fixes overflow
   when result can not be represented by number of given size.  Result
   can be placed in operand. */

pub fn ammunition_change_integer_size(
    operand_size: usize,
    operand: &[u8],
    result_size: usize,
    result: &mut [u8],
    overflow_bit: &mut bool,
) {
  *overflow_bit = false;
  let operand_sign = integer_sign(operand[0]);
  if operand_size <= result_size {
    memcpy(&mut result[(result_size - operand_size)..], &operand, operand_size);
    memset(result, if operand_sign != 0 { UCHAR_MAX } else { 0 }, result_size - operand_size);
  } else {
    // loopbound min 2 max 2
    for operand_digit_number in 0..(operand_size - result_size) {
      if operand[operand_digit_number] != (if operand_sign != 0 { UCHAR_MAX } else { 0 }) {
        *overflow_bit = true;
        break;
      }
    }
    memcpy(result, &operand[(operand_size - result_size)..], result_size);
    if operand_sign != integer_sign(result[0]) {
      *overflow_bit = true;
    }
  }
  if *overflow_bit {
    ammunition_arithmetic_overflow_reaction();
  }
}

/* This page contains functions for conversion of arbitrary precision
   numbers to ascii representation. */

/* This function transforms unsigned integer of given size to BASE
   ascii representation.  BASE should be between 2 and 36 including
   them.  Digits more 9 are represented by 'a', 'b' etc.  Sign is
   absent in result string.  The function returns the result
   string. */

fn ammunition_unsigned_integer_to_based_string(
    size: usize,
    operand: &[u8],
    base: i32,
    result: &mut [u8],
) {
  let mut operand_copy: [u8; MAX_INTEGER_OPERAND_SIZE] = [0; MAX_INTEGER_OPERAND_SIZE];
  memcpy(&mut operand_copy, operand, size);
  let mut length = 0;
  // loopbound min 1 max 10
  loop {
    let mut nonzero_flag = false /* FALSE */;
    let mut remainder: u64 = 0;
    // loopbound min 2 max 6
    for digit_num in 0..size {
      let divisable = remainder * (UCHAR_MAX  as u64+ 1) + operand_copy[digit_num] as u64;
      remainder = divisable % base as u64;
      operand_copy[digit_num] =  (divisable / base as u64) as u8;
      if operand_copy[digit_num] != 0 {
        nonzero_flag = true /* TRUE */;
      }
    }
    result[length] = if remainder < 10 { b'0' + remainder as u8 } else { b'a' + remainder as u8 - 10 };
    length += 1;
    if !nonzero_flag { break; }
  }
  result[length] = b'\0';
  // loopbound min 0 max 5
  for i in 0..(length / 2) {
    let temporary = result[i];
    result[i] = result[length - i - 1];
    result[length - i - 1] = temporary;
  }
}


/* This function transforms unsigned integer of given size to decimal
   ascii representation.  Sign is absent in result string.  The
   function returns the result string. */


pub fn ammunition_unsigned_integer_to_string(
    size: usize,
    operand: &[u8],
    result: &mut [u8],
) {
  ammunition_unsigned_integer_to_based_string(size, operand, 10, result);
}


/* This function transforms integer of given size to BASE ascii
   representation.  BASE should be between 2 and 36 including them.
   Digits more 9 are represented by 'a', 'b' etc.  Sign is present in
   result string only for negative numbers.  The function returns the
   result string. */

fn ammunition_integer_to_based_string(
    size: usize,
    operand: &[u8],
    base: i32,
    result: &mut [u8],
) {
  let mut operand_copy = [0u8; MAX_INTEGER_OPERAND_SIZE];

  if integer_sign(operand[0]) == 0 {
    ammunition_unsigned_integer_to_based_string(size, operand, base, result);
    return;
  }
  memcpy(&mut operand_copy, &operand, size);
  /* May be integer overflow. But result is correct because it is unsigned. */
  ammunition_make_complementary_code(size, &operand, &mut operand_copy);
  result[0] = b'-';
  ammunition_unsigned_integer_to_based_string(size, &operand_copy, base, &mut result[1..]);
}

/* This function transforms integer of given size to decimal ascii
   representation.  Sign is present in result string only for negative
   numbers.  The function returns the result string. */

pub fn ammunition_integer_to_string(
    size: usize,
    operand: &[u8],
    result: &mut [u8],
) {
  ammunition_integer_to_based_string(size, operand, 10, result);
}

/* This page contains functions for conversion of decimal ascii
   representation to arbitrary precision numbers.  */

/* The function adds digit (byte size) to unsigned integer.  The
   function returns 1 if unsigned integer overflow is fixed, 0
   otherwise.
*/

fn ammunition_add_digit_to_unsigned_integer_without_overflow_reaction(
    size: usize,
    operand: &mut [u8],
    digit: u32,
) -> bool {
    let mut carry = digit;
    for byte in operand[..size].iter_mut().rev() {
        let sum = (*byte as u32) + carry;
        *byte = (sum % 256) as u8;
        carry = sum / 256;
    }
    carry != 0
}

/* This function transforms source string (decimal ascii
   representation without sign) to given size unsigned integer and
   returns pointer to first non digit in the source string through a
   parameter.  If the string started with invalid integer
   representation the result will be zero and returns the operand
   through the parameter.  The function returns 1 if unsigned integer
   overflow is fixed, 0 otherwise. */

fn ammunition_string_to_unsigned_integer_without_overflow_reaction(
    size: usize,
    operand: &[u8],
    result: &mut [u8],
    first_nondigit: &mut usize,
) -> bool {
  let mut overflow_flag = false;

  memset(result, 0u8,  size);
  let mut index = 0;
  // loopbound min 1 max 10
  while index < operand.len() && operand[index].is_ascii_digit() {
    overflow_flag = overflow_flag
      || ammunition_multiply_unsigned_integer_by_digit_without_overflow_reaction(size, result, 10);
    overflow_flag = overflow_flag
        || ammunition_add_digit_to_unsigned_integer_without_overflow_reaction(size, result, (operand[index] - b'0') as u32);
    index += 1;
  }
  *first_nondigit = index;
  overflow_flag
}

/* This function skips all white spaces at the begin of source string
   and transforms tail of the source string (decimal ascii
   representation without sign) to given size unsigned integer with
   the aid of function
   `string_to_unsigned_integer_without_overflow_reaction'.  If the
   string started with invalid unsigned integer representation the
   result will be zero.  The function fixes overflow when result can
   not be represented by number of given size.  The function returns
   address of the first nondigit in the source string. */

pub fn ammunition_unsigned_integer_from_string(
    size: usize,
    operand: &[u8],
    result: &mut [u8],
    overflow_bit: &mut bool,
) -> usize {
  let mut first_nondigit = 0;

  let mut index = 0;
  // loopbound min 0 max 0
  while index < operand.len() && operand[index].is_ascii_whitespace() {
    index += 1;
  }
  *overflow_bit = ammunition_string_to_unsigned_integer_without_overflow_reaction(
      size, &operand[index..], result, &mut first_nondigit,
  );
  if *overflow_bit {
    ammunition_arithmetic_unsigned_overflow_reaction();
  }
  first_nondigit
}

/* This function skips all white spaces at the begin of source string
   and transforms tail of the source string (decimal ascii
   representation with possible sign `+' or `-') to given size integer
   with the aid of function
   `string_to_unsigned_integer_without_overflow_reaction'.  If the
   string started with invalid integer representation the result will
   be zero.  The function fixes overflow when result can not be
   represented by number of given size.  the function returns Address
   of the first nondigit in the source string. */


pub fn ammunition_integer_from_string(
    size: usize,
    operand: &[u8],
    result: &mut [u8],
    overflow_bit: &mut bool,
) -> usize {
  let mut first_nondigit: usize = 0;

  let mut index = 0;
  // loopbound min 0 max 0
  while index < operand.len() && operand[index].is_ascii_whitespace() {
    index += 1;
  };
  let mut negative_number_flag = false; /* FALSE */
  if operand[index] == b'+' {
    index += 1;
  } else if operand[index] == b'-' {
    index += 1;
    negative_number_flag = true; /* TRUE */
  }
  *overflow_bit = ammunition_string_to_unsigned_integer_without_overflow_reaction(
    size, &operand[index..], result, &mut first_nondigit
  );

  let unsigned_result_sign = integer_sign(result[0]);
  if negative_number_flag {
    /* May be integer overflow when `result' is correct.  But result
       is correct because it is unsigned. */
    let mut result_copy: [u8; MAX_INTEGER_OPERAND_SIZE] = [0u8; MAX_INTEGER_OPERAND_SIZE];
    result_copy[..result.len()].copy_from_slice(result);
    let result_copy = &result_copy[..result.len()];
    ammunition_make_complementary_code(size, result_copy, result);
  }
  *overflow_bit = *overflow_bit
    || (unsigned_result_sign != 0
      && (!negative_number_flag
      || integer_sign(result[0]) != unsigned_result_sign));
  if *overflow_bit {
    ammunition_arithmetic_unsigned_overflow_reaction();
  }
  first_nondigit
}
