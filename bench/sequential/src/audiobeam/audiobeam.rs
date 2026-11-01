/*

  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license (MIT) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

  Version V 2.0

  Name: audiobeam

  Author: Eugene Weinstein

  Function: Audio beam former

  Source: StreamIt
          http://groups.csail.mit.edu/cag/streamit/

  Changes: no functional changes

  License: see license.txt

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::{f32, hint::black_box};

mod input;
mod memory;
mod math;

use core::mem::size_of;
use memory::{ HeapSlice, SimulatedHeap };
use math::{ ceilf, floorf, sqrtf, cosf, PI };

use crate::input::{
    START_INPUT, INPUT_MIC_LOCATIONS, INPUT_ORIGIN_LOCATION, INPUT_SOURCE_LOCATION
};

/*
##################
    audiobeam.h
##################
*/

#[allow(dead_code)]
struct DataQueue {
    sample_queue: HeapSlice,
    sample_queue_dim: (usize, usize),
    head: i32,
    tail: i32,
    full: u8,
}

impl DataQueue {
    const QUEUE_SERIALIZED_SIZE:
        usize =
            size_of::<HeapSlice>()
            + size_of::<(usize, usize)>()
            + size_of::<i32>()
            + size_of::<i32>()
            + size_of::<u8>();

    fn init(max_delay: usize, num_mic: usize, heap: &mut SimulatedHeap) -> HeapSlice {
        let queue = heap.malloc(DataQueue::QUEUE_SERIALIZED_SIZE, DataQueue::QUEUE_SERIALIZED_SIZE);
        let sample_queue = heap.malloc((max_delay + 1) * (num_mic) * size_of::<f32>(), size_of::<f32>());
        // loopbound min 15 max 15
        for i in 0..(max_delay + 1) {
            // loopbound min 15 max 15
            for j in 0..num_mic {
                let index = sample_queue.start + i * num_mic * size_of::<f32>() + j * size_of::<f32>();
                // Initialize values to 0
                heap.heap_content[index..(index + size_of::<f32>())].copy_from_slice(&[0; size_of::<f32>()]);
            }
        }
        // sample_queue
        let ary = sample_queue.to_ne_bytes();
        heap.heap_content[queue.start..(queue.start + size_of::<HeapSlice>())].copy_from_slice(&ary);
        // sample_queue_dim
        let index: usize = queue.start + size_of::<HeapSlice>();
        let ary = (max_delay + 1).to_ne_bytes();
        heap.heap_content[index..(index + size_of::<usize>())].copy_from_slice(&ary);
        let index: usize = index + size_of::<usize>();
        let ary = num_mic.to_ne_bytes();
        heap.heap_content[index..(index + size_of::<usize>())].copy_from_slice(&ary);
        // head
        let index: usize = index + size_of::<usize>();
        heap.heap_content[index..(index + size_of::<i32>())].copy_from_slice(&[0; size_of::<i32>()]);
        // tail
        let index: usize = index + size_of::<i32>();
        heap.heap_content[index..(index + size_of::<i32>())].copy_from_slice(&[0; size_of::<i32>()]);
        // full
        let index: usize = index + size_of::<i32>();
        heap.heap_content[index] = 0;
        
        queue
    }

    fn from_ne_bytes(bytes: [u8; DataQueue::QUEUE_SERIALIZED_SIZE]) -> DataQueue {
        let (a, r1) = bytes.split_at(size_of::<HeapSlice>());
        let (b, r2) = r1.split_at(size_of::<usize>());
        let (c, r3) = r2.split_at(size_of::<usize>());
        let (d, r4) = r3.split_at(size_of::<i32>());
        let (e, f) = r4.split_at(size_of::<i32>());

        let sample_queue = HeapSlice::from_ne_bytes(<[u8; 3 * size_of::<usize>()]>::try_from(a).unwrap());
        let dim_x = usize::from_ne_bytes(<[u8; size_of::<usize>()]>::try_from(b).unwrap());
        let dim_y = usize::from_ne_bytes(<[u8; size_of::<usize>()]>::try_from(c).unwrap());
        let head = i32::from_ne_bytes(<[u8; size_of::<i32>()]>::try_from(d).unwrap());
        let tail = i32::from_ne_bytes(<[u8; size_of::<i32>()]>::try_from(e).unwrap());
        let full = u8::from_ne_bytes(<[u8; size_of::<u8>()]>::try_from(f).unwrap());
        DataQueue { sample_queue, sample_queue_dim: (dim_x, dim_y), head, tail, full }
    }
}

