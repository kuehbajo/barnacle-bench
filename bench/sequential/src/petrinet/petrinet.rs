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

  Name: Petrinet

  Author: Friedhelm Stappert, C-LAB, Paderborn, Germany

  Function: Simulate an extended Petri Net
            Automatically generated code containing large amounts of
            if-statements (more than 250)

  Source: Mälardalen benchmark suite

  Changes: no major functional changes

  License: may be used, modified, and re-distributed freely

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

const CHECKSUM: i64 = 0;

#[derive(Clone, Copy)]
enum PermutIndex {
    P1 { i1: usize, i2: usize, i3: usize },
    P2 { i1: usize, i2: usize, i3: usize, i4: usize },
}

#[inline(always)]
fn petrinet_permutation_p1_condition(
    p1_marking_member_0: &[i64],
    p1_is_marked: i32,
    permut_i: PermutIndex
) -> bool
{
    /* Transition T1 */
    if let PermutIndex::P1 { i1: _, i2, i3 } = permut_i {
        return p1_is_marked >= 3
            && p1_marking_member_0[i2] == p1_marking_member_0[i3];
    }
    false
}

#[inline(always)]
fn petrinet_permutation_p2_condition(
    p2_marking_member_0: &[i64],
    p2_is_marked: i32,
    permut_i: PermutIndex,
) -> bool
{
    /* Transition T2 */
    if let PermutIndex::P2 { i1: _, i2, i3, i4 } = permut_i {
        return p2_is_marked >= 4
            && p2_marking_member_0[i2] == p2_marking_member_0[i3]
            && p2_marking_member_0[i2] == p2_marking_member_0[i4];
    }
    false
}

#[inline(always)]
fn petrinet_permutation_marking(
    p_marking_member_0: &[i64],
    p_is_marked: &mut i32,
    p3_marking_member_0: &mut [i64; 6],
    p3_is_marked: &mut i32,
    permut_i: PermutIndex
) {
    let (pi1, pi2, f1, f2) = match permut_i {
        PermutIndex::P1 { i1, i2, i3: _ } => (
            i1, i2, 3, -1
        ),
        PermutIndex::P2 { i1, i2, i3: _, i4: _ } => (
            i1, i2, 4, 1
        ),
    };
    
    let x = p_marking_member_0[pi1];
    let y = p_marking_member_0[pi2];

    /* Transition condition */
    if x < y {

        /* demarking of input places */
        *p_is_marked -= f1;

        /* preaction */
        let z = x + f2 * y;

        /* marking of output places */
        p3_marking_member_0[(*p3_is_marked + 0) as usize] = x;
        p3_marking_member_0[(*p3_is_marked + 1) as usize] = y;
        p3_marking_member_0[(*p3_is_marked + 2) as usize] = z;
        *p3_is_marked += 3;
    }
}

