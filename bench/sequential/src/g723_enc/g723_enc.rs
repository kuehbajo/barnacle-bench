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

  Version V 1.x

  Name: g723_enc

  Author: Unknown

  Function: g723 encoder.

  Source: SUN Microsystems

  Changes: The benchmark was changed to use the g723 encoder

  License: "Unrestricted use" (see license.txt)

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
  Declaration of data types
*/

/*
   The following is the definition of the state structure
   used by the G.721/G.723 encoder and decoder to preserve their internal
   state between successive calls.  The meanings of the majority
   of the state structure fields are explained in detail in the
   CCITT Recommendation G.721.  The field names are essentially indentical
   to variable names in the bit level description of the coding algorithm
   included in this Recommendation.
*/

struct StateT {
    yl: i64,  /* Locked or steady state step size multiplier. */
    yu: i16, /* Unlocked or non-steady state step size multiplier. */
    dms: i16,  /* Short term energy estimate. */
    dml: i16,  /* Long term energy estimate. */
    ap: i16, /* Linear weighting coefficient of 'yl' and 'yu'. */

    a: [i16; 2], /* Coefficients of pole portion of prediction filter. */
    b: [i16; 6], /* Coefficients of zero portion of prediction filter. */
    pk: [i16; 2],  /*
            Signs of previous two samples of a partially
            reconstructed signal.
    */
    dq: [i16; 6],  /*
            Previous 6 samples of the quantized difference
            signal represented in an internal floating point
            format.
    */
    sr: [i16; 2],  /*
            Previous 2 samples of the quantized difference
            signal represented in an internal floating point
            format.
    */
    td: i8,  /* delayed tone detect, new in 1988 version */
}

impl StateT {
    const fn new() -> Self {
        StateT {
            yl: 0,
            yu: 0,
            dms: 0,
            dml: 0,
            ap: 0,
            a: [0; 2],
            b: [0; 6],
            pk: [0; 2],
            dq: [0; 6],
            sr: [0; 2],
            td: 0,
        }
    }
}

/*
  Declaration of global variables
*/

const INPUT_TAB: [u32; 256] = [
    51, 17, 31, 53, 95, 17, 70, 22, 49, 12,  8, 39, 28, 37, 99, 54,
    77, 65, 77, 78, 83, 15, 63, 31, 35, 92, 52, 40, 61, 79, 94, 87,
    87, 68, 76, 58, 39, 35, 20, 83, 42, 46, 98, 12, 21, 96, 74, 41,
    78, 76, 96,  2, 32, 76, 24, 59,  4, 96, 32,  5, 44, 92, 57, 12,
    57, 25, 50, 23, 48, 41, 88, 43, 36, 38,  4, 16, 52, 70,  9, 40,
    78, 24, 34, 23, 30, 30, 89,  3, 65, 40, 68, 73, 94, 23, 84, 97,
    78, 43, 68, 81, 16, 28, 13, 87, 75, 21, 14, 29, 81, 22, 56, 72,
    19, 99, 25, 43, 76, 86, 90, 98, 39, 43, 12, 46, 24, 99, 65, 61,
    24, 45, 79,  7, 48, 15, 24, 95, 62, 99, 48, 80, 75, 38, 48, 53,
    9, 60, 35, 14, 78, 71, 45, 71,  9, 97, 55, 74, 58, 64, 78, 18,
    30, 28, 69, 29, 57, 42, 30, 44, 57, 49, 61, 42, 13, 25,  3, 98,
    11, 38, 65, 35, 55, 36, 57, 48, 16, 62, 17, 56, 29, 88, 84, 85,
    90, 60, 54, 16, 66, 69, 26, 10, 82, 19, 42, 35, 84, 13, 26, 17,
    48, 38, 50, 50, 35, 53, 12, 52, 61, 74, 56, 34, 80, 59, 26, 67,
    55, 79, 89, 89,  6, 80, 91, 65, 16, 30, 16, 28, 85, 54,  3, 20,
    2, 36, 62, 52, 55, 15, 83,  3,  2, 38, 62,  2, 63, 92, 37, 73,
];