struct Delays {
    delay_values: HeapSlice,
    delay_values_dim: (usize, usize),
    max_delay: i64,
}

impl Delays {
    const DELAYS_SERIALIZED_SIZE:
        usize =
            size_of::<HeapSlice>()
            + 2 * size_of::<usize>()
            + size_of::<i64>();

    fn init(num_angles: usize, num_mic: usize, heap: &mut SimulatedHeap) -> HeapSlice {
        let delays = heap.malloc(Delays::DELAYS_SERIALIZED_SIZE, Delays::DELAYS_SERIALIZED_SIZE);

        // Initialize the delays array
        // loopbound min 1 max 1
        let delay_values = heap.malloc(num_angles * num_mic * size_of::<f32>(), size_of::<f32>());

        // delay_values
        let ary = delay_values.to_ne_bytes();
        heap.heap_content[delays.start..(delays.start + size_of::<HeapSlice>())].copy_from_slice(&ary);
        // delay_values_dim
        let index: usize = delays.start + size_of::<HeapSlice>();
        let ary = num_angles.to_ne_bytes();
        heap.heap_content[index..(index + size_of::<usize>())].copy_from_slice(&ary);
        let index: usize = index + size_of::<usize>();
        let ary = num_mic.to_ne_bytes();
        heap.heap_content[index..(index + size_of::<usize>())].copy_from_slice(&ary);
        // max_delay
        let index = delays.start + size_of::<HeapSlice>() + 2 * size_of::<usize>();
        heap.heap_content[index..index + size_of::<i64>()].copy_from_slice(&0i64.to_ne_bytes());

        delays
    }

    fn delays_col(self, i: usize) -> HeapSlice {
        let row_size = self.delay_values_dim.1 * size_of::<f32>();
        let start_i = self.delay_values.start + i * row_size;
        let end_i = start_i + row_size;
        HeapSlice { start: start_i, end: end_i, type_size: size_of::<f32>() }
    }

    fn from_ne_bytes(bytes: [u8; Delays::DELAYS_SERIALIZED_SIZE]) -> Delays {
        let (a, r1) = bytes.split_at(size_of::<HeapSlice>());
        let (b, r2) = r1.split_at(size_of::<usize>());
        let (c, d) = r2.split_at(size_of::<usize>());

        let delay_values = HeapSlice::from_ne_bytes(<[u8; 3 * size_of::<usize>()]>::try_from(a).unwrap());
        let dim_x = usize::from_ne_bytes(<[u8; size_of::<usize>()]>::try_from(b).unwrap());
        let dim_y = usize::from_ne_bytes(<[u8; size_of::<usize>()]>::try_from(c).unwrap());
        let max_delay = i64::from_ne_bytes(<[u8; size_of::<i64>()]>::try_from(d).unwrap());
        Delays { delay_values, delay_values_dim: (dim_x, dim_y), max_delay }
    }
}

#[derive(Default, Clone, Copy)]
struct PreprocessedDelays {
    delay: f32,
    low: i32,
    high: i32,
    offset: f32,
}

/*
##################
    audiobeam.c
##################
*/

/*
  Initialization- and return-value-related functions
*/

fn audiobeam_init(
    inp: &mut [f32; 5760],
    input_pos: &mut usize,
    mic_locations: &mut [[f32; 3]; 15],
    source_location: &mut [f32; 3],
    origin_location: &mut [f32; 3],
    checksum: &mut i32,
    heap: &mut SimulatedHeap,
) {
    *input_pos = 0;
    *checksum = 0;
    heap.reset();

    let bitmask: u32 = 0;

    /*
        Apply volatile XOR-bitmask to entire input array.
    */
    // loopbound min 23040 max 23040
    for elem in inp.iter_mut() {
        *elem = f32::from_bits(elem.to_bits() ^ bitmask);
    }

    // loopbound min 180 max 180
    for col in mic_locations.iter_mut() {
        for elem in col.iter_mut() {
            *elem = f32::from_bits(elem.to_bits() ^ bitmask);
        }
    }

    // loopbound min 12 max 12
    for elem in source_location.iter_mut() {
        *elem = f32::from_bits(elem.to_bits() ^ bitmask);
    }

    // loopbound min 12 max 12
    for elem in origin_location.iter_mut() {
        *elem = f32::from_bits(elem.to_bits() ^ bitmask);
    }
}

