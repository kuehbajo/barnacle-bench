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

  Name: ndes

  Author: unknown

  Function: A lot of bit manipulation, shifts, array and matrix calculations.

  Source:

  Changes: no major functional changes

  License: May be used, modified, and re-distributed freely.

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

/* A read from this address will result in an known value of 1 */
const KNOWN_VALUE: u64 = 1;
const NDES_WORSTCASE: bool = true;

#[derive(Clone, Copy)]
struct Immense {
    l: u64,
    r: u64,
}

impl Immense {
    const fn new() -> Self {
        Immense {
            l: 0,
            r: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Great {
    l: u64,
    c: u64,
    r: u64,
}

impl Great {
    const fn new() -> Self {
        Great {
            l: 0,
            c: 0,
            r: 0,
        }
    }
}

struct Parameters {
    bit: [u64; 33],
    inp: Immense,
    key: Immense,
    out: Immense,
    newkey: i32,
    isw: i32,
    value: i32,   
}

impl  Parameters {
    const fn new() -> Self {
        Parameters {
            bit: [0; 33],
            inp: Immense::new(),
            key: Immense::new(),
            out: Immense::new(),
            newkey: 0,
            isw: 0,
            value: if NDES_WORSTCASE { 1 } else { 0 },
        }
    }
}

/*
   Initialization
*/

fn ndes_init(
    ipc1: &mut [i8; 57],
    ipc2: &mut [i8; 49],
    parameter: &mut Parameters,
    
) {
    static IPC1_TMP: [i8; 57] = [
        0, 57, 49, 41, 33, 25, 17, 9, 1, 58, 50,
        42, 34, 26, 18, 10, 2, 59, 51, 43, 35, 27, 19, 11, 3, 60,
        52, 44, 36, 63, 55, 47, 39, 31, 23, 15, 7, 62, 54, 46, 38,
        30, 22, 14, 6, 61, 53, 45, 37, 29, 21, 13, 5, 28, 20, 12, 4
    ];
    static IPC2_TMP: [i8; 49 ] = [
        0, 14, 17, 11, 24, 1, 5, 3, 28, 15, 6, 21,
        10, 23, 19, 12, 4, 26, 8, 16, 7, 27, 20, 13, 2, 41, 52, 31,
        37, 47, 55, 30, 40, 51, 45, 33, 48, 44, 49, 39, 56, 34,
        53, 46, 42, 50, 36, 29, 32
    ];
    // loopbound min 57 max 57
    for (e1, e2) in ipc1.iter_mut().zip(IPC1_TMP.iter()) {
        *e1 = *e2;
    }
        
    // loopbound min 49 max 49
    for (e1, e2) in ipc2.iter_mut().zip(IPC2_TMP.iter()) {
        *e1 = *e2;
    }

    parameter.inp.l = KNOWN_VALUE * 35;
    parameter.inp.r = KNOWN_VALUE * 26;
    parameter.key.l = KNOWN_VALUE * 2;
    parameter.key.r = KNOWN_VALUE * 16;

    parameter.newkey = parameter.value;
    parameter.isw = parameter.value;
}
/*
   Arithmetic functions
*/

// moved kns and initflag declaration
fn ndes_des(
    icd: &mut Immense,
    ipc1: &mut [i8; 57],
    ipc2: &mut [i8; 49],
    initflag: &mut bool,
    kns: &mut [Great; 17],
    parameter: &mut Parameters,
) {
    static IP: [i8; 65] = [
        0, 58, 50, 42, 34, 26, 18, 10, 2, 60, 52, 44, 36,
        28, 20, 12, 4, 62, 54, 46, 38, 30, 22, 14, 6, 64, 56, 48, 40,
        32, 24, 16, 8, 57, 49, 41, 33, 25, 17, 9, 1, 59, 51, 43, 35,
        27, 19, 11, 3, 61, 53, 45, 37, 29, 21, 13, 5, 63, 55, 47, 39,
        31, 23, 15, 7
    ];

    static IPM: [i8;  65] = [
        0, 40, 8, 48, 16, 56, 24, 64, 32, 39, 7, 47, 15,
        55, 23, 63, 31, 38, 6, 46, 14, 54, 22, 62, 30, 37, 5, 45, 13,
        53, 21, 61, 29, 36, 4, 44, 12, 52, 20, 60, 28, 35, 3, 43, 11,
        51, 19, 59, 27, 34, 2, 42, 10, 50, 18, 58, 26, 33, 1, 41, 9,
        49, 17, 57, 25
    ];

    *initflag = if NDES_WORSTCASE {
        true
    } else {
        false
    };

    if *initflag {
        *initflag = false;
        parameter.bit[1] = 1u64;
        let mut shifter = 1u64;

        // loopbound min 31 max 31
        for el in parameter.bit[2..].iter_mut() {
            *el = shifter << 1;
            shifter <<= 1;
        }
    }

    if parameter.newkey > 0 {
        parameter.newkey = 0;
        icd.r = 0u64;
        icd.l = 0u64;

        let mut k = 56;
        // loopbound min 28 max 28
        for j in (1..=28).rev() {
            icd.r = (icd.r << 1) | ndes_getbit(&parameter.key, ipc1[j] as usize, 32, &parameter.bit);
            icd.l = icd.l << 1;
            icd.l = ( icd.l ) | ndes_getbit(&parameter.key, ipc1[k] as usize, 32, &parameter.bit);
            k -= 1;
        }

        // loopbound min 16 max 16
        for (i, el) in kns[1..].iter_mut().enumerate() {
            let mut pg = *el;
            ndes_ks(i as i32, &mut pg, icd, ipc2, parameter);
            *el = pg;
        }
    }

    let mut itmp: Immense = Immense::new();
    itmp.r = 0u64;
    itmp.l = 0u64;

    // loopbound min 32 max 32
    let mut k = 64;
    for j in (1..=32).rev() {
        itmp.r = itmp.r << 1;
        itmp.r = ( itmp.r ) | ndes_getbit(&parameter.inp, IP[j] as usize, 32, &parameter.bit);
        itmp.l = itmp.l << 1;
        itmp.l = ( itmp.l ) | ndes_getbit(&parameter.inp, IP[k] as usize, 32, &parameter.bit);
        k -= 1;
    }

    let mut ic: u64 = 0;
    // loopbound min 16 max 16
    for i in 1..=16 {
        let ii =  if parameter.isw == 1 { 17 - i } else { i };
        ndes_cyfun(itmp.l, &kns[ii], &mut ic, parameter);
        ic ^= itmp.r;
        itmp.r = itmp.l;
        itmp.l = ic;
    }

    ic = itmp.r;
    itmp.r = itmp.l;
    itmp.l = ic;
    parameter.out.r = 0u64;
    parameter.out.l = 0u64;

    // loopbound min 32 max 32
    let mut k = 64;
    for j in 1..=32 {
        parameter.out.r = parameter.out.r << 1;
        parameter.out.r = parameter.out.r | ndes_getbit(&itmp, IPM[j] as usize, 32, &parameter.bit);
        parameter.out.l = parameter.out.l << 1;
        parameter.out.l = parameter.out.l | ndes_getbit(&itmp, IPM[k] as usize, 32, &parameter.bit);
        k -= 1;
    }
}

fn ndes_cyfun(
    ir: u64,
    k: &Great,
    iout: &mut u64,
    parameter: &mut Parameters
) {
    static IET: [i64; 49] = [
        0, 32, 1, 2, 3, 4, 5, 4, 5, 6, 7, 8, 9, 8, 9,
        10, 11, 12, 13, 12, 13, 14, 15, 16, 17, 16, 17, 18, 19,
        20, 21, 20, 21, 22, 23, 24, 25, 24, 25, 26, 27, 28, 29,
        28, 29, 30, 31, 32, 1
    ];
    static IPP: [i64; 33] = [
        0, 16, 7, 20, 21, 29, 12, 28, 17, 1, 15,
        23, 26, 5, 18, 31, 10, 2, 8, 24, 14, 32, 27, 3, 9, 19, 13,
        30, 6, 22, 11, 4, 25
    ];
    static IS: [[[i64; 9]; 4]; 16] = [
        [
            [0, 14, 15, 10, 7, 2, 12, 4, 13],
            [0, 0, 3, 13, 13, 14, 10, 13, 1],
            [0, 4, 0, 13, 10, 4, 9, 1, 7],
            [0, 15, 13, 1, 3, 11, 4, 6, 2],
        ], [
            [0, 4, 1, 0, 13, 12, 1, 11, 2],
            [0, 15, 13, 7, 8, 11, 15, 0, 15],
            [0, 1, 14, 6, 6, 2, 14, 4, 11],
            [0, 12, 8, 10, 15, 8, 3, 11, 1],
        ], [
            [0, 13, 8, 9, 14, 4, 10, 2, 8],
            [0, 7, 4, 0, 11, 2, 4, 11, 13],
            [0, 14, 7, 4, 9, 1, 15, 11, 4],
            [0, 8, 10, 13, 0, 12, 2, 13, 14],
        ], [
            [0, 1, 14, 14, 3, 1, 15, 14, 4],
            [0, 4, 7, 9, 5, 12, 2, 7, 8],
            [0, 8, 11, 9, 0, 11, 5, 13, 1],
            [0, 2, 1, 0, 6, 7, 12, 8, 7],
        ], [
            [0, 2, 6, 6, 0, 7, 9, 15, 6],
            [0, 14, 15, 3, 6, 4, 7, 4, 10],
            [0, 13, 10, 8, 12, 10, 2, 12, 9],
            [0, 4, 3, 6, 10, 1, 9, 1, 4],
        ], [
            [0, 15, 11, 3, 6, 10, 2, 0, 15],
            [0, 2, 2, 4, 15, 7, 12, 9, 3],
            [0, 6, 4, 15, 11, 13, 8, 3, 12],
            [0, 9, 15, 9, 1, 14, 5, 4, 10],
        ], [
            [0, 11, 3, 15, 9, 11, 6, 8, 11],
            [0, 13, 8, 6, 0, 13, 9, 1, 7],
            [0, 2, 13, 3, 7, 7, 12, 7, 14],
            [0, 1, 4, 8, 13, 2, 15, 10, 8],
        ], [
            [0, 8, 4, 5, 10, 6, 8, 13, 1],
            [0, 1, 14, 10, 3, 1, 5, 10, 4],
            [0, 11, 1, 0, 13, 8, 3, 14, 2],
            [0, 7, 2, 7, 8, 13, 10, 7, 13],
        ], [
            [0, 3, 9, 1, 1, 8, 0, 3, 10],
            [0, 10, 12, 2, 4, 5, 6, 14, 12],
            [0, 15, 5, 11, 15, 15, 7, 10, 0],
            [0, 5, 11, 4, 9, 6, 11, 9, 15],
        ], [
            [0, 10, 7, 13, 2, 5, 13, 12, 9],
            [0, 6, 0, 8, 7, 0, 1, 3, 5],
            [0, 12, 8, 1, 1, 9, 0, 15, 6],
            [0, 11, 6, 15, 4, 15, 14, 5, 12],
        ], [
            [0, 6, 2, 12, 8, 3, 3, 9, 3],
            [0, 12, 1, 5, 2, 15, 13, 5, 6],
            [0, 9, 12, 2, 3, 12, 4, 6, 10],
            [0, 3, 7, 14, 5, 0, 1, 0, 9],
        ], [
            [0, 12, 13, 7, 5, 15, 4, 7, 14],
            [0, 11, 10, 14, 12, 10, 14, 12, 11],
            [0, 7, 6, 12, 14, 5, 10, 8, 13],
            [0, 14, 12, 3, 11, 9, 7, 15, 0],
        ], [
            [0, 5, 12, 11, 11, 13, 14, 5, 5],
            [0, 9, 6, 12, 1, 3, 0, 2, 0],
            [0, 3, 9, 5, 5, 6, 1, 0, 15],
            [0, 10, 0, 11, 12, 10, 6, 14, 3],
        ], [
            [0, 9, 0, 4, 12, 0, 7, 10, 0],
            [0, 5, 9, 11, 10, 9, 11, 15, 14],
            [0, 10, 3, 10, 2, 3, 13, 5, 3],
            [0, 0, 5, 5, 7, 4, 0, 2, 5],
        ], [
            [0, 0, 5, 2, 4, 14, 5, 6, 12],
            [0, 3, 11, 15, 14, 8, 3, 8, 9],
            [0, 5, 2, 14, 8, 0, 11, 9, 5],
            [0, 6, 14, 2, 2, 5, 8, 3, 6],
        ], [
            [0, 7, 10, 8, 15, 9, 11, 1, 7],
            [0, 8, 5, 1, 9, 6, 8, 6, 2],
            [0, 0, 15, 7, 4, 14, 6, 2, 8],
            [0, 13, 9, 12, 14, 3, 13, 12, 11],
        ],
    ];

    let ibin: [u8; 16] = [ 0, 8, 4, 12, 2, 10, 6, 14, 1, 9, 5, 13, 3, 11, 7, 15 ];
    let mut ie: Great = Great::new();
    let mut iec: [u8; 9] = [0; 9];
    let p = &parameter.bit;

    ie.r = 0;
    ie.c = 0;
    ie.l = 0;

    // loopbound min 16 max 16
    let mut l = 32;
    let mut m = 48;
    for j in (1..=16).rev() {
        let or_val = if (p[IET[j] as usize] & ir) > 0 { 1 } else { 0 };
        ie.r = (ie.r << 1) | or_val;
        let or_val = if (p[IET[l] as usize] & ir) > 0 { 1 } else { 0 };
        ie.c = (ie.c << 1) | or_val;
        let or_val = if (p[IET[m] as usize] & ir) > 0 { 1 } else { 0 };
        ie.l = (ie.l << 1) | or_val;
        l -= 1;
        m -= 1;
    }
    ie.r ^= k.r;
    ie.c ^= k.c;
    ie.l ^= k.l;
    let mut ietmp1 = (  (ie.c as u64) << 16 ) + ie.r as u64;
    let mut ietmp2 = ( (ie.l as u64) << 8 ) + ( ie.c as u64 >> 8 );

    // loopbound min 4 max 4
    let mut m: usize = 5;
    for j in 1..=4 {
        iec[j] = (ietmp1 & 0x3fu64) as u8;
        iec[m] = (ietmp2 & 0x3fu64) as u8;
        ietmp1 >>= 6;
        ietmp2 >>= 6;
        m += 1;
    }

    let mut itmp = 0u64;

    // loopbound min 8 max 8
    for jj in (1..=8).rev() {
        let j = iec[jj];
        let irow = ((j & 0x1) << 1) + ((j & 0x20) >> 5);
        let icol = ((j & 0x2) << 2) + (j & 0x4) + ((j & 0x8) >> 2) + ((j & 0x10) >> 4);
        let iss = IS[icol as usize][irow as usize][jj];
        itmp = (itmp << 4) | ibin[iss as usize] as u64;
    }

    *iout = 0u64;

    // loopbound min 32 max 32
    let mut j = 32;
    while j >= 1 {
        *iout = *iout << 1;
        j -= 1;
    }
    let and_val = if itmp > 0 { 1 } else { 0 };
    *iout |= p[IPP[j] as usize] & and_val;
}

fn ndes_getbit(
    source: &Immense,
    bitno: usize,
    nbits: usize,
    bit: &[u64; 33],
) -> u64 {
    if bitno <= nbits {
        if (bit[bitno] & source.r) > 0 { 1u64 } else { 0u64 }
    } else {
        if (bit[bitno - nbits] & source.l) > 0 { 1u64 } else { 0u64 }
    }
}

fn ndes_ks(
    n: i32,
    kn: &mut Great,
    icd: &mut Immense,
    ipc2: &mut [i8; 49],
    parameter: &mut Parameters
) {
    if n == 1 || n == 2 || n == 9 || n == 16 {
        icd.r = (icd.r | ((icd.r & 1u64) << 28)) >> 1;
        icd.l = (icd.l | ((icd.l & 1u64) << 28)) >> 1;
    } else {
        // loopbound min 2 max 2
        for _ in 1..=2 {
            icd.r = (icd.r | ((icd.r & 1u64) << 28)) >> 1;
            icd.l = (icd.l | ((icd.l & 1u64) << 28)) >> 1;
        }
    }

    kn.r = 0; 
    kn.c = 0;
    kn.l = 0;

    // loopbound min 16 max 16
    let mut k = 32;
    let mut l = 48;
    for j in 1..=16 {
        kn.r = kn.r << 1;
        kn.r = kn.r | ndes_getbit(icd, ipc2[j] as usize, 28, &parameter.bit);
        kn.c = kn.c << 1;
        kn.c = kn.c | ndes_getbit(icd, ipc2[k] as usize, 28, &parameter.bit);
        kn.l = kn.l << 1;
        kn.l = kn.l | ndes_getbit(icd, ipc2[l] as usize, 28, &parameter.bit);
        k -= 1;
        l -= 1;
    }
}

fn ndes_return(icd: &mut Immense) -> bool {
    ((icd.r + icd.l) as i64 - 8390656) != 0
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn ndes_main(
    icd: &mut Immense,
    ipc1: &mut [i8; 57],
    ipc2: &mut [i8; 49],
    initflag: &mut bool,
    kns: &mut [Great; 17],
    parameter: &mut Parameters,
) {
    ndes_des(
        icd, ipc1, ipc2, initflag, kns, parameter,
    );
}

/* main function */

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn ndes_log(
    icd: &mut Immense,
    ipc1: &mut [i8; 57],
    ipc2: &mut [i8; 49],
    initflag: &mut bool,
    kns: &mut [Great; 17],
    parameter: &mut Parameters,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        ndes_init(
            black_box(ipc1), black_box(ipc2), black_box(parameter),
        );
        let start = black_box(DWT::cycle_count());
        ndes_main(
            black_box(icd),
            black_box(ipc1),
            black_box(ipc2),
            black_box(initflag),
            black_box(kns),
            black_box(parameter),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("ndes - elapsed: {}", cycles);
    });

    black_box(ndes_return(black_box(icd)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn ndes_std(
    icd: &mut Immense,
    ipc1: &mut [i8; 57],
    ipc2: &mut [i8; 49],
    initflag: &mut bool,
    kns: &mut [Great; 17],
    parameter: &mut Parameters,
) -> bool {
    ndes_init(
        black_box(ipc1), black_box(ipc2), black_box(parameter),
    );
    ndes_main(
        black_box(icd),
        black_box(ipc1),
        black_box(ipc2),
        black_box(initflag),
        black_box(kns),
        black_box(parameter),
    );
    black_box(ndes_return(black_box(icd)))
}

fn ndes() -> bool {
    static mut ICD: Immense = Immense::new();
    static mut IPC1: [i8; 57] = [0; 57];
    static mut IPC2: [i8; 49] = [0; 49];
    static mut INITFLAG: bool = false;
    static mut KNS: [Great; 17] = [Great::new(); 17];
    static mut PARAMETER: Parameters = Parameters::new();

    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let icd = unsafe { &mut *{ &raw mut ICD } };
    let ipc1 = unsafe { &mut *{ &raw mut IPC1 } };
    let ipc2 = unsafe { &mut *{ &raw mut IPC2 } };
    let initflag = unsafe { &mut *{ &raw mut INITFLAG} };
    let kns = unsafe { &mut *{ &raw mut KNS } };
    let parameter = unsafe { &mut *{ &raw mut PARAMETER } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(ndes_log(
            black_box(icd),
            black_box(ipc1),
            black_box(ipc2),
            black_box(initflag),
            black_box(kns),
            black_box(parameter),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(ndes_std(
            black_box(icd),
            black_box(ipc1),
            black_box(ipc2),
            black_box(initflag),
            black_box(kns),
            black_box(parameter),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(ndes());
    
    #[cfg(feature = "log")]
    {
        rprintln!("ndes: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(ndes());

    #[cfg(feature = "log")]
    std::println!("ndes: {}", success);

    std::process::exit(!success as i32);
}
