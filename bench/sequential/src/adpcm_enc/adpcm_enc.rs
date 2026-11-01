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

  Name: adpcm_enc

  Author: Sung-Soo Lim

  Function: CCITT G.722 ADPCM (Adaptive Differential Pulse Code Modulation)
    algorithm. 16khz sample rate data is stored in the array test_data[ SIZE ].
    Results are stored in the array compressed[ SIZE ].
    Execution time is determined by the constant SIZE (default value is 2000).


  Source: C Algorithms for Real-Time DSP by P. M. Embree
    and SNU-RT Benchmark Suite for Worst Case Timing Analysis
    collected and modified by S.-S. Lim <sslim@archi.snu.ac.kr>

  Original name: adpcm_encoder

  Changes: no major functional changes

  License: may be used, modified, and re-distributed freely, but the
           SNU-RT Benchmark Suite must be acknowledged

*/

/*
    This program is derived from the SNU-RT Benchmark Suite for Worst
    Case Timing Analysis by Sung-Soo Lim
*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#![allow(non_upper_case_globals)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

/* common sampling rate for sound cards on IBM/PC */
const PI: i32 = 3141;
const SIZE: usize = 3;
const IN_END: usize = 4;

struct AdpcmEncData {
    test_data: [i32; SIZE * 2],
    compressed: [i32; SIZE],
    xl: i32,
    xh: i32,
    /* variables for encoder (hi and lo) here */
    il: i32,
    szl: i32,
    spl: i32,
    sl: i32,
    el: i32,
    tqmf: [i32; 24], /* variables for transimit quadrature mirror filter here */
    delay_bpl: [i32; 6],
    delay_dltx: [i32; 6],
    nbl: i32,       /* delay line */
    al1: i32,
    al2: i32,
    plt: i32,
    plt1: i32,
    plt2: i32,
    dlt: i32,
    rlt: i32,
    rlt1: i32,
    rlt2: i32,
    detl: i32,
    deth: i32,
    sh: i32,         /* this comes from adaptive predictor */
    eh: i32,
    dh: i32,
    ih: i32,
    nbh: i32,
    szh: i32,
    sph: i32,
    ph: i32,
    yh: i32,
    delay_dhx: [i32; 6],
    delay_bph: [i32; 6],
    ah1: i32,
    ah2: i32,
    ph1: i32,
    ph2: i32,
    rh1: i32,
    rh2: i32,
}

impl AdpcmEncData {
    const fn new() -> Self {
        Self {
            test_data: [0; SIZE * 2],
            compressed: [0; SIZE],
            xl: 0,
            xh: 0,
            il: 0,
            szl: 0,
            spl: 0,
            sl: 0,
            el: 0,
            tqmf: [0; 24],
            delay_bpl: [0; 6],
            delay_dltx: [0; 6],
            nbl: 0,
            al1: 0,
            al2: 0,
            plt: 0,
            plt1: 0,
            plt2: 0,
            dlt: 0,
            rlt: 0,
            rlt1: 0,
            rlt2: 0,
            detl: 0,
            deth: 0,
            sh: 0,
            eh: 0,
            dh: 0,
            ih: 0,
            nbh: 0,
            szh: 0,
            sph: 0,
            ph: 0,
            yh: 0,
            delay_dhx: [0; 6],
            delay_bph: [0; 6],
            ah1: 0,
            ah2: 0,
            ph1: 0,
            ph2: 0,
            rh1: 0,
            rh2: 0,
        }
    }
}

/* G722 C code */

/* QMF filter coefficients:
  scaled by a factor of 4 compared to G722 CCITT recommendation */
const adpcm_enc_h: [i32; 24] = [
      12,   -44,   -44,   212,    48,  -624,   128,  1448,
    -840, -3220,  3804, 15504, 15504,  3804, -3220,  -840,
    1448,   128,  -624,    48,   212,   -44,   -44,    12,
];

const adpcm_enc_qq4_code4_table: [i32; 16] = [
        0,  -20456,  -12896,   -8968,   -6288,   -4240,   -2584,   -1200,
    20456,   12896,    8968,    6288,    4240,    2584,    1200,       0,
];