fn audiobeam_return(checksum: &mut i32) -> bool {
    *checksum + 1 == 0
}

/*
  Algorithm core functions
*/

fn audiobeam_preprocess_delays(prep_delays: &mut [PreprocessedDelays], delays: HeapSlice, heap: &mut SimulatedHeap) {
    // loopbound min 15 max 15
    for (i, prep_delay) in prep_delays.iter_mut().enumerate() {
        let start_i = delays.start + i * delays.type_size;
        let end_i = start_i + delays.type_size;
        let ary: [u8; size_of::<f32>()] = heap.heap_content[start_i..end_i].try_into().unwrap();
        let delay = f32::from_ne_bytes(ary);
        prep_delay.delay = delay;
        prep_delay.high = ceilf(delay) as i32;
        prep_delay.low = floorf(delay) as i32;
        prep_delay.offset = delay - prep_delay.low as f32;
    }
}

fn audiobeam_parse_line(
    float_arr: HeapSlice,
    num_mic: usize,
    inp: &mut [f32; 5760],
    input_pos: &mut usize,
    heap: &mut SimulatedHeap
) {
    // loopbound min 15 max 15
    for i in 0..num_mic {
        let start_i = float_arr.start + i * float_arr.type_size;
        let end_i = start_i + float_arr.type_size;
        let bytes = inp[*input_pos].to_ne_bytes();
        heap.heap_content[start_i..end_i].copy_from_slice(&bytes);
        *input_pos += 1;
    }
}

fn audiobeam_find_max_in_arr(arr: &mut [f32]) -> i64 {
    // loopbound min 15 max 15
    let max: f32 = arr.iter()
                      .copied()
                      .filter(|x| !x.is_nan())
                      .max_by(|a, b| a.partial_cmp(b).unwrap())
                      .unwrap();

    ceilf(max) as i64
}

fn audiobeam_find_min_in_arr(arr: &mut [f32]) -> i64 {
    // loopbound min 15 max 15
    let min: f32 = arr.iter()
                      .copied()
                      .filter(|x| !x.is_nan())
                      .min_by(|a, b| a.partial_cmp(b).unwrap())
                      .unwrap();
    floorf(min) as i64
}

fn audiobeam_wrapped_inc_amount(i: i32, a: i32, max_i: i32) -> i32 {
    if i + a > max_i {
        i + a - max_i - a
    } else {
        i + a
    }
}

fn audiobeam_calc_distances(
      source_location: &mut [f32; 3],
      mic_locations: &mut [[f32; 3]; 15],
      distances: &mut [f32; 15],
) {
    // loopbound min 15 max 15
    for (dist, loc) in distances.iter_mut().zip(mic_locations.iter()) {
        *dist = sqrtf((loc[0] - source_location[0]) * (loc[0] - source_location[0])
            + (loc[1] - source_location[1]) * (loc[1] - source_location[1])
            + (loc[2] - source_location[2]) * (loc[2] - source_location[2]));
    }
}

fn audiobeam_calc_delays(
    distances: &mut [f32; 15],
    delays: HeapSlice,
    sound_speed: i32,
    sampling_rate: i32,
    heap: &mut SimulatedHeap,
) {
    // loopbound min 15 max 15
    for (i, &dist) in distances.iter().enumerate() {
        let val = (dist / sound_speed as f32) * sampling_rate as f32;
        let bytes: [u8; 4] = val.to_ne_bytes();
        let start_i = delays.start + i * delays.type_size;
        let end_i = start_i + delays.type_size;
        heap.heap_content[start_i..end_i].copy_from_slice(&bytes);
    }

}

