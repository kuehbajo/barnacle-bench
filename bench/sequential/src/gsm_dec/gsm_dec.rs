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
  
  Version V 2.0

  Name: gsm_dec

  Author: Jutta Degener and Carsten Bormann,
   Technische Universitaet Berlin

  Function: Decoding of GSM data

  Source: included in MediaBench/gsm

  Original name: gsm_decode

  Changes: no major functional changes

  License: see the accompanying COPYRIGHT file

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
use core::hint::black_box;

mod data;
use data::{ GsmSignal, GsmByte, GSM_DEC_PCMDATA, GSM_DEC_GSMDATA };

type Word = i16;        /* 16 bit signed int  */
type Longword = i32;    /* 32 bit signed int  */

type GsmFrame = [GsmByte; 33];

const GSM_MAGIC: Word =  0xD; /* 13 kbit/s RPE-LTP */
const SAMPLES: usize = 20;

struct GsmState {
    dp0: [Word; 280],

    _z1: Word,                   /* preprocessing.c, Offset_com. */
    _l_z2: Longword,             /*                  Offset_com. */
    _mp: i16,                    /*                  Preemphasis */

    _u: [Word; 8],               /* short_term_aly_filter.c  */
    la_rpp: [[Word; 8]; 2],      /*                              */
    j: Word,                    /*                              */
    v: [Word; 9],               /* short_term.c, synthesis  */

    nrp: Word,                  /* 40 */ /* long_term.c, synthesis */
    msr: Word,                  /* decoder.c, Postprocessing  */
}

impl GsmState {
    const fn new() -> Self {
        Self {
            dp0: [0; 280],
            _z1: 0,
            _l_z2: 0,
            _mp: 0,
            _u: [0; 8],
            la_rpp: [[0; 8]; 2],
            j: 0,
            v: [0; 9],
            nrp: 0,
            msr: 0,
        }
    }
}

impl Default for GsmState {
    fn default() -> Self {
        GsmState {
            dp0: [0; 280],
            _z1: 0,
            _l_z2: 0,
            _mp: 0,
            _u: [0; 8],
            la_rpp: [[0; 8]; 2],
            j: 0,
            v: [0; 9],
            nrp: 0,
            msr: 0,
        }
    }
}


const MIN_WORD: Word = -32768;
const MAX_WORD: Word = 32767;

/*   Table 4.6   Normalized direct mantissa used to compute xM/xmax
*/
/*                        i         0      1      2      3      4      5      6      7   */
const GSM_DEC_FAC: [Word; 8] = [18431, 20479, 22527, 24575, 26623, 28671, 30719, 32767];

/*   Table 4.3b   Quantization levels of the LTP gain quantizer
*/
/*                        bc       0      1      2      3      */
const GSM_DEC_QLB: [Word; 4] = [3277, 11469, 21299, 32767];

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

#[inline]
pub fn gsm_add<T: SaturatingAdd>(a: T, b: T) -> T {
    a.sat_add(b)
}

#[inline]
pub fn gsm_sub<T: SaturatingSub>(a: T, b: T) -> T {
    a.sat_sub(b)
}

/* add.c */

fn gsm_dec_asl(a: Word, n: i16) -> Word {
    if n >= 16 { 0 }
    else if n <= -16 { -((a < 0) as Word) }
    else if n < 0 { gsm_dec_asr(a, -n) }
    else { a << n }
}

fn gsm_dec_asr(a: Word, n: i16) -> Word {
    if n >= 16 { -((a < 0) as Word) }
    else if n <= -16 { 0 }
    else if n < 0 { a << -n }
    else { a >> n }
}

/* short_term.c */
/*
  SHORT TERM ANALYSIS FILTERING SECTION
*/

/* 4.2.8 */