const adpcm_enc_wl_code_table: [i32; 16] = [
     -60,  3042,  1198,   538,   334,   172,    58,   -30,
    3042,  1198,   538,   334,   172,    58,   -30,   -60,
];

const adpcm_enc_ilb_table: [i32; 32] = [
    2048,  2093,  2139,  2186,  2233,  2282,  2332,  2383,
    2435,  2489,  2543,  2599,  2656,  2714,  2774,  2834,
    2896,  2960,  3025,  3091,  3158,  3228,  3298,  3371,
    3444,  3520,  3597,  3676,  3756,  3838,  3922,  4008
];

/* decision levels - pre-multiplied by 8, 0 to indicate end */
const adpcm_enc_decis_levl: [i32; 30] = [
      280,   576,   880,  1200,  1520,  1864,  2208,  2584,
     2960,  3376,  3784,  4240,  4696,  5200,  5712,  6288,
     6864,  7520,  8184,  8968,  9752, 10712, 11664, 12896,
    14120, 15840, 17560, 20456, 23352, 32767,
];


/* quantization table 31 long to make quantl look-up easier,
  last entry is for mil=30 case when wd is max */
const adpcm_enc_quant26bt_pos: [i32; 31] = [
    61,    60,    59,    58,    57,    56,    55,    54,
    53,    52,    51,    50,    49,    48,    47,    46,
    45,    44,    43,    42,    41,    40,    39,    38,
    37,    36,    35,    34,    33,    32,    32,
];

/* quantization table 31 long to make quantl look-up easier,
  last entry is for mil=30 case when wd is max */
const adpcm_enc_quant26bt_neg: [i32; 31] = [
    63,    62,    31,    30,    29,    28,    27,    26,
    25,    24,    23,    22,    21,    20,    19,    18,
    17,    16,    15,    14,    13,    12,    11,    10,
     9,     8,     7,     6,     5,     4,     4,
];

const adpcm_enc_qq2_code2_table: [i32; 4] = [
    -7408,   -1616,   7408,  1616
];

const adpcm_enc_wh_code_table: [i32; 4] = [
    798,   -214,    798,   -214
];

/* G722 encode function two ints in, one 8 bit output */

/* put input samples in xin1 = first value, xin2 = second value */
/* returns il and ih stored together */


/* MAX: 1 */
#[inline]
fn adpcm_enc_abs(n: i32) -> i32 {
    n.abs()
}

fn adpcm_enc_sin(rad: i32) -> i32 {
    let mut diff: i32;
    let mut app: i32;
    let mut inc: i32 = 1;
    /* MAX dependent on rad's value, say 50 */
    // loopbound min 0 max 1999 -> removed both while loops
    let period = 2 * PI;
    let mut rad = rad % period;
    rad = rad - if rad > (2 * PI) { period } else { 0 };

    diff = rad;
    app = diff;
    diff = (diff * (-(rad * rad))) / ((2 * inc) * (2 * inc + 1));
    app = app + diff;
    inc += 1;

    /* REALLY: while(my_fabs(diff) >= 0.00001) { */
    /* MAX: 1000 */
    // loopbound min 849 max 2424
    while adpcm_enc_abs(diff) >= 1 {
        diff = (diff * (-( rad * rad))) / ((2 * inc) * (2 * inc + 1));
        app = app + diff;
        inc += 1;
    }
    app
}

fn adpcm_enc_cos(rad: i32) -> i32 {
    adpcm_enc_sin(PI / 2 - rad)
}