fn audiobeam_adjust_delays(delays: HeapSlice, heap: &mut SimulatedHeap) {
    // get floats from delays col
    let mut delays_f: [f32; 15] = [0.; 15];
    {
        let mut delay_val_bytes: [u8; 15 * size_of::<f32>()] = [0; 15 * size_of::<f32>()];
        delay_val_bytes.copy_from_slice(&heap.heap_content[delays.start..delays.end]);

        delay_val_bytes.chunks_exact(4)
                       .enumerate()
                       .for_each(|(i, chunk)| {
                            let arr: [u8; 4] = chunk.try_into().unwrap();
                            delays_f[i] = f32::from_ne_bytes(arr);
                       });
    }

    let min_delay: i64 = audiobeam_find_min_in_arr(&mut delays_f) - 1;
    // loopbound min 15 max 15
    let size = (delays.end - delays.start) / delays.type_size;
    for i in 0..size {
        let start_i = delays.start + i * delays.type_size;
        let end_i = start_i + delays.type_size;
        let ary: [u8; size_of::<f32>()] = heap.heap_content[start_i..end_i].try_into().unwrap();
        let f = f32::from_ne_bytes(ary);
        let new_val = f - min_delay as f32;
        let bytes = new_val.to_ne_bytes();
        heap.heap_content[start_i..end_i].copy_from_slice(&bytes);
    }
}

fn audiobeam_calc_weights_lr(num_mic: usize, heap: &mut SimulatedHeap) -> HeapSlice {
    let weights = heap.malloc(num_mic * size_of::<f32>(), size_of::<f32>());
    let half: isize = num_mic as isize / 4;
    let mut index: usize = weights.start;
    // loopbound min 0 max 0
    for _ in (0..=2).step_by(2) {
        // loopbound min 0 max 0
        for y in 0..half {
            let val = 0.54f64 + 0.46f64 * cosf(PI * (y as f32 / half as f32)) as f64;
            let ary = (val as f32).to_ne_bytes();
            heap.heap_content[index..(index + size_of::<f32>())].copy_from_slice(&ary);
            index += size_of::<f32>();
        }
        // loopbound min 0 max 0
        for y in 0..half {
            let val = 0.54f64 + 0.46f64 * cosf(PI * (-y as f32 / half as f32)) as f64;
            let ary = (val as f32).to_ne_bytes();
            heap.heap_content[index..(index + size_of::<f32>())].copy_from_slice(&ary);
            index += size_of::<f32>();
        }
    }
    weights
}

fn audiobeam_calc_weights_left_only (num_mic: usize, heap: &mut SimulatedHeap) -> HeapSlice {
    let weights = heap.malloc(num_mic * size_of::<f32>(), size_of::<f32>());
    let half = (num_mic / 2) as i32;
    let mut index: usize = 0;
    let step: usize = size_of::<f32>();

    // loopbound min 15 max 15
    for y in (-half)..=(half) {
        let val = 0.54f64 + 0.46f64 * cosf(PI * (y as f32 / half as f32)) as f64;
        let ary = (val as f32).to_ne_bytes();
        heap.heap_content[(weights.start + index)..(weights.start + index + step)].copy_from_slice(&ary);
        index += size_of::<f32>();
    }
    weights
}

fn audiobeam_calculate_energy(samples: HeapSlice, num_samples: isize, heap: &mut SimulatedHeap) -> f32 {
    let mut sum: f32 = 0.0;
    let mut i = 0;
    // loopbound min 0 max 0
    while i < num_samples {
        let start_i = samples.start + i as usize * samples.type_size;
        let end_i = start_i + samples.type_size;
        let ary = heap.heap_content[start_i..end_i].try_into().unwrap();
        let val = f32::from_ne_bytes(ary);
        sum += val * val;
        i += 1;
    }
    sum
}