const POWER2: [i16; 15] = [
    1, 2, 4, 8, 0x10, 0x20, 0x40, 0x80,
    0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000, 0x4000,
];

/*
   Maps G.723_24 code word to reconstructed scale factor normalized log
   magnitude values.
*/

const QTAB_723_24: [i16; 3] = [8, 218, 331];

/*
   Maps G.721 code word to reconstructed scale factor normalized log
   magnitude values.
*/
const DQLNTAB: [i16; 16] = [
    -2048, 4, 135, 213, 273, 323, 373, 425,
    425, 373, 323, 273, 213, 135, 4, -2048,
];

/* Maps G.721 code word to log of scale factor multiplier. */
const WITAB: [i16; 16] = [
    -12, 18, 41, 64, 112, 198, 355, 1122,
    1122, 355, 198, 112, 64, 41, 18, -12,
];

/*
   Maps G.721 code words to a set of values whose long and short
   term averages are computed and then compared to give an indication
   how stationary (steady state) the signal is.
*/
const FITAB: [i16; 16] = [
    0, 0, 0, 0x200, 0x200, 0x200, 0x600, 0xE00,
    0xE00, 0x600, 0x200, 0x200, 0x200, 0, 0, 0,
];

/*
  Declaration of macros
*/

const AUDIO_ENCODING_ULAW: i16 = 1; /* ISDN u-law */
const AUDIO_ENCODING_ALAW: i16 = 2; /* ISDN A-law */
const AUDIO_ENCODING_LINEAR: i16 = 3; /* PCM 2's-complement (0-center) */

const BIAS: i16 = 0x84;    /* Bias for linear code. */

const SIGN_BIT: i16 = 0x80;    /* Sign bit for a A-law byte. */
const QUANT_MASK: i16 = 0xf;   /* Quantization field mask. */
const SEG_SHIFT: i16 = 4;   /* Left shift for segment number. */
const SEG_MASK: i16 = 0x70;    /* Segment field mask. */

/*
  Arithmetic math functions
*/

/*
   g723_enc_fmult()

   returns the integer product of the 14-bit integer "an" and
   "floating point" representation (4-bit exponent, 6-bit mantessa) "srn".
*/
fn g723_enc_fmult(
  an: i16,
  srn: i16
) -> i16 {
    let anmag = if an > 0 { an } else { -an & 0x1FFF };
    let anexp = g723_enc_quan(anmag, &POWER2, 3) - 6;
    let anmant = if anmag == 0 { 32 } else if anexp >= 0 { anmag >> anexp } else { anmag << -anexp };
    let wanexp = anexp + ((srn >> 6) & 0xF) - 13;

    let wanmant = ( anmant * ( srn & 077 ) + 0x30 ) >> 4;
    let retval = if wanexp >= 0 { (wanmant << wanexp) & 0x7FFF } else { wanmant >> -wanexp };

    if an ^ srn < 0 { -retval } else { retval }
}

/*
  Algorithm core functions
*/

/*
   g723_enc_quan()

   quantizes the input val against the table of size short integers.
   It returns i if table[ i - 1 ] <= val < table[ i ].

   Using linear search for simple coding.
*/
fn g723_enc_quan(
  val: i16,
  table: &[i16],
  size: usize
) -> i16 {
    let mut j: i16 = 0;
    let mut k = true;

    // loopbound min 3 max 15
    for i in 0..size {
        if k {
            if val < table[i] {
                j = i as i16;
                k = false;
            }
        }
    }
    j
}


