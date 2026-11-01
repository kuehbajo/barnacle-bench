/*

  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license as its origin.

  THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND,
  EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
  OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
  IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
  DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
  OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR
  THE USE OR OTHER DEALINGS IN THE SOFTWARE.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

   Copyright 1992 by Jutta Degener and Carsten Bormann, Technische
   Universitaet Berlin.  See the accompanying file "COPYRIGHT" for
   details.  THERE IS ABSOLUTELY NO WARRANTY FOR THIS SOFTWARE.
*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#![allow(overflowing_literals)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::ops::{ Mul, Shr };
use core::cmp::max;
use core::hint::black_box;

mod data;
use data::{ GsmSignal, GsmByte, GSM_ENC_PCMDATA, GSM_ENC_GSMDATA };

type Word = i16;        /* 16 bit signed int  */
type Longword = i32;    /* 32 bit signed int  */

struct GsmState {
    dp0: [Word; 280],

    z1: Word,                   /* preprocessing.c, Offset_com. */
    l_z2: Longword,             /*                  Offset_com. */
    mp: i16,                    /*                  Preemphasis */

    u: [Word; 8],               /* short_term_aly_filter.c  */
    larpp: [[Word; 8]; 2],     /*                              */
    j: Word,                    /*                              */
    _v: [Word; 9],              /* short_term.c, synthesis  */

    nrp: Word,                  /* 40 */ /* long_term.c, synthesis */
    _msr: Word,                 /* decoder.c, Postprocessing  */
}

impl GsmState {
    const fn new() -> Self {
        Self {
            dp0: [0; 280],
            z1: 0,
            l_z2: 0,
            mp: 0,
            u: [0; 8],
            larpp: [[0; 8]; 2],
            j: 0,
            _v: [0; 9],
            nrp: 0,
            _msr: 0,
        }
    }
}

impl Default for GsmState {
    fn default() -> Self {
        GsmState {
            dp0: [0; 280],
            z1: 0,
            l_z2: 0,
            mp: 0,
            u: [0; 8],
            larpp: [[0; 8]; 2],
            j: 0,
            _v: [0; 9],
            nrp: 0,
            _msr: 0,
        }
    }
}


const MIN_WORD: Word = -32768;
const MAX_WORD: Word = 32767;

/*   Table 4.3a  Decision level of the LTP gain quantizer
*/
/*                        bc       0      1      2      3      */
const GSM_ENC_DLB: [Word; 4] = [6554, 16384, 26214, 32767];


/*   Table 4.5   Normalized inverse mantissa used to compute xM/xmax
*/
/*                          i         0      1      2      3      4      5      6      7  */
const GSM_ENC_NRFAC: [Word; 8] = [29128, 26215, 23832, 21846, 20165, 18725, 17476, 16384];


/*   Table 4.6   Normalized direct mantissa used to compute xM/xmax
*/
/*                        i         0      1      2      3      4      5      6      7   */
const GSM_ENC_FAC: [Word; 8] = [18431, 20479, 22527, 24575, 26623, 28671, 30719, 32767];


/* gsm_enc_encode.c */
/*
   Copyright 1992 by Jutta Degener and Carsten Bormann, Technische
   Universitaet Berlin.  See the accompanying file "COPYRIGHT" for
   details.  THERE IS ABSOLUTELY NO WARRANTY FOR THIS SOFTWARE.
*/

// this is done to simulate the macros without std or a crate
// other option is to simply use saturating_* funcs without wrapping inlined call
pub trait SaturatingAdd: Copy { fn sat_add(self, rhs: Self) -> Self; }
pub trait SaturatingSub: Copy { fn sat_sub(self, rhs: Self) -> Self; }

macro_rules! impl_saturating_funcs {
    ($($t:ty),*) => {
        $(
            impl SaturatingAdd for $t {
                #[inline]
                fn sat_add(self, rhs: Self) -> Self {
                    self.saturating_add(rhs)
                }
            }
            impl SaturatingSub for $t {
                #[inline]
                fn sat_sub(self, rhs: Self) -> Self {
                    self.saturating_sub(rhs)
                }
            }
        )*
    };
}

impl_saturating_funcs!(i8, i16, i32, i64, i128, isize,
                       u8, u16, u32, u64, u128, usize);

/*
    Inlined functions from add.h
*/

#[inline]
fn sasr<T, U>(x: T, by: U) -> T
where
    T: Shr<U, Output = T>,
{
    x >> by
}

/* word a, word b, !(a == b == MIN_WORD) */
#[inline]
fn gsm_mult_r<T>(a: T, b: T) -> Word
where
    T: Into<i32>,
    T: Mul<T, Output = T>,
{
    let tmp: Longword = (a * b).into() + 16384;
    sasr(tmp, 15) as Word
}

/* word a, word b, !(a == b == MIN_WORD) */
#[inline]
fn gsm_mult<T>(a: T, b: T) -> Word
where
    T: Into<i32>,
    T: Mul<T, Output = T>,
{
    let tmp: Longword = (a * b).into();
    sasr(tmp, 15) as Word
}

#[inline]
pub fn gsm_add<T: SaturatingAdd>(a: T, b: T) -> T {
    a.sat_add(b)
}

#[inline]
pub fn gsm_sub<T: SaturatingSub>(a: T, b: T) -> T {
    a.sat_sub(b)
}

#[inline]
fn gsm_abs(a: Word) -> Word {
    if a == MIN_WORD { MAX_WORD } else { a.abs() }
}

#[inline]
fn saturate(x: Longword) -> Word {
    x.clamp(MIN_WORD as Longword, MAX_WORD as Longword) as Word
}

/* long_term.c */

const UMUL_TABLE_C: [[u16; 256]; 513] = [[0; 256]; 513];

#[inline]
fn umul(
  umul_table: &[[u16; 256]; 513],
  x9: Word,
  x15: Word,
) -> i16
{
    (umul_table[x9 as usize][(x15 & 0x0FF) as usize] + (umul_table[x9 as usize][(x15 >> 8) as usize] << 8)) as i16
} 

#[inline]
fn table_mul(
    umul_table: &[[u16; 256]; 513],
    a: Word,
    b: Word,
) -> i16 {
    if a < 0 {
        if b < 0 {
            umul(umul_table, -a, -b)
        } else {
            -umul(umul_table, -a, b)
      }
    } else {
        if b < 0 {
            -umul(umul_table, a, -b)
        } else {
            umul(umul_table, a, b)
        }
    }
}

/*
    Interface
*/

const GSM_MAGIC: Word = 0xD;        /* 13 kbit/s RPE-LTP */

const SAMPLES: usize = 20;

const GSM_ENC_BITOFF: [u8; 256] = [
    8, 7, 6, 6, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 4,
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
];