fn audiobeam_do_beamforming(
    prep_delays: &[PreprocessedDelays],
    sample_queue: HeapSlice,
    queue_dim: usize,
    queue_head: usize,
    max_delay: i64,
    num_mic: usize,
    weights: Option<HeapSlice>,
    heap: &mut SimulatedHeap,
) -> f32
{
    // add up all the num_mic delayed samples
    let mut sum: f32 = 0.;
    // loopbound min 15 max 15
    for i in 0..num_mic {
        let delay_floor = prep_delays[i].low;
        let delay_ceil = prep_delays[i].high;

        // Inline wrap around here
        // Low index gets index of sample right before desired sample
        let mut low_index = queue_head + sample_queue.type_size * queue_dim * delay_floor as usize;
        // let mut low_index = queue_head + delay_floor as usize;

        if low_index > (max_delay as usize * queue_dim * sample_queue.type_size) {
            low_index -= (max_delay + 1) as usize * queue_dim * sample_queue.type_size;
        }

        // High index gets index of sample right after desired sample
        let mut high_index = queue_head + sample_queue.type_size * queue_dim * delay_ceil as usize;
        // let mut high_index = queue_head + delay_ceil as usize;
        if high_index > (max_delay as usize * queue_dim * sample_queue.type_size) {
            high_index -= (max_delay + 1) as usize * queue_dim * sample_queue.type_size;
        }

        // i gives the value of the microphone we want. However, since
        // the array only has microphones first_mic to last_mic, we
        // need to offset our index by first_mic

        let sq_high_start_i = sample_queue.start
            + high_index
            + i * sample_queue.type_size;
        let sq_high_end_i = sq_high_start_i + sample_queue.type_size;
        let sq_low_start_i = sample_queue.start
            + low_index
            + i * sample_queue.type_size;
        let sq_low_end_i = sq_low_start_i + sample_queue.type_size;

        let ary: [u8; size_of::<f32>()] = heap.heap_content[sq_high_start_i..sq_high_end_i].try_into().unwrap();
        let high_val = f32::from_ne_bytes(ary);
        let ary: [u8; size_of::<f32>()] = heap.heap_content[sq_low_start_i..sq_low_end_i].try_into().unwrap();
        let low_val = f32::from_ne_bytes(ary);

        let interpolated_value: f32 = ((high_val - low_val) * (prep_delays[i].offset)) + low_val;

        // If we have microphone weights, multiply the value by the weight
        if weights.is_some() {
            let index = weights.unwrap().start + i * weights.unwrap().type_size;
            let weight = f32::from_ne_bytes(heap.heap_content[index..(index + weights.unwrap().type_size)].try_into().unwrap());
            sum += interpolated_value * weight;
        } else {
            sum += interpolated_value;
        }
    }
    sum
}