#[inline(always)]
fn petrinet_permutation<F>(
    f: F,
    p_marking_member_0: &[i64],
    p_is_marked: &mut i32,
    p3_marking_member_0: &mut [i64; 6],
    p3_is_marked: &mut i32,
    permut_i: PermutIndex,
)
where
    F: Fn(&[i64], i32, PermutIndex) -> bool,
{
    if f(p_marking_member_0, *p_is_marked, permut_i) {
        petrinet_permutation_marking(
            p_marking_member_0,
            p_is_marked,
            p3_marking_member_0,
            p3_is_marked,
            permut_i,
        );
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn petrinet_main(
  p1_marking_member_0: &[i64; 3],
  p1_is_marked: &mut i32,
  p2_marking_member_0: &[i64; 5],
  p2_is_marked: &mut i32,
  p3_marking_member_0: &mut [i64; 6],
  p3_is_marked: &mut i32,
) {
    /*   dummy_i = 17; Takes too much time */
    let mut dummy_i = 2;

    let p1_permuts = [
        PermutIndex::P1 { i1: 0, i2: 1, i3: 2 }, /* Permutation for petrinet_Place P1 : 0, 1, 2 */
        PermutIndex::P1 { i1: 0, i2: 2, i3: 1 }, /* Permutation for petrinet_Place P1 : 0, 2, 1 */
        PermutIndex::P1 { i1: 1, i2: 0, i3: 2 }, /* Permutation for Place petrinet_P1 : 1, 0, 2 */
        PermutIndex::P1 { i1: 1, i2: 2, i3: 0 }, /* Permutation for Place petrinet_P1 : 1, 2, 0 */
        PermutIndex::P1 { i1: 2, i2: 0, i3: 1 }, /* Permutation for Place petrinet_P1 : 2, 0, 1 */
        PermutIndex::P1 { i1: 2, i2: 1, i3: 0 }, /* Permutation for Place petrinet_P1 : 2, 1, 0 */
    ];

    let p2_permuts = [
        PermutIndex::P2 { i1: 0, i2: 1, i3: 2, i4: 3 }, /* Permutation for Place petrinet_P2 : 0, 1, 2, 3 */
        PermutIndex::P2 { i1: 0, i2: 1, i3: 3, i4: 2 }, /* Permutation for Place petrinet_P2 : 0, 1, 3, 2 */
        PermutIndex::P2 { i1: 0, i2: 2, i3: 1, i4: 3 }, /* Permutation for Place petrinet_P2 : 0, 2, 1, 3 */
        PermutIndex::P2 { i1: 0, i2: 2, i3: 3, i4: 1 }, /* Permutation for Place petrinet_P2 : 0, 2, 3, 1 */
        PermutIndex::P2 { i1: 0, i2: 3, i3: 1, i4: 2 }, /* Permutation for Place petrinet_P2 : 0, 3, 1, 2 */
        PermutIndex::P2 { i1: 0, i2: 3, i3: 2, i4: 1 }, /* Permutation for Place petrinet_P2 : 0, 3, 2, 1 */
        PermutIndex::P2 { i1: 1, i2: 0, i3: 2, i4: 3 }, /* Permutation for Place petrinet_P2 : 1, 0, 2, 3 */
        PermutIndex::P2 { i1: 1, i2: 0, i3: 3, i4: 2 }, /* Permutation for Place petrinet_P2 : 1, 0, 3, 2 */
        PermutIndex::P2 { i1: 1, i2: 2, i3: 0, i4: 3 }, /* Permutation for Place petrinet_P2 : 1, 2, 0, 3 */
        PermutIndex::P2 { i1: 1, i2: 2, i3: 3, i4: 0 }, /* Permutation for Place petrinet_P2 : 1, 2, 3, 0 */
        PermutIndex::P2 { i1: 1, i2: 3, i3: 0, i4: 2 }, /* Permutation for Place petrinet_P2 : 1, 3, 0, 2 */
        PermutIndex::P2 { i1: 1, i2: 3, i3: 2, i4: 0 }, /* Permutation for Place petrinet_P2 : 1, 3, 2, 0 */
        PermutIndex::P2 { i1: 2, i2: 0, i3: 1, i4: 3 }, /* Permutation for Place petrinet_P2 : 2, 0, 1, 3 */
        PermutIndex::P2 { i1: 2, i2: 0, i3: 3, i4: 1 }, /* Permutation for Place petrinet_P2 : 2, 0, 3, 1 */
        PermutIndex::P2 { i1: 2, i2: 1, i3: 0, i4: 3 }, /* Permutation for Place petrinet_P2 : 2, 1, 0, 3 */
        PermutIndex::P2 { i1: 2, i2: 1, i3: 3, i4: 0 }, /* Permutation for Place petrinet_P2 : 2, 1, 3, 0 */
        PermutIndex::P2 { i1: 2, i2: 3, i3: 0, i4: 1 }, /* Permutation for Place petrinet_P2 : 2, 3, 0, 1 */
        PermutIndex::P2 { i1: 2, i2: 3, i3: 1, i4: 0 }, /* Permutation for Place petrinet_P2 : 2, 3, 1, 0 */
        PermutIndex::P2 { i1: 3, i2: 0, i3: 1, i4: 2 }, /* Permutation for Place petrinet_P2 : 3, 0, 1, 2 */
        PermutIndex::P2 { i1: 3, i2: 0, i3: 2, i4: 1 }, /* Permutation for Place petrinet_P2 : 3, 0, 2, 1 */
    ];

    // loopbound min 2 max 2
    while dummy_i > 0 {
        dummy_i -= 1;

        // loopbound min 6 max 6 -> new loop for less code
        for permut_i in p1_permuts {
            petrinet_permutation(
                petrinet_permutation_p1_condition,
                p1_marking_member_0,
                p1_is_marked,
                p3_marking_member_0,
                p3_is_marked,
                permut_i,
            );
        }
    
        // loopbound min 20 max 20 -> new loop for less code
        for permut_i in p2_permuts {
            petrinet_permutation(
                petrinet_permutation_p2_condition,
                p2_marking_member_0,
                p2_is_marked,
                p3_marking_member_0,
                p3_is_marked,
                permut_i,
            );
        }
    }
}


fn petrinet_init(
    p1_marking_member_0: &mut [i64; 3],
    p1_is_marked: &mut i32,
    p2_marking_member_0: &mut [i64; 5],
    p2_is_marked: &mut i32,
    p3_marking_member_0: &mut [i64; 6],
    p3_is_marked: &mut i32,
) {
    *p1_is_marked = 3;
    *p2_is_marked = 5;
    *p3_is_marked = 0;

    *p1_marking_member_0 = [0; 3];
    *p2_marking_member_0 = [0; 5];
    *p3_marking_member_0 = [0; 6];
}

fn petrinet_return(
    p1_marking_member_0: &[i64; 3],
    p2_marking_member_0: &[i64; 5],
    p3_marking_member_0: &[i64; 6],
) -> bool
{
    // TODO: use something from the Px_... arrays
    let mut checksum: i64 = 0;

    // loopbound min 3 max 3
    for elem in p1_marking_member_0.iter() {
        checksum += *elem;
    }

    // loopbound min 5 max 5
    for elem in p2_marking_member_0.iter() {
        checksum += *elem;
    }

    // loopbound min 6 max 6
    for elem in p3_marking_member_0.iter() {
        checksum += *elem;
    }

    checksum == CHECKSUM
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn petrinet_log(
    p1_marking_member_0: &mut [i64; 3],
    p1_is_marked: &mut i32,
    p2_marking_member_0: &mut [i64; 5],
    p2_is_marked: &mut i32,
    p3_marking_member_0: &mut [i64; 6],
    p3_is_marked: &mut i32,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        petrinet_init(
            black_box(p1_marking_member_0), black_box(p1_is_marked),
            black_box(p2_marking_member_0), black_box(p2_is_marked),
            black_box(p3_marking_member_0), black_box(p3_is_marked),
        );
        let start = black_box(DWT::cycle_count());
        petrinet_main(
            black_box(p1_marking_member_0), black_box(p1_is_marked),
            black_box(p2_marking_member_0), black_box(p2_is_marked),
            black_box(p3_marking_member_0), black_box(p3_is_marked),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("petrinet - elapsed: {}", cycles);
    });

    black_box(petrinet_return(
        black_box(p1_marking_member_0),
        black_box(p2_marking_member_0),
        black_box(p3_marking_member_0),
    ))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn petrinet_std(
    p1_marking_member_0: &mut [i64; 3],
    p1_is_marked: &mut i32,
    p2_marking_member_0: &mut [i64; 5],
    p2_is_marked: &mut i32,
    p3_marking_member_0: &mut [i64; 6],
    p3_is_marked: &mut i32,
) -> bool {
    petrinet_init(
        black_box(p1_marking_member_0), black_box(p1_is_marked),
        black_box(p2_marking_member_0), black_box(p2_is_marked),
        black_box(p3_marking_member_0), black_box(p3_is_marked),
    );
    petrinet_main(
        black_box(p1_marking_member_0), black_box(p1_is_marked),
        black_box(p2_marking_member_0), black_box(p2_is_marked),
        black_box(p3_marking_member_0), black_box(p3_is_marked),
    );
    black_box(petrinet_return(
        black_box(p1_marking_member_0),
        black_box(p2_marking_member_0),
        black_box(p3_marking_member_0),
    ))
}

fn petrinet() -> bool {
    static mut P1_MARKING_MEMBER_0: [i64; 3] = [0; 3];
    static mut P1_IS_MARKED: i32 = 0;
    static mut P2_MARKING_MEMBER_0: [i64; 5] = [0; 5];
    static mut P2_IS_MARKED: i32 = 0;
    static mut P3_MARKING_MEMBER_0: [i64; 6] = [0; 6];
    static mut P3_IS_MARKED: i32 = 0;
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let p1_marking_member_0 = unsafe { &mut *{ &raw mut P1_MARKING_MEMBER_0 } };
    let p1_is_marked = unsafe { &mut *{ &raw mut P1_IS_MARKED } };
    let p2_marking_member_0 = unsafe { &mut *{ &raw mut P2_MARKING_MEMBER_0 } };
    let p2_is_marked = unsafe { &mut *{ &raw mut P2_IS_MARKED } };
    let p3_marking_member_0 = unsafe { &mut *{ &raw mut P3_MARKING_MEMBER_0 } };
    let p3_is_marked = unsafe { &mut *{ &raw mut P3_IS_MARKED } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(petrinet_log(
            black_box(p1_marking_member_0), black_box(p1_is_marked),
            black_box(p2_marking_member_0), black_box(p2_is_marked),
            black_box(p3_marking_member_0), black_box(p3_is_marked),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(petrinet_std(
            black_box(p1_marking_member_0), black_box(p1_is_marked),
            black_box(p2_marking_member_0), black_box(p2_is_marked),
            black_box(p3_marking_member_0), black_box(p3_is_marked),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(petrinet());
    
    #[cfg(feature = "log")]
    {
        rprintln!("petrinet: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(petrinet());

    #[cfg(feature = "log")]
    std::println!("petrinet: {}", success);

    std::process::exit(!success as i32);
}
