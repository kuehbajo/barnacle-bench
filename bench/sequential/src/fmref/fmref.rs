/*

  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license (MIT) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite.

  fmref.c: C reference implementation of FM Radio
  David Maze <dmaze@cag.lcs.mit.edu>
  $Id: fmref.c,v 1.2 2010-10-04 21:21:26 garus Exp $
*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;
use core::mem::size_of;

mod math;

use math::{ sinf, cosf, atanf };

const M_PI: f64 = 3.1415926535897932384626433832795;

// Defines
const SAMPLING_RATE: i32 = 250000000;
const CUTOFF_FREQUENCY: f32 = 108000000.;
const NUM_TAPS: usize = 64;
const MAX_AMPLITUDE: f32 = 27000.0;
const BANDWIDTH: i32 = 10000;
const DECIMATION: usize = 4;
/* Must be at least NUM_TAPS+1: */
const IN_BUFFER_LEN: usize = 200;
const EQUALIZER_BANDS: usize = 10;

const EQ_CUTOFFS: [f32; EQUALIZER_BANDS + 1] = [
    55.000004, 77.78174, 110.00001, 155.56354, 220.00002, 311.12695,
    440.00003, 622.25415, 880.00006, 1244.5078, 1760.0001
];

// Type declarations
#[derive(Clone, Copy, Debug)]
struct FloatBuffer {
    buff: [f32; IN_BUFFER_LEN],
    rpos: usize,
    rlen: usize,
}

impl Default for FloatBuffer {
    fn default() -> Self {
        FloatBuffer {
            buff: [0.; IN_BUFFER_LEN],
            rpos: 0,
            rlen: 0,
        }
    }
}

impl FloatBuffer {
    const fn new() -> Self {
        FloatBuffer {
            buff: [0.; IN_BUFFER_LEN],
            rpos: 0,
            rlen: 0,
        }
    }
}

/* Low pass filter: */
#[derive(Clone, Copy)]
struct LPFData {
    coeff: [f32; NUM_TAPS],
    freq: f32,
    taps: usize,
    decimation: usize,
}

impl Default for LPFData {
    fn default() -> Self {
        LPFData {
            coeff: [0.; NUM_TAPS],
            freq: 0.,
            taps: 0,
            decimation: 0,
        }
    }
}

impl LPFData {
    const fn new() -> Self {
        LPFData {
            coeff: [0.; NUM_TAPS],
            freq: 0.,
            taps: 0,
            decimation: 0,
        }
    }
}

#[derive(Default)]
struct EqualizerData {
    lpf: [LPFData; EQUALIZER_BANDS + 1],
    fb: [FloatBuffer; EQUALIZER_BANDS + 1],
    gain: [f32; EQUALIZER_BANDS],
}

impl EqualizerData {
    const fn new() -> Self {
        EqualizerData {
            lpf: [LPFData::new(); 11],
            fb: [FloatBuffer::new(); 11],
            gain: [0.; EQUALIZER_BANDS],
        }
    }
}

// changed: resets EqualizerData now
fn fmref_init(
    eq_data: &mut EqualizerData,
    numiters: &mut usize,
    x: &mut i32,
) {
    *numiters = 2;
    *x = 0;
    for lpf in &mut eq_data.lpf {
        *lpf = LPFData::new();
    }

    for fb in &mut eq_data.fb {
        *fb = FloatBuffer::new();
    }
    eq_data.gain = [0.0; EQUALIZER_BANDS];
}

fn fmref_return(_fb4: &mut FloatBuffer) -> bool {
    // new checksum to test the result
    // let mut checksum = 0.0f32;
    // for i in 0..fb4.rlen {
    //     checksum += fb4.buff[i];
    // }

    // dummy return value
    true
}

