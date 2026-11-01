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

    Name: adpcm_dec

    Author: Sung-Soo Lim

    Function:
      CCITT G.722 ADPCM (Adaptive Differential Pulse Code Modulation)
      algorithm.
      16khz sample rate data is stored in the array test_data[ SIZE ].
      Results are stored in the array compressed[ SIZE ] and result[ SIZE ].
      Execution time is determined by the constant SIZE (default value
      is 2000).

    Source: SNU-RT Benchmark Suite

    Changes: adpcm benchmark was split into decode and encode benchmark

    License: may be used, modified, and re-distributed freely, but
            the SNU-RT Benchmark Suite must be acknowledged

*/

/*
    This program is derived from the SNU-RT Benchmark Suite for Worst
    Case Timing Analysis by Sung-Soo Lim

    Original source: C Algorithms for Real-Time DSP by P. M. Embree
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

/*
  Declaration of global variables
*/

struct AdpcmDecData {
    test_data: [i32; SIZE * 2],
    result: [i32; SIZE * 2],
    /* variables for receive quadrature mirror filter here */
    accumc: [i32; 11],
    accumd: [i32; 11],
    /* outputs of decode() */
    xout1: i32,
    xout2: i32,

    xs: i32,
    xd: i32,
    /* variables for encoder (hi and lo) here */
    il: i32,

    nbl: i32,     /* delay line */
    al1: i32,
    al2: i32,
    plt1: i32,
    plt2: i32,
    rlt1: i32,
    rlt2: i32,
    detl: i32,
    deth: i32,

    ih: i32,
    nbh: i32,

    rh: i32,
    delay_dhx: [i32; 6],
    delay_bph: [i32; 6],
    ah1: i32,
    ah2: i32,
    ph1: i32,
    ph2: i32,
    rh1: i32,
    rh2: i32,
    /* variables for decoder here */
    ilr: i32,
    rl: i32,
    dec_deth: i32,
    dec_detl: i32,
    dec_dlt: i32,

    dec_del_bpl: [i32; 6],
    dec_del_dltx: [i32; 6],
    dec_plt: i32,
    dec_plt1: i32,
    dec_plt2: i32,
    dec_szl: i32,
    dec_spl: i32,
    dec_sl: i32,
    dec_rlt1: i32,
    dec_rlt2: i32,
    dec_rlt: i32,
    dec_al1: i32,
    dec_al2: i32,
    dl: i32,
    dec_nbl: i32,
  
    dec_dh: i32,
    dec_nbh: i32,
    /* variables used in filtez */
    dec_del_bph: [i32; 6],
    dec_del_dhx: [i32; 6],
    dec_szh: i32,
    /* variables used in filtep */
    dec_rh1: i32,
    dec_rh2: i32,
    dec_ah1: i32,
    dec_ah2: i32,
    dec_ph: i32,
    dec_sph: i32,
    dec_sh: i32,
    dec_ph1: i32,
    dec_ph2: i32,
}

impl AdpcmDecData {
    const fn new() -> Self {
        Self {
            test_data: [0; SIZE * 2],
            result: [0; SIZE * 2],
            accumc: [0; 11],
            accumd: [0; 11],
            xout1: 0,
            xout2: 0,
            xs: 0,
            xd: 0,
            il: 0,
            nbl: 0,
            al1: 0,
            al2: 0,
            plt1: 0,
            plt2: 0,
            rlt1: 0,
            rlt2: 0,
            detl: 0,
            deth: 0,
            ih: 0,
            nbh: 0,
            rh: 0,
            delay_dhx: [0; 6],
            delay_bph: [0; 6],
            ah1: 0,
            ah2: 0,
            ph1: 0,
            ph2: 0,
            rh1: 0,
            rh2: 0,
            ilr: 0,
            rl: 0,
            dec_deth: 0,
            dec_detl: 0,
            dec_dlt: 0,
            dec_del_bpl: [0; 6],
            dec_del_dltx: [0; 6],
            dec_plt: 0,
            dec_plt1: 0,
            dec_plt2: 0,
            dec_szl: 0,
            dec_spl: 0,
            dec_sl: 0,
            dec_rlt1: 0,
            dec_rlt2: 0,
            dec_rlt: 0,
            dec_al1: 0,
            dec_al2: 0,
            dl: 0,
            dec_nbl: 0,
            dec_dh: 0,
            dec_nbh: 0,
            dec_del_bph: [0; 6],
            dec_del_dhx: [0; 6],
            dec_szh: 0,
            dec_rh1: 0,
            dec_rh2: 0,
            dec_ah1: 0,
            dec_ah2: 0,
            dec_ph: 0,
            dec_sph: 0,
            dec_sh: 0,
            dec_ph1: 0,
            dec_ph2: 0,
        }
    }
}