/*
   g723_enc_predictor_zero()

   computes the estimated signal from 6-zero predictor.

*/
fn g723_enc_predictor_zero(state: &mut StateT) -> i16 {
    let mut sezi = g723_enc_fmult(state.b[0] >> 2, state.dq[0]);
    // loopbound min 5 max 5
    for i in 1..6 { /* ACCUM */
        sezi += g723_enc_fmult(state.b[i] >> 2, state.dq[i]);
    }
    sezi
}

/*
   g723_enc_predictor_pole()

   computes the estimated signal from 2-pole predictor.

*/
fn g723_enc_predictor_pole(state: &mut StateT) -> i16 {
    g723_enc_fmult(state.a[1] >> 2, state.sr[1]) +
        g723_enc_fmult(state.a[0] >> 2, state.sr[0])
}

/*
   g723_enc_step_size()

   computes the quantization step size of the adaptive quantizer.

*/
fn g723_enc_step_size(state: &mut StateT) -> i16 {
    if state.ap >= 256 {
        state.yu
    } else {
        let mut y = (state.yl >> 6) as i16;
        let dif = state.yu - y;
        let al = state.ap >> 2;
        if dif > 0 {
            y += dif * al >> 6;
        } else {
            if dif < 0 {
                y += ( dif * al + 0x3F ) >> 6;
            }
        }
        y
    }
}

/*
   g723_enc_quantize()

   Given a raw sample, 'd', of the difference signal and a
   quantization step size scale factor, 'y', this routine returns the
   ADPCM codeword to which that sample gets quantized.  The step
   size scale factor division operation is done in the log base 2 domain
   as a subtraction.
*/
fn g723_enc_quantize(
    d: i16,  /* Raw difference signal sample */
    y: i16,  /* Step size multiplier */
    table: &[i16], /* quantization table */
    size: usize,  /* table size of short integers */
) -> i16 {
    /*
        LOG

        Compute base 2 log of 'd', and store in 'dl'.
    */
    let dqm = d.abs();  /* Magnitude of 'd' */
    let exp = g723_enc_quan(dqm >> 1, &POWER2, 15);    /* Integer part of base 2 log of 'd' */
    let mant = ((((dqm as i32) << 7) >> exp) & 0x7F) as i16; /* Fractional portion of base 2 log */
    let dl = (exp << 7) + mant; /* Log of magnitude of 'd' */

    /*
        SUBTB

        "Divide" by step size multiplier.
    */
    let dln = dl - ( y >> 2 );  /* Step size scale factor normalized log */

    /*
        QUAN

        Obtain codword i for 'd'.
    */
    let i = g723_enc_quan(dln, table, size);

    if d < 0  {          /* take 1's complement of i */
        (size << 1) as i16 + 1 - i
    } else if i == 0 {      /* take 1's complement of 0 */
        ((size << 1) + 1) as i16 /* new in 1988 */
    } else {
        i
    }   
}
/*
   g723_enc_reconstruct()

   Returns reconstructed difference signal 'dq' obtained from
   codeword 'i' and quantization step size scale factor 'y'.
   Multiplication is performed in log base 2 domain as addition.
*/
fn g723_enc_reconstruct(
    sign: i16, /* 0 for non-negative value */
    dqln: i16, /* G.72x codeword */
    y: i16,  /* Step size multiplier */
) -> i16 {
    let dql = dqln + (y >> 2);  /* ADDA */  /* Log of 'dq' magnitude */

    if dql < 0 {
        if sign != 0 { -0x8000 } else { 0 }
    } else {    /* ANTILOG */
        let dex = (dql >> 7) & 15;  /* Integer part of log */
        let dqt = 128 + (dql & 127);    /* Reconstructed difference signal sample */
        let dq = (dqt << 7) >> (14 - dex);
        if sign != 0 { ((dq as i32) - 0x8000) as i16 } else { dq }
    }
}


