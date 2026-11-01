/*

  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license (GPL) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

  Version V 2.0

  Name: test3

  Author: Rathijit Sen

  Function: This testcase walks through parts of 8 arrays in a 10 x 10 grid of
    functions. function f_i_j calls f_i_j+1 and f_i+1,j except at the grid
    boundaries.

  Source: Universitaet des Saarlandes, Saarbruecken, Germany
          Compiler Research Group

  Original name: test3

  Changes:
          26-10-2007 Creation at Saarbruecken

  License: GPL

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

/* 
  overall changes:
  - moving global variables to call-by-reference vars necessitated additional array to keep parameter count low
  - remove heavy code duplication via more generic functions
*/

/*
  Initialization- and return-value-related functions
*/

/*
  "random" number generator
  in original: initSeed is NOT called
  in Rust version: RandomInt::init() is called in test3_init()
  -> this is analogous to st benchmark as "test3_randomInteger()" is the same as "st_randomInteger()" (barring int vs long return type)
*/

struct RandomInt(i32);

/*
  generates "random" integers between 0 and 8094.
*/
impl RandomInt {
    fn init() -> Self {
        RandomInt(0)
    }

    fn next(&mut self) -> i32 {
        self.0 = ((black_box(self.0) * 133) + 81) % 8095;
        self.0
    }
}

fn test3_init(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    let mut rand = RandomInt::init();

    // loopbound min 8 max 8
    arrays.iter_mut().for_each(|array| {
        // loopbound min 32 max 32
        array.iter_mut().for_each(|row| {
              // loopbound min 32 max 32
              row.iter_mut().for_each(|val| {
                  *val = rand.next();
              });  
        }); 
    });
    
    *result = black_box(0);
}

fn test3_return(result: &mut i32) -> i32 {
    *result
}

/*
  Algorithm core functions
*/

// iterator based
fn test3_func_base_it(
    array: &mut [[i32; 32]; 32],
    result: &mut i32,
    mut row: usize, mut col: usize
) {
    let mut x = 8;
    // loopbound min 4 max 4
    for _ in 0..4 {
        *result += (0..12)
            .fold(0, |acc, _| {
                let curr = array[row][col];  
                row += if col + x >= 32 { 1 } else { 0 };
                col = (col + x) % 32;
                acc + curr
            });
        x >>= 1;
    }
}

// loop based; close to original (e.g. loop unrolled)
#[allow(dead_code)]
fn test3_func_base_lo(
    array: &mut [[i32; 32]; 32],
    result: &mut i32,
    mut row: usize, mut col: usize
) {
    let mut x = 8;
    // loopbound min 4 max 4
    for _ in 0..4 {
        *result += array[row][col];
        row += if col + x >= 32 { 1 } else { 0 };
        col = (col + x) % 32;
        *result += array[row][col];
        row += if col + x >= 32 { 1 } else { 0 };
        col = (col + x) % 32;
        *result += array[row][col];
        row += if col + x >= 32 { 1 } else { 0 };
        col = (col + x) % 32;
        *result += array[row][col];
        row += if col + x >= 32 { 1 } else { 0 };
        col = (col + x) % 32;
        *result += array[row][col];
        row += if col + x >= 32 { 1 } else { 0 };
        col = (col + x) % 32;
        *result += array[row][col];
        row += if col + x >= 32 { 1 } else { 0 };
        col = (col + x) % 32;
        *result += array[row][col];
        row += if col + x >= 32 { 1 } else { 0 };
        col = (col + x) % 32;
        *result += array[row][col];
        row += if col + x >= 32 { 1 } else { 0 };
        col = (col + x) % 32;
        *result += array[row][col];
        row += if col + x >= 32 { 1 } else { 0 };
        col = (col + x) % 32;
        *result += array[row][col];
        row += if col + x >= 32 { 1 } else { 0 };
        col = (col + x) % 32;
        *result += array[row][col];
        row += if col + x >= 32 { 1 } else { 0 };
        col = (col + x) % 32;
        *result += array[row][col];
        row += if col + x >= 32 { 1 } else { 0 };
        col = (col + x) % 32;
        x >>= 1;
    }
}