fn gsm_enc_encode(
    s: &mut GsmState,
    source: &mut [GsmSignal; 160],
    c: &mut [GsmByte; 33],
) {
    let mut larc: [Word; 8] = [0; 8];
    let nc: [Word; 4] = [0; 4];
    let mc: [Word; 4] = [0; 4];
    let bc: [Word; 4] = [0; 4];
    let xmaxc: [Word; 4] = [0; 4];
    let mut xmc: [Word; 13*4] = [0; 13*4];

    gsm_enc_gsm_coder(
        s,
        source,
        &mut larc,
        &nc,
        &bc,
        &mc,
        &xmaxc,
        &mut xmc
    );

    /*  variable  size

      GSM_MAGIC 4

      larc[0]   6
      larc[1]   6
      larc[2]   5
      larc[3]   5
      larc[4]   4
      larc[5]   4
      larc[6]   3
      larc[7]   3

      nc[0]   7
      bc[0]   2
      mc[0]   2
      xmaxc[0]  6
      xmc[0]    3
      xmc[1]    3
      xmc[2]    3
      xmc[3]    3
      xmc[4]    3
      xmc[5]    3
      xmc[6]    3
      xmc[7]    3
      xmc[8]    3
      xmc[9]    3
      xmc[10]   3
      xmc[11]   3
      xmc[12]   3

      nc[1]   7
      bc[1]   2
      mc[1]   2
      xmaxc[1]  6
      xmc[13]   3
      xmc[14]   3
      xmc[15]   3
      xmc[16]   3
      xmc[17]   3
      xmc[18]   3
      xmc[19]   3
      xmc[20]   3
      xmc[21]   3
      xmc[22]   3
      xmc[23]   3
      xmc[24]   3
      xmc[25]   3

      nc[2]   7
      bc[2]   2
      mc[2]   2
      xmaxc[2]  6
      xmc[26]   3
      xmc[27]   3
      xmc[28]   3
      xmc[29]   3
      xmc[30]   3
      xmc[31]   3
      xmc[32]   3
      xmc[33]   3
      xmc[34]   3
      xmc[35]   3
      xmc[36]   3
      xmc[37]   3
      xmc[38]   3

      nc[3]   7
      bc[3]   2
      mc[3]   2
      xmaxc[3]  6
      xmc[39]   3
      xmc[40]   3
      xmc[41]   3
      xmc[42]   3
      xmc[43]   3
      xmc[44]   3
      xmc[45]   3
      xmc[46]   3
      xmc[47]   3
      xmc[48]   3
      xmc[49]   3
      xmc[50]   3
      xmc[51]   3
    */

    let mut i: usize = 0;
    c[i] = (((GSM_MAGIC & 0xF) << 4) /* 1 */
            | ((larc[0] >> 2) & 0xF)) as u8;
    i += 1;
    c[i] = (((larc[0] & 0x3) << 6)
            | (larc[1] & 0x3F))  as u8;
    i += 1;
    c[i] = (((larc[2] & 0x1F) << 3)
            | ((larc[3] >> 2) & 0x7)) as u8;
    i += 1;
    c[i] = (((larc[3] & 0x3) << 6)
            | ((larc[4] & 0xF) << 2)
            | ((larc[5] >> 2) & 0x3)) as u8;
    i += 1;
    c[i] = (((larc[5] & 0x3) << 6)
            | ((larc[6] & 0x7) << 3)
            | (larc[7] & 0x7)) as u8;
    i += 1;
    c[i] = (((nc[0] & 0x7F) << 1)
            | ((bc[0] >> 1) & 0x1)) as u8;
    i += 1;
    c[i] = (((bc[0] & 0x1) << 7)
            | ((mc[0] & 0x3) << 5)
            | ((xmaxc[0] >> 1) & 0x1F)) as u8;
    i += 1;
    c[i] = (((xmaxc[0] & 0x1) << 7)
            | ((xmc[0] & 0x7) << 4)
            | ((xmc[1] & 0x7) << 1)
            | ((xmc[2] >> 2) & 0x1)) as u8;
    i += 1;
    c[i] = (((xmc[2] & 0x3) << 6)
            | ((xmc[3] & 0x7) << 3)
            | (xmc[4] & 0x7)) as u8;
    i += 1;
    c[i] = (((xmc[5] & 0x7) << 5)  /* 10 */
            | ((xmc[6] & 0x7) << 2)
            | ((xmc[7] >> 1) & 0x3)) as u8;
    i += 1;
    c[i] = (((xmc[7] & 0x1) << 7)
            | ((xmc[8] & 0x7) << 4)
            | ((xmc[9] & 0x7) << 1)
            | ((xmc[10] >> 2) & 0x1)) as u8;
    i += 1;
    c[i] = (((xmc[10] & 0x3) << 6)
            | ((xmc[11] & 0x7) << 3)
            | (xmc[12] & 0x7)) as u8;
    i += 1;
    c[i] = (((nc[1] & 0x7F) << 1)
            | ((bc[1] >> 1) & 0x1)) as u8;
    i += 1;
    c[i] = (((bc[1] & 0x1) << 7)
            | ((mc[1] & 0x3) << 5)
            | ((xmaxc[1] >> 1) & 0x1F)) as u8;
    i += 1;
    c[i] = (((xmaxc[1] & 0x1) << 7)
            | ((xmc[13] & 0x7) << 4)
            | ((xmc[14] & 0x7) << 1)
            | ((xmc[15] >> 2) & 0x1)) as u8;
    i += 1;
    c[i] = (((xmc[15] & 0x3) << 6)
            | ((xmc[16] & 0x7) << 3)
            | (xmc[17] & 0x7)) as u8;
    i += 1;
    c[i] = (((xmc[18] & 0x7) << 5)
            | ((xmc[19] & 0x7) << 2)
            | ((xmc[20] >> 1) & 0x3)) as u8;
    i += 1;
    c[i] = (((xmc[20] & 0x1) << 7)
            | ((xmc[21] & 0x7) << 4)
            | ((xmc[22] & 0x7) << 1)
            | ((xmc[23] >> 2) & 0x1)) as u8;
    i += 1;
    c[i] = (((xmc[23] & 0x3) << 6)
            | ((xmc[24] & 0x7) << 3)
            | (xmc[25] & 0x7)) as u8;
    i += 1;
    c[i] = (((nc[2] & 0x7F) << 1)  /* 20 */
            | ((bc[2] >> 1) & 0x1)) as u8;
    i += 1;
    c[i] = (((bc[2] & 0x1) << 7)
            | ((mc[2] & 0x3) << 5)
            | ((xmaxc[2] >> 1) & 0x1F)) as u8;
    i += 1;
    c[i] = (((xmaxc[2] & 0x1) << 7)
            | ((xmc[26] & 0x7) << 4)
            | ((xmc[27] & 0x7) << 1)
            | ((xmc[28] >> 2) & 0x1)) as u8;
    i += 1;
    c[i] = (((xmc[28] & 0x3) << 6)
            | ((xmc[29] & 0x7) << 3)
            | (xmc[30] & 0x7)) as u8;
    i += 1;
    c[i] = (((xmc[31] & 0x7) << 5)
            | ((xmc[32] & 0x7) << 2)
            | ((xmc[33] >> 1) & 0x3)) as u8;
    i += 1;
    c[i] = (((xmc[33] & 0x1) << 7)
            | ((xmc[34] & 0x7) << 4)
            | ((xmc[35] & 0x7) << 1)
            | ((xmc[36] >> 2) & 0x1)) as u8;
    i += 1;
    c[i] = (((xmc[36] & 0x3) << 6)
            | ((xmc[37] & 0x7) << 3)
            | (xmc[38] & 0x7)) as u8;
    i += 1;
    c[i] = (((nc[3] & 0x7F) << 1)
            | ((bc[3] >> 1) & 0x1)) as u8;
    i += 1;
    c[i] = (((bc[3] & 0x1) << 7)
            | ((mc[3] & 0x3) << 5)
            | ((xmaxc[3] >> 1) & 0x1F)) as u8;
    i += 1;
    c[i] = (((xmaxc[3] & 0x1) << 7)
            | ((xmc[39] & 0x7) << 4)
            | ((xmc[40] & 0x7) << 1)
            | ((xmc[41] >> 2) & 0x1)) as u8;
    i += 1;
    c[i] = (((xmc[41] & 0x3) << 6) /* 30 */
            | ((xmc[42] & 0x7) << 3)
            | (xmc[43] & 0x7)) as u8;
    i += 1;
    c[i] = (((xmc[44] & 0x7) << 5)
            | ((xmc[45] & 0x7) << 2)
            | ((xmc[46] >> 1) & 0x3)) as u8;
    i += 1;
    c[i] = (((xmc[46] & 0x1) << 7)
            | ((xmc[47] & 0x7) << 4)
            | ((xmc[48] & 0x7) << 1)
            | ((xmc[49] >> 2) & 0x1)) as u8;
    i += 1;
    c[i] = (((xmc[49] & 0x3) << 6)
            | ((xmc[50] & 0x7) << 3)
            | (xmc[51] & 0x7)) as u8;
}

