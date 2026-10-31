/*

  Rust Implementation

  Author: David Heim
  License: The Rust implementation shares the same license (BSD 3-clause) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

  Version 2.0

  Name: pm

  Author: Hector Chan
          MIT Lincoln Laboratory

  Function: This is the ANSI C Pattern Match kernel. It finds the closest
            match of a pattern from a library of patterns.  The code below
            serves as a reference implemenation of the pattern match kernel.

  Source: HPEC Challenge Benchmark Suite, Pattern Match Kernel Benchmark

  Original name: pm

  Changes: See ChangeLog.txt

  License: BSD 3-clause

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

use core::hint::black_box;
use input::{PM_LIB_DATA, PM_PATTERN_DATA};

use crate::math::{pm_ceilf, pm_floorf};

const PM_MIN_NOISE: f32 = 1e-10;

static mut PM_INIT_ARRAY_1: [u8; 64] = [0; 64];
static mut PM_INIT_ARRAY_2: [f32; 21] = [0.; 21];
static mut PM_INIT_ARRAY_3: [f32; 64] = [0.; 64];
static mut PM_INIT_ARRAY_4: [f32; 64] = [0.; 64];
static mut PM_INIT_ARRAY_5: [f32; 21] = [0.; 21];
static mut PM_INIT_ARRAY_6: [f32; 21] = [0.; 21];
static mut PM_INIT_ARRAY_7: [f32; 72] = [0.; 72];
static mut PM_INIT_ARRAY_8: [f32; 110] = [0.; 110];

struct Array<'a> {
    data: &'a mut [f32],
    size: (usize, usize),
}

impl<'a> Array<'a> {
    fn new<const N: usize, const M: usize>(data: &'a mut [[f32; N]; M]) -> Self {
        Self {
            size: (data.len(), data[0].len()),
            data: data.as_flattened_mut(),
        }
    }

    fn reset<const N: usize, const M: usize>(&mut self, ary: &'a [[f32; N]; M]) {
        for(d, s) in self.data.iter_mut().zip(ary.iter().flat_map(|row| row.iter())) {
            *d = *s;
        }
    }
}

struct PmData<'a> {
    template_profiles_db: &'a mut [f32],
    test_profile_db: &'a [f32],

    template_copy: &'a mut [f32],
    test_noise_db_array: &'a mut [f32],

    mse_scores: &'a mut [f32],
    mag_shift_scores: &'a mut [f32],
    minimum_mse_score: &'a mut [f32],

    all_shifted_test_db: &'a mut [f32],
    template_exceed: &'a mut [u8],
    test_exceed_means: &'a mut [f32],

    shift_size: usize,
    profile_size: usize,
    num_templates: usize,
}

impl<'a> PmData<'a> {
    fn new(lib: &'a mut Array<'static>, pattern: &'a Array<'static>) -> Self {
        let profile_size = lib.size.1;
        let num_templates = lib.size.0;

        let shift_ratio = 3.0;
        let x = profile_size as f32 / shift_ratio;
        let shs = if (x - (x as i32) as f32) < 0.5f32 {pm_floorf(x) as i32} else {pm_ceilf(x) as i32}; 

        Self {
            profile_size,
            num_templates,

            template_profiles_db: lib.data,
            test_profile_db: pattern.data,

            /* Equivalent to shift_size = roundf((float)profile_size / shift_ratio) */
            shift_size: shs as usize,

            // SAFETY: This is the only place we reference these and there is only ever one instance of PmData
            template_exceed: unsafe { &mut *{ &raw mut PM_INIT_ARRAY_1 } },
            test_exceed_means: unsafe { &mut *{ &raw mut PM_INIT_ARRAY_2 } },

            template_copy: unsafe { &mut *{ &raw mut PM_INIT_ARRAY_3 } },
            test_noise_db_array: unsafe { &mut *{ &raw mut PM_INIT_ARRAY_4 } },

            mse_scores: unsafe { &mut *{ &raw mut PM_INIT_ARRAY_5 } },
            mag_shift_scores: unsafe { &mut *{ &raw mut PM_INIT_ARRAY_6 } },

            minimum_mse_score: unsafe { &mut *{ &raw mut PM_INIT_ARRAY_7 } },
            all_shifted_test_db: unsafe { &mut *{ &raw mut PM_INIT_ARRAY_8 } },
        }
    }
}

fn pm_init(lib: &mut Array<'static>, pattern: &mut Array<'static>) {
    lib.reset(&PM_LIB_DATA);
    pattern.reset(&PM_PATTERN_DATA);
}