// col 10
fn test3_func_10_10(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_base_it(arrays[4], result, 10, 10);
}

fn test3_func_9_10(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_base_it(arrays[3], result, 9, 10);
    test3_func_10_10(arrays, result);
}

fn test3_func_8_10(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_base_it(arrays[2], result, 8, 10);
    test3_func_9_10(arrays, result);
}

fn test3_func_7_10(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_base_it(arrays[1], result, 7, 10);
    test3_func_8_10(arrays, result);
}

fn test3_func_6_10(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_base_it(arrays[0], result, 6, 10);
    test3_func_7_10(arrays, result);
}

fn test3_func_5_10(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_base_it(arrays[7], result, 5, 10);
    test3_func_6_10(arrays, result);
}

fn test3_func_4_10(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_base_it(arrays[6], result, 4, 10);
    test3_func_5_10(arrays, result);
}

fn test3_func_3_10(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_base_it(arrays[5], result, 3, 10);
    test3_func_4_10(arrays, result);
}

fn test3_func_2_10(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_base_it(arrays[4], result, 2, 10);
    test3_func_3_10(arrays, result);
}

fn test3_func_1_10(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_base_it(arrays[3], result, 1, 10);
    test3_func_2_10(arrays, result);
}

fn test3_func_0_10(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_base_it(arrays[2], result, 0, 10);
    test3_func_1_10(arrays, result);
}

// col 9
fn test3_func_10_9(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_10_10(arrays, result);
    test3_func_base_it(arrays[3], result, 10, 9);
}

fn test3_func_9_9(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_9_10(arrays, result);
    test3_func_base_it(arrays[2], result, 9, 9);
    test3_func_10_9(arrays, result);
}

fn test3_func_8_9(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_8_10(arrays, result);
    test3_func_base_it(arrays[1], result, 8, 9);
    test3_func_9_9(arrays, result);
}

fn test3_func_7_9(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_7_10(arrays, result);
    test3_func_base_it(arrays[0], result, 7, 9);
    test3_func_8_9(arrays, result);
}

fn test3_func_6_9(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_6_10(arrays, result);
    test3_func_base_it(arrays[7], result, 6, 9);
    test3_func_7_9(arrays, result);
}

fn test3_func_5_9(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_5_10(arrays, result);
    test3_func_base_it(arrays[6], result, 5, 9);
    test3_func_6_9(arrays, result);
}

fn test3_func_4_9(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_4_10(arrays, result);
    test3_func_base_it(arrays[5], result, 4, 9);
    test3_func_5_9(arrays, result);
}

fn test3_func_3_9(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_3_10(arrays, result);
    test3_func_base_it(arrays[4], result, 3, 9);
    test3_func_4_9(arrays, result);
}

fn test3_func_2_9(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_2_10(arrays, result);
    test3_func_base_it(arrays[3], result, 2, 9);
    test3_func_3_9(arrays, result);
}

fn test3_func_1_9(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_1_10(arrays, result);
    test3_func_base_it(arrays[2], result, 1, 9);
    test3_func_2_9(arrays, result);
}

fn test3_func_0_9(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_0_10(arrays, result);
    test3_func_base_it(arrays[1], result, 0, 9);
    test3_func_1_9(arrays, result);
}

// col 8
fn test3_func_10_8(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_10_9(arrays, result);
    test3_func_base_it(arrays[2], result, 10, 8);
}

fn test3_func_9_8(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_9_9(arrays, result);
    test3_func_base_it(arrays[1], result, 9, 8);
    test3_func_10_8(arrays, result);
}