fn audiobeam_process_signal(
    delays: HeapSlice,
    num_mic: usize,
    beamform_results: Option<HeapSlice>,
    queue: HeapSlice,
    num_beams: usize,
    window: i32,
    weights: Option<HeapSlice>,
    inp: &mut [f32; 5760],
    input_pos: &mut usize,
    heap: &mut SimulatedHeap,
) -> i32 {
    let mut done = 0;

    // get delay data from heap
    let mut delays_bytes: [u8; Delays::DELAYS_SERIALIZED_SIZE] = [0; Delays::DELAYS_SERIALIZED_SIZE];
    delays_bytes.copy_from_slice(&heap.heap_content[delays.start..delays.end]);
    let d = Delays::from_ne_bytes(delays_bytes);
    let delay_max = d.max_delay;
    // get queue indices
    let head_start_i = queue.start + size_of::<HeapSlice>() + 2 * size_of::<usize>();
    let head_end_i = head_start_i + size_of::<i32>();
    let tail_start_i = head_end_i;
    let tail_end_i = tail_start_i + size_of::<i32>();

    let mut preprocessed_delays: [PreprocessedDelays; 15] = [PreprocessedDelays::default(); 15];
    audiobeam_preprocess_delays(&mut preprocessed_delays, d.delays_col(0), heap);

    let mut queue_bytes: [u8; DataQueue::QUEUE_SERIALIZED_SIZE] = [0; DataQueue::QUEUE_SERIALIZED_SIZE];
    queue_bytes.copy_from_slice(&heap.heap_content[queue.start..queue.end]);
    let queue_h = DataQueue::from_ne_bytes(queue_bytes);
    let q_xdim = queue_h.sample_queue_dim.0;

    // loopbound min 13 max 13
    for _ in 0..(delay_max - 1) {
        let ary: [u8; size_of::<i32>()] = heap.heap_content[head_start_i..head_end_i].try_into().unwrap();
        let head = i32::from_ne_bytes(ary);
        if *input_pos < 5760 {   
            let sq_col_start_i = queue_h.sample_queue.start + head as usize;
            let sq_col_end_i = sq_col_start_i + 15 * size_of::<f32>();
            let sq_col: HeapSlice = HeapSlice {
                start: sq_col_start_i,
                end: sq_col_end_i,
                type_size: size_of::<f32>(),
            };
            audiobeam_parse_line(
                sq_col, 15, inp, input_pos, heap
            );
        } else {
            return -1;
        }
        let val: i32 = audiobeam_wrapped_inc_amount(
            head, (q_xdim * queue_h.sample_queue.type_size) as i32,
            (delay_max as usize * q_xdim * queue_h.sample_queue.type_size) as i32,
        );
        let bytes: [u8; size_of::<i32>()] = val.to_ne_bytes();
        heap.heap_content[head_start_i..head_end_i].copy_from_slice(&bytes);
    }
    
    // loopbound min 372 max 372 -> original showed 371
    let mut i: usize = 0;
    while (i as i32) < window || window < 0 {
        let ary: [u8; size_of::<i32>()] = heap.heap_content[head_start_i..head_end_i].try_into().unwrap();
        let head = i32::from_ne_bytes(ary);
        if *input_pos < 5760 {
            let sq_col_start_i = queue_h.sample_queue.start + head as usize;
            let sq_col_end_i = sq_col_start_i + 15 * size_of::<f32>();
            let sq_col: HeapSlice = HeapSlice {
                start: sq_col_start_i,
                end: sq_col_end_i,
                type_size: size_of::<f32>(),
            };
            audiobeam_parse_line(sq_col, 15, inp, input_pos, heap);
        } else {
            done = 1;
            break;
        }
        // loopbound min 1 max 1
        for j in 0..num_beams {
            let head_inc = audiobeam_wrapped_inc_amount(
                head, (q_xdim * queue_h.sample_queue.type_size) as i32,
                (delay_max as usize * q_xdim * queue_h.sample_queue.type_size) as i32,
            );

            let mut value = audiobeam_do_beamforming(
                &mut preprocessed_delays,
                queue_h.sample_queue,
                queue_h.sample_queue_dim.0,
                head_inc as usize,
                delay_max, num_mic, weights, heap);

            value = value / num_mic as f32;

            if beamform_results.is_some() {    
                let beamform_start_i = beamform_results.unwrap().start
                    + j * 0 * beamform_results.unwrap().type_size
                    + i * beamform_results.unwrap().type_size;
                let beamform_end_i = beamform_start_i + beamform_results.unwrap().type_size;
                let bytes = value.to_ne_bytes();
                heap.heap_content[beamform_start_i..beamform_end_i].copy_from_slice(&bytes);
            }
        }
        let ary: [u8; size_of::<i32>()] = head.to_ne_bytes();
        heap.heap_content[tail_start_i..tail_end_i].copy_from_slice(&ary);
        let val: i32 = audiobeam_wrapped_inc_amount(
            head, (q_xdim * queue_h.sample_queue.type_size) as i32,
            (delay_max as usize * q_xdim * queue_h.sample_queue.type_size) as i32,
        );
        let bytes: [u8; size_of::<i32>()] = val.to_ne_bytes();
        heap.heap_content[head_start_i..head_end_i].copy_from_slice(&bytes);
        i += 1;
    }
    done
}

