/*

  Rust Implementation

  Author: David Heim
  License: The Rust implementation shares the same license (BSD 3-clause) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.
  
  Version 2.0

  Name: pm_libm.c

  Author: Hector Chan
          MIT Lincoln Laboratory

  Function: This file contains the C math library functions used by pm.

  Source: HPEC Challenge Benchmark Suite, Pattern Match Kernel Benchmark

  Original name: pm

  Changes: See ChangeLog.txt

  License: BSD 3-clause

*/

#![allow(clippy::excessive_precision)]

const POW_COEFF: [f32; 19] = [
    0.5f32,             /* 1/2! */
    0.166666667f32,     /* 1/3! */
    0.041666666f32,     /* 1/4! */
    8.333333333e-3f32,
    1.388888889e-3f32,
    1.984126984e-4f32,
    2.480158730e-5f32,
    2.755731922e-6f32,
    2.755731922e-7f32,
    2.505210839e-8f32,
    2.087675699e-9f32,
    1.605904384e-10f32,
    1.147074560e-11f32,
    7.647163732e-13f32,
    4.779477332e-14f32,
    2.811457254e-15f32,
    1.561920697e-16f32,
    8.220635247e-18f32,
    4.110317623e-19f32,
];
const LOG_COEFF: [f32; 16] = [
    0.333333333f32,         /* 1/3 */
    0.2f32,                 /* 1/5 */
    0.142857143f32,         /* 1/7 */
    0.111111111f32,         /* 1/9 */
    9.090909091e-2f32,      /* 1/11 */
    7.692307692e-2f32,      /* 1/13 */
    6.666666667e-2f32,      /* 1/15 */
    5.882352941e-2f32,      /* 1/17 */
    5.263157895e-2f32,      /* 1/19 */
    4.761904762e-2f32,      /* 1/21 */
    4.347826087e-2f32,      /* 1/23 */
    0.04f32,                /* 1/25 */
    3.703703704e-2f32,      /* 1/27 */
    3.448275862e-2f32,      /* 1/29 */
    3.225806452e-2f32,      /* 1/31 */
    3.030303030e-2f32,      /* 1/33 */
];
#[expect(clippy::approx_constant)]
const LOG10: f32 = 2.302585093f32;

pub(super) fn pm_pow10f(exp: f32) -> f32 {
    let mut index: usize = 0;
    let term: f32 = exp * LOG10;
    let mut mul: f32 = exp * LOG10;
    let mut ans: f32 = 1.0f32;

    ans += mul;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += POW_COEFF[index] * mul;
    
    ans
}

pub(super) fn pm_log10f(exp: f32) -> f32 {
    let mut index: usize = 0;
    let mut mul: f32 = (exp - 1.) / (exp + 1.);
    let term: f32 = mul * mul;
    let mut ans: f32 = mul;

    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;
    index += 1;
    mul *= term;
    ans += LOG_COEFF[index] * mul;

    ans *= 0.86858896381f32;  /* ans = ans * 2 / log(10) */

    ans
}

pub(super) fn pm_floorf(arg: f32) -> f32 {
    if arg >= 0. {
        return (arg as i32) as f32;
    }
    (-((-arg) as i32 + 1)) as f32
}

pub(super) fn pm_ceilf(arg: f32) -> f32 {
    if arg > 0. {
        return ((arg + 1.) as i32) as f32;
    }
    (arg as i32) as f32
}