/* decode.c */
/*
    4.3 FIXED POINT IMPLEMENTATION OF THE RPE-LTP DECODER
*/

/* code.c */
fn gsm_enc_gsm_coder(
    state: &mut GsmState,
    s: &mut [Word; 160],  /* [ 0..159 ] samples       IN  */
    /*
      The RPE-LTD coder works on a frame by frame basis.  The length of
      the frame is equal to 160 samples.  Some computations are done
      once per frame to produce at the output of the coder the
      larc[ 1..8 ] parameters which are the coded LAR coefficients and
      also to realize the inverse filtering operation for the entire
      frame (160 samples of signal d[ 0..159 ]).  These parts produce at
      the output of the coder:
    */
    larc: &mut [Word; 8], /* [ 0..7 ] LAR coefficients    OUT */
    /*
      Procedure 4.2.11 to 4.2.18 are to be executed four times per
      frame.  That means once for each sub-segment RPE-LTP analysis of
      40 samples.  These parts produce at the output of the coder:
    */
    nc: &[Word; 4],    /* [ 0..3 ] LTP lag     OUT   */
    bc: &[Word; 4],    /* [ 0..3 ] coded LTP gain    OUT   */

    mc: &[Word; 4],    /* [ 0..3 ] RPE grid selection    OUT     */
    xmaxc: &[Word; 4], /* [ 0..3 ] Coded maximum amplitude OUT */
    xmc: &mut [Word; 52]   /* [ 13*4 ] normalized RPE samples  OUT */
) {
    let mut e: [Word; 50] = [0; 50];

    let mut so: [Word; 160] = [0; 160];

    gsm_enc_gsm_preprocess(state, s, &mut so);
    gsm_enc_gsm_lpc_analysis(&mut so, larc);
    gsm_enc_gsm_short_term_analysis_filter(state, larc, &mut so);

    // loopbound min 4 max 4
    for k in 0..=3 {
        let mut nci: Word =  nc[k];
        let mut bci: Word = bc[k];

        let (left, right) = state.dp0.split_at_mut((k*40) + 120);
        let dp = &mut left[(k*40) .. (k*40) + 120]; /* [  -120...-1  ] */
        let dpp = &mut right[0 .. 40]; /* [  0...39  ]  */

        gsm_enc_gsm_long_term_predictor(
            &so[(k*40)..((k+1)*40)],     /* d      [ 0..39 ] IN */
            dp,   /* dp  [ -120..-1 ] IN  */
            &mut e,    /* e      [ 0..39 ] OUT */
            dpp,    /* dpp    [ 0..39 ] OUT */
            &mut nci,
            &mut bci,
        );

        let mut xmaxci: Word = xmaxc[k];
        let mut mci: Word = mc[k];
        gsm_enc_gsm_rpe_encoding(
            &mut e,  /* e     ][ 0..39 ][  IN/OUT */
            &mut xmaxci,
            &mut mci,
            &mut xmc[(k*13)..((k + 1) * 13)]
        );
        {
            // loopbound min 40 max 40
            for i in 0..=39 {
                dp[i] = gsm_add(e[5 + i], dpp[i]);
            }
        }
    }
}

/* rpe.c */
/*  4.2.13 .. 4.2.17  RPE ENCODING SECTION
*/

/* 4.2.13 */

fn gsm_enc_weighting_filter(
    e: &[Word],    /* signal [ -5..0.39.44 ] IN  */
    x: &mut [Word; 40]   /* signal [ 0..39 ] OUT */
) {
    /*
      The coefficients of the weighting filter are stored in a table
      (see table 4.4).  The following scaling is used:

      H[ 0..10 ] = integer( real_H[  0..10 ] * 8192 );
  */

    let mut l_result: Longword;

    /*  Initialization of a temporary working array wt[ 0...49 ] */

    let step = |i: usize, h: Word, k: usize| {
        (e[k + i] * h) as Longword
    };

    /*  Compute the signal x[ 0..39 ] */
    // loopbound min 40 max 40
    for k in 0..=39 {
        l_result = 8192 >> 1;
        /*  Every one of these multiplications is done twice --
            but I don't see an elegant way to optimize this.
            Do you?
        */
        l_result +=
            step(0, -134, k)
            + step(1, -374, k)
            + step(3,  2054, k)
            + step(4,  5741, k)
            + step(5,  8192, k)
            + step(6,  5741, k)
            + step(7,  2054, k)
            + step(9,  -374, k)
            + step(10,  -134, k);

        /*
            2 adds vs. >>16 => 14, minus one shift to compensate for
            those we lost when replacing L_MULT by '*'.
        */

        l_result = sasr(l_result, 13);
        x[k] = saturate(l_result);
    }
}

/* 4.2.14 */

fn gsm_enc_rpe_grid_selection(
    x: &[Word; 40],    /* [ 0..39 ]    IN  */
    xm: &mut [Word; 13],   /* [ 0..12 ]    OUT */
    mc_out: &mut Word,  /*      OUT */
) {
    /*
        The signal x[ 0..39 ] is used to select the RPE grid which is
        represented by mc.
    */
    let mut mc: Word = 0;
    let mut l_result: Longword = 0;
    let step = |m: Word, i: Word, l_result: &mut Longword| {
        let l_temp = sasr(x[(m + 3 * i) as usize], 2) as Longword;
        *l_result += l_temp * l_temp;
    };

    /* common part of 0 and 3 */
    for i in 1..13 {
        step(0, i, &mut l_result);
    }
    let l_common_0_3: Longword = l_result;

    /* i = 0 */

    step(0, 0, &mut l_result);
    l_result <<= 1; /* implicit in L_MULT */
    let mut em: Longword = l_result;    /* xxx should be L_EM? */

    /* i = 1 */

    l_result = 0;
    for i in 0..13 {
        step(1, i, &mut l_result);
    }
    l_result <<= 1;
    if l_result > em {
        mc = 1;
        em = l_result;
    }

    /* i = 2 */

    l_result = 0;
    for i in 0..13 {
        step(2, i, &mut l_result);
    }
    l_result <<= 1;
    if l_result > em {
        mc = 2;
        em = l_result;
    }

    /* i = 3 */

    l_result = l_common_0_3;
    step(3, 12, &mut l_result);
    l_result <<= 1;
    if l_result > em {
        mc = 3;
    }

    /**/

    /*  Down-sampling by a factor 3 to get the selected xM[ 0..12 ] RPE sequence. */
    // loopbound min 13 max 13
    for i in 0..=12 {
        xm[i] = x[mc as usize + 3 * i];
    }
    *mc_out = mc;
}

/* 4.12.15 */

fn gsm_enc_apcm_quantization_xmaxc_to_exp_mant(xmaxc: Word) -> (Word, Word) {
    /* Compute exponent and mantissa of the decoded version of xmaxc */
    let mut exp = if xmaxc > 15 { sasr(xmaxc, 3) - 1 } else { 0 };
    let mut mant = xmaxc - (exp << 3);

    if mant == 0 {
        exp  = -4;
        mant = 7;
    } else {
        // loopbound min 0 max 3
        while mant <= 7 {
            mant = mant << 1 | 1;
            exp -= 1;
        }
        mant -= 8;
    }
    (exp, mant)
}