fn test3_func_8_8(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_8_9(arrays, result);
    test3_func_base_it(arrays[0], result, 8, 8);
    test3_func_9_8(arrays, result);
}

fn test3_func_7_8(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_7_9(arrays, result);
    test3_func_base_it(arrays[7], result, 7, 8);
    test3_func_8_8(arrays, result);
}

fn test3_func_6_8(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_6_9(arrays, result);
    test3_func_base_it(arrays[6], result, 6, 8);
    test3_func_7_8(arrays, result);
}

fn test3_func_5_8(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_5_9(arrays, result);
    test3_func_base_it(arrays[5], result, 5, 8);
    test3_func_6_8(arrays, result);
}

fn test3_func_4_8(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_4_9(arrays, result);
    test3_func_base_it(arrays[4], result, 4, 8);
    test3_func_5_8(arrays, result);
}

fn test3_func_3_8(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_3_9(arrays, result);
    test3_func_base_it(arrays[3], result, 3, 8);
    test3_func_4_8(arrays, result);
}

fn test3_func_2_8(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_2_9(arrays, result);
    test3_func_base_it(arrays[2], result, 2, 8);
    test3_func_3_8(arrays, result);
}

fn test3_func_1_8(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_1_9(arrays, result);
    test3_func_base_it(arrays[1], result, 1, 8);
    test3_func_2_8(arrays, result);
}

fn test3_func_0_8(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_0_9(arrays, result);
    test3_func_base_it(arrays[0], result, 0, 8);
    test3_func_1_8(arrays, result);
}

// col 7
fn test3_func_10_7(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_10_8(arrays, result);
    test3_func_base_it(arrays[1], result, 10, 7);
}

fn test3_func_9_7(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_9_8(arrays, result);
    test3_func_base_it(arrays[0], result, 9, 7);
    test3_func_10_7(arrays, result);
}

fn test3_func_8_7(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_8_8(arrays, result);
    test3_func_base_it(arrays[7], result, 8, 7);
    test3_func_9_7(arrays, result);
}

fn test3_func_7_7(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_7_8(arrays, result);
    test3_func_base_it(arrays[6], result, 7, 7);
    test3_func_8_7(arrays, result);
}

fn test3_func_6_7(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_6_8(arrays, result);
    test3_func_base_it(arrays[5], result, 6, 7);
    test3_func_7_7(arrays, result);
}

fn test3_func_5_7(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_5_8(arrays, result);
    test3_func_base_it(arrays[4], result, 5, 7);
    test3_func_6_7(arrays, result);
}

fn test3_func_4_7(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_4_8(arrays, result);
    test3_func_base_it(arrays[3], result, 4, 7);
    test3_func_5_7(arrays, result);
}

fn test3_func_3_7(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_3_8(arrays, result);
    test3_func_base_it(arrays[2], result, 3, 7);
    test3_func_4_7(arrays, result);
}

fn test3_func_2_7(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_2_8(arrays, result);
    test3_func_base_it(arrays[1], result, 2, 7);
    test3_func_3_7(arrays, result);
}

fn test3_func_1_7(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_1_8(arrays, result);
    test3_func_base_it(arrays[0], result, 1, 7);
    test3_func_2_7(arrays, result);
}

fn test3_func_0_7(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_0_8(arrays, result);
    test3_func_base_it(arrays[7], result, 0, 7);
    test3_func_1_7(arrays, result);
}

// col 6
fn test3_func_10_6(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_10_7(arrays, result);
    test3_func_base_it(arrays[0], result, 10, 6);
}

fn test3_func_9_6(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_9_7(arrays, result);
    test3_func_base_it(arrays[7], result, 9, 6);
    test3_func_10_6(arrays, result);
}

fn test3_func_8_6(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_8_7(arrays, result);
    test3_func_base_it(arrays[6], result, 8, 6);
    test3_func_9_6(arrays, result);
}