/* Input data for the decoder usually generated by the encoder. */
const adpcm_dec_compressed: [i32; SIZE] = [ 0, 253, 32 ];

/* G722 C code */

/* QMF filter coefficients:
  scaled by a factor of 4 compared to G722 CCITT recommendation */

const adpcm_dec_h: [i32; 24] = [
      12,   -44,   -44,   212,    48,  -624,   128,  1448,
    -840, -3220,  3804, 15504, 15504,  3804, -3220,  -840,
    1448,   128,  -624,    48,   212,   -44,   -44,    12,
];

const adpcm_dec_qq4_code4_table: [i32; 16] = [
        0,  -20456,  -12896,   -8968,   -6288,   -4240,   -2584,   -1200,
    20456,   12896,    8968,    6288,    4240,    2584,    1200,       0,
];

const adpcm_dec_qq6_code6_table: [i32; 64] = [
      -136,    -136,    -136,    -136,  -24808,  -21904,  -19008,  -16704,
    -14984,  -13512,  -12280,  -11192,  -10232,   -9360,   -8576,   -7856,
     -7192,   -6576,   -6000,   -5456,   -4944,   -4464,   -4008,   -3576,
     -3168,   -2776,   -2400,   -2032,   -1688,   -1360,   -1040,    -728,
     24808,   21904,   19008,   16704,   14984,   13512,   12280,   11192,
     10232,    9360,    8576,    7856,    7192,    6576,    6000,    5456,
      4944,    4464,    4008,    3576,    3168,    2776,    2400,    2032,
      1688,    1360,    1040,     728,     432,     136,    -432,    -136,
];

const adpcm_dec_wl_code_table: [i32; 16] = [
     -60,  3042,  1198,   538,   334,   172,    58,   -30,
    3042,  1198,   538,   334,   172,    58,   -30,   -60,
];

const adpcm_dec_ilb_table: [i32; 32] = [
    2048,  2093,  2139,  2186,  2233,  2282,  2332,  2383,
    2435,  2489,  2543,  2599,  2656,  2714,  2774,  2834,
    2896,  2960,  3025,  3091,  3158,  3228,  3298,  3371,
    3444,  3520,  3597,  3676,  3756,  3838,  3922,  4008
];

const adpcm_dec_qq2_code2_table: [i32; 4] = [ -7408, -1616, 7408, 1616 ];

const adpcm_dec_wh_code_table: [i32; 4] = [ 798, -214, 798, -214 ];

/*
  Arithmetic math functions
*/

/* MAX: 1 */
/* MAX: 1 */
#[inline]
fn adpcm_dec_abs(n: i32) -> i32 {
    n.abs()
}

fn adpcm_dec_sin(rad: i32) -> i32 {
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
    while adpcm_dec_abs(diff) >= 1 {
        diff = (diff * (-( rad * rad))) / ((2 * inc) * (2 * inc + 1));
        app = app + diff;
        inc += 1;
    }
    app
}

fn adpcm_dec_cos(rad: i32) -> i32 {
    adpcm_dec_sin(PI / 2 - rad)
}

/*
  Algorithm core functions
*/