// entrypoint
// changed: moved EqualizerData to static memory
#[inline(never)]
#[unsafe(no_mangle)]
fn fmref_main(
    eq_data: &mut EqualizerData,
    fb1: &mut FloatBuffer,
    fb2: &mut FloatBuffer,
    fb3: &mut FloatBuffer,
    fb4: &mut FloatBuffer,
    lpf_data: &mut LPFData,
    numiters: &mut usize,
    x: &mut i32,
) {
    fb1.rpos = 0;
    fb1.rlen = 0;
    fb2.rpos = 0;
    fb2.rlen = 0;
    fb3.rpos = 0;
    fb3.rlen = 0;
    fb4.rpos = 0;
    fb4.rlen = 0;

    fmref_init_lpf_data(lpf_data, CUTOFF_FREQUENCY, NUM_TAPS, DECIMATION);
    fmref_init_equalizer(eq_data);

    /* Startup: */
    fmref_get_floats(fb1, x);
    /* LPF needs at least NUM_TAPS+1 inputs; fmref_get_floats is fine. */
    fmref_run_lpf(fb1, fb2, lpf_data);
    /* run_demod needs 1 input, OK here. */
    /* run_equalizer needs 51 inputs (same reason as for LPF).  This means
        running the pipeline up to demod 50 times in advance: */
    // loopbound min 64 max 64
    for _ in 0..64 {
        if fb1.rlen - fb1.rpos < NUM_TAPS + 1 {
            fmref_get_floats(fb1, x);
        }
        fmref_run_lpf(fb1, fb2, lpf_data);
        fmref_run_demod(fb2, fb3);
    }

    /* Main loop: */
    // loopbound min 2 max 2
    while *numiters > 0 {
        *numiters -= 1;
        /* The low-pass filter will need NUM_TAPS+1 items; read them if we need to. */
        if fb1.rlen - fb1.rpos < NUM_TAPS + 1 {
            fmref_get_floats(fb1, x);
        }
        fmref_run_lpf(fb1, fb2, lpf_data);
        fmref_run_demod(fb2, fb3);
        fmref_run_equalizer(fb3, fb4, eq_data);
    }
}

fn fmref_fb_compact(fb: &mut FloatBuffer) {
    // convert to bytes for bytewise memmove
    let mut bytes: [u8; size_of::<f32>() * IN_BUFFER_LEN] = [0; size_of::<f32>() * IN_BUFFER_LEN];
    for (i, f) in fb.buff.iter().enumerate() {
        bytes[i * 4..(i + 1) * 4].copy_from_slice(&f.to_ne_bytes());
    }

    let start = fb.rpos * size_of::<f32>();
    let num = fb.rlen - fb.rpos;
    // loopbound min 0 max 60
    bytes.copy_within(start..(start + num), 0);
    // writing it back to fb.buff
    for (dst, chunk) in fb.buff.iter_mut().zip(bytes.chunks_exact(4)) {
        *dst = f32::from_ne_bytes(chunk.try_into().unwrap());
    }
    
    fb.rlen -= fb.rpos;
    fb.rpos = 0;
}

fn fmref_fb_ensure_writable(
    fb: &mut FloatBuffer,
    amount: usize
) -> bool {
    let available = IN_BUFFER_LEN - fb.rlen;
    if available >= amount {
        return true;
    }

    /* Nope, not enough room, move current contents back to the beginning. */
    fmref_fb_compact(fb);

    let available = IN_BUFFER_LEN - fb.rlen;
    if available >= amount {
        return true;
    }
    false
}

fn fmref_get_floats(fb: &mut FloatBuffer, x: &mut i32) {
    fmref_fb_compact(fb);

    /* Fill the remaining space in fb with 1.0. */
    // loopbound min 140 max 200
    while fb.rlen < IN_BUFFER_LEN {
        fb.buff[fb.rlen] = *x as f32;
        fb.rlen += 1;
        *x += 1;
    }
}