fn gsm_dec_decoding_of_the_coded_log_area_ratios(
    la_rc: &mut [Word; 8],   /* coded log area ratio [ 0..7 ]  IN  */
    la_rpp: &mut [Word; 8]   /* out: decoded ..      */
) {
    /*  This procedure requires for efficient implementation
        two tables.

        INVA[ 1..8 ] = integer( (32768 * 8) / real_A[ 1..8 ])
        MIC[ 1..8 ]  = minimum value of the LARc[ 1..8 ]
    */

    let mut i: usize = 0;
    /*  Compute the LARpp[ 1..8 ] */
    let mut step = |b: Word, mic: Word, inva: Word| {
        let mut temp1 = gsm_add(la_rc[i], mic) << 10;
        temp1 = gsm_sub(temp1, if b >= 0 { b << 1 } else { -((-b) << 1) });
        temp1 = gsm_mult_r(inva, temp1);
        la_rpp[i] = gsm_add(temp1, temp1);
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

    /* NOTE: the addition of *MIC is used to restore
        the sign of *LARc.
    */
}

/* 4.2.9 */
/* Computation of the quantized reflection coefficients
*/

/* 4.2.9.1  Interpolation of the LARpp[ 1..8 ] to get the LARp[ 1..8 ]
*/

/*
  Within each frame of 160 analyzed speech samples the short term
  analysis and synthesis filters operate with four different sets of
  coefficients, derived from the previous set of decoded LARs(LARpp(j-1))
  and the actual set of decoded LARs (LARpp(j))

  (Initial value: LARpp(j-1)[ 1..8 ] = 0.)
*/

fn gsm_dec_coefficients_0_12(
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

/* 4.2.9.2 */

fn gsm_dec_larp_to_rp(
    larp: &mut [Word],      /* [ 0..7 ] IN/OUT  */
) {
    /*
        The input of this procedure is the interpolated LARp[ 0..7 ] array.
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


fn gsm_dec_decoder(
    state: &mut GsmState,
    larcr: &mut [Word; 8],      /* [ 0..7 ]    IN  */
    ncr: &[Word; 4],        /* [ 0..3 ]    IN  */
    bcr: &[Word; 4],        /* [ 0..3 ]    IN  */
    mcr: &[Word; 4],        /* [ 0..3 ]    IN  */
    xmaxcr: &[Word; 4],     /* [ 0..3 ]    IN  */
    x_mcr: &[Word; 13*4],    /* [ 0..13*4 ] IN  */
    s: &mut [Word; 160],        /* [ 0..159 ]  OUT */
) {
    let mut erp: [Word; 40] = [0; 40];
    let mut wt: [Word; 160] = [0; 160];
    let drp: &mut [Word] = &mut state.dp0[120..280];
    
    // loopbound min 4 max 4
    for j in 0..4 {
        gsm_dec_rpe_decoding(xmaxcr[j], mcr[j], &x_mcr[(j*13)..((j+1)*13)], &mut erp);
        gsm_dec_long_term_synthesis_filtering(ncr[j], bcr[j], &erp, drp, &mut state.nrp);

        // loopbound min 40 max 40
        for k in 0..=39 {
            wt[j * 40 + k] = drp[k];
        }
    }
    gsm_dec_short_term_synthesis_filter(state, larcr, &mut wt, s);
    gsm_dec_postprocessing(state, s);
}

/* 4.3.2 */
fn gsm_dec_long_term_synthesis_filtering(
    ncr: Word,
    bcr: Word,
    erp: &[Word; 40],    /* [ 0..39 ]    IN              */
    drp: &mut [Word],   /* [ -120..-1 ] IN, [ 0..40 ] OUT */
    nrp: &mut Word,
) {
    /*
      This procedure uses the bcr and Ncr parameter to realize the
      long term synthesis filtering.  The decoding of bcr needs
      table 4.3b.
    */

    /*  Check the limits of Nr.
    */
    let nr: Word = if ncr < 40 || ncr > 120 { *nrp } else { ncr };
    *nrp = nr;

    /*  Decoding of the LTP gain bcr
    */
    let brp = GSM_DEC_QLB[bcr as usize];

    /*  Computation of the reconstructed short term residual signal drp[ 0..39 ] */

    // loopbound min 40 max 40
    for k in 0..40 {
        let drpp: Word = gsm_mult_r(brp, drp[(k + 120 - nr) as usize]);
        drp[k as usize] = gsm_add(erp[k as usize], drpp);
    }

    /* Update of the reconstructed short term residual signal drp[  -1..-120  ] */

    // loopbound min 120 max 120
    for k in 0..120 {
        drp[k] = drp[k + 40];
    }
}

fn gsm_dec_rpe_decoding(
  xmaxcr: Word,
  mcr: Word,
  x_mcr: &[Word],   /* [ 0..12 ], 3 bits IN */
  erp: &mut [Word; 40],    /* [ 0..39 ]         OUT */
) {
    let mut x_mp: [Word; 13] = [0; 13];

    let (exp, mant): (Word, Word) = gsm_dec_apcm_quantization_xmaxc_to_exp_mant(xmaxcr);
    gsm_dec_apcm_inverse_quantization(x_mcr, mant, exp, &mut x_mp);
    gsm_dec_rpe_grid_positioning(mcr, &mut x_mp, erp);
}

/* 4.2.17 */
fn gsm_dec_rpe_grid_positioning(
    mc: Word,           /* grid position  IN  */
    x_mp: &mut [Word; 13],    /* [ 0..12 ]    IN  */
    ep: &mut [Word],     /* [ 0..39 ]    OUT */
) {
    /*
        This procedure computes the reconstructed long term residual signal
        ep[ 0..39 ] for the LTP analysis filter.  The inputs are the Mc
        which is the grid position selection and the xMp[ 0..12 ] decoded
        RPE samples which are upsampled by a factor of 3 by inserting zero
        values.
    */
    let mut i = 13;
    
    ep.fill(0);
    if 0 <= mc && mc <= 3 {
        ep[mc as usize] = x_mp[0];
        i -= 1;
    }

    let offset_ep: usize = (mc + 1) as usize;
    let mut offset_x_mp = 1;
    // loopbound min 12 max 12
    for k in 0..i {
        ep[3 * k + 2 + offset_ep] = x_mp[offset_x_mp];
        offset_x_mp += 1;
    }
}

/* 4.2.16 */
fn gsm_dec_apcm_inverse_quantization(
    x_mc: &[Word],    /* [ 0..12 ]      IN  */
    mant: Word,
    exp: Word,
    x_mp: &mut [Word; 13],    /* [ 0..12 ]      OUT   */
) {
    /*
        This part is for decoding the RPE sequence of coded xMc[ 0..12 ]
        samples to obtain the xMp[ 0..12 ] array.  Table 4.6 is used to get
        the mantissa of xmaxc (FAC[ 0..7 ]).
    */
    let temp1: Word = GSM_DEC_FAC[mant as usize];  /* see 4.2-15 for mant */
    let temp2: Word = gsm_sub(6, exp);  /* see 4.2-15 for exp  */
    let temp3: Word = gsm_dec_asl(1, gsm_sub(temp2, 1));

    // loopbound min 13 max 13
    for i in 0..13 {
        let mut temp: Word = (x_mc[i] << 1) - 7;       /* restore sign   */

        temp <<= 12;        /* 16 bit signed  */
        temp = gsm_mult_r(temp1, temp);
        temp = gsm_add(temp, temp3);

        x_mp[i] = gsm_dec_asr(temp, temp2);
    }
}

/*
  4.2.4 .. 4.2.7 LPC ANALYSIS SECTION
*/

/* rpe.c */
/* 4.12.15 */
fn gsm_dec_apcm_quantization_xmaxc_to_exp_mant(xmaxc: Word) -> (Word, Word) {
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

/* decode.c */
/*
  4.3 FIXED POINT IMPLEMENTATION OF THE RPE-LTP DECODER
*/
fn gsm_dec_postprocessing(
    state: &mut GsmState,
    s: &mut [Word; 160],
) {
    let msr: &mut Word = &mut state.msr;

    // loopbound min 159 max 159
    for i in 1..160 {
        let tmp = gsm_mult_r(*msr, 28180);
        *msr = gsm_add(s[i], tmp);          /* Deemphasis       */
        s[i] = gsm_add(*msr, *msr) & 0xFFF8; /* Truncation & Upscaling */
    }
}

fn gsm_dec_coefficients_13_26(
    larpp_j_1: &mut [Word],
    larpp_j: &mut [Word],
    larp: &mut [Word],
) {
    // loopbound min 8 max 8
    for i in 0..7 {
        larp[i] = gsm_add(sasr(larpp_j_1[i], 1), sasr(larpp_j[i], 1));
    }
}

fn gsm_dec_coefficients_40_159(
    larpp_j: &mut [Word],
    larp: &mut [Word],
) {
    // loopbound min 8 max 8
    for i in 0..8 {
        larp[i] = larpp_j[i];
    }
}

fn gsm_dec_short_term_synthesis_filtering(
    v: &mut [Word; 9],
    rrp: [Word; 8], /* [ 0..7 ]      IN  */
    k: usize,       /* k_end - k_start */
    wt: &[Word],    /* [ 0..k-1 ]    IN  */
    sr: &mut [Word],    /* [ 0..k-1 ]    OUT */
)
{
    // loopbound min 12 max 119
    for m in (0..k).rev() {
        let mut sri: Word = wt[m];

        // loopbound min 8 max 8
        for i in (0..8).rev() {
            /* sri = GSM_SUB( sri, gsm_mult_r( rrp[ i ], v[ i ] ) ); */
            let tmp = if rrp[i] == MIN_WORD && v[i] == MIN_WORD {
                MAX_WORD
            } else {
                0x0FFFF & ((rrp[i] as Longword * v[i] as Longword + 16384 ) >> 15) as Word
            };
            sri = gsm_sub(sri, tmp);

            /* v[ i+1 ] = GSM_ADD( v[ i ], gsm_mult_r( rrp[ i ], sri ) ); */
            let tmp = if rrp[i] == MIN_WORD && sri == MIN_WORD {
                MAX_WORD
            } else {
                0x0FFFF & ((rrp[i] as Longword * sri as Longword + 16384) >> 15) as Word
            };
            v[i + 1] = gsm_add(v[i], tmp);
        }
        v[0] = sri;
        sr[m] = sri;
    }
}

fn gsm_dec_short_term_synthesis_filter(
    state: &mut GsmState,
    larc: &mut [Word; 8],       /* coded log area ratio [ 0..7 ]  IN  */
    wt: &mut [Word; 160],       /* received d [ 0..159 ]             IN  */
    s: &mut [Word; 160],        /* signal   s [ 0..159 ]             OUT  */
) {
    let larpp_j: &mut [Word; 8];
    let larpp_j_1: &mut [Word; 8];

    let (a, b) = state.la_rpp.split_at_mut((state.j + 1) as usize);
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

    gsm_dec_decoding_of_the_coded_log_area_ratios(larc, larpp_j);

    gsm_dec_coefficients_0_12(larpp_j_1, larpp_j, &mut larp);
    gsm_dec_larp_to_rp(&mut larp);
    gsm_dec_short_term_synthesis_filtering(&mut state.v, larp, 13, &wt[0..160], &mut s[0..160]);

    gsm_dec_coefficients_13_26(larpp_j_1, larpp_j, &mut larp);
    gsm_dec_larp_to_rp(&mut larp);
    gsm_dec_short_term_synthesis_filtering(&mut state.v, larp, 14, &wt[13..160], &mut s[13..160]);

    gsm_dec_coefficients_27_39(larpp_j_1, larpp_j, &mut larp);
    gsm_dec_larp_to_rp(&mut larp);
    gsm_dec_short_term_synthesis_filtering(&mut state.v, larp, 13, &wt[27..160], &mut s[27..160]);

    gsm_dec_coefficients_40_159(larpp_j, &mut larp);
    gsm_dec_larp_to_rp(&mut larp);
    gsm_dec_short_term_synthesis_filtering(&mut state.v, larp, 120, &wt[40..160], &mut s[40..160]);
}

fn gsm_dec_coefficients_27_39(
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

/*
  Initialization- and return-value-related functions
*/

fn gsm_dec_create() -> GsmState {
    // loopbound min 648 max 648
    let mut r = GsmState::default();

    r.nrp = 40;
    return r;
}

fn gsm_dec_init(state: &mut GsmState) {
    *state = gsm_dec_create();
}

/*
  Main functions
*/

/* gsm_decode.c */
fn gsm_dec_decode(
    state: &mut GsmState,
    c: &mut [GsmByte; 33],
    target: &mut [GsmSignal; 160],
) -> bool
{
    let mut larc:[Word; 8] = [0; 8];
    let mut nc: [Word; 4] = [0; 4];
    let mut mc: [Word; 4] = [0; 4];
    let mut bc: [Word; 4] = [0; 4];
    let mut xmaxc: [Word; 4] = [0; 4];
    let mut xmc: [Word; 13 * 4] = [0; 13 * 4];

    /* GSM_MAGIC  = (*c >> 4) & 0xF; */
    
    if ((c[0] >> 4) & 0x0F) != GSM_MAGIC as u8{
        return false;
    }

    let mut i = 0;
    let ci = c[i] as i16;
    larc[0] = (ci & 0xF) << 2; /* 1 */
    i += 1;
    let ci = c[i] as i16;
    larc[0] |= (ci >> 6) & 0x3;
    larc[1] = ci & 0x3F;
    i += 1;
    let ci = c[i] as i16;
    larc[2] = (ci >> 3) & 0x1F;
    larc[3] = (ci & 0x7) << 2;
    i += 1;
    let ci = c[i] as i16;
    larc[3] |= (ci >> 6) & 0x3;
    larc[4] = (ci >> 2) & 0xF;
    larc[5] = (ci & 0x3) << 2;
    i += 1;
    let ci = c[i] as i16;
    larc[5] |= (ci >> 6) & 0x3;
    larc[6] = (ci >> 3) & 0x7;
    larc[7] = ci & 0x7;
    i += 1;
    let ci = c[i] as i16;
    nc[0] = (ci >> 1) & 0x7F;
    bc[0] = (ci & 0x1) << 1;
    i += 1;
    let ci = c[i] as i16;
    bc[0] |= (ci >> 7) & 0x1;
    mc[0] = (ci >> 5) & 0x3;
    xmaxc[0] = (ci & 0x1F) << 1;
    i += 1;
    let ci = c[i] as i16;
    xmaxc[0] |= (ci >> 7) & 0x1;
    xmc[0] = (ci >> 4) & 0x7;
    xmc[1] = (ci >> 1) & 0x7;
    xmc[2] = (ci & 0x1) << 2;
    i += 1;
    let ci = c[i] as i16;
    xmc[2] |= (ci >> 6) & 0x3;
    xmc[3] = (ci >> 3) & 0x7;
    xmc[4] = ci & 0x7;
    i += 1;
    let ci = c[i] as i16;
    xmc[5] = (ci >> 5) & 0x7;
    xmc[6] = (ci >> 2) & 0x7;
    xmc[7] = (ci & 0x3) << 1; /* 10 */
    i += 1;
    let ci = c[i] as i16;
    xmc[7] |= (ci >> 7) & 0x1;
    xmc[8] = (ci >> 4) & 0x7;
    xmc[9] = (ci >> 1) & 0x7;
    xmc[10] = (ci & 0x1) << 2;
    i += 1;
    let ci = c[i] as i16;
    xmc[10] |= (ci >> 6) & 0x3;
    xmc[11] = (ci >> 3) & 0x7;
    xmc[12] = ci & 0x7;
    i += 1;
    let ci = c[i] as i16;
    nc[1] = (ci >> 1) & 0x7F;
    bc[1] = (ci & 0x1) << 1;
    i += 1;
    let ci = c[i] as i16;
    bc[1] |= (ci >> 7) & 0x1;
    mc[1] = (ci >> 5) & 0x3;
    xmaxc[1] = (ci & 0x1F) << 1;
    i += 1;
    let ci = c[i] as i16;
    xmaxc[1] |= (ci >> 7) & 0x1;
    xmc[13] = (ci >> 4) & 0x7;
    xmc[14] = (ci >> 1) & 0x7;
    xmc[15] = (ci & 0x1) << 2;
    i += 1;
    let ci = c[i] as i16;
    xmc[15] |= (ci >> 6) & 0x3;
    xmc[16] = (ci >> 3) & 0x7;
    xmc[17] = ci & 0x7;
    i += 1;
    let ci = c[i] as i16;
    xmc[18] = (ci >> 5) & 0x7;
    xmc[19] = (ci >> 2) & 0x7;
    xmc[20] = (ci & 0x3) << 1;
    i += 1;
    let ci = c[i] as i16;
    xmc[20] |= (ci >> 7) & 0x1;
    xmc[21] = (ci >> 4) & 0x7;
    xmc[22] = (ci >> 1) & 0x7;
    xmc[23] = (ci & 0x1) << 2;
    i += 1;
    let ci = c[i] as i16;
    xmc[23] |= (ci >> 6) & 0x3;
    xmc[24] = (ci >> 3) & 0x7;
    xmc[25] = ci & 0x7;
    i += 1;
    let ci = c[i] as i16;
    nc[2] = (ci >> 1) & 0x7F;
    bc[2] = (ci & 0x1) << 1; /* 20 */
    i += 1;
    let ci = c[i] as i16;
    bc[2] |= (ci >> 7) & 0x1;
    mc[2] = (ci >> 5) & 0x3;
    xmaxc[2] = (ci & 0x1F) << 1;
    i += 1;
    let ci = c[i] as i16;
    xmaxc[2] |= (ci >> 7) & 0x1;
    xmc[26] = (ci >> 4) & 0x7;
    xmc[27] = (ci >> 1) & 0x7;
    xmc[28] = (ci & 0x1) << 2;
    i += 1;
    let ci = c[i] as i16;
    xmc[28] |= (ci >> 6) & 0x3;
    xmc[29] = (ci >> 3) & 0x7;
    xmc[30] = ci & 0x7;
    i += 1;
    let ci = c[i] as i16;
    xmc[31] = (ci >> 5) & 0x7;
    xmc[32] = (ci >> 2) & 0x7;
    xmc[33] = (ci & 0x3) << 1;
    i += 1;
    let ci = c[i] as i16;
    xmc[33] |= (ci >> 7) & 0x1;
    xmc[34] = (ci >> 4) & 0x7;
    xmc[35] = (ci >> 1) & 0x7;
    xmc[36] = (ci & 0x1) << 2;
    i += 1;
    let ci = c[i] as i16;
    xmc[36] |= (ci >> 6) & 0x3;
    xmc[37] = (ci >> 3) & 0x7;
    xmc[38] = ci & 0x7;
    i += 1;
    let ci = c[i] as i16;
    nc[3] = (ci >> 1) & 0x7F;
    bc[3] = (ci & 0x1) << 1;
    i += 1;
    let ci = c[i] as i16;
    bc[3] |= (ci >> 7) & 0x1;
    mc[3] = (ci >> 5) & 0x3;
    xmaxc[3] = (ci & 0x1F ) << 1;
    i += 1;
    let ci = c[i] as i16;
    xmaxc[3] |= (ci >> 7) & 0x1;
    xmc[39] = (ci >> 4) & 0x7;
    xmc[40] = (ci >> 1) & 0x7;
    xmc[41] = (ci & 0x1) << 2;
    i += 1;
    let ci = c[i] as i16;
    xmc[41] |= (ci >> 6) & 0x3;
    xmc[42] = (ci >> 3) & 0x7;
    xmc[43] = ci & 0x7; /* 30  */
    i += 1;
    let ci = c[i] as i16;
    xmc[44] = (ci >> 5) & 0x7;
    xmc[45] = (ci >> 2) & 0x7;
    xmc[46] = (ci & 0x3) << 1;
    i += 1;
    let ci = c[i] as i16;
    xmc[46] |= (ci >> 7) & 0x1;
    xmc[47] = (ci >> 4) & 0x7;
    xmc[48] = (ci >> 1) & 0x7;
    xmc[49] = (ci & 0x1) << 2;
    i += 1;
    let ci = c[i] as i16;
    xmc[49] |= (ci >> 6) & 0x3;
    xmc[50] = (ci >> 3) & 0x7;
    xmc[51] = ci & 0x7; /* 33 */

    gsm_dec_decoder(state, &mut larc, &nc, &bc, &mc, &xmaxc, &xmc, target);

    return true;
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn gsm_dec_main(state: &mut GsmState) -> bool {
    // loopbound min 20 max 20
    for i in 0..SAMPLES {
        let mut c: GsmFrame = [0; 33];
        c.copy_from_slice(&GSM_DEC_GSMDATA[i * 33..(i + 1) * 33]);
        let mut target: [GsmSignal; 160] = [0; 160];
        target.copy_from_slice(&GSM_DEC_PCMDATA[i * 160..(i + 1) * 160]);
        let gsm_dec_result = gsm_dec_decode(
            state,
            &mut c,
            &mut target,
        );
        if !gsm_dec_result { return false; }
    }
    true
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn gsm_dec_log(state: &mut GsmState) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result: bool = false;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        gsm_dec_init(black_box(state));
        let start = black_box(DWT::cycle_count());
        result = black_box(gsm_dec_main(black_box(state)));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("gsm_dec - elapsed: {}", cycles);
    });

    result
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn gsm_dec_std(state: &mut GsmState) -> bool {
    gsm_dec_init(black_box(state));
    let result = black_box(gsm_dec_main(black_box(state)));
    result
}

fn gsm_dec() -> bool {
     static mut STATE: GsmState = GsmState::new();
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let state: &'static mut GsmState = unsafe { &mut *{ &raw mut STATE } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(gsm_dec_log(black_box(state)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(gsm_dec_std(black_box(state)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(gsm_dec());
    
    #[cfg(feature = "log")]
    {
        rprintln!("gsm_dec: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(gsm_dec());

    #[cfg(feature = "log")]
    std::println!("gsm_dec: {}", success);

    std::process::exit(!success as i32);
}