/* decode function, result in xout1 and xout2 */
fn adpcm_dec_decode(input: i32, data: &mut AdpcmDecData) {
    /* split transmitted word from input into ilr and ih */
    data.ilr = input & 0x3f;
    data.ih = input >> 6;

    /* LOWER SUB_BAND DECODER */

    /* filtez: compute predictor output for zero section */
    data.dec_szl = adpcm_dec_filtez(&data.dec_del_bpl,
                                     &data.dec_del_dltx,
                                  );

    /* filtep: compute predictor output signal for pole section */
    data.dec_spl = adpcm_dec_filtep(data.dec_rlt1,
                                    data.dec_al1,
                                    data.dec_rlt2,
                                    data.dec_al2
                                  );

    data.dec_sl = data.dec_spl + data.dec_szl;

    /* invqxl: compute quantized difference signal for adaptive predic */
    data.dec_dlt = ((data.dec_detl as i64
        * adpcm_dec_qq4_code4_table[(data.ilr >> 2) as usize] as i64) >> 15i64) as i32;

    /* invqxl: compute quantized difference signal for decoder output */
    data.dl = ((data.dec_detl as i64
        * adpcm_dec_qq6_code6_table[data.il as usize] as i64) >> 15i64) as i32;

    data.rl = data.dl + data.dec_sl;

    /* logscl: quantizer scale factor adaptation in the lower sub-band */
    data.dec_nbl = adpcm_dec_logscl(data.ilr, data.dec_nbl);

    /* scalel: computes quantizer scale factor in the lower sub band */
    data.dec_detl = adpcm_dec_scalel(data.dec_nbl, 8);

    /* parrec - add pole predictor output to quantized diff. signal */
    /* for partially reconstructed signal */
    data.dec_plt = data.dec_dlt + data.dec_szl;

    /* upzero: update zero section predictor coefficients */
    adpcm_dec_upzero(data.dec_dlt,
                     &mut data.dec_del_dltx,
                     &mut data.dec_del_bpl
                    );

    /* uppol2: update second predictor coefficient apl2 and delay it as al2 */
    data.dec_al2 = adpcm_dec_uppol2(data.dec_al1, data.dec_al2,
                                    data.dec_plt, data.dec_plt1,
                                    data.dec_plt2
                                  );

    /* uppol1: update first predictor coef. (pole setion) */
    data.dec_al1 = adpcm_dec_uppol1(data.dec_al1,
                                    data.dec_al2,
                                    data.dec_plt,
                                    data.dec_plt1
                                  );

    /* recons : compute recontructed signal for adaptive predictor */
    data.dec_rlt = data.dec_sl + data.dec_dlt;

    /* done with lower sub band decoder, implement delays for next time */
    data.dec_rlt2 = data.dec_rlt1;
    data.dec_rlt1 = data.dec_rlt;
    data.dec_plt2 = data.dec_plt1;
    data.dec_plt1 = data.dec_plt;

    /* HIGH SUB-BAND DECODER */

    /* filtez: compute predictor output for zero section */
    data.dec_szh = adpcm_dec_filtez(&mut data.dec_del_bph,
                                    &mut data.dec_del_dhx
                                    );

    /* filtep: compute predictor output signal for pole section */
    data.dec_sph = adpcm_dec_filtep(data.dec_rh1,
                                    data.dec_ah1,
                                    data.dec_rh2,
                                    data.dec_ah2
                                  );

    /* predic:compute the predictor output value in the higher sub_band decoder */
    data.dec_sh = data.dec_sph + data.dec_szh;

    /* invqah: in-place compute the quantized difference signal */
    data.dec_dh = (( data.dec_deth as i64 *
        adpcm_dec_qq2_code2_table[ data.ih as usize ] as i64) >> 15i64) as i32;

    /* logsch: update logarithmic quantizer scale factor in hi sub band */
    data.dec_nbh = adpcm_dec_logsch(data.ih, data.dec_nbh);

    /* scalel: compute the quantizer scale factor in the higher sub band */
    data.dec_deth = adpcm_dec_scalel(data.dec_nbh, 10);

    /* parrec: compute partially recontructed signal */
    data.dec_ph = data.dec_dh + data.dec_szh;

    /* upzero: update zero section predictor coefficients */
    adpcm_dec_upzero(data.dec_dh, &mut data.dec_del_dhx, &mut data.dec_del_bph);

    /* uppol2: update second predictor coefficient aph2 and delay it as ah2 */
    data.dec_ah2 = adpcm_dec_uppol2(data.dec_ah1,
                                    data.dec_ah2,
                                    data.dec_ph,
                                    data.dec_ph1,
                                    data.dec_ph2
                                  );

    /* uppol1: update first predictor coef. (pole setion) */
    data.dec_ah1 = adpcm_dec_uppol1(data.dec_ah1,
                                    data.dec_ah2,
                                    data.dec_ph,
                                    data.dec_ph1
                                  );

    /* recons : compute recontructed signal for adaptive predictor */
    data.rh = data.dec_sh + data.dec_dh;

    /* done with high band decode, implementing delays for next time here */
    data.dec_rh2 = data.dec_rh1;
    data.dec_rh1 = data.rh;
    data.dec_ph2 = data.dec_ph1;
    data.dec_ph1 = data.dec_ph;

    /* end of higher sub_band decoder */

    /* end with receive quadrature mirror filters */
    data.xd = data.rl - data.rh;
    data.xs = data.rl + data.rh;

    /* receive quadrature mirror filters implemented here */
    let h_ary = adpcm_dec_h;
    let mut xa1: i64 = data.xd as i64 * h_ary[0] as i64;    /* qmf accumulators */
    let mut xa2: i64 = data.xs as i64 * h_ary[1] as i64;    /* qmf accumulators */
    /* main multiply accumulate loop for samples and coefficients */
    /* MAX: 11 */
    // loopbound min 11 max 11 -> changed; original did end separately
    let it = h_ary[2..].chunks_exact(2)
                                                          .zip(data.accumc)
                                                          .zip(data.accumd)
                                                          .map(|((a, b), c)| (a, b, c));
    for (h_chunk, ac, ad) in it {
        xa1 += ac as i64 * h_chunk[0] as i64;
        xa2 += ad as i64 * h_chunk[1] as i64;
    }

    /* scale by 2^14 */
    data.xout1 = (xa1 >> 14) as i32;
    data.xout2 = (xa2 >> 14) as i32;

    /* update delay lines */
    // loopbound min 10 max 10
    for i in (1..11).rev() {
        data.accumc[i] = data.accumc[i - 1];
        data.accumd[i] = data.accumd[i - 1];
    }
    data.accumc[0] = data.xd;
    data.accumd[0] = data.xs;
}