fn fmref_init_lpf_data(
    data: &mut LPFData,
    freq: f32,
    taps: usize,
    decimation: usize,
) {
    /* Assume that CUTOFF_FREQUENCY is non-zero.  See comments in
      StreamIt LowPassFilter.java for origin. */
    let w: f32 = (2. * M_PI * freq as f64 / SAMPLING_RATE as f64) as f32;
    let m: f32 = taps as f32 - 1.0;

    data.freq = freq;
    data.taps = taps;
    data.decimation = decimation;

    // loopbound min 64 max 64
    for (i, el) in data.coeff.iter_mut().enumerate() {
        if i as f32 - m / 2. == 0.0 {
            *el = (w as f64 / M_PI) as f32;
        } else {
            *el = (sinf(w * (i as f32 - m / 2.)) as f64
                / M_PI
                / (i as f32 - m / 2.) as f64
                * (0.54 - 0.46 * cosf((2. * M_PI * i as f64 / m as f64) as f32)) as f64
            ) as f32;
        }
    }
}

fn fmref_run_lpf(
    fbin: &mut FloatBuffer,
    fbout: &mut FloatBuffer,
    data: &mut LPFData,
) {
    let mut sum: f32 = 0.0;

    // loopbound min 64 max 64
    for (i, el) in data.coeff.iter().enumerate() {
        sum += fbin.buff[fbin.rpos + i] * *el;
    }

    fbin.rpos += data.decimation + 1;

    /* Check that there's room in the output buffer; move data if necessary. */
    fmref_fb_ensure_writable(fbout, 1);
    fbout.buff[fbout.rlen] = sum;
    fbout.rlen += 1;
}

fn fmref_run_demod(
    fbin: &mut FloatBuffer,
    fbout: &mut FloatBuffer,
) {
    let gain: f32 = (MAX_AMPLITUDE as f64 * SAMPLING_RATE as f64 / (BANDWIDTH as f64 * M_PI)) as f32;
    let mut temp: f32 = fbin.buff[fbin.rpos] * fbin.buff[fbin.rpos + 1];
    temp = gain * atanf(temp);
    fbin.rpos += 1;
    fmref_fb_ensure_writable(fbout, 1);
    fbout.buff[fbout.rlen] = temp;
    fbout.rlen += 1;
}

fn fmref_init_equalizer(data: &mut EqualizerData) {
    /* Equalizer structure: there are ten band-pass filters, with
      cutoffs as shown below.  The outputs of these filters get added
      together.  Each band-pass filter is LPF(high)-LPF(low). */
    // loopbound min 11 max 11
    for (i, el) in data.lpf.iter_mut().enumerate() {
        fmref_init_lpf_data(el, EQ_CUTOFFS[i], 64, 0);
    }

    /* Also initialize member buffers. */
    // loopbound min 11 max 11
    for el in data.fb.iter_mut() {
        el.rpos = 0;
        el.rlen = 0;
    } 

    // loopbound min 10 max 10
    for (i, el) in data.gain.iter_mut().enumerate() {
        // the gain amplifies the middle bands the most
        let val: f32 = (i as f32 - ((EQUALIZER_BANDS - 1) as f32 / 2.0)) / 5.0;
        *el = if val > 0. { 2.0 - val } else { 2.0 + val };
    }
}