fn gsm_enc_apcm_quantization(
    xm: &mut [Word; 13],   /* [ 0..12 ]    IN  */
    xmc: &mut [Word],    /* [ 0..12 ]    OUT */
) -> (Word, Word, Word)
{
    /*  Find the maximum absolute value xmax of xM[ 0..12 ]. */

    // loopbound min 13 max 13
    let xmax: Word = xm.iter().map(|x| gsm_abs(*x)).max().unwrap_or(0);

    /*  Qantizing and coding of xmax to get xmaxc. */

    let mut exp: Word = 0;
    let mut temp: Word = sasr(xmax, 9);
    let mut itest: Word = 0;

    // loopbound min 6 max 6
    for _ in 0..=5 {
        itest |= (temp <= 0) as Word;
        temp = sasr(temp, 1);

        if itest == 0 { exp += 1; }  // exp = add (exp, 1)
    }

    temp = exp + 5;

    let xmaxc: Word = saturate(sasr(xmax as Longword, temp) + (exp << 3) as Longword);

    /*   Quantizing and coding of the xM[ 0..12 ] RPE sequence to get the xmc[ 0..12 ] */

    let (exp, mant): (Word, Word) = gsm_enc_apcm_quantization_xmaxc_to_exp_mant(xmaxc);

    /*  This computation uses the fact that the decoded version of xmaxc
        can be calculated by using the exponent and the mantissa part of
        xmaxc (logarithmic table).
        So, this method avoids any division and uses only a scaling
        of the RPE samples by a function of the exponent.  A direct
        multiplication by the inverse of the mantissa (NRFAC[ 0..7 ]
        found in table 4.5) gives the 3 bit coded version xmc[ 0..12 ]
        of the RPE samples.
    */


    /* Direct computation of xmc[ 0..12 ] using table 4.5 */

    let temp1: Word = 6 - exp;          /* normalization by the exponent */
    let temp2 = GSM_ENC_NRFAC[mant as usize];    /* inverse mantissa      */

    // loopbound min 13 max 13
    for i in 0..=12 {
        let mut temp = xm[i] << temp1;
        temp = gsm_mult(temp, temp2);
        temp = sasr(temp, 12);
        xmc[i] = temp + 4;    /* see note below */
    }

    /*  NOTE: This equation is used to make all the xmc[ i ] positive. */

    (mant, exp, xmaxc)
}

/* 4.2.16 */

fn gsm_enc_apcm_inverse_quantization(
    xmc: &[Word],    /* [ 0..12 ]      IN  */
    mant: Word,
    exp: Word,
    xmp: &mut [Word; 13],    /* [ 0..12 ]      OUT   */
) {
    /*
        This part is for decoding the RPE sequence of coded xmc[ 0..12 ]
        samples to obtain the xmp[ 0..12 ] array.  Table 4.6 is used to get
        the mantissa of xmaxc (FAC[ 0..7 ]).
    */
    let temp1: Word = GSM_ENC_FAC[mant as usize];  /* see 4.2-15 for mant */
    let temp2: Word = gsm_sub(6, exp);  /* see 4.2-15 for exp  */
    let temp3: Word = gsm_enc_asl(1, gsm_sub(temp2, 1));

    // loopbound min 13 max 13
    for i in 0..13 {
        let mut temp: Word = (xmc[i] << 1) - 7;       /* restore sign   */

        temp <<= 12;        /* 16 bit signed  */
        temp = gsm_mult_r(temp1, temp);
        temp = gsm_add(temp, temp3);

        xmp[i] = gsm_enc_asr(temp, temp2);
    }
}

/* 4.2.17 */

fn gsm_enc_rpe_grid_positioning(
    mc: Word,           /* grid position  IN  */
    xmp: &mut [Word; 13],    /* [ 0..12 ]    IN  */
    ep: &mut [Word],     /* [ 0..39 ]    OUT */
) {
    /*
        This procedure computes the reconstructed long term residual signal
        ep[ 0..39 ] for the LTP analysis filter.  The inputs are the mc
        which is the grid position selection and the xMp[ 0..12 ] decoded
        RPE samples which are upsampled by a factor of 3 by inserting zero
        values.
    */
    let mut i = 13;
    
    ep.fill(0);
    if 0 <= mc && mc <= 3 {
        ep[mc as usize] = xmp[0];
        i -= 1;
    }

    let offset_ep: usize = (mc + 1) as usize;
    let mut offset_xmp = 1;
    // loopbound min 12 max 12
    for k in 0..i {
        ep[3 * k + 2 + offset_ep] = xmp[offset_xmp];
        offset_xmp += 1;
    }
}

/* 4.2.18 */

fn gsm_enc_gsm_rpe_encoding(
    e: &mut [Word],          /* -5..-1 ][ 0..39 ][ 40..44  IN/OUT  */
    xmaxc: &mut Word,           /*        OUT */
    mc: &mut Word,              /*          OUT */
    xmc: &mut [Word],        /* [ 0..12 ]      OUT */
) {
    let mut x: [Word; 40] = [0; 40];
    let mut xm: [Word; 13] = [0; 13];
    let mut xmp: [Word; 13] = [0; 13];

    gsm_enc_weighting_filter(&e, &mut x);
    gsm_enc_rpe_grid_selection(&mut x, &mut xm, mc);

    let (mant, exp, xmax) = gsm_enc_apcm_quantization(&mut xm, xmc);
    *xmaxc = xmax;
    gsm_enc_apcm_inverse_quantization(xmc,  mant,  exp, &mut xmp);

    gsm_enc_rpe_grid_positioning(*mc, &mut xmp, e);
}

/*
    4.2.11 .. 4.2.12 LONG TERM PREDICTOR (LTP) SECTION
*/


/*
   This procedure computes the LTP gain (bc) and the LTP lag (nc)
   for the long term analysis filter.   This is done by calculating a
   maximum of the cross-correlation function between the current
   sub-segment short term residual signal d[ 0..39 ] (output of
   the short term analysis filter; for simplification the index
   of this array begins at 0 and ends at 39 for each sub-segment of the
   RPE-LTP analysis) and the previous reconstructed short term
   residual signal dp[  -120 .. -1  ].  A dynamic scaling must be
   performed to avoid overflow.
*/

/* This procedure exists in four versions.  First, the two integer
   versions with or without table-multiplication (as one function);
   then, the two floating point versions (as another function), with
   or without scaling.
*/

trait LtpSpecific {
    type Elem: Copy + PartialOrd;
    type Corr: Copy + AsRef<[Self::Elem]>;

    const BATCH_SIZE: usize;    // 1: word  9: float
    const TABLE_MUL: bool = false;
    const TABLE: [[u16; 256]; 513] = UMUL_TABLE_C;

    fn zero() -> Self::Elem;
    fn to_longword(x: Self::Elem) -> Longword;
    fn convert_d(x: Word, scal: Word) -> Self::Elem;
    fn convert_dp(x: Word) -> Self::Elem;
    fn correlate(
      wt: &[Self::Elem; 40],
      dp: &[Self::Elem],
      lambda: usize
    ) -> Self::Corr;
}

struct IntSpecific;

impl LtpSpecific for IntSpecific {
    type Elem = Word;
    type Corr = [Word; 1];

    const BATCH_SIZE: usize = 1;
    const TABLE_MUL: bool = true;

    fn zero() -> Self::Elem { 0 }
    fn to_longword(x: Self::Elem) -> Longword { x as Longword }
    fn convert_d(x: Word, scal: Word) -> Self::Elem { sasr(x, scal) }
    fn convert_dp(x: Word) -> Self::Elem { x }