fn test3_func_7_6(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_7_7(arrays, result);
    test3_func_base_it(arrays[5], result, 7, 6);
    test3_func_8_6(arrays, result);
}

fn test3_func_6_6(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_6_7(arrays, result);
    test3_func_base_it(arrays[4], result, 6, 6);
    test3_func_7_6(arrays, result);
}

fn test3_func_5_6(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_5_7(arrays, result);
    test3_func_base_it(arrays[3], result, 5, 6);
    test3_func_6_6(arrays, result);
}

fn test3_func_4_6(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_4_7(arrays, result);
    test3_func_base_it(arrays[2], result, 4, 6);
    test3_func_5_6(arrays, result);
}

fn test3_func_3_6(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_3_7(arrays, result);
    test3_func_base_it(arrays[1], result, 3, 6);
    test3_func_4_6(arrays, result);
}

fn test3_func_2_6(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_2_7(arrays, result);
    test3_func_base_it(arrays[0], result, 2, 6);
    test3_func_3_6(arrays, result);
}

fn test3_func_1_6(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_1_7(arrays, result);
    test3_func_base_it(arrays[7], result, 1, 6);
    test3_func_2_6(arrays, result);
}

fn test3_func_0_6(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_0_7(arrays, result);
    test3_func_base_it(arrays[6], result, 0, 6);
    test3_func_1_6(arrays, result);
}

// col 5
fn test3_func_10_5(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_10_6(arrays, result);
    test3_func_base_it(arrays[7], result, 10, 5);
}

fn test3_func_9_5(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_9_6(arrays, result);
    test3_func_base_it(arrays[6], result, 9, 5);
    test3_func_10_5(arrays, result);
}

fn test3_func_8_5(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_8_6(arrays, result);
    test3_func_base_it(arrays[5], result, 8, 5);
    test3_func_9_5(arrays, result);
}

fn test3_func_7_5(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_7_6(arrays, result);
    test3_func_base_it(arrays[4], result, 7, 5);
    test3_func_8_5(arrays, result);
}

fn test3_func_6_5(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_6_6(arrays, result);
    test3_func_base_it(arrays[3], result, 6, 5);
    test3_func_7_5(arrays, result);
}

fn test3_func_5_5(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_5_6(arrays, result);
    test3_func_base_it(arrays[2], result, 5, 5);
    test3_func_6_5(arrays, result);
}

fn test3_func_4_5(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_4_6(arrays, result);
    test3_func_base_it(arrays[1], result, 4, 5);
    test3_func_5_5(arrays, result);
}

fn test3_func_3_5(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_3_6(arrays, result);
    test3_func_base_it(arrays[0], result, 3, 5);
    test3_func_4_5(arrays, result);
}

fn test3_func_2_5(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_2_6(arrays, result);
    test3_func_base_it(arrays[7], result, 2, 5);
    test3_func_3_5(arrays, result);
}

fn test3_func_1_5(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_1_6(arrays, result);
    test3_func_base_it(arrays[6], result, 1, 5);
    test3_func_2_5(arrays, result);
}

fn test3_func_0_5(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_0_6(arrays, result);
    test3_func_base_it(arrays[5], result, 0, 5);
    test3_func_1_5(arrays, result);
}

// col 4
fn test3_func_10_4(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_10_5(arrays, result);
    test3_func_base_it(arrays[6], result, 10, 4);
}

fn test3_func_9_4(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_9_5(arrays, result);
    test3_func_base_it(arrays[5], result, 9, 4);
    test3_func_10_4(arrays, result);
}

fn test3_func_8_4(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_8_5(arrays, result);
    test3_func_base_it(arrays[4], result, 8, 4);
    test3_func_9_4(arrays, result);
}

fn test3_func_7_4(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_7_5(arrays, result);
    test3_func_base_it(arrays[3], result, 7, 4);
    test3_func_8_4(arrays, result);
}