/* MAX: 1 */
fn adpcm_enc_encode(
    xin1: i32, xin2: i32,
    data: &mut AdpcmEncData,
) -> i32 {
    /* transmit quadrature mirror filters implemented here */
    let h_ary = adpcm_enc_h;
    let mut xa: i64 = 0;
    let mut xb: i64 = 0;
    /* main multiply accumulate loop for samples and coefficients */
    /* MAX: 12 */
    // loopbound min 12 max 12 -> changed; original did start and end separately
    for (tqmf_chunk, h_chunk) in data.tqmf.chunks_exact(2).zip(h_ary.chunks_exact(2)) {
        xa += tqmf_chunk[0] as i64 * h_chunk[0] as i64;
        xb += tqmf_chunk[1] as i64 * h_chunk[1] as i64;
    }

    /* update delay line tqmf */
    /* MAX: 22 */
    // loopbound min 22 max 22
    for i in (2..data.tqmf.len()).rev() {
        data.tqmf[i] = data.tqmf[i - 2]
    }
    data.tqmf[1] = xin1;
    data.tqmf[0] = xin2;

    /* scale outputs */
    data.xl = ((xa + xb) >> 15) as i32;
    data.xh = ((xa - xb) >> 15) as i32;

    /* end of quadrature mirror filter code */

    /* starting with lower sub band encoder */

    /* filtez - compute predictor output section - zero section */
    data.szl = adpcm_enc_filtez(&data.delay_bpl,
                                &data.delay_dltx,
                            );

    /* filtep - compute predictor output signal (pole section) */
    data.spl = adpcm_enc_filtep(data.rlt1,
                                data.al1,
                                data.rlt2,
                                data.al2,
                            );

    /* compute the predictor output value in the lower sub_band encoder */
    data.sl = data.szl + data.spl;
    data.el = data.xl - data.sl;

    /* quantl: quantize the difference signal */
    data.il = adpcm_enc_quantl(data.el, data.detl);

    /* invqxl: computes quantized difference signal */
    /* for invqbl, truncate by 2 lsbs, so mode = 3 */
    data.dlt = ((data.detl as i64
        * adpcm_enc_qq4_code4_table[(data.il >> 2) as usize] as i64) >> 15) as i32;

    /* logscl: updates logarithmic quant. scale factor in low sub band */
    data.nbl = adpcm_enc_logscl(data.il, data.nbl);

    /* scalel: compute the quantizer scale factor in the lower sub band */
    /* calling parameters nbl and 8 (constant such that scalel can be scaleh) */
    data.detl = adpcm_enc_scalel(data.nbl, 8);

    /* parrec - simple addition to compute recontructed signal for adaptive pred */
    data.plt = data.dlt + data.szl;

    /* upzero: update zero section predictor coefficients (sixth order)*/
    /* calling parameters: dlt, dlt1, dlt2, ..., dlt6 from dlt */
    /*  bpli (linear_buffer in which all six values are delayed */
    /* return params:      updated bpli, delayed dltx */
    adpcm_enc_upzero(data.dlt, &mut data.delay_dltx, &mut data.delay_bpl);

    /* uppol2- update second predictor coefficient apl2 and delay it as al2 */
    /* calling parameters: al1, al2, plt, plt1, plt2 */
    data.al2 = adpcm_enc_uppol2(data.al1, 
                                data.al2,
                                data.plt,
                                data.plt1,
                                data.plt2,
                            );

    /* uppol1 :update first predictor coefficient apl1 and delay it as al1 */
    /* calling parameters: al1, apl2, plt, plt1 */
    data.al1 = adpcm_enc_uppol1(data.al1,
                                data.al2,
                                data.plt,
                                data.plt1,
                            );

    /* recons : compute recontructed signal for adaptive predictor */
    data.rlt = data.sl + data.dlt;

    /* done with lower sub_band encoder; now implement delays for next time*/
    data.rlt2 = data.rlt1;
    data.rlt1 = data.rlt;
    data.plt2 = data.plt1;
    data.plt1 = data.plt;

    /* high band encode */

    data.szh = adpcm_enc_filtez(&data.delay_bph,
                                &data.delay_dhx,
                            );

    data.sph = adpcm_enc_filtep(data.rh1,
                                data.ah1,
                                data.rh2,
                                data.ah2
                            );

    /* predic: sh = sph + szh */
    data.sh = data.sph + data.szh;
    /* subtra: eh = xh - sh */
    data.eh = data.xh - data.sh;

    /* quanth - quantization of difference signal for higher sub-band */
    /* quanth: in-place for speed params: eh, deth (has init. value) */
    if data.eh >= 0 {
        data.ih = 3;     /* 2,3 are pos codes */
    } else {
        data.ih = 1;     /* 0,1 are neg codes */
    }

    let decis: i32 = ((564i64 * data.deth as i64) >> 12i64) as i32;
    if adpcm_enc_abs(data.eh) > decis {
        data.ih -= 1;     /* mih = 2 case */
    }

    /* invqah: compute the quantized difference signal, higher sub-band*/
    data.dh = ((data.deth as i64
        * adpcm_enc_qq2_code2_table[data.ih as usize] as i64) >> 15i64) as i32;

    /* logsch: update logarithmic quantizer scale factor in hi sub-band*/
    data.nbh = adpcm_enc_logsch(data.ih, data.nbh);

    /* note : scalel and scaleh use same code, different parameters */
    data.deth = adpcm_enc_scalel(data.nbh, 10);

    /* parrec - add pole predictor output to quantized diff. signal */
    data.ph = data.dh + data.szh;

    /* upzero: update zero section predictor coefficients (sixth order) */
    /* calling parameters: dh, dhi, bphi */
    /* return params: updated bphi, delayed dhx */
    adpcm_enc_upzero(data.dh,
                     &mut data.delay_dhx,
                     &mut data.delay_bph,
                    );

    /* uppol2: update second predictor coef aph2 and delay as ah2 */
    /* calling params: ah1, ah2, ph, ph1, ph2 */
    data.ah2 = adpcm_enc_uppol2(data.ah1,
                                data.ah2,
                                data.ph,
                                data.ph1,
                                data.ph2,
                            );

    /* uppol1:  update first predictor coef. aph2 and delay it as ah1 */
    data.ah1 = adpcm_enc_uppol1(data.ah1,
                                data.ah2,
                                data.ph,
                                data.ph1,
                            );

    /* recons for higher sub-band */
    data.yh = data.sh + data.dh;

    /* done with higher sub-band encoder, now Delay for next time */
    data.rh2 = data.rh1;
    data.rh1 = data.yh;
    data.ph2 = data.ph1;
    data.ph1 = data.ph;

    /* multiplex ih and il to get signals together */
    data.il | (data.ih << 6)
}