    fn correlate(
        wt: &[Self::Elem; 40],
        dp: &[Self::Elem],
        lambda: usize,
    ) -> Self::Corr
    {
        let mut acc: Word = 0;
        for k in 0..40 {
            if Self::TABLE_MUL {
                acc += table_mul(&Self::TABLE, wt[k], dp[120 - lambda + k]);
            } else {
                acc += ((wt[k] as Longword) * (dp[120 - lambda + k] as Longword)) as Word;
            }
        }
        [acc]
    }
}

pub struct FloatSpecific;

impl LtpSpecific for FloatSpecific {
    type Elem = f32;
    type Corr = [f32; 9];

    const BATCH_SIZE: usize = 9;

    fn zero() -> Self::Elem { 0.0 }
    fn to_longword(x: Self::Elem) -> Longword { x as Longword }
    fn convert_d(x: Word, scal: Word) -> Self::Elem { sasr(x, scal) as f32 }
    fn convert_dp(x: Word) -> Self::Elem { x as f32 }

    fn correlate(
        wt: &[Self::Elem; 40],
        dp: &[Self::Elem],
        lambda: usize,
    ) -> Self::Corr
    {
        let mut a = dp[120 - lambda - 8];
        let b = dp[120 - lambda - 7];
        let c = dp[120 - lambda - 6];
        let d = dp[120 - lambda - 5];
        let e = dp[120 - lambda - 4];
        let f = dp[120 - lambda - 3];
        let g = dp[120 - lambda - 2];
        let h = dp[120 - lambda - 1];

        let mut s = [0.0f32; 9];

        for k in 0..40 {
            let w = wt[k];

            s[8] += w * a;
            s[7] += w * b;
            s[6] += w * c;
            s[5] += w * d;
            s[4] += w * e;
            s[3] += w * f;
            s[2] += w * g;
            s[1] += w * h;

            a = dp[120 - lambda + k];

            s[0] += w * a;
        }

        s
    }

}

fn gsm_enc_calculation_of_the_ltp_parameters<T: LtpSpecific>
(
    d: &[Word],
    dp: &[Word],
    bc_out: &mut Word,
    nc_out: &mut Word,
) {
    /*  Search of the optimum scaling of d[ 0..39 ]. */
    
    // loopbound min 40 max 40
    let dmax: Word = d.iter().map(|x| gsm_abs(*x)).max().unwrap_or(0);
    let temp: Word = if dmax != 0 { gsm_enc_norm((dmax as Longword) << 16) } else { 0 };
    let scal: Word = max(6 - temp, 0);

    /*  Initialization of a working array wt */
    let mut wt = [T::convert_d(0, 0); 40];
    // loopbound min 40 max 40
    for (k, w) in wt.iter_mut().enumerate() {
        *w = T::convert_d(d[k], scal);
    }

    /* Conversion of dp */
    let mut dp_c: [T::Elem; 120] = [T::convert_dp(0); 120];
    // loopbound min 120 max 120
    for (i, x) in dp.iter().enumerate() {
        dp_c[i] = T::convert_dp(*x);
    }

    /* Search for the maximum cross-correlation and coding of the LTP lag */
    let mut l_max: T::Elem = T::zero();
    let mut nc: Word = 40;          /* index for the maximum cross-correlation */

    // word:  loopbound min 81 max 81
    // float: loopbound min 9 max 9
    let step = T::BATCH_SIZE;
    let mut lambda = 40;
    while lambda <= 120 {
        let corr = T::correlate(&wt, &dp_c, lambda);
        for (offset, &val) in corr.as_ref().iter().enumerate() {
            let lag = lambda + offset;
            if lag > 120 { break; }
            if val > l_max {
                l_max = val;
                nc = lag as Word;
            }
        }
        lambda += step;
    }


    *nc_out = nc as Word;

    let mut l_max: Longword = T::to_longword(l_max) << 1;

    /*  Rescaling of L_max */
    l_max = l_max >> (6 - scal);     /* sub(6, scal) */

    /*   Compute the power of the reconstructed short term residual signal dp[ .. ] */
    let mut l_power: Longword = 0;
    // loopbound min 40 max 40
    for k in 0..40 {
        let l_temp: Longword = sasr(dp[(k + 120 - nc) as usize] as Longword, 3);
        l_power += l_temp * l_temp;
    }
    l_power *= 2;   /* from L_MULT */

    /*  Normalization of L_max and L_power */
  
    if l_max <= 0 {
        *bc_out = 0;
        return;
    }
    if l_max >= l_power {
        *bc_out = 3;
        return;
    }

    let temp = gsm_enc_norm(l_power);

    let r: i32 = sasr(l_max << temp, 16);
    let s = sasr(l_power << temp, 16);

    /*  Coding of the LTP gain */

    /*  Table 4.3a must be used to obtain the level DLB[ i ] for the
        quantization of the LTP gain b to get the coded version bc.
    */
    // loopbound min 3 max 3
    let mut bc: Word = 0;
    for _ in 0..=2 {
        if r <= gsm_mult(s, GSM_ENC_DLB[bc as usize] as Longword) as Longword {
            bc += 1;
            break;
        }
    }
    *bc_out = bc;
}

/* 4.2.12 */

fn gsm_enc_long_term_analysis_filtering(
    bc: Word,           /*          IN  */
    nc: Word,           /*          IN  */
    dp: &[Word],    /* previous d [ -120..-1 ]    IN  */
    d: &[Word],      /* d    [ 0..39 ]     IN  */
    dpp: &mut [Word],  /* estimate [ 0..39 ]     OUT */
    e: &mut [Word] /* long term res. signal [ 0..39 ]  OUT */
) {
    /*
        In this part, we have to decode the bc parameter to compute
        the samples of the estimate dpp[ 0..39 ].  The decoding of bc needs the
        use of table 4.3b.  The long term residual signal e[ 0..39 ]
        is then calculated to be fed to the RPE encoding section.
    */

    let mut step = |bp: Word| {
        // loopbound min 40 max 40
        for k in 0..=39 {
            dpp[k]  = gsm_mult_r(bp, dp[120 + k - nc as usize]);
            e[k]  = gsm_sub(d[k], dpp[k]);
        }
    };

    match bc {
        0 => {
            step(3277);
        },
        1 => {
            step(11469);
        },
        2 => {
            step(21299);
        },
        3 => {
            step(32767);
        },
        _ => {}
    }
}

fn gsm_enc_gsm_long_term_predictor(
    d: &[Word],  /* [ 0..39 ]   residual signal  IN  */
    dp: &[Word], /* [ -120..-1 ] d'    IN  */

    e: &mut [Word],  /* [ 0..39 ]      OUT */
    dpp: &mut [Word],  /* [ 0..39 ]      OUT */
    nc: &mut Word, /* correlation lag    OUT */
    bc: &mut Word  /* gain factor      OUT */
) {
    gsm_enc_calculation_of_the_ltp_parameters::<IntSpecific>(d, dp, bc, nc);

    gsm_enc_long_term_analysis_filtering(*bc, *nc, dp, d, dpp, e);
}

/* short_term.c */
/*
    SHORT TERM ANALYSIS FILTERING SECTION
*/

/* 4.2.8 */