fn test3_func_6_4(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_6_5(arrays, result);
    test3_func_base_it(arrays[2], result, 6, 4);
    test3_func_7_4(arrays, result);
}

fn test3_func_5_4(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_5_5(arrays, result);
    test3_func_base_it(arrays[1], result, 5, 4);
    test3_func_6_4(arrays, result);
}

fn test3_func_4_4(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_4_5(arrays, result);
    test3_func_base_it(arrays[0], result, 4, 4);
    test3_func_5_4(arrays, result);
}

fn test3_func_3_4(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_3_5(arrays, result);
    test3_func_base_it(arrays[7], result, 3, 4);
    test3_func_4_4(arrays, result);
}

fn test3_func_2_4(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_2_5(arrays, result);
    test3_func_base_it(arrays[6], result, 2, 4);
    test3_func_3_4(arrays, result);
}

fn test3_func_1_4(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_1_5(arrays, result);
    test3_func_base_it(arrays[5], result, 1, 4);
    test3_func_2_4(arrays, result);
}

fn test3_func_0_4(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_0_5(arrays, result);
    test3_func_base_it(arrays[4], result, 0, 4);
    test3_func_1_4(arrays, result);
}

// col 3
fn test3_func_10_3(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_10_4(arrays, result);
    test3_func_base_it(arrays[5], result, 10, 3);
}

fn test3_func_9_3(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_9_4(arrays, result);
    test3_func_base_it(arrays[4], result, 9, 3);
    test3_func_10_3(arrays, result);
}

fn test3_func_8_3(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_8_4(arrays, result);
    test3_func_base_it(arrays[3], result, 8, 3);
    test3_func_9_3(arrays, result);
}

fn test3_func_7_3(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_7_4(arrays, result);
    test3_func_base_it(arrays[2], result, 7, 3);
    test3_func_8_3(arrays, result);
}

fn test3_func_6_3(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_6_4(arrays, result);
    test3_func_base_it(arrays[1], result, 6, 3);
    test3_func_7_3(arrays, result);
}

fn test3_func_5_3(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_5_4(arrays, result);
    test3_func_base_it(arrays[0], result, 5, 3);
    test3_func_6_3(arrays, result);
}

fn test3_func_4_3(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_4_4(arrays, result);
    test3_func_base_it(arrays[7], result, 4, 3);
    test3_func_5_3(arrays, result);
}

fn test3_func_3_3(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_3_4(arrays, result);
    test3_func_base_it(arrays[6], result, 3, 3);
    test3_func_4_3(arrays, result);
}

fn test3_func_2_3(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_2_4(arrays, result);
    test3_func_base_it(arrays[5], result, 2, 3);
    test3_func_3_3(arrays, result);
}

fn test3_func_1_3(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_1_4(arrays, result);
    test3_func_base_it(arrays[4], result, 1, 3);
    test3_func_2_3(arrays, result);
}

fn test3_func_0_3(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_0_4(arrays, result);
    test3_func_base_it(arrays[3], result, 0, 3);
    test3_func_1_3(arrays, result);
}

// col 2
fn test3_func_10_2(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_10_3(arrays, result);
    test3_func_base_it(arrays[4], result, 10, 2);
}

fn test3_func_9_2(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_9_3(arrays, result);
    test3_func_base_it(arrays[3], result, 9, 2);
    test3_func_10_2(arrays, result);
}

fn test3_func_8_2(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_8_3(arrays, result);
    test3_func_base_it(arrays[2], result, 8, 2);
    test3_func_9_2(arrays, result);
}

fn test3_func_7_2(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_7_3(arrays, result);
    test3_func_base_it(arrays[1], result, 7, 2);
    test3_func_8_2(arrays, result);
}

fn test3_func_6_2(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_6_3(arrays, result);
    test3_func_base_it(arrays[0], result, 6, 2);
    test3_func_7_2(arrays, result);
}