/* filtez - compute predictor output signal (zero section) */
/* input: bpl1-6 and dlt1-6, output: szl */
fn adpcm_enc_filtez(bpl: &[i32; 6], dlt: &[i32; 6]) -> i32 {
    let mut zl: i64 = 0;
    /* MAX: 6 */
    // loopbound min 6 max 6 -> changed from 5 to 6 due to code structure change
    for (e1, e2) in bpl.iter().zip(dlt.iter()) {
        zl += (*e1 as i64) * (*e2 as i64);
    }

    (zl >> 14) as i32 /* x2 here */
}


/* filtep - compute predictor output signal (pole section) */
/* input rlt1-2 and al1-2, output spl */
fn adpcm_enc_filtep(rlt1: i32, al1: i32, rlt2: i32, al2: i32) -> i32 {
    let mut pl: i64 = 2 * rlt1 as i64;
    pl = al1 as i64 * pl;
    let pl2: i64 = 2 * rlt2 as i64;
    pl += al2 as i64 * pl2;

    (pl >> 15) as i32
}

/* quantl - quantize the difference signal in the lower sub-band */
fn adpcm_enc_quantl(el: i32, detl: i32) -> i32 {
    /* abs of difference signal */
    let wd: i64 = adpcm_enc_abs(el) as i64;

    /* determine mil based on decision levels and detl gain */
    /* MAX: 30 */
    // loopbound min 1 max 30
    let mil: usize = 'bp: {
        for (i, elem) in adpcm_enc_decis_levl.iter().enumerate() {
            let decis: i64 = ((*elem * detl) as i64 ) >> 15i64;
            if wd <= decis {
                break 'bp i
            }
        }
        adpcm_enc_decis_levl.len() - 1
    };    

    /* if mil=30 then wd is less than all decision levels */
    let ril = if el >= 0 { adpcm_enc_quant26bt_pos[mil] } else { adpcm_enc_quant26bt_neg[mil] };
    ril
}