fn gsm_enc_decoding_of_the_coded_log_area_ratios(
    larc: &mut [Word; 8],   /* coded log area ratio [ 0..7 ]  IN  */
    larpp: &mut [Word; 8]   /* out: decoded ..      */
) {
    /*  This procedure requires for efficient implementation
        two tables.

        INVA[ 1..8 ] = integer( (32768 * 8) / real_A[ 1..8 ])
        MIC[ 1..8 ]  = minimum value of the larc[ 1..8 ]
    */

    let mut i: usize = 0;
    /*  Compute the larpp[ 1..8 ] */
    let mut step = |b: Word, mic: Word, inva: Word| {
        let mut temp1 = gsm_add(larc[i], mic) << 10;
        temp1 = gsm_sub(temp1, if b >= 0 { b << 1 } else { -((-b) << 1) });
        temp1 = gsm_mult_r(inva, temp1);
        larpp[i] = gsm_add(temp1, temp1);
        i += 1;
    };

    step(0, -32, 13107);
    step(0, -32, 13107);
    step(2048, -16, 13107);
    step(-2560, -16, 13107);

    step(94, -8, 19223);
    step(-1792, -8, 17476);
    step(-341, -4, 31454);
    step(-1144, -4, 29708);

    /* NOTE: the addition of *MIC is used to restore the sign of *larc. */
}

/* 4.2.9 */
/* Computation of the quantized reflection coefficients
*/

/* 4.2.9.1  Interpolation of the larpp[ 1..8 ] to get the larp[ 1..8 ]
*/

/*
    Within each frame of 160 analyzed speech samples the short term
    analysis and synthesis filters operate with four different sets of
    coefficients, derived from the previous set of decoded LARs(larpp(j-1))
    and the actual set of decoded LARs (larpp(j))

   (Initial value: larpp(j-1)[ 1..8 ] = 0.)
*/

fn gsm_enc_coefficients_0_12(
    larpp_j_1: &mut [Word],
    larpp_j: &mut [Word],
    larp: &mut [Word],
) {
    // loopbound min 8 max 8
    for i in 0..8 {
        larp[i] = gsm_add(sasr(larpp_j_1[i], 2), sasr(larpp_j[i], 2));
        larp[i] = gsm_add(larp[i], sasr(larpp_j_1[i], 1));
    }
}

fn gsm_enc_coefficients_13_26(
    larpp_j_1: &mut [Word],
    larpp_j: &mut [Word],
    larp: &mut [Word],
) {
    // loopbound min 8 max 8
    for i in 0..8 {
        larp[i] = gsm_add(sasr(larpp_j_1[i], 1), sasr(larpp_j[i], 1));
    }
}

fn gsm_enc_coefficients_27_39(
    larpp_j_1: &mut [Word],
    larpp_j: &mut [Word],
    larp: &mut [Word],
) {
    // loopbound min 8 max 8
    for i in 0..8 {
        larp[i] = gsm_add(sasr(larpp_j_1[i], 2), sasr(larpp_j[i], 2));
        larp[i] = gsm_add(larp[i], sasr(larpp_j[i], 1));
    }
}


fn gsm_enc_coefficients_40_159(
    larpp_j: &mut [Word],
    larp: &mut [Word],
) {
    // loopbound min 8 max 8
    for i in 0..8 {
        larp[i] = larpp_j[i];
    }
}

/* 4.2.9.2 */

fn gsm_enc_larp_to_rp(
    larp: &mut [Word],      /* [ 0..7 ] IN/OUT  */
) {
    /*
        The input of this procedure is the interpolated larp[ 0..7 ] array.
        The reflection coefficients, rp[ i ], are used in the analysis
        filter and in the synthesis filter.
    */
    // loopbound min 8 max 8
    for i in 0..8 {
        let temp: Word;
        let factor: Word;
        if larp[i] < 0 {
            temp = if larp[i] == MIN_WORD { MAX_WORD } else { -larp[i] };
            factor = -1;
        } else {
            temp = larp[i];
            factor = -1;
        }

        larp[i] = factor * (if temp < 11059 {
            temp << 1
        } else {
            if temp < 20070 {
                temp + 11059
            } else {
                gsm_add(temp >> 2, 26112)
            }
        });
    }
}


/* 4.2.10 */
fn gsm_enc_short_term_analysis_filtering(
    u: &mut [Word; 8],
    rp: [Word; 8],      /* [ 0..7 ] IN  */
    k_n: i16,           /*   k_end - k_start  */
    s: &mut Word            /* [ 0..n-1 ] IN/OUT -> no array reference as only first element is used */
) {
    /*
        This procedure computes the short term residual signal d[ .. ] to be fed
        to the RPE-LTP loop from the s[ .. ] signal and from the local rp[ .. ]
        array (quantized reflection coefficients).  As the call of this
        procedure can be done in many ways (see the interpolation of the LAR
        coefficient), it is assumed that the computation begins with index
        k_start (for arrays d[ .. ] and s[ .. ]) and stops with index k_end
        (k_start and k_end are defined in 4.2.9.1).  This procedure also
        needs to keep the array u[ 0..7 ] in memory for each call.
    */
    // loopbound min 13 max 120
    for _ in 0..k_n {
        let mut sav: Word = *s;
        let mut di: Word = *s;

        // loopbound min 8 max 8
        for i in 0..8 { /* YYY */
            let ui: Word = u[i];
            u[i] = sav;

            let zzz: Word = gsm_mult_r(rp[i], di);
            sav = gsm_add(ui,  zzz);

            let zzz = gsm_mult_r(rp[i], ui);
            di = gsm_add(di,  zzz);

        }

        *s = di;
    }
}

fn gsm_enc_gsm_short_term_analysis_filter(
    state: &mut GsmState,
    larc: &mut [Word; 8],      /* coded log area ratio [ 0..7 ]  IN  */
    s: &mut [Word; 160],       /* signal [ 0..159 ]    IN/OUT  */
) {
    let larpp_j: &mut [Word; 8];
    let larpp_j_1: &mut [Word; 8];

    let (a, b) = state.larpp.split_at_mut((state.j + 1) as usize);
    // even
    if (state.j % 2) == 0 {
        larpp_j = &mut a[if a.len() == 0 { 0 } else { a.len() - 1 }];
        larpp_j_1 = &mut b[0];
    // odd
    } else {
        larpp_j = &mut b[0];
        larpp_j_1 = &mut a[if a.len() == 0 { 0 } else { a.len() - 1 }];
    }
    let mut larp: [Word; 8] = [0; 8];

    gsm_enc_decoding_of_the_coded_log_area_ratios(larc, larpp_j);

    gsm_enc_coefficients_0_12(larpp_j_1, larpp_j, &mut larp);
    gsm_enc_larp_to_rp(&mut larp);
    gsm_enc_short_term_analysis_filtering(&mut state.u, larp, 13, &mut s[0]);

    gsm_enc_coefficients_13_26(larpp_j_1, larpp_j, &mut larp);
    gsm_enc_larp_to_rp(&mut larp);
    gsm_enc_short_term_analysis_filtering(&mut state.u, larp, 14, &mut s[13]);

    gsm_enc_coefficients_27_39(larpp_j_1, larpp_j, &mut larp);
    gsm_enc_larp_to_rp(&mut larp);
    gsm_enc_short_term_analysis_filtering(&mut state.u, larp, 13, &mut s[27]);

    gsm_enc_coefficients_40_159(larpp_j, &mut larp);
    gsm_enc_larp_to_rp(&mut larp);
    gsm_enc_short_term_analysis_filtering(&mut state.u, larp, 120, &mut s[40]);
}

/* lpc.c */

/*
    4.2.4 .. 4.2.7 LPC ANALYSIS SECTION
*/

/* 4.2.4 */