fn fmref_run_equalizer(
    fbin: &mut FloatBuffer,
    fbout: &mut FloatBuffer,
    data: &mut EqualizerData,
) {
    let mut lpf_out: [f32; EQUALIZER_BANDS + 1] = [0.; EQUALIZER_BANDS + 1];
    let mut sum: f32 = 0.0;

    /* Save the input read location; we can reuse the same input data on all
      of the LPFs. */
    let rpos = fbin.rpos;

    /* Run the child filters. */
    // loopbound min 11 max 11
    for i in 0..(EQUALIZER_BANDS + 1) {
        fbin.rpos = rpos;
        fmref_run_lpf(fbin, &mut data.fb[i], &mut data.lpf[i]);
        lpf_out[i] = data.fb[i].buff[data.fb[i].rpos];
        data.fb[i].rpos += 1;
    }

    /* Now process the results of the filters.  Remember that each band is
      output(hi)-output(lo). */
    // loopbound min 10 max 10
    for (i, &gain) in data.gain.iter().enumerate() {
        sum += (lpf_out[i + 1] - lpf_out[i]) * gain;
    }      

    /* Write that result.  */
    fmref_fb_ensure_writable(fbout, 1);
    fbout.buff[fbout.rlen] = sum;
    fbout.rlen += 1;
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn fmref_log(
    eq_data: &mut EqualizerData,
    fb1: &mut FloatBuffer,
    fb2: &mut FloatBuffer,
    fb3: &mut FloatBuffer,
    fb4: &mut FloatBuffer,
    lpf_data: &mut LPFData,
    numiters: &mut usize,
    x: &mut i32,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        fmref_init(
            black_box(eq_data), black_box(numiters), black_box(x),
        );
        let start = black_box(DWT::cycle_count());
        fmref_main(
            black_box(eq_data),
            black_box(fb1), black_box(fb2),
            black_box(fb3), black_box(fb4),
            black_box(lpf_data), black_box(numiters),
            black_box(x),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("fmref - elapsed: {}", cycles);
    });

    black_box(fmref_return(fb4))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn fmref_std(
    eq_data: &mut EqualizerData,
    fb1: &mut FloatBuffer,
    fb2: &mut FloatBuffer,
    fb3: &mut FloatBuffer,
    fb4: &mut FloatBuffer,
    lpf_data: &mut LPFData,
    numiters: &mut usize,
    x: &mut i32,
) -> bool {
    fmref_init(
        black_box(eq_data), black_box(numiters), black_box(x),
    );
    fmref_main(
        black_box(eq_data),
        black_box(fb1), black_box(fb2),
        black_box(fb3), black_box(fb4),
        black_box(lpf_data), black_box(numiters),
        black_box(x),
    );
    black_box(fmref_return(fb4))
}

fn fmref() -> bool {
    static mut EQ_DATA: EqualizerData = EqualizerData::new();
    static mut FB1: FloatBuffer = FloatBuffer::new();
    static mut FB2: FloatBuffer = FloatBuffer::new();
    static mut FB3: FloatBuffer = FloatBuffer::new();
    static mut FB4: FloatBuffer = FloatBuffer::new();
    static mut LPF_DATA: LPFData = LPFData::new();
    static mut NUMITERS: usize = 2;
    static mut X: i32 = 0;

    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let eq_data = unsafe { &mut *{ &raw mut EQ_DATA } };
    let fb1 = unsafe { &mut *{ &raw mut FB1 } };
    let fb2 = unsafe { &mut *{ &raw mut FB2 } };
    let fb3 = unsafe { &mut *{ &raw mut FB3 } };
    let fb4 = unsafe { &mut *{ &raw mut FB4 } };
    let lpf_data = unsafe { &mut *{ &raw mut LPF_DATA } };
    let numiters = unsafe { &mut *{ &raw mut NUMITERS } };
    let x = unsafe { &mut *{ &raw mut X } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(fmref_log(
            black_box(eq_data),
            black_box(fb1), black_box(fb2),
            black_box(fb3), black_box(fb4),
            black_box(lpf_data), black_box(numiters),
            black_box(x),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(fmref_std(
            black_box(eq_data),
            black_box(fb1), black_box(fb2),
            black_box(fb3), black_box(fb4),
            black_box(lpf_data), black_box(numiters),
            black_box(x),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(fmref());
    
    #[cfg(feature = "log")]
    {
        rprintln!("fmref: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(fmref());

    #[cfg(feature = "log")]
    std::println!("fmref: {}", success);

    std::process::exit(!success as i32);
}