fn test3_func_5_2(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_5_3(arrays, result);
    test3_func_base_it(arrays[7], result, 5, 2);
    test3_func_6_2(arrays, result);
}

fn test3_func_4_2(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_4_3(arrays, result);
    test3_func_base_it(arrays[6], result, 4, 2);
    test3_func_5_2(arrays, result);
}

fn test3_func_3_2(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_3_3(arrays, result);
    test3_func_base_it(arrays[5], result, 3, 2);
    test3_func_4_2(arrays, result);
}

fn test3_func_2_2(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_2_3(arrays, result);
    test3_func_base_it(arrays[4], result, 2, 2);
    test3_func_3_2(arrays, result);
}

fn test3_func_1_2(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_1_3(arrays, result);
    test3_func_base_it(arrays[3], result, 1, 2);
    test3_func_2_2(arrays, result);
}

fn test3_func_0_2(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_0_3(arrays, result);
    test3_func_base_it(arrays[2], result, 0, 2);
    test3_func_1_2(arrays, result);
}

// col 1
fn test3_func_10_1(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_10_2(arrays, result);
    test3_func_base_it(arrays[3], result, 10, 1);
}

fn test3_func_9_1(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_9_2(arrays, result);
    test3_func_base_it(arrays[2], result, 9, 1);
    test3_func_10_1(arrays, result);
}

fn test3_func_8_1(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_8_2(arrays, result);
    test3_func_base_it(arrays[1], result, 8, 1);
    test3_func_9_1(arrays, result);
}

fn test3_func_7_1(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_7_2(arrays, result);
    test3_func_base_it(arrays[0], result, 7, 1);
    test3_func_8_1(arrays, result);
}

fn test3_func_6_1(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_6_2(arrays, result);
    test3_func_base_it(arrays[7], result, 6, 1);
    test3_func_7_1(arrays, result);
}

fn test3_func_5_1(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_5_2(arrays, result);
    test3_func_base_it(arrays[6], result, 5, 1);
    test3_func_6_1(arrays, result);
}

fn test3_func_4_1(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_4_2(arrays, result);
    test3_func_base_it(arrays[5], result, 4, 1);
    test3_func_5_1(arrays, result);
}

fn test3_func_3_1(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_3_2(arrays, result);
    test3_func_base_it(arrays[4], result, 3, 1);
    test3_func_4_1(arrays, result);
}

fn test3_func_2_1(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_2_2(arrays, result);
    test3_func_base_it(arrays[3], result, 2, 1);
    test3_func_3_1(arrays, result);
}

fn test3_func_1_1(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_1_2(arrays, result);
    test3_func_base_it(arrays[2], result, 1, 1);
    test3_func_2_1(arrays, result);
}

fn test3_func_0_1(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_0_2(arrays, result);
    test3_func_base_it(arrays[1], result, 0, 1);
    test3_func_1_1(arrays, result);
}

// col 0
fn test3_func_10_0(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_10_1(arrays, result);
    test3_func_base_it(arrays[2], result, 10, 0);
}

fn test3_func_9_0(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_9_1(arrays, result);
    test3_func_base_it(arrays[1], result, 9, 0);
    test3_func_10_0(arrays, result);
}

fn test3_func_8_0(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_8_1(arrays, result);
    test3_func_base_it(arrays[0], result, 8, 0);
    test3_func_9_0(arrays, result);
}

fn test3_func_7_0(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_7_1(arrays, result);
    test3_func_base_it(arrays[7], result, 7, 0);
    test3_func_8_0(arrays, result);
}

fn test3_func_6_0(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_6_1(arrays, result);
    test3_func_base_it(arrays[6], result, 6, 0);
    test3_func_7_0(arrays, result);
}

fn test3_func_5_0(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_5_1(arrays, result);
    test3_func_base_it(arrays[5], result, 5, 0);
    test3_func_6_0(arrays, result);
}