fn audiobeam_calc_beamforming_result(
    delays: HeapSlice,
    beamform_results: Option<HeapSlice>,
    energies: HeapSlice,
    queue: HeapSlice,
    num_beams: usize,
    window: i32,
    hamming: bool,
    inp: &mut [f32; 5760],
    input_pos: &mut usize,
    heap: &mut SimulatedHeap,
) -> i32
{
    let mut weights: Option<HeapSlice> = None;

    if hamming {
        if ( 15 % 2 ) == 1 {
            weights = Some(audiobeam_calc_weights_left_only(15, heap));
        } else {
            weights = Some(audiobeam_calc_weights_lr(15, heap));
        }
    }
    
    let done: i32 = audiobeam_process_signal(delays, 15,
                                             beamform_results, queue, num_beams,
                                             window, weights, inp, input_pos, heap,
                                            );

    if beamform_results.is_some() {
        // loopbound min 1 max 1
        for i in 0..num_beams {
            let start_i = beamform_results.unwrap().start + i * beamform_results.unwrap().type_size;
            let end_i = start_i + beamform_results.unwrap().type_size;
            let result: HeapSlice = HeapSlice {
                start: start_i,
                end: end_i,
                type_size: beamform_results.unwrap().type_size,
            };
            let energy = audiobeam_calculate_energy(result, window as isize, heap);
            let bytes = energy.to_ne_bytes();
            let start_i = energies.start + i * energies.type_size;
            let end_i = start_i + energies.type_size;
            heap.heap_content[start_i..end_i].copy_from_slice(&bytes);
        }
    }
    done
}

fn audiobeam_calc_single_pos(
    hamming: bool,
    inp: &mut [f32; 5760],
    input_pos: &mut usize,
    mic_locations: &mut [[f32; 3]; 15],
    source_location: &mut [f32; 3],
    checksum: &mut i32,
    heap: &mut SimulatedHeap,
) {
    let mut mic_distances: [f32; 15] = [0.; 15];
    let delays_alloc: HeapSlice = Delays::init(1, 15, heap);

    let beamform_results_alloc: HeapSlice = heap.malloc(384 * size_of::<f32>(), size_of::<f32>());
    let energies_alloc: HeapSlice = heap.malloc(size_of::<f32>(), size_of::<f32>());

    // Calculate distances from source to each of mics
    audiobeam_calc_distances(source_location, mic_locations, &mut mic_distances);

    // Calculate HeapSlice for col 0 of delays
    let delays_col_alloc: HeapSlice;
    {
        let mut delays_bytes: [u8; Delays::DELAYS_SERIALIZED_SIZE] = [0; Delays::DELAYS_SERIALIZED_SIZE];
        delays_bytes.copy_from_slice(&heap.heap_content[delays_alloc.start..delays_alloc.end]);
        let delays = Delays::from_ne_bytes(delays_bytes);
        delays_col_alloc = delays.delays_col(0);
    }

    audiobeam_calc_delays(
        &mut mic_distances, delays_col_alloc, 342, 16000, heap
    );

    audiobeam_adjust_delays(delays_col_alloc, heap);

    // get floats from delays col
    let mut delays_col_f: [f32; 15] = [0.; 15];
    {
        let mut delay_val_bytes: [u8; 15 * size_of::<f32>()] = [0; 15 * size_of::<f32>()];
        delay_val_bytes.copy_from_slice(&heap.heap_content[delays_col_alloc.start..delays_col_alloc.end]);

        delay_val_bytes.chunks_exact(size_of::<f32>())
                       .enumerate()
                       .for_each(|(i, chunk)| {
                            let arr: [u8; 4] = chunk.try_into().unwrap();
                            delays_col_f[i] = f32::from_ne_bytes(arr);
                       });
    }

    let max_delay = audiobeam_find_max_in_arr(&mut delays_col_f);
    // set heap data
    let start_i = delays_alloc.start + size_of::<HeapSlice>() + 2 * size_of::<usize>();
    let end_i = start_i + size_of::<i64>();
    let bytes: [u8; size_of::<i64>()] = max_delay.to_ne_bytes();
    heap.heap_content[start_i..end_i].copy_from_slice(&bytes);

    let queue_alloc: HeapSlice = DataQueue::init(max_delay as usize, 15, heap);

    audiobeam_calc_beamforming_result(
        delays_alloc, Some(beamform_results_alloc),
        energies_alloc, queue_alloc,
        1, -1, hamming,
        inp, input_pos, heap,
    );

    let start_i = beamform_results_alloc.start;
    let end_i = start_i + beamform_results_alloc.type_size;
    let tmp: [u8; size_of::<f32>()]
        = heap.heap_content[start_i..end_i].try_into().unwrap();
    let val = f32::from_ne_bytes(tmp);
    *checksum += (val * 1000f32) as i32;
}