/* filtez - compute predictor output signal (zero section) */
/* input: bpl1-6 and dlt1-6, output: szl */
fn adpcm_dec_filtez(bpl: &[i32; 6], dlt: &[i32; 6]) -> i32 {
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
fn adpcm_dec_filtep(rlt1: i32, al1: i32, rlt2: i32, al2: i32) -> i32 {
    let mut pl: i64 = 2 * rlt1 as i64;
    pl = al1 as i64 * pl;
    let pl2: i64 = 2 * rlt2 as i64;
    pl += al2 as i64 * pl2;

    (pl >> 15) as i32
}

/* logscl - update log quantizer scale factor in lower sub-band */
/* note that nbl is passed and returned */
fn adpcm_dec_logscl(il: i32, nbl: i32) -> i32 {
    // long long int wd;

    let wd: i64 = ( nbl as i64 * 127i64 ) >> 7i64; /* leak factor 127/128 */
    let mut nbl: i32 = wd as i32 + adpcm_dec_wl_code_table[(il >> 2) as usize];

    if nbl < 0 {
        nbl = 0;
    }
    if nbl > 18432 {
        nbl = 18432;
    }
    nbl
}

/* scalel: compute quantizer scale factor in lower or upper sub-band*/
fn adpcm_dec_scalel(nbl: i32, shift_constant: i32) -> i32 {
    let wd1: i32 = (nbl >> 6) & 31;
    let wd2: i32 = nbl >> 11;
    let wd3: i32 = adpcm_dec_ilb_table[wd1 as usize] >> (shift_constant + 1 - wd2);

    wd3 << 3
}

/* upzero - inputs: dlt, dlti[ 0-5 ], bli[ 0-5 ], outputs: updated bli[ 0-5 ] */
/* also implements delay of bli and update of dlti from dlt */
fn adpcm_dec_upzero(dlt: i32, dlti: &mut [i32; 6], bli: &mut [i32; 6]) {
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
fn adpcm_dec_uppol2(al1: i32, al2: i32, plt: i32, plt1: i32, plt2: i32) -> i32 {
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
fn adpcm_dec_uppol1(al1: i32, apl2: i32, plt: i32, plt1: i32) -> i32 {
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
fn adpcm_dec_logsch(ih: i32, nbh: i32) -> i32 {
    let wd: i32 = ((nbh as i64 * 127i64) >> 7i64) as i32;   /* leak factor 127/128 */
    let mut nbh = wd + adpcm_dec_wh_code_table[ih as usize];

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

fn adpcm_dec_reset(data: &mut AdpcmDecData) {
    /* reset to min scale factor */
    data.detl = 32;
    data.dec_detl = 32;

    data.deth = 8;
    data.dec_deth = 8;

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
    data.dec_nbl = 0;
    data.dec_al1 = 0;
    data.dec_al2 = 0;
    data.dec_plt1 = 0;
    data.dec_plt2 = 0;
    data.dec_rlt1 = 0;
    data.dec_rlt2 = 0;
    data.dec_nbh = 0;
    data.dec_ah1 = 0;
    data.dec_ah2 = 0;
    data.dec_ph1 = 0;
    data.dec_ph2 = 0;
    data.dec_rh1 = 0;
    data.dec_rh2 = 0;

    // loopbound min 6 max 6
    data.delay_dhx.fill(0);
    data.dec_del_dltx.fill(0);
    data.dec_del_dhx.fill(0);

    // loopbound min 6 max 6
    data.delay_bph.fill(0);
    data.dec_del_bpl.fill(0);
    data.dec_del_bph.fill(0);

    // loopbound min 11 max 11
    data.accumc.fill(0);
    data.accumd.fill(0);
}

fn adpcm_dec_init(data: &mut AdpcmDecData) {
    /* reset, initialize required memory */
    adpcm_dec_reset(data);

    /* read in amplitude and frequency for test data */
    let j = 10;
    let f = 2000;

    /* 16 KHz sample rate */
    /* XXmain_0, MAX: 2 */
    /* Since the number of times we loop in my_sin depends on the argument we
        add the fact: xxmain_0:[  ]: */
    // loopbound min 3 max 3
    for (i, elem) in data.test_data.iter_mut().enumerate() {
        *elem = j * adpcm_dec_cos(f * PI * i as i32);
    }
}

fn adpcm_dec_return(data: &mut AdpcmDecData) -> bool {
    let mut checksum: i32 = 0;
    // loopbound min 2 max 2
    for i in (0..IN_END).step_by(2) {
        checksum += data.result[i] + data.result[i + 1];
    }
    checksum == -2
}

/*
  Main functions
*/

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn adpcm_dec_main(data: &mut AdpcmDecData) {
    // loopbound min 2 max 2
    for i in (0..IN_END).step_by(2) {
        adpcm_dec_decode(adpcm_dec_compressed[i / 2], data);
        data.result[i] = data.xout1;
        data.result[i + 1] = data.xout2;
    }
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn adpcm_dec_log(data: &mut AdpcmDecData) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        adpcm_dec_init(black_box(data));
        let start = black_box(DWT::cycle_count());
        adpcm_dec_main(black_box(data));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("adpcm_dec - elapsed: {}", cycles);
    });

    black_box(adpcm_dec_return(black_box(data)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn adpcm_dec_std(data: &mut AdpcmDecData) -> bool {
    black_box(adpcm_dec_init(data));
    adpcm_dec_main(black_box(data));
    black_box(adpcm_dec_return(black_box(data)))
}

fn adpcm_dec() -> bool {
    static mut DATA: AdpcmDecData = AdpcmDecData::new();
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let data = unsafe { &mut *{ &raw mut DATA } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(adpcm_dec_log(black_box(data)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(adpcm_dec_std(black_box(data)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(adpcm_dec());
    
    #[cfg(feature = "log")]
    {
        rprintln!("adpcm_dec: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(adpcm_dec());

    #[cfg(feature = "log")]
    std::println!("adpcm_dec: {}", success);

    std::process::exit(!success as i32);
}
