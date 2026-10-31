/*

  Rust Implementation

  Author: David Heim
  License: The Rust implementation shares the same license (GPL) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

  Version V 2.0

  Name: quicksort

  Author: Matthew R. Guthaus

  Function: quicksort applies a recursive quicksort algorithm to two different
    input sets.

  Source: MiBench
          http://wwweb.eecs.umich.edu/mibench

  Original name: qsort

  Changes: No major functional changes.

  License: GPL

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

mod input;
mod math;

use core::{cmp::Ordering, ffi::CStr, hint::black_box};

use input::{QUICKSORT_INPUT_STRING, QUICKSORT_INPUT_VECTOR};

#[derive(Debug, Default, Copy, Clone)]
struct Vertex3D {
    x: u32,
    y: u32,
    z: u32,
    distance: f64,
}

impl PartialEq for Vertex3D {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl PartialOrd for Vertex3D {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Vertex3D {}

impl Ord for Vertex3D {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.distance.total_cmp(&other.distance)
    }
}

#[derive(Debug, Copy, Clone)]
enum Dir {
    Ascending,
    Descending,
}

fn quicksort_init(strings: &mut [&CStr; 681], vectors: &mut [Vertex3D; 1000]) {
    // loopbound min 681 max 681
    for (string, &istring) in strings.iter_mut().zip(&QUICKSORT_INPUT_STRING) {
        // loopbound min 1 max 20
        *string = black_box(istring);
    }

    for (vector, ivector) in vectors
        .iter_mut()
        .zip(QUICKSORT_INPUT_VECTOR.chunks_exact(3))
    {
        vector.x = black_box(ivector[0]);
        vector.y = black_box(ivector[1]);
        vector.z = black_box(ivector[2]);

        vector.distance = math::sqrtf(
            math::powf(vector.x as f32, 2.)
                + math::powf(vector.y as f32, 2.)
                + math::powf(vector.z as f32, 2.),
        ) as f64;
    }
}

fn quicksort_return(strings: &[&CStr; 681], vectors: &[Vertex3D; 1000]) -> bool {
    let checksum = strings[42].to_bytes()[1] as u32 + vectors[42].x + vectors[42].y + vectors[42].z;
    checksum == 1527923179
}

fn quicksort_pivot<T>(arr: &[T]) -> usize
where
    T: Ord + Copy,
{
    let i = arr.len() / 6;
    let si = arr[i]; // 1/6

    let j = 3 * i;
    let sj = arr[j]; // 1/2

    let k = 5 * i;
    let sk = arr[k]; // 5/6

    if si < sj {
        if si < sk {
            if sj < sk {
                return j;
            }
            return k;
        }
        return i;
    }

    if sj < sk {
        if si < sk {
            return i;
        }
        return k;
    }
    j
}

fn quicksort_sort<T>(mut arr: &mut [T], dir: Dir)
where
    T: Ord + Copy,
{
    let (ord_lo, ord_hi) = match dir {
        Dir::Ascending => (Ordering::Less, Ordering::Greater),
        Dir::Descending => (Ordering::Greater, Ordering::Less),
    };

    // str: loopbound min 0 max 8
    // vec: loopbound min 0 max 15
    while arr.len() > 1 {
        let pivot = if arr.len() > 10 {
            quicksort_pivot(arr)
        } else {
            arr.len() >> 1
        };

        arr.swap(0, pivot);
        let mut pi = 0;
        let mut pj = arr.len();
        let pn = pj;

        // str: loopbound min 0 max 169
        // vec: loopbound min 1 max 250
        loop {
            // str: loopbound min 1 max 26
            // vec: loopbound min 1 max 51
            loop {
                pi += 1;

                if !(pi < pn && arr[pi].cmp(&arr[0]) == ord_lo) {
                    break;
                }
            }

            // str: loopbound min 1 max 23
            // vec: loopbound min 1 max 27
            loop {
                pj -= 1;

                if !(pj > 0 && arr[pj].cmp(&arr[0]) == ord_hi) {
                    break;
                }
            }

            if pj < pi {
                break;
            }

            arr.swap(pi, pj);
        }

        arr.swap(0, pj);

        let n = arr.len() - pj - 1;
        if pj >= n {
            quicksort_sort(&mut arr[..pj], dir);
            arr = &mut arr[pj + 1..]
        } else {
            quicksort_sort(&mut arr[pj + 1..], dir);
            arr = &mut arr[..pj];
        }
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn quicksort_main(strings: &mut [&CStr; 681], vectors: &mut [Vertex3D; 1000]) {
    // marker recursivecall
    // flowrestriction 1*quicksort_str <= 521*recursivecall
    quicksort_sort(strings, Dir::Descending);

    // marker recursivecall2
    // flowrestriction 1*quicksort_vec <= 650*recursivecall2
    quicksort_sort(vectors, Dir::Ascending);
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn quicksort_log(strings: &mut [&CStr; 681], vectors: &mut [Vertex3D; 1000]) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        quicksort_init(black_box(strings), black_box(vectors));
        let start = black_box(DWT::cycle_count());
        quicksort_main(black_box(strings), black_box(vectors));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("quicksort_custom - elapsed: {}", cycles);
    });

    black_box(quicksort_return(black_box(strings), black_box(vectors)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn quicksort_std(strings: &mut [&CStr; 681], vectors: &mut [Vertex3D; 1000]) -> bool {
    quicksort_init(black_box(strings), black_box(vectors));
    quicksort_main(black_box(strings), black_box(vectors));

    black_box(quicksort_return(black_box(strings), black_box(vectors)))
}

fn quicksort() -> bool {
    static mut STRINGS: [&CStr; 681] = [c""; 681];
    static mut VECTORS: [Vertex3D; 1000] = [Vertex3D {
        x: 0,
        y: 0,
        z: 0,
        distance: 0.,
    }; 1000];
    // SAFETY: This is the only place we ever reference these and this function is only ever called once.
    let strings = unsafe { &mut *{ &raw mut STRINGS } };
    let vectors = unsafe { &mut *{ &raw mut VECTORS } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(quicksort_log(black_box(strings), black_box(vectors)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(quicksort_std(black_box(strings), black_box(vectors)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(quicksort());

    #[cfg(feature = "log")]
    {
        rprintln!("quicksort_custom: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(quicksort());

    #[cfg(feature = "log")]
    std::println!("quicksort_custom: {}", success);

    std::process::exit(!success as i32);
}