/*
   g723_enc_update()

   updates the state variables for each output code
*/
fn g723_enc_update(
    code_size: i16,     /* distinguish 723_40 with others */
    y: i16,             /* quantizer step size */
    wi: i16,            /* scale factor multiplier */
    fi: i16,            /* for long/short term energies */
    dq: i16,            /* quantized prediction difference */
    sr: i16,            /* reconstructed signal */
    dqsez: i16,         /* difference from 2-pole predictor */
    state: &mut StateT  /* coder state pointer */
) {
    let tr; /* tone/transition detector */
    let mut exp: i16;   /* FLOAT A */
    let mut a2p: i16 = 0;    /* LIMC */
    let a1ul: i16;   /* UPA1 */
    let pks1: i16;  /* UPA2 */

    let pk0 = if dqsez < 0 { 1 } else { 0 };  /* needed in updating predictor poles */

    let mut mag = dq & 0x7FFF;    /* Adaptive predictor, prediction difference magnitude */
    /* TRANS */
    let ylint = (state.yl >> 15) as i16;  /* exponent part of yl */
    let ylfrac: i16 = ((state.yl >> 10) & 0x1F) as i16;  /* fractional part of yl */
    let thr1 = (32 + ylfrac) << ylint;    /* threshold */
    let thr2 = if ylint > 9 { 31 << 10 } else { thr1 }; /* limit thr2 to 31 << 10 */
    let dqthr = (thr2 + (thr2 >> 1)) >> 1;  /* dqthr = 0.75 * thr2 */
    if state.td == 0 {  /* signal supposed voice */
        tr = false;
    } else if mag <= dqthr {    /* supposed data, but small mag */
        tr = false;     /* treated as voice */
    } else {    /* signal is data (modem) */
        tr = true;
    }

    /*
        Quantizer scale factor adaptation.
    */

    /* FUNCTW & FILTD & DELAY */
    /* update non-steady state step size multiplier */
    state.yu = y + ((wi - y) >> 5);

    /* LIMB */
    if state.yu < 544 { /* 544 <= yu <= 5120 */
        state.yu = 544;
    } else if state.yu > 5120 {
        state.yu = 5120;
    }   

    /* FILTE & DELAY */
    /* update steady state step size multiplier */
    state.yl += state.yu as i64 + (-state.yl >> 6);

    /*
        Adaptive predictor coefficients.
    */
    if tr {      /* reset a's and b's for modem signal */
        state.a[0] = 0;
        state.a[1] = 0;
        state.b[0] = 0;
        state.b[1] = 0;
        state.b[2] = 0;
        state.b[3] = 0;
        state.b[4] = 0;
        state.b[5] = 0;
    } else {      /* update a's and b's */
        pks1 = pk0 ^ state.pk[0];    /* UPA2 */

        /* update predictor pole a[ 1 ] */
        a2p = state.a[1] - (state.a[1] >> 7);
        if dqsez != 0 {
            let fa1 = if pks1 != 0 { state.a[0] } else { -state.a[0] };
            if fa1 < -8191 {    /* a2p = function of fa1 */
                a2p -= 0x100;
            } else if fa1 > 8191 {
                a2p += 0xFF;
            } else {
                a2p += fa1 >> 5;
            }
            if (pk0 ^ state.pk[1]) != 0 {
                /* LIMC */
                if a2p <= -12160 {
                    a2p = -12288;
                } else if a2p >= 12416 {
                    a2p = 12288;
                } else {
                    a2p -= 0x80;
                }
            } else {
                if a2p <= -12416 {
                    a2p = -12288;
                } else if a2p >= 12160 {
                    a2p = 12288;
                } else {
                    a2p += 0x80;
                }
            }
        }

        /* TRIGB & DELAY */
        state.a[1] = a2p;

        /* UPA1 */
        /* update predictor pole a[ 0 ] */
        state.a[0] -= state.a[0] >> 8;
        if dqsez != 0 {
            if pks1 == 0 {
                state.a[0] += 192;
            } else {
                state.a[0] -= 192;
            }
        }

        /* LIMD */
        a1ul = 15360 - a2p;
        if state.a[0] < -a1ul {
            state.a[0] = -a1ul;
        } else if state.a[0] > a1ul {
            state.a[0] = a1ul;
        }

        /* UPB : update predictor zeros b[ 6 ] */
        // loopbound min 6 max 6
        for (i, el) in state.b.iter_mut().enumerate() {
            if code_size == 5 { /* for 40Kbps G.723 */
                *el -= *el >> 9;
            } else {    /* for G.721 and 24Kbps G.723 */
                *el -= *el >> 8;
            }
            if (dq & 0x7FFF) != 0 {      /* XOR */
                if (dq ^ state.dq[i]) >= 0 {
                    *el += 128;
                } else {
                    *el -= 128;
                }
            }
        }
    }

    // loopbound min 5 max 5
    for cnt in (1..state.dq.len()).rev() {
        state.dq[cnt] = state.dq[cnt - 1];
    }
        
    /* FLOAT A : convert dq[ 0 ] to 4-bit exp, 6-bit mantissa f.p. */
    if mag == 0 {
        state.dq[0] = if dq >= 0 { 0x20 } else { 0xFC20u16 as i16 };
    } else {
        exp = g723_enc_quan(mag, &POWER2, 15);
        state.dq[0] = if dq >= 0 {
                        (exp << 6) + ((mag << 6) >> exp)
                    } else {
                        (exp << 6) + ((mag << 6) >> exp) - 0x400
                    };
    }

    state.sr[1] = state.sr[0];
    /* FLOAT B : convert sr to 4-bit exp., 6-bit mantissa f.p. */
    if sr == 0 {
        state.sr[0] = 0x20;
    } else if sr > 0 {
        exp = g723_enc_quan(sr, &POWER2, 15);
        state.sr[0] = (exp << 6) + ((sr << 6) >> exp);
    } else if sr > -32768 {
        mag = -sr;
        exp = g723_enc_quan(mag, &POWER2, 15);
        state.sr[0] =  (exp << 6) + ((mag << 6) >> exp) - 0x400;
    } else {
        state.sr[0] = 0xFC20u16 as i16;
    }

    /* DELAY A */
    state.pk[1] = state.pk[0];
    state.pk[0] = pk0;

    /* TONE */
    if tr {    /* this sample has been treated as data */
        state.td = 0;  /* next one will be treated as voice */
    } else if a2p < -11776 {  /* small sample-to-sample correlation */
        state.td = 1;  /* signal may be data */
    } else {    /* signal is voice */
        state.td = 0;
    }

    /*
        Adaptation speed control.
    */
    state.dms += (fi - state.dms) >> 5;   /* FILTA */
    state.dml += ((fi << 2) - state.dml) >> 7;  /* FILTB */

    if tr {
        state.ap = 256;
    } else if y < 1536 {  /* SUBTC */
        state.ap += (0x200 - state.ap) >> 4;
    } else if state.td == 1 {
        state.ap += (0x200 - state.ap) >> 4;
    } else if ((state.dms << 2) - state.dml).abs() >= (state.dml >> 3) {
        state.ap += (0x200 - state.ap) >> 4;
    } else {
        state.ap += -state.ap >> 4;
    }
}