fn pm_kernel(pmdata: &mut PmData<'_>) -> usize {
    let sum_weights_inv = (pmdata.profile_size as f32).recip();

    let test_noise = (math::pm_pow10f(pmdata.test_profile_db[0] * 0.1)
        + math::pm_pow10f(pmdata.test_profile_db[pmdata.profile_size - 1] * 0.1))
        * 0.5;
    let half_shift_size = math::pm_ceilf(pmdata.shift_size as f32 / 2.0) as usize;

    let test_noise_db =
        if test_noise == 0.0 { -100. } else { 10. } * math::pm_log10f(test_noise.abs());
    let test_noise_db_plus_3 = test_noise_db + 3.0;

    pmdata.test_noise_db_array[..pmdata.profile_size].fill(test_noise_db);

    // loopbound min 64 max 64
    let test_peak = pmdata.test_profile_db[..pmdata.profile_size]
        .iter()
        .copied()
        .max_by(f32::total_cmp)
        .expect("array is not empty");

    pmdata.all_shifted_test_db[..half_shift_size]
        .copy_from_slice(&pmdata.test_noise_db_array[..half_shift_size]);
    pmdata.all_shifted_test_db[half_shift_size..half_shift_size + pmdata.profile_size]
        .copy_from_slice(&pmdata.test_profile_db[..pmdata.profile_size]);
    pmdata.all_shifted_test_db[half_shift_size + pmdata.profile_size
        ..half_shift_size + pmdata.profile_size + half_shift_size]
        .copy_from_slice(&pmdata.test_noise_db_array[..half_shift_size]);

    /* Set the pixels to test noise in dB domain if pixel is less than test
    noise in dB */

    // loopbound min 64 max 64
    for f in pmdata.all_shifted_test_db[half_shift_size..half_shift_size + pmdata.profile_size]
        .iter_mut()
    {
        if *f < test_noise_db {
            *f = test_noise_db;
        }
    }

    /* Calculating the mean of the pixels that exceeded twice test noise for each
    possible shift of the test profile */

    // loopbound min 21 max 21
    for (current_shift, exceed_mean) in pmdata.test_exceed_means[..pmdata.shift_size]
        .iter_mut()
        .enumerate()
    {
        let f = if current_shift < half_shift_size {
            &pmdata.all_shifted_test_db[half_shift_size..current_shift + pmdata.profile_size]
        } else {
            &pmdata.all_shifted_test_db[current_shift..half_shift_size + pmdata.profile_size]
        };

        let mut sum_exceed = 0.0;
        let mut num_test_exceed = 0;
        // loopbound min 53 max 64
        for &x in f {
            if x > test_noise_db_plus_3 {
                num_test_exceed += 1;
                sum_exceed += x;
            }
        }
        *exceed_mean = if num_test_exceed != 0 {
            sum_exceed / num_test_exceed as f32
        } else {
            0.0
        };
    }

    /* Loop over all the templates. Determine the best shift distance, then
    the best gain adjustment. */
    // loopbound min 60 max 60
    for template_index in 0..pmdata.num_templates {
        let cur_tp = &mut pmdata.template_profiles_db[template_index * pmdata.profile_size..]
            [..pmdata.profile_size];

        // loopbound min 64 max 64
        let template_peak = cur_tp
            .iter()
            .copied()
            .max_by(f32::total_cmp)
            .expect("array is not empty");

        /* Additively adjust the noise level of this template profile in the
        raw power domain so that its noise level matches the noise level
        of the test profile */

        let noise_shift = test_peak - template_peak;
        pmdata.template_exceed[..pmdata.profile_size].fill(0);
        let mut sum_exceed = 0.0;
        let mut num_template_exceed = 0;

        // loopbound min 64 max 64
        for x in cur_tp.iter_mut() {
            let tmp = *x + noise_shift;
            *x = math::pm_pow10f(tmp * 0.1);
        }

        let mut template_exceed_mean = 0.0;
        let template_noise = (cur_tp[0] + cur_tp[pmdata.profile_size - 1]) * 0.5;
        let noise_shift2 = test_noise - template_noise;

        // loopbound min 64 max 64
        for (x, template_exceed) in cur_tp.iter_mut().zip(pmdata.template_exceed.iter_mut()) {
            let mut tmp = *x + noise_shift2;

            if tmp == 0.0 {
                tmp = PM_MIN_NOISE;
            }

            *x = 10.0 * math::pm_log10f(tmp.abs()) + test_noise_db;

            /* Because many of the operations in the search for the best shift
            amount depend on knowledge of which pixels in the template
            have values exceeding twice test_noise (recall that 3db is roughly
            equivalent to a doubling of raw power), we'll put those indices in
            template_exceed */

            if *x > test_noise_db_plus_3 {
                *template_exceed = 1;
                num_template_exceed += 1;
                sum_exceed += *x;
            }
        }

        /* If there is at least one pixel in the template we're
        currently working on whose value exceeds twice test_noise */
        if num_template_exceed > 0 {
            template_exceed_mean = sum_exceed / num_template_exceed as f32;

            // loopbound min 21 max 21
            for (current_shift, (test_exceed_mean, mse_score)) in pmdata.test_exceed_means
                [..pmdata.shift_size]
                .iter()
                .zip(pmdata.mse_scores.iter_mut())
                .enumerate()
            {
                /* Work on a copy of the template we're currently working on */
                pmdata.template_copy.copy_from_slice(cur_tp);

                /* If there is at least one pixel in the shifted test profile
                whose value exceeds twice test noise. */
                if *test_exceed_mean != 0.0 {
                    /* CASE 1 */
                    /* Considering only those pixels whose powers exceed twice
                    test noise, compute the difference of the mean power in
                    template we're currently working on. */
                    let power_ratio = *test_exceed_mean - template_exceed_mean;

                    /* Scale template values that exceed twice test noise by power ratio
                    and set the values that are less than test noise in db to test
                    noise in db */
                    // loopbound min 64 max 64
                    for (f, &b) in pmdata
                        .template_copy
                        .iter_mut()
                        .zip(pmdata.template_exceed.iter())
                    {
                        if b > 0 {
                            *f += power_ratio;
                        }
                        if *f < test_noise_db {
                            *f = test_noise_db;
                        }
                    }
                } else {
                    /* CASE 2 */
                    /* Set those pixels in the template we're currently working on
                    whose values are less than test_noise to test_noise. */
                    // loopbound min 64 max 64
                    for (template_copy, &x) in pmdata.template_copy.iter_mut().zip(cur_tp.iter()) {
                        if x < test_noise_db {
                            *template_copy = test_noise_db;
                        }
                    }
                }

                /* Compute the weighted MSE */
                // loopbound min 64 max 64
                let weighted_mse: f32 = pmdata
                    .template_copy
                    .iter()
                    .zip(pmdata.all_shifted_test_db[current_shift..].iter())
                    .map(|(tc, atdb)| tc - atdb)
                    .map(|x| x * x)
                    .sum();
                *mse_score = weighted_mse * sum_weights_inv;
            }
        } else {
            // loopbound min 21 max 21
            for (current_shift, &test_exceed_mean) in pmdata.test_exceed_means[..pmdata.shift_size]
                .iter()
                .enumerate()
            {
                /* CASE 3 */
                /* If there is at least one pixel that exceeds twice test noise */

                let f = if test_exceed_mean > 0.0 {
                    &mut *cur_tp
                } else {
                    /* CASE 4 */
                    /* Work on a copy of the template we're currently working on. */
                    pmdata.template_copy.copy_from_slice(cur_tp);

                    // loopbound min 64 max 64
                    for (copy, &cur) in pmdata.template_copy.iter_mut().zip(cur_tp.iter()) {
                        if cur < test_noise_db {
                            *copy = test_noise_db;
                        }
                    }

                    &mut pmdata.template_copy
                };

                /* Compute the weighted MSE */
                // loopbound min 64 max 64
                let weighted_mse: f32 = f
                    .iter()
                    .zip(pmdata.all_shifted_test_db[current_shift..].iter())
                    .map(|(tc, atdb)| tc - atdb)
                    .map(|x| x * x)
                    .sum();
                pmdata.mse_scores[current_shift] = weighted_mse * sum_weights_inv;
            }
        }

        /* Finding the minimum MSE for range shifting */
        // loopbound min 20 max 20
        let (min_mse_index, _) = pmdata
            .mse_scores
            .iter()
            .enumerate()
            .min_by(|&x, &y| f32::total_cmp(x.1, y.1))
            .expect("array is not empty");

        /* Work on a copy of the template we're currently working on. */
        pmdata.template_copy.copy_from_slice(cur_tp);

        let mut mag_shift_scores_flag = true;
        if pmdata.test_exceed_means[min_mse_index] != 0.0 {
            if num_template_exceed > 0 {
                /* Compute the difference of the average shifted test profile
                power to the average template power */
                let avg_power_ratio =
                    pmdata.test_exceed_means[min_mse_index] - template_exceed_mean;

                /* Loop over all possible magnitude shifts */
                // loopbound min 21 max 21
                for (j, mag_shift_score) in pmdata.mag_shift_scores.iter_mut().enumerate() {
                    let mag_db = -5.0 + (j as f32 * 0.5);
                    let power_shift = avg_power_ratio + mag_db;

                    // loopbound min 64 max 64
                    for ((copy, &exceed), &tp) in pmdata
                        .template_copy
                        .iter_mut()
                        .zip(pmdata.template_exceed.iter())
                        .zip(cur_tp.iter())
                    {
                        if exceed > 0 {
                            *copy = tp + power_shift;
                        }
                    }

                    /* Compute the weighted MSE */
                    // loopbound min 64 max 64
                    let weighted_mse: f32 = pmdata
                        .template_copy
                        .iter()
                        .zip(pmdata.all_shifted_test_db[min_mse_index..].iter())
                        .map(|(tc, atdb)| tc - atdb)
                        .map(|x| x * x)
                        .sum();

                    *mag_shift_score = weighted_mse * sum_weights_inv;
                }
            }
        } else {
            /* Set those pixels in the template we're currently working on
            whose values are less than test_noise to test_noise. */
            // loopbound min 64 max 64
            for (copy, &tp) in pmdata.template_copy.iter_mut().zip(cur_tp.iter()) {
                if tp < test_noise_db {
                    *copy = test_noise_db;
                }
            }

            /* Compute the weighted MSE */
            // loopbound min 64 max 64
            let weighted_mse: f32 = pmdata
                .template_copy
                .iter()
                .zip(pmdata.all_shifted_test_db[min_mse_index..].iter())
                .map(|(tc, atdb)| tc - atdb)
                .map(|x| x * x)
                .sum();

            pmdata.minimum_mse_score[template_index] = weighted_mse * sum_weights_inv;
            mag_shift_scores_flag = false;
        }

        /* If magnitude shifting has performed above */
        if mag_shift_scores_flag {
            /* Find the minimum MSE for magnitude scaling */
            // loopbound min 20 max 20
            let min_mse = pmdata
                .mag_shift_scores
                .iter()
                .copied()
                .min_by(f32::total_cmp)
                .expect("array is not empty");

            pmdata.minimum_mse_score[template_index] = min_mse;
        }
    }

    /* Find the minimum mean square error */
    // loopbound min 59 max 59
    let (match_index, _) = pmdata.minimum_mse_score[..pmdata.num_templates]
        .iter()
        .enumerate()
        .min_by(|x, y| f32::total_cmp(x.1, y.1))
        .expect("array is not empty");

    match_index
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn pm_main(lib: &mut Array<'static>, pattern: &mut Array<'static>) -> usize {
    let mut pmdata = PmData::new(lib, pattern);

    pm_kernel(&mut pmdata)
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn pm_log(lib: &mut Array<'static>, pattern: &mut Array<'static>) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];
    let mut result: usize = 0;

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        pm_init(black_box(lib), black_box(pattern));
        let start = black_box(DWT::cycle_count());
        result = black_box(pm_main(black_box(lib), black_box(pattern)));
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("pm_custom - elapsed: {}", cycles);
    });

    result == 12
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn pm_std(lib: &mut Array<'static>, pattern: &mut Array<'static>) -> bool {
    pm_init(black_box(lib), black_box(pattern));
    let result = black_box(pm_main(black_box(lib), black_box(pattern)));

    result == 12
}

fn pm() -> bool {
    static mut PM_LIB_DATA_COPY: [[f32; 64]; 60] = [[0.; 64]; 60];
    static mut PM_PATTERN_DATA_COPY: [[f32; 64]; 60] = [[0.; 64]; 60];
    // SAFETY: This is the only place we ever reference these and this function is only ever called once.
    let mut lib: Array<'static> = Array::new(unsafe { &mut *{ &raw mut PM_LIB_DATA_COPY } });
    let mut pattern: Array<'static> = Array::new(unsafe { &mut *{ &raw mut PM_PATTERN_DATA_COPY } });

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(pm_log(black_box(&mut lib), black_box(&mut pattern)))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(pm_std(black_box(&mut lib), black_box(&mut pattern)))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(pm());

    #[cfg(feature = "log")]
    {
        rprintln!("pm_custom: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(pm());

    #[cfg(feature = "log")]
    std::println!("pm_custom: {}", success);
    
    std::process::exit(!success as i32);
}