fn gsm_enc_autocorrelation(
    s: &mut [Word; 160],            /* [ 0..159 ] IN/OUT  */
    l_acf: &mut [Longword; 9],      /* [ 0..8 ] OUT     */
) {
    /*
        The goal is to compute the array l_acf[ k ].  The signal s[ i ] must
        be scaled in order to avoid an overflow situation.
    */

    /*  Dynamic scaling of the array  s[ 0..159 ] */

    /*  Search for the maximum. */
    // loopbound min 160 max 160
    let smax: Word = s.iter().map(|x| gsm_abs(*x)).max().unwrap_or(0);

    /*  Computation of the scaling factor. */
    let scalauto: Word = if smax == 0 {
        0
    } else {
        4 - gsm_enc_norm((smax as Longword) << 16) /* sub(4,..) */
    };

    /*  Scaling of the array s[ 0...159 ]
    */

    if 1 <= scalauto && scalauto <= 4 {
        let factor: Word = 16384 >> (scalauto - 1);
        // loopbound min 160 max 160
        for val in s.iter_mut() {
            *val = gsm_mult_r(*val, factor);
        }
    }

    /*  Compute the l_acf[ .. ]. */
    {
        // loopbound min 9 max 9
        l_acf.fill(0);

        let mut step = |i: usize, j: usize, sl: Word| {
            l_acf[j] += (sl as Longword) * (s[i-j] as Longword);
        };

        for i in 0..=7 {
            let sl: Word = s[i];
            for j in 0..=i {
                step(i, j, sl)
            }
        }

        // loopbound min 152 max 152
        for i in 8..=159 {
            let sl: Word = s[i];
            for j in 0..=8 {
                step(i, j, sl);
            }
        }

        for val in l_acf.iter_mut() {
            *val <<= 1;
        }
    }
    /*   Rescaling of the array s[ 0..159 ] */
    if scalauto > 0 {
        // loopbound min 160 max 160
        for val in s.iter_mut() {
            *val <<= scalauto;
        }
    }
}

/* 4.2.5 */

fn gsm_enc_reflection_coefficients(
    l_acf: &[Longword; 9], /* 0...8  IN  */
    r: &mut [Word; 8],
) {
    let mut acf: [Word; 9] = [0; 9];     /* 0..8 */
    let mut p: [Word; 9];       /* 0..8 */
    let mut k: [Word; 9] = [0; 9];       /* 2..8 */

    /*  Schur recursion with 16 bits arithmetic. */

    if l_acf[ 0 ] == 0 {
        // loopbound min 8 max 8
        r.fill(0);
        return;
    }

    let mut temp = gsm_enc_norm(l_acf[0]);

    /* ? overflow ? */
    // loopbound min 9 max 9
    for (i, val) in acf.iter_mut().enumerate() {
        *val = sasr(l_acf[i] << temp, 16) as Word;
    }

    /*   Initialize array P[ .. ] and K[ .. ] for the recursion.
    */

    // loopbound min 7 max 7
    for i in 1..=7 {
        k[i] = acf[i];
    }

    // loopbound min 9 max 9
    p = acf;

    /* Compute reflection coefficients */


    // loopbound min 8 max 8
    // marker outer-marker
    for n in 1..=8 {
        temp = p[1];
        temp = gsm_abs(temp);

        if p[0] < temp { 
            // loopbound min 1 max 8
            for i in n..=8 {
                r[i] = 0; // marker inner-marker
            }
            return; // flowrestriction 1*inner-marker <= 36*outer-marker
        }

        r[n - 1] = gsm_enc_div(temp, p[0]);

        if p[1] > 0 { r[n - 1] = -r[n - 1]; } /* r[ n ] = sub(0, r[ n ]) */
        if n == 8 { return; }

        /*  Schur recursion
        */
        temp = gsm_mult_r(p[1], r[n]);
        p[0] = gsm_add(p[0], temp);

        // loopbound min 1 max 7
        for m in 1..=(8 - n) {
            temp = gsm_mult_r(k[m], r[n]);
            p[m] = gsm_add(p[m + 1], temp);

            temp = gsm_mult_r(p[m + 1], r[n]);
            k[m] = gsm_add(k[m], temp);
        }
    }
}

/* 4.2.6 */

fn gsm_enc_transformation_to_log_area_ratios(
    r: &mut [Word; 8]       /* 0..7    IN/OUT */
) {
    /*
        The following scaling for r[ .. ] and LAR[ .. ] has been used:

        r[ .. ]   = integer( real_r[ .. ]*32768. ); -1 <= real_r < 1.
        LAR[ .. ] = integer( real_LAR[ .. ] * 16384 );
        with -1.625 <= real_LAR <= 1.625
*/
    /* Computation of the LAR[ 0..7 ] from the r[ 0..7 ]
    */
    // loopbound min 8 max 8
    for val in r.iter_mut() {
      let mut temp = gsm_abs(*val);
      if temp < 22118 {
          temp >>= 1
      } else {
          if temp < 31130 {
              temp -= 11059;
          } else {
              temp -= 26112;
              temp <<= 2;
          }
      }
      *val = if *val < 0 { -temp } else { temp };
    }
}

/* 4.2.7 */

fn gsm_enc_quantization_and_coding(lar: &mut [Word; 8]) {
    /*  This procedure needs four tables; the following equations
        give the optimum scaling for the constants:

        A[ 0..7 ] = integer( real_A[ 0..7 ] * 1024 )
        B[ 0..7 ] = integer( real_B[ 0..7 ] *  512 )
        MAC[ 0..7 ] = maximum of the larc[ 0..7 ]
        MIC[ 0..7 ] = minimum of the larc[ 0..7 ]
    */
    let mut step = |a: Word, b: Word, mac: Word, mic: Word, i: &mut usize| {
        let mut temp = gsm_mult(a, lar[*i]);
        temp = gsm_add(temp, b);
        temp = gsm_add(temp, 256);
        temp = sasr(temp, 9);
        lar[*i] = if temp > mac {
            mac - mic
        } else {
            if temp < mic {
                0
            } else {
                temp - mic
            }
        };
        *i += 1;
    };

    let mut i: usize = 0;
    step(20480, 0, 31, -32, &mut i);
    step(20480, 0, 31, -32, &mut i);
    step(20480, 2048, 15, -16, &mut i);
    step(20480, -2560, 15, -16, &mut i);

    step(13964, 94, 7, -8, &mut i);
    step(15360, -1792, 7, -8, &mut i);
    step(8534, -341, 3, -4, &mut i);
    step(9036, -1144, 3, -4, &mut i);
}

fn gsm_enc_gsm_lpc_analysis(
    s: &mut [Word; 160],   /* 0..159 signals IN/OUT  */
    larc: &mut [Word; 8],  /* 0..7   larc's  OUT */
) {
    let mut l_acf: [Longword; 9] = [0; 9];
    gsm_enc_autocorrelation(s, &mut l_acf);
    gsm_enc_reflection_coefficients(&l_acf, larc);
    gsm_enc_transformation_to_log_area_ratios(larc);
    gsm_enc_quantization_and_coding(larc);
}

/* preprocess.c */
/*  4.2.0 .. 4.2.3  PREPROCESSING SECTION

      After A-law to linear conversion (or directly from the
      Ato D converter) the following scaling is assumed for
    input to the RPE-LTP algorithm:

        in:  0.1.....................12
         S.v.v.v.v.v.v.v.v.v.v.v.v.*.*.

    Where S is the sign bit, v a valid bit, and * a "don't care" bit.
    The original signal is called sop[ .. ]

        out:   0.1................... 12
         S.S.v.v.v.v.v.v.v.v.v.v.v.v.0.0
*/