/*
   g723_enc_alaw2linear() - Convert an A-law value to 16-bit linear PCM

*/
fn g723_enc_alaw2linear(a_val: u8) -> i16 {
    let a_val = (a_val ^ 0x55) as i16;

    let mut t = (a_val & QUANT_MASK) << 4;
    let seg = (a_val & SEG_MASK) >> SEG_SHIFT;
    match seg {
        0 => {
            t += 8;
        },
        1 => {
            t += 0x108;
        },
        _ => {
            t += 0x108;
            t <<= seg - 1;
        },
    }
    if (a_val & SIGN_BIT) != 0 { t } else { -t }
}

/*
   g723_enc_ulaw2linear() - Convert a u-law value to 16-bit linear PCM

   First, a biased linear code is derived from the code word. An unbiased
   output can then be obtained by subtracting 33 from the biased code.

   Note that this function expects to be passed the complement of the
   original code word. This is in keeping with ISDN conventions.
*/
fn g723_enc_ulaw2linear(u_val: u8) -> i16 {
    /* Complement to obtain normal u-law value. */
    let u_val = !u_val as i16;

    /*
        Extract and bias the quantization bits. Then
        shift up by the segment number and subtract out the bias.
    */
    let mut t = ((u_val & QUANT_MASK) << 3) + BIAS;
    t <<= (u_val & SEG_MASK) >> SEG_SHIFT; // ( unsigned int )

    if (u_val & SIGN_BIT) != 0 { BIAS - t } else { t - BIAS }
}

/*
   g723_enc_g723_24_encoder()

   Encodes a linear PCM, A-law or u-law input sample and returns its 3-bit code.
   Returns -1 if invalid input coding value.
*/
fn g723_enc_g723_24_encoder(
    sl: i16,
    in_coding: i16,
    state: &mut StateT,
) -> i16 {
    let mut sl = sl;
    match in_coding { /* linearize input sample to 14-bit PCM */
        AUDIO_ENCODING_ALAW => {
            sl = g723_enc_alaw2linear(sl as u8) >> 2;
        },
        AUDIO_ENCODING_ULAW => {
            sl = g723_enc_ulaw2linear(sl as u8) >> 2;
        },
        AUDIO_ENCODING_LINEAR => {
            sl >>= 2;   /* sl of 14-bit dynamic range */
        },
        _ => {
            return -1;
        }
    }

    let sezi = g723_enc_predictor_zero(state);  /* ACCUM */
    let sez = sezi >> 1;    /* ACCUM */
    let sei = sezi + g723_enc_predictor_pole(state);    /* ACCUM */
    let se = sei >> 1;      /* ACCUM, se = estimated signal */

    let d = sl - se;      /* SUBTA, d = estimation diff. */

    /* quantize prediction difference d */
    let y = g723_enc_step_size(state); /* MIX, quantizer step size */
    let i = g723_enc_quantize(d, y, &QTAB_723_24, 3); /* i = ADPCM code */
    let dq = g723_enc_reconstruct(i & 4, DQLNTAB[i as usize], y); /* quantized diff. */

    let sr = if dq < 0 { se - (dq & 0x3FFF) } else  { se + dq }; /* ADDB, reconstructed signal */

    let dqsez = sr + sez - se;    /* ADDC, pole prediction diff. */

    g723_enc_update(3, y, WITAB[i as usize], FITAB[i as usize], dq, sr, dqsez, state);

    i
}

/*
   Pack output codes into bytes and write them to stdout.
   Returns 1 if there is residual output, else returns 0.
*/

fn g723_enc_pack_output(
    code: u8,
    bits: i16,
    output:&mut [u16; 256],
    out_buffer: &mut u16,
    out_bits: &mut i16,
    index: &mut usize,
) -> bool {
    *out_buffer |= (code as u16) << *out_bits;
    *out_bits += bits;
    if *out_bits >= 8 {
        let out_byte = *out_buffer & 0xff;
        *out_bits -= 8;
        *out_buffer >>= 8;
        output[*index] = out_byte;
        *index =  *index + 1;
    }

    *out_bits > 0
}

/*
  Initialization- and return-value-related functions
*/

/*
   g723_enc_init_state()

   This routine initializes and/or resets the g72x_state structure
   pointed to by 'state'.
   All the initial state values are specified in the CCITT G.721 document.
*/
fn g723_enc_init_state(state: &mut StateT) {
    state.yl = 34816;
    state.yu = 544;
    state.dms = 0;
    state.dml = 0;
    state.ap = 0;

    // loopbound min 2 max 2
    for cnta in 0..2 {
        state.a[cnta] = 0;
        state.pk[cnta] = 0;
        state.sr[cnta] = 32;
    }
    // loopbound min 6 max 6
    for cnta in 0..6 {
        state.b[cnta] = 0;
        state.dq[cnta] = 32;
    }
    state.td = 0;
}

fn g723_enc_init(
    state: &mut StateT,
    out_buffer: &mut u16,
    out_bits: &mut i16,
    index: &mut usize,
) {
    g723_enc_init_state(state);
    *out_buffer = 0;
    *out_bits = 0;
    *index = 0;
}

fn g723_enc_return(output:&mut [u16; 256]) -> bool {
    let mut checksum = 0;

    // loopbound min 256 max 256
    for el in output.iter() {
        checksum += *el;
    }
    checksum == 24284
}

/*
  Main functions
*/

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn g723_enc_main(
    state: &mut StateT,
    output:&mut [u16; 256],
    out_buffer: &mut u16,
    out_bits: &mut i16,
    index: &mut usize
) {
    let enc_bits = 3;
    let in_coding = AUDIO_ENCODING_ALAW;
    let mut resid = false;

    // loopbound min 256 max 256
    for &el in INPUT_TAB.iter() {
        let sample_short = el;
        let code = g723_enc_g723_24_encoder(sample_short as i16, in_coding, state);

        resid = g723_enc_pack_output(
            code as u8, enc_bits, output, out_buffer, out_bits, index
        );
    }

    /* Write zero codes until all residual codes are written out */
    // loopbound min 0 max 0
    while resid {
        resid = g723_enc_pack_output(
            0, enc_bits, output, out_buffer, out_bits, index
        );
    }
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn g723_enc_log(
    state: &mut StateT,
    output:&mut [u16; 256],
    out_buffer: &mut u16,
    out_bits: &mut i16,
    index: &mut usize
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        g723_enc_init(
            black_box(state), black_box(out_buffer),
            black_box(out_bits), black_box(index),
        );
        let start = black_box(DWT::cycle_count());
        g723_enc_main(
            black_box(state), black_box(output),
            black_box(out_buffer), black_box(out_bits),
            black_box(index),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("g723_enc - elapsed: {}", cycles);
    });

    black_box(g723_enc_return(black_box(output)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn g723_enc_std(
    state: &mut StateT,
    output:&mut [u16; 256],
    out_buffer: &mut u16,
    out_bits: &mut i16,
    index: &mut usize
) -> bool {
    g723_enc_init(
        black_box(state), black_box(out_buffer),
        black_box(out_bits), black_box(index),
    );
    g723_enc_main(
        black_box(state), black_box(output),
        black_box(out_buffer), black_box(out_bits),
        black_box(index),
    );
    black_box(g723_enc_return(black_box(output)))
}

fn g723_enc() -> bool {
    static mut STATE: StateT = StateT::new();
    static mut OUTPUT: [u16; 256] = [0; 256];
    static mut OUT_BUFFER: u16 = 0;
    static mut OUT_BITS: i16 = 0; 
    static mut INDEX: usize = 0;

    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let state = unsafe { &mut *{ &raw mut STATE } };
    let output = unsafe { &mut *{ &raw mut OUTPUT } };
    let out_buffer = unsafe { &mut *{ &raw mut OUT_BUFFER } };
    let out_bits = unsafe { &mut *{ &raw mut OUT_BITS } };
    let index = unsafe { &mut *{ &raw mut INDEX } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(g723_enc_log(
            black_box(state), black_box(output),
            black_box(out_buffer), black_box(out_bits),
            black_box(index),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(g723_enc_std(
            black_box(state), black_box(output),
            black_box(out_buffer), black_box(out_bits),
            black_box(index),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(g723_enc());
    
    #[cfg(feature = "log")]
    {
        rprintln!("g723_enc: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(g723_enc());

    #[cfg(feature = "log")]
    std::println!("g723_enc: {}", success);

    std::process::exit(!success as i32);
}