/*
  Main functions
*/

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn audiobeam_main(
    inp: &mut [f32; 5760],
    input_pos: &mut usize,
    mic_locations: &mut [[f32; 3]; 15],
    source_location: &mut [f32; 3],
    checksum: &mut i32,
    heap: &mut SimulatedHeap,
) {
    let hamming: bool = true;
    audiobeam_calc_single_pos(
        hamming, inp, input_pos,
        mic_locations, source_location,
        checksum, heap,
    );
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn audiobeam_log(
    inp: &mut [f32; 5760],
    input_pos: &mut usize,
    mic_locations: &mut [[f32; 3]; 15],
    source_location: &mut [f32; 3],
    origin_location: &mut [f32; 3],
    checksum: &mut i32,
    heap: &mut SimulatedHeap,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        audiobeam_init(
            black_box(inp),
            black_box(input_pos),
            black_box(mic_locations),
            black_box(source_location),
            black_box(origin_location),
            black_box(checksum),
            black_box(heap),
        );
        let start = black_box(DWT::cycle_count());
        audiobeam_main(
            black_box(inp),
            black_box(input_pos),
            black_box(mic_locations),
            black_box(source_location),
            black_box(checksum),
            black_box(heap),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("audiobeam - elapsed: {}", cycles);
    });

    black_box(audiobeam_return(black_box(checksum)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn audiobeam_std(
    inp: &mut [f32; 5760],
    input_pos: &mut usize,
    mic_locations: &mut [[f32; 3]; 15],
    source_location: &mut [f32; 3],
    origin_location: &mut [f32; 3],
    checksum: &mut i32,
    heap: &mut SimulatedHeap,
) -> bool {
    audiobeam_init(
        black_box(inp),
        black_box(input_pos),
        black_box(mic_locations),
        black_box(source_location),
        black_box(origin_location),
        black_box(checksum),
        black_box(heap),
    );
    audiobeam_main(
        black_box(inp),
        black_box(input_pos),
        black_box(mic_locations),
        black_box(source_location),
        black_box(checksum),
        black_box(heap),
    );
    black_box(audiobeam_return(black_box(checksum)))
}

fn audiobeam() -> bool {
    static mut INP: [f32; 5760] = START_INPUT;
    static mut INPUT_POS: usize = 0;
    static mut MIC_LOCATIONS: [[f32; 3]; 15] = INPUT_MIC_LOCATIONS;
    static mut SOURCE_LOCATION: [f32; 3] = INPUT_SOURCE_LOCATION;
    static mut ORIGIN_LOCATION: [f32; 3] = INPUT_ORIGIN_LOCATION;
    static mut CHECKSUM: i32 = 0;
    static mut HEAP: SimulatedHeap = SimulatedHeap::new();

    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let inp = unsafe { &mut *{ &raw mut INP } };
    let input_pos = unsafe { &mut *{ &raw mut INPUT_POS } };
    let mic_locations = unsafe { &mut *{ &raw mut MIC_LOCATIONS } };
    let source_location = unsafe { &mut *{ &raw mut SOURCE_LOCATION } };
    let origin_location = unsafe { &mut *{ &raw mut ORIGIN_LOCATION } };
    let checksum = unsafe { &mut *{ &raw mut CHECKSUM } };
    let heap = unsafe { &mut *{ &raw mut HEAP } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(audiobeam_log(
            black_box(inp),
            black_box(input_pos),
            black_box(mic_locations),
            black_box(source_location),
            black_box(origin_location),
            black_box(checksum),
            black_box(heap),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(audiobeam_std(
            black_box(inp),
            black_box(input_pos),
            black_box(mic_locations),
            black_box(source_location),
            black_box(origin_location),
            black_box(checksum),
            black_box(heap),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(audiobeam());
    
    #[cfg(feature = "log")]
    {
        rprintln!("audiobeam: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(audiobeam());

    #[cfg(feature = "log")]
    std::println!("audiobeam: {}", success);

    std::process::exit(!success as i32);
}