fn gsm_enc_gsm_preprocess(
    state: &mut GsmState,
    s: &[Word; 160],
    so: &mut [Word; 160],    /* [ 0..159 ]   IN/OUT  */
) {
    let mut z1: Word = state.z1;
    let mut l_z2: Longword = state.l_z2;
    let mut mp: Word = state.mp;

    // loopbound min 160 max 160
    for i in 0..160 {
        /*  4.2.1   Downscaling of the input signal */
        let s_o: Word = sasr(s[i], 3) << 2;

        /*  4.2.2   Offset compensation

            This part implements a high-pass filter and requires extended
            arithmetic precision for the recursive part of this filter.
            The input of this procedure is the array so[ 0...159 ] and the
            output the array sof[  0...159  ].
        */
        /*   Compute the non-recursive part */

        let s1: Word = s_o - z1;     /* s1 = gsm_enc_sub( *so, z1 ); */
        z1 = s_o;

        /*   Compute the recursive part */
        let mut l_s2: Longword = s1 as Longword;
        l_s2 <<= 15;

        /*   Execution of a 31 bv 16 bits multiplication */

        let mut msp: Word = sasr(l_z2, 15) as Word;
        let lsp: Word = (l_z2 - ((msp as Longword) << 15)) as Word; /* gsm_enc_L_sub(L_z2,(msp<<15)); */

        l_s2  += gsm_mult_r(lsp, 32735) as Longword;
        let mut l_temp: Longword = (msp as Longword) * 32735; /* GSM_L_MULT(msp,32735) >> 1;*/
        l_z2   = gsm_add(l_temp, l_s2);

        /*    Compute sof[ k ] with rounding */
        l_temp = gsm_add(l_z2, 16384) as Longword;

        /*   4.2.3  Preemphasis */

        msp = gsm_mult_r(mp, -28180);
        mp = sasr(l_temp, 15) as i16;
        so[i] = gsm_add(mp, msp);
    }

    state.z1 = z1;
    state.l_z2 = l_z2;
    state.mp = mp;
}

/* gsm_enc_bench.c */

fn gsm_enc_norm(mut a: Longword) -> Word {
    /*
        the number of left shifts needed to normalize the 32 bit
        variable L_var1 for positive values on the interval

        with minimum of
        minimum of 1073741824  (01000000000000000000000000000000) and
        maximum of 2147483647  (01111111111111111111111111111111)


        and for negative values on the interval with
        minimum of -2147483648 (-10000000000000000000000000000000) and
        maximum of -1073741824 ( -1000000000000000000000000000000).

        in order to normalize the result, the following
        operation must be done: L_norm_var1 = L_var1 << norm( L_var1 );

        (That's 'ffs', only from the left, not the right..)
    */
    if a < 0 {
      if a <= -1073741824 {
        return 0;
      }
      a = !a;
    }

    if (a & 0xffff0000) != 0 {
        if (a & 0xff00000) != 0 {
            -1 + GSM_ENC_BITOFF[(0xFF & (a >> 24)) as usize] as i16
        } else {
            7 + GSM_ENC_BITOFF[(0xFF & (a >> 16)) as usize] as i16
        }
    } else {
        if (a & 0xff00) != 0 {
            15 + GSM_ENC_BITOFF[(0xFF & (a >> 8)) as usize] as i16
        } else {
            23 + GSM_ENC_BITOFF[(0xFF & a) as usize] as i16
        }
    }
}

fn gsm_enc_asl(a: Word, n: i16) -> Word {
    if n >= 16 { 0 }
    else if n <= -16 { -((a < 0) as Word) }
    else if n < 0 { gsm_enc_asr(a, -n) }
    else { a << n }
}

// here the version where SASR is not defined is omitted
fn gsm_enc_asr(a: Word, n: i16) -> Word {
    if n >= 16 { -((a < 0) as Word) }
    else if n <= -16 { 0 }
    else if n < 0 { a << -n }
    else { a >> n }
}

/*
    (From p. 46, end of section 4.2.5)

    NOTE: The following lines gives [ sic ] one correct implementation
        of the div(num, denum) arithmetic operation.  Compute div
          which is the integer division of num by denum: with denum
      >= num > 0
*/

fn gsm_enc_div(
    num: Word,
    denum: Word,
) -> Word
{
    let mut l_num: Longword = num as Longword;
    let l_denum: Longword = denum as Longword;
    let mut div: Word = 0;

    /* The parameter num sometimes becomes zero.
      Although this is explicitly guarded against in 4.2.5,
      we assume that the result should then be zero as well.
    */

    if num == 0 { return 0; }

    // loopbound min 15 max 15
    for _ in 0..15 {
        div <<= 1;
        l_num <<= 1;

        if l_num >= l_denum {
            l_num -= l_denum;
            div += 1;
        }  
    }
    div
}



fn gsm_enc_create() -> GsmState {
    // loopbound min 648 max 648
    let mut r = GsmState::default();

    r.nrp = 40;
    return r;
}

fn gsm_enc_init(
    state: &mut GsmState,
    pcmdata: &mut [GsmSignal; 3200],
    gsmdata: &mut [GsmByte; 660],
) {
    *state = gsm_enc_create();
    *pcmdata = GSM_ENC_PCMDATA;
    *gsmdata = GSM_ENC_GSMDATA;
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn gsm_enc_main(
    state: &mut GsmState,
    pcmdata: &mut [GsmSignal; 3200],
    gsmdata: &mut [GsmByte; 660],
) -> Word {
    let gsm_enc_result: Word = 0;

    // loopbound min 20 max 20
    for i in 0..SAMPLES {
        // let source: &mut [GsmSignal; 160] = [0; 160];
        // for j in 0..160 {
        //     source[j] = GSM_ENC_PCMDATA[i*160+j];
        // }
        // let mut c: GsmFrame = [0; 33];
        // for j in 0..33 {
        //     c[j] = GSM_ENC_GSMDATA[i*33+j];
        // }
        gsm_enc_encode(
            state,
            (&mut pcmdata[(i * 160)..((i + 1) * 160)]).try_into().unwrap(),
            (&mut gsmdata[(i * 33)..((i + 1) * 33)]).try_into().unwrap(),
        );
    }
    gsm_enc_result
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn gsm_enc_log(
    state: &mut GsmState,
    pcmdata: &mut [GsmSignal; 3200],
    gsmdata: &mut [GsmByte; 660],
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result: i16 = 0;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        gsm_enc_init(
            black_box(state), black_box(pcmdata), black_box(gsmdata),
        );
        let start = black_box(DWT::cycle_count());
        result = black_box(gsm_enc_main(
            black_box(state), black_box(pcmdata), black_box(gsmdata),
        ));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("gsm_enc - elapsed: {}", cycles);
    });

    result == 0
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn gsm_enc_std(
    state: &mut GsmState,
    pcmdata: &mut [GsmSignal; 3200],
    gsmdata: &mut [GsmByte; 660],
) -> bool {
    gsm_enc_init(
        black_box(state), black_box(pcmdata), black_box(gsmdata),
    );
    let result = black_box(gsm_enc_main(
        black_box(state), black_box(pcmdata), black_box(gsmdata),
    ));

    result == 0
}

fn gsm_enc() -> bool {
    static mut STATE: GsmState = GsmState::new();
    static mut PCMDATA: [GsmSignal; 3200] = [0; 3200];
    static mut GSMDATA: [GsmByte; 660] = [0; 660];

    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let state = unsafe { &mut *{ &raw mut STATE } };
    let pcmdata = unsafe { &mut *{ &raw mut PCMDATA } };
    let gsmdata = unsafe { &mut *{ &raw mut GSMDATA } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(gsm_enc_log(
            black_box(state), black_box(pcmdata), black_box(gsmdata),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(gsm_enc_std(
            black_box(state), black_box(pcmdata), black_box(gsmdata),
    ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(gsm_enc());
    
    #[cfg(feature = "log")]
    {
        rprintln!("gsm_enc: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(gsm_enc());

    #[cfg(feature = "log")]
    std::println!("gsm_enc: {}", success);

    std::process::exit(!success as i32);
}