fn test3_func_4_0(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_4_1(arrays, result);
    test3_func_base_it(arrays[4], result, 4, 0);
    test3_func_5_0(arrays, result);
}

fn test3_func_3_0(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_3_1(arrays, result);
    test3_func_base_it(arrays[3], result, 3, 0);
    test3_func_4_0(arrays, result);
}

fn test3_func_2_0(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_2_1(arrays, result);
    test3_func_base_it(arrays[2], result, 2, 0);
    test3_func_3_0(arrays, result);
}

fn test3_func_1_0(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_1_1(arrays, result);
    test3_func_base_it(arrays[1], result, 1, 0);
    test3_func_2_0(arrays, result);
}

fn test3_func_0_0(arrays: &mut [&mut [[i32; 32]; 32]], result: &mut i32) {
    test3_func_0_1(arrays, result);
    test3_func_base_it(arrays[0], result, 0, 0);
    test3_func_1_0(arrays, result);
}

/*
  Main functions
*/

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn test3_main(
    arrays: &mut [&mut [[i32; 32]; 32]],
    result: &mut i32,
) {
    test3_func_0_0(arrays, result);
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 25;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn test3_log(
    arrays: &mut [&mut [[i32; 32]; 32]],
    result: &mut i32,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result: u32 = 0;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        test3_init(black_box(arrays), black_box(result));
        let start = black_box(DWT::cycle_count());
        test3_main(black_box(arrays), black_box(result));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("test3 - elapsed: {}", cycles);
    });
 
    test3_return(black_box(result)) - 1377313800 != 0
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn test3_std(
    arrays: &mut [&mut [[i32; 32]; 32]],
    result: &mut i32,
) -> bool {
    test3_init(black_box(arrays), black_box(result));
    test3_main(black_box(arrays), black_box(result));

    test3_return(black_box(result))- 1377313800 == 0
}

fn test3() -> bool{
    static mut ARRAY1: [[i32; 32]; 32] = [[0; 32]; 32];
    static mut ARRAY2: [[i32; 32]; 32] = [[0; 32]; 32];
    static mut ARRAY3: [[i32; 32]; 32] = [[0; 32]; 32];
    static mut ARRAY4: [[i32; 32]; 32] = [[0; 32]; 32];
    static mut ARRAY5: [[i32; 32]; 32] = [[0; 32]; 32];
    static mut ARRAY6: [[i32; 32]; 32] = [[0; 32]; 32];
    static mut ARRAY7: [[i32; 32]; 32] = [[0; 32]; 32];
    static mut ARRAY8: [[i32; 32]; 32] = [[0; 32]; 32];
    static mut RESULT: i32 = 0;

    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let test3_array1 = unsafe { &mut *{ &raw mut ARRAY1 } };
    let test3_array2 = unsafe { &mut *{ &raw mut ARRAY2 } };
    let test3_array3 = unsafe { &mut *{ &raw mut ARRAY3 } };
    let test3_array4 = unsafe { &mut *{ &raw mut ARRAY4 } };
    let test3_array5 = unsafe { &mut *{ &raw mut ARRAY5 } };
    let test3_array6 = unsafe { &mut *{ &raw mut ARRAY6 } };
    let test3_array7 = unsafe { &mut *{ &raw mut ARRAY7 } };
    let test3_array8 = unsafe { &mut *{ &raw mut ARRAY8 } };
    let mut test3_arrays = [
      test3_array1, test3_array2, test3_array3, test3_array4,
      test3_array5, test3_array6, test3_array7, test3_array8
    ];
    let test3_result = unsafe { &mut *{ &raw mut RESULT } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(test3_log(black_box(test3_arrays), black_box(test3_result)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(test3_std(black_box(&mut test3_arrays), black_box(test3_result)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(test3());

    #[cfg(feature = "log")]
    {
        rprintln!("test3: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(test3());

    #[cfg(feature = "log")]
    std::println!("test3: {}", success);

    std::process::exit(!success as i32);
}