/* logscl - update log quantizer scale factor in lower sub-band */
/* note that nbl is passed and returned */
fn adpcm_enc_logscl(il: i32, nbl: i32) -> i32 {
    // long long int wd;

    let wd: i64 = ( nbl as i64 * 127i64 ) >> 7i64; /* leak factor 127/128 */
    let mut nbl: i32 = wd as i32 + adpcm_enc_wl_code_table[(il >> 2) as usize];

    if nbl < 0 {
        nbl = 0;
    }
    if nbl > 18432 {
        nbl = 18432;
    }
    nbl
}

/* scalel: compute quantizer scale factor in lower or upper sub-band*/
fn adpcm_enc_scalel(nbl: i32, shift_constant: i32) -> i32 {
    let wd1: i32 = (nbl >> 6) & 31;
    let wd2: i32 = nbl >> 11;
    let wd3: i32 = adpcm_enc_ilb_table[wd1 as usize] >> (shift_constant + 1 - wd2);

    wd3 << 3
}

/* upzero - inputs: dlt, dlti[ 0-5 ], bli[ 0-5 ], outputs: updated bli[ 0-5 ] */
/* also implements delay of bli and update of dlti from dlt */
fn adpcm_enc_upzero(dlt: i32, dlti: &mut [i32; 6], bli: &mut [i32; 6]) {
    /*if dlt is zero, then no sum into bli */
    if dlt == 0 {
        // loopbound min 6 max 6
        for elem in bli.iter_mut() {
            *elem = ((255i64 * (*elem as i64)) >> 8i64) as i32; /* leak factor of 255/256 */
        }
    } else {
        // loopbound min 6 max 6
        for i in 0..6 {
            let wd2: i32;
            if dlt as i64 * dlti[i] as i64 >= 0 {
                wd2 = 128;
            } else {
                wd2 = -128;
            }
            let wd3 = ((255i64 * bli[i] as i64) >> 8i64) as i32; /* leak factor of 255/256 */
            bli[i] = wd2 + wd3;
        }
    }
    /* implement delay line for dlt */
    dlti[5] = dlti[4];
    dlti[4] = dlti[3];
    dlti[3] = dlti[2];
    dlti[1] = dlti[0];
    dlti[0] = dlt;
}

/* uppol2 - update second predictor coefficient (pole section) */
/* inputs: al1, al2, plt, plt1, plt2. outputs: apl2 */
fn adpcm_enc_uppol2(al1: i32, al2: i32, plt: i32, plt1: i32, plt2: i32) -> i32 {
    let mut wd2 = 4i64 * al1 as i64;
    if plt as i64 * plt1 as i64 >= 0i64 {
        wd2 = -wd2;    /* check same sign */
    }        
    wd2 = wd2 >> 7;      /* gain of 1/128 */

    let wd4: i64 = wd2 + if plt as i64 * plt2 as i64 >= 0i64 { 128 } else { -128 };
    let mut apl2: i32 = (wd4 + (127i64 * al2 as i64 >> 7i64)) as i32; /* leak factor of 127/128 */

    /* apl2 is limited to +-.75 */
    if apl2 > 12288 {
        apl2 = 12288;
    }
    if apl2 < -12288 {
        apl2 = -12288;
    }
    apl2
}

/* uppol1 - update first predictor coefficient (pole section) */
/* inputs: al1, apl2, plt, plt1. outputs: apl1 */
fn adpcm_enc_uppol1(al1: i32, apl2: i32, plt: i32, plt1: i32) -> i32 {
    let wd2 = (al1 as i64 * 255i64) >> 8i64; /* leak factor of 255/256 */
    let mut apl1: i32 = wd2 as i32 + if plt as i64 * plt1 as i64 >= 0i64 { 192 } else { -192 };

    /* note: wd3= .9375-.75 is always positive */
    let wd3 = 15360 - apl2;     /* limit value */
    if apl1 > wd3 {
        apl1 = wd3;
    }
    if apl1 < -wd3 {
        apl1 = -wd3;
    }
    apl1
}

/* logsch - update log quantizer scale factor in higher sub-band */
/* note that nbh is passed and returned */
fn adpcm_enc_logsch(ih: i32, nbh: i32) -> i32 {
    let wd: i32 = ((nbh as i64 * 127i64) >> 7i64) as i32;   /* leak factor 127/128 */
    let mut nbh = wd + adpcm_enc_wh_code_table[ih as usize];

    if nbh < 0 {
        nbh = 0;
    }
    if nbh > 22528 {
        nbh = 22528;
    }
    nbh
}

/*
  Initialization- and return-value-related functions
*/

/* clear all storage locations */

fn adpcm_enc_reset(data: &mut AdpcmEncData) {
    data.detl = 32;   /* reset to min scale factor */
    data.deth = 8;

    data.nbl = 0;
    data.al1 = 0;
    data.al2 = 0;
    data.plt1 = 0;
    data.plt2 = 0;
    data.rlt1 = 0;
    data.rlt2 = 0;
    data.nbh = 0;
    data.ah1 = 0;
    data.ah2 = 0;
    data.ph1 = 0;
    data.ph2 = 0;
    data.rh1 = 0;
    data.rh2 = 0;

    // loopbound min 6 max 6
    data.delay_dltx.fill(0);
    data.delay_dhx.fill(0);

    // loopbound min 6 max 6
    data.delay_bpl.fill(0);
    data.delay_bph.fill(0);

    // loopbound min 23 max 23
    for i in 0..23 {
        data.tqmf[i] = 0;
    }
}

fn adpcm_enc_init(data: &mut AdpcmEncData) {
    /* reset, initialize required memory */
    adpcm_enc_reset(data);

    /* read in amplitude and frequency for test data */
    let j = 10;
    let f = 2000;

    /* 16 KHz sample rate */
    /* XXmain_0, MAX: 2 */
    /* Since the number of times we loop in my_sin depends on the argument we
        add the fact: xxmain_0:[  ]: */
    // loopbound min 3 max 3
    for i in 0..SIZE {
        data.test_data[i] = j * adpcm_enc_cos(f * PI * i as i32);
    }
}

fn adpcm_enc_return(data: &mut AdpcmEncData) -> bool {
    let mut checksum: i32 = 0;
    // loopbound min 2 max 2
    for i in (0..IN_END).step_by(2) {
        checksum += data.compressed[i / 2];
    }
    checksum == 385
}

/*
  Main functions
*/

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn adpcm_enc_main(data: &mut AdpcmEncData) {
    /* MAX: 2 */
    // loopbound min 2 max 2
    for i in (0..IN_END).step_by(2) {
        data.compressed[i / 2] = adpcm_enc_encode(data.test_data[i],
                                                  data.test_data[i + 1],
                                                  data,
                                                );
    }
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn adpcm_enc_log(data: &mut AdpcmEncData) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        adpcm_enc_init(black_box(data));
        let start = black_box(DWT::cycle_count());
        adpcm_enc_main(black_box(data));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("adpcm_enc - elapsed: {}", cycles);
    });

    black_box(adpcm_enc_return(black_box(data)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn adpcm_enc_std(data: &mut AdpcmEncData) -> bool {
    black_box(adpcm_enc_init(data));
    adpcm_enc_main(black_box(data));
    black_box(adpcm_enc_return(black_box(data)))
}

fn adpcm_enc() -> bool {
    static mut DATA: AdpcmEncData = AdpcmEncData::new();
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let data = unsafe { &mut *{ &raw mut DATA } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(adpcm_enc_log(black_box(data)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(adpcm_enc_std(black_box(data)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(adpcm_enc());
    
    #[cfg(feature = "log")]
    {
        rprintln!("adpcm_enc: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(adpcm_enc());

    #[cfg(feature = "log")]
    std::println!("adpcm_enc: {}", success);

    std::process::exit(!success as i32);
}