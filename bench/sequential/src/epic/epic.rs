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

  Name: epic

  Author: Designed by Eero P. Simoncelli and Edward H. Adelson
          Written by Eero P. Simoncelli
          Developed at the Vision Science Group, The Media Laboratory
          Copyright 1989, Massachusetts Institute of Technology
          All rights reserved.

  Function: Efficient Pyramid Image Coder

  Source: MediaBench


  Original name: epic

  Changes: no major functional changes

  License:
    Permission to use, copy, or modify this software and its documentation
    for educational and research purposes only and without fee is hereby
    granted, provided that this copyright notice appear on all copies and
    supporting documentation.  For any other uses of this software, in
    original or modified form, including but not limited to distribution
    in whole or in part, specific prior permission must be obtained from
    M.I.T. and the authors.  These programs shall not be used, rewritten,
    or adapted as the basis of a commercial software or hardware product
    without first obtaining appropriate licenses from M.I.T.  M.I.T. makes
    no representations about the suitability of this software for any
    purpose.  It is provided "as is" without express or implied warranty.
*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

mod input;
use input::EPIC_IMAGE;

/*
##################
    epic.h
##################
*/

/* ============= FUNDAMENTAL LIMITATIONS ============= */

/* Maximum x- or y-size of image */
const _MAX_IMAGE_DIM: i32 = 16384;

/* Maximum number of pyramid levels (value 3*levs+1 stored in 5 bits).
   This doesn't need to be larger than log2(MAX_IMAGE_DIM/FILTER_SIZE). */
const _MAX_LEVELS: i32 = 10;

/* Maximum number of quantization bins.  This essentially determines
   the maximum depth image to be represented. */
const _MAX_BINS: i32 = 511;

/* ============= SECONDARY (derived) LIMITATIONS ============= */

/* This number determines the precision of the stored binsizes:
   stored coefficients are accurate to +/- (1/SCALE_FACTOR).
   On the other hand, this number also will limit the maximum amount
   of compression.
   It should not be more than [ 2^(8*sizeof(BinValueType)) ]/256. */
const SCALE_FACTOR: i32 = 128;

/* This number must be consistent with the filters that are
   hardwired into epic.c */
const FILTER_SIZE: usize = 15;

/* Log (base 2) of MAX_IMAGE_DIM^2: (bits required to store the dimensions) */
const _LOG_MAX_IMAGE_SIZE: i32 = 32;

/* The type of the quantized images. Must be SIGNED, and capable of holding
   values  in the range [ -MAX_BINS, MAX_BINS ] */
type _BinIndexType = i16;

/* The type used to represent the binsizes. Should be UNSIGNED. If this is
   changed, be sure to change the places in epic.c and unepic.c where
   binsizes are written or read from files.  */
type _BinValueType = u16;

/* Number of possible values for a symbol.  This must be at least
   (MAX_BINS * 4)  (one sign bit, one tag bit)... */
const _NUM_SYMBOL_VALUES: i32 = 65536;

/*
##################
    epic.c
##################
*/

const X_SIZE: usize = 64;
const Y_SIZE: usize = 64;
const FILTER: u8 = 0;
const EXPAND: u8 = 1;
const NUM_LEVELS: usize = 4;

// #define IS    ==
// #define ISNT  !=
// #define AND &&
// #define OR ||

const EPIC_HI_FILTER: [f32; FILTER_SIZE] = [
    0.0012475221, -0.0024950907, -0.0087309530, 0.0199579580,
    0.0505290000, -0.1205509700, -0.2930455800,
    0.7061761600,
    -0.2930455800, -0.1205509700, 0.0505290000,
    0.0199579580, -0.0087309530, -0.0024950907, 0.0012475221
];

const EPIC_LO_FILTER: [f32; FILTER_SIZE] = [
    -0.0012475221, -0.0024950907, 0.0087309530, 0.0199579580,
    -0.0505290000, -0.1205509700, 0.2930455800,
    0.7061761600,
    0.2930455800, -0.1205509700, -0.0505290000,
    0.0199579580, 0.0087309530, -0.0024950907, -0.0012475221
];

/*
  Initialization function
*/

fn epic_init(
    image: &mut [f32; 4096],
    hi_filter: &mut [f32; FILTER_SIZE],
    lo_filter: &mut [f32; FILTER_SIZE],
) {
    *image = EPIC_IMAGE;
    // loopbound min 4096 max 4096
    for elem in image.iter_mut() {
        *elem *= SCALE_FACTOR as f32;
    }
    *hi_filter = EPIC_HI_FILTER;
    *lo_filter = EPIC_LO_FILTER;
}

/*
  Algorithm core functions
*/

/*
  ======================================================================
  epic_build_pyr() -- builds a separable QMF-style pyramid.  The pyramid
  is written over the original image.  NOTE: the image size must be
  divisible by 2^num_levels, but we do not check this here.
  ======================================================================
*/
fn epic_build_pyr(
    image: &mut [f32; 4096],
    x_size: usize,
    y_size: usize,
    num_levels: usize,
    hi_filter: &mut [f32],
    lo_filter: &mut [f32],
    filter_size: usize,
    filtertemp: &mut [f32; FILTER_SIZE],
    hi_imagetemp: &mut [f32; X_SIZE * Y_SIZE / 2],
    lo_imagetemp: &mut [f32; X_SIZE * Y_SIZE / 2]
) {
    let mut x_level = x_size;
    let mut y_level = y_size;
    // loopbound min 4 max 4
    for _ in 0..num_levels {
        epic_build_level(
            image, x_level, y_level, 
            lo_filter, hi_filter, filter_size,
            filtertemp, hi_imagetemp, lo_imagetemp,

        );
        x_level /= 2;
        y_level /= 2;
    }
}

/*
  ======================================================================
  epic_build_level() -- builds a level of the pyramid by computing 4
  filtered and subsampled images.  Since the convolution is separable,
  image and result-block can point to the same place!  Image order is
  lowpass, horizontal, vertical (transposed), and diagonal.
  ======================================================================
*/
fn epic_build_level(
    image: &mut [f32; 4096],
    level_x_size: usize,
    level_y_size: usize,
    lo_filter: &mut [f32],
    hi_filter: &mut [f32],
    filter_size: usize,
    filtertemp: &mut [f32; FILTER_SIZE],
    hi_imagetemp: &mut [f32; X_SIZE * Y_SIZE / 2],
    lo_imagetemp: &mut [f32; X_SIZE * Y_SIZE / 2],
) {
    let total_size = level_x_size * level_y_size;

    /* filter and subsample in the X direction */
    epic_internal_filter(
        image, level_x_size as isize, level_y_size as isize,
        lo_filter, filtertemp, filter_size as isize, 1,
        0, 2, 0, 1, lo_imagetemp,
    );

    epic_internal_filter(
        image, level_x_size as isize, level_y_size as isize,
        hi_filter, filtertemp, filter_size as isize, 1,
        1, 2, 0, 1, hi_imagetemp,
    );

    let level_x_size = level_x_size / 2;
    /* now filter and subsample in the Y direction */
    let result_block = image;
    let step = total_size / 4;
    let mut result_start = 0usize;
    let result_slice = &mut result_block[result_start..];

    epic_internal_filter(
        lo_imagetemp, level_x_size as isize,
        level_y_size as isize, /* lowpass */
        lo_filter, filtertemp, 1, filter_size as isize,
        0, 1, 0, 2, result_slice,
    );

    result_start += step;
    let result_slice = &mut result_block[result_start..];
    epic_internal_filter(
        lo_imagetemp, level_x_size as isize,
        level_y_size as isize, /* horizontal */
        hi_filter, filtertemp, 1, filter_size as isize,
        0, 1, 1, 2, result_slice,
    );

    result_start += step;
    let result_slice = &mut result_block[result_start..];
    epic_internal_filter(
        hi_imagetemp, level_x_size as isize,
        level_y_size as isize, /* vertical */
        lo_filter, filtertemp, 1, filter_size as isize,
        0, 1, 0, 2, result_slice,
    );
    /* transpose the vertical band for more efficient scanning */
    epic_internal_transpose(
        result_slice, level_y_size / 2, level_x_size
    );

    result_start += step;
    let result_slice = &mut result_block[result_start..];
    epic_internal_filter(
        hi_imagetemp, level_x_size as isize,
        level_y_size as isize, /* diagonal */
        hi_filter, filtertemp, 1, filter_size as isize,
        0, 1, 1, 2, result_slice,
    );
}

/*
  ======================================================================
  In-place matrix tranpose algorithm.  Handles non-square matrices,
  too!  Is there a faster algorithm??
  ======================================================================
*/

fn epic_internal_transpose(
    mat: &mut [f32],
    rows: usize,
    cols: usize,
) {
    let modulus: usize = rows * cols - 1;

    /* loop, ignoring first and last elements */
    // loopbound min 14 max 2399
    for current_pos in 1..modulus {
        /* Compute swap position */
        let mut swap_pos = current_pos;

        // loopbound min 1 max 2
        loop {
            swap_pos = (swap_pos * cols) % modulus;
            if swap_pos >= current_pos { break; }
        }

        if current_pos != swap_pos {
            mat.swap(swap_pos, current_pos);
        }
    }
}

#[inline(always)]
fn epic_convolution(
    image: &[f32],
    kernel: &[f32],
    x_dim: isize,
    x_fdim: isize,
    filt_size: isize,
    im_base: isize,
    y_start: isize,
) -> f32
{
    let mut sum: f32 = 0.0;
    let mut x_filt: usize = 0;
    let mut y_im_lin = y_start as isize;

    let mut y_filt_lin = x_fdim;
    // loopbound min 1 max 15
    while y_filt_lin <= filt_size {
        let mut im_pos = (im_base as isize + y_im_lin) as usize;
        // loopbound min 1 max 15
        while (x_filt as isize) < y_filt_lin {
            sum += image[im_pos] * kernel[x_filt];
            im_pos += 1;
            x_filt += 1;
        }
        y_im_lin += x_dim;
        y_filt_lin += x_fdim;
    }
    sum
}

/* --------------------------------------------------------------------
  Correlate FILT with IMAGE, subsampling according to GRID parameters,
  with values placed into result array.  TEMP is a temporary
  array the size of the filter.  EDGES is a string -- see convolve.h.
  The convolution is done in 9 sections, where the border sections use
  specially computed edge-handling filters (see edges.c). The origin
  of the filter is assumed to be (floor(x_fdim/2), floor(y_fdim/2)).
  10/6/89 - approximately optimized the choice of register vars on SPARCS.
  ------------------------------------------------------------------------ */
fn epic_internal_filter(
    image: &[f32],
    x_dim: isize,
    y_dim: isize,
    filt: &[f32],
    temp: &mut [f32],
    x_fdim: isize,
    y_fdim: isize,
    xgrid_start: isize,
    xgrid_step: isize,
    ygrid_start: isize,
    ygrid_step: isize,
    result: &mut [f32],
) {
    let filt_size = x_fdim * y_fdim;
    let last_ctr_col = x_dim - x_fdim;
    let last_ctr_row = (y_dim - y_fdim) * x_dim;
    let x_fmid = x_fdim / 2;
    let y_fmid = y_fdim / 2;
    let x_stop = x_fdim - x_fmid + 1;
    let y_stop = y_fdim - y_fmid + 1;
    let ygrid_step_full: usize = (ygrid_step * x_dim) as usize;
    let x_res_dim = (x_dim - xgrid_start + xgrid_step - 1) / xgrid_step;

    let mut rt_edge_res_pos = x_res_dim;
    let mut res_pos = 0;
    let mut first_col = xgrid_start - x_fmid + xgrid_step;
    let mut x_pos;
    let mut y_pos = ygrid_start - y_fmid - 1;

    // loopbound min 1 max 4
    while y_pos < 0 {
        x_pos = xgrid_start - x_fmid;
        // loopbound min 1 max 4
        while x_pos < 0 { /* top-left corner */
            epic_reflect1( filt, x_fdim, y_fdim, x_pos, y_pos, temp, FILTER );
            let sum = epic_convolution(
                image, temp, x_dim, x_fdim, filt_size, 0, 0
            );
            result[res_pos] = sum;
            res_pos += 1;
            x_pos += xgrid_step;
        }
        first_col = x_pos + 1;
        epic_reflect1( filt, x_fdim, y_fdim, 0, y_pos, temp, FILTER );
        // loopbound min 41 max 46
        x_pos = first_col;
        while x_pos < last_ctr_col { /* top edge */
            let sum = epic_convolution(
                image, temp, x_dim, x_fdim, filt_size, x_pos, 0
            );
            result[res_pos] = sum;
            res_pos += 1;
            x_pos += xgrid_step;
        }
        rt_edge_res_pos = res_pos as isize + x_res_dim;   /* save this for later ... */
        // loopbound min 1 max 4
        x_pos += 1 - last_ctr_col;
        while x_pos < x_stop { /* top-right corner */
            epic_reflect1( filt, x_fdim, y_fdim, x_pos, y_pos, temp, FILTER );
            let sum = epic_convolution(
                image, temp, x_dim, x_fdim, filt_size, last_ctr_col, 0
            );
            result[res_pos] = sum;
            res_pos += 1;
            x_pos += xgrid_step;
        }
        y_pos += ygrid_step;
    }                        /* end top */

    let first_row = x_dim * (y_pos + 1); /* need this to go down the sides */
    let mut prev_res_pos = res_pos;
    // loopbound min 1 max 4
    x_pos = xgrid_start - x_fmid;
    while x_pos < 1 { /* left edge */
        res_pos = prev_res_pos;
        epic_reflect1( filt, x_fdim, y_fdim, x_pos, 0, temp, FILTER );
        // loopbound min 41 max 97
        y_pos = first_row;
        while y_pos < last_ctr_row {
            let sum = epic_convolution(
                image, temp, x_dim, x_fdim, filt_size, 0, y_pos
            );
            result[res_pos] = sum;
            res_pos += x_res_dim as usize;
            y_pos += ygrid_step_full as isize;
        }
        prev_res_pos += 1;
        x_pos += xgrid_step;
    }
    epic_reflect1( filt, x_fdim, y_fdim, 0, 0, temp, FILTER );
    // loopbound min 41 max 97
    y_pos = first_row;
    while y_pos < last_ctr_row { /* center region of image */
        res_pos = prev_res_pos;
        // loopbound min 41 max 46
        x_pos = first_col;
        while x_pos < last_ctr_col {
            let sum = epic_convolution(
                image, temp, x_dim, x_fdim, filt_size, x_pos, y_pos
            );
            result[res_pos] = sum;
            res_pos += 1;
            x_pos += xgrid_step;
        }
        prev_res_pos += x_res_dim as usize;
        y_pos += ygrid_step_full as isize;
    }
    
    prev_res_pos = rt_edge_res_pos as usize;
    // loopbound min 1 max 4
    x_pos += 1 - last_ctr_col;
    while x_pos < x_stop { /* right edge */
        res_pos = prev_res_pos;
        epic_reflect1( filt, x_fdim, y_fdim, x_pos, 0, temp, FILTER );
        // loopbound min 41 max 97
        y_pos = first_row;
        while y_pos < last_ctr_row {
            let sum = epic_convolution(
                image, temp, x_dim, x_fdim, filt_size, last_ctr_col, y_pos
            );
            result[ res_pos ] = sum;
            res_pos += x_res_dim as usize;
            y_pos += ygrid_step_full as isize;
        }
        prev_res_pos += 1;
        x_pos += xgrid_step;
    }       /* end mid */
    res_pos -= (x_res_dim - 1) as usize;          /* go to lower left corner */
    // loopbound min 1 max 4
    y_pos = ((y_pos - last_ctr_row) / x_dim) + 1;
    while y_pos < y_stop { /* bottom */
        // loopbound min 1 max 4
        x_pos = xgrid_start - x_fmid;
        while x_pos < 1 { /* bottom-left corner */
            epic_reflect1( filt, x_fdim, y_fdim, x_pos, y_pos, temp, FILTER );
            let sum = epic_convolution(
                image, temp, x_dim, x_fdim, filt_size, 0, last_ctr_row
            );
            result[res_pos] = sum;
            res_pos += 1;
            x_pos += xgrid_step;
        }
        epic_reflect1( filt, x_fdim, y_fdim, 0, y_pos, temp, FILTER );
        // loopbound min 41 max 46
        x_pos = first_col;
        while x_pos < last_ctr_col { /* bottom edge */
            let sum = epic_convolution(
                image, temp, x_dim, x_fdim, filt_size, x_pos, last_ctr_row
            );
            result[res_pos] = sum;
            res_pos += 1;
            x_pos += xgrid_step
        }
        // loopbound min 1 max 4
        x_pos += 1 - last_ctr_col;
        while x_pos < x_stop { /* bottom-right corner */
            epic_reflect1( filt, x_fdim, y_fdim, x_pos, y_pos, temp, FILTER );
            let sum = epic_convolution(
                image, temp, x_dim, x_fdim, filt_size, last_ctr_col, last_ctr_row
            );
            result[res_pos] = sum;
            res_pos += 1;
            x_pos += xgrid_step
        }
        y_pos += ygrid_step;
    }       /* end bottom */
}     /* end of epic_internal_filter */


/*
  The following function determine how edges are to be handled
  when performing convolutions of images with linear filters.
  Any edge handling function which is local and linear may be defined,
  except (unfortunately) constants cannot be added.  So to treat the
  edges as if the image is surrounded by a gray field, you must paste it
  into a gray image, convolve, and crop it out...
  The main convolution function is called epic_internal_filter.  The idea
  is that the convolution function calls the edge handling function which
  computes a new filter based on the old filter and the distance to the
  edge of the image.  For example, reflection is done by reflecting the
  filter through the appropriate axis and summing.
*/

/*
  ---------------- EDGE HANDLER ARGUMENTS ------------------------
  filt - floating point array of filter taps.
  x_dim, y_dim - x and y dimensions of filt.
  x_pos - position of filter relative to the horizontal image edges. Negative
       values indicate left edge, positive indicate right edge.  Zero
       indicates that the filter is not touching either edge.  An absolute
       value of 1 indicates that the edge tap of the filter is over the
       edge pixel of the image.
  y_pos - analogous to x_pos.
  result - floating point array where the resulting filter will go.  The edge
       of this filter will be aligned with the image for application...
  f_or_e - equal to one of the two constants EXPAND or FILTER.
  --------------------------------------------------------------------
*/

/* --------------------------------------------------------------------
  epic_reflect1() - Reflection through the edge pixels.  This is the right
  thing to do if you are subsampling by 2, since it maintains parity (even
  pixels positions remain even, odd ones remain odd). (note: procedure differs
  depending on f_or_e parameter).  */
fn epic_reflect1(
    filt: &[f32],
    x_dim: isize,
    y_dim: isize,
    x_pos: isize,
    y_pos: isize,
    result: &mut [f32],
    f_or_e: u8,
) {
    let filt_sz = x_dim * y_dim;
    let mut x_start = 0;
    let mut y_start = 0;
    let mut x_stop = x_dim;
    let mut y_stop = filt_sz;

    let x_base = if x_pos > 0 { x_dim - 1 } else { 0 };
    let y_base = if y_pos > 0 { x_dim * (y_dim - 1) } else { 0 };
    let mut x_edge_dist = if x_pos > 0 {
        x_pos - x_dim as isize
    } else if x_pos < -1 {
        x_pos + 1
    } else {
        0
    };
    let mut y_edge_dist = x_dim * if y_pos > 0 {
        y_pos - y_dim as isize
    } else if y_pos < -1 {
        y_pos + 1
    } else {
        0
    };

    let mx_pos = (x_dim / 2) + 1;
    let my_pos = (y_dim / 2) + 1;

    // loopbound min 15 max 15
    result.fill(0.);

    /* if EXPAND and filter is centered on image edge, do not reflect */
    if f_or_e == EXPAND {
        if x_pos == mx_pos as isize {
            x_stop = (x_dim + 1) / 2;
        } else if x_pos == -(mx_pos as isize) {
            x_start = x_dim / 2;
            x_edge_dist = 0;
        }

        if y_pos == my_pos as isize {
            y_stop = x_dim * ( ( y_dim + 1 ) / 2 );
        } else if y_pos == -(my_pos as isize) {
            y_start = x_dim * (y_dim / 2);
            y_edge_dist = 0;
        }
    }

    let mut y_edge = y_edge_dist;
    /* reflect at boundary of image */
    // loopbound min 1 max 15
    for y_filt in (y_start..y_stop).step_by(x_dim as usize) {
        let mut x_edge = x_edge_dist;
        // loopbound min 1 max 15
        for x_filt in (y_filt + x_start)..(y_filt + x_stop) {
            let index  = (y_base as isize - y_edge.abs()).abs() + (x_base as isize - x_edge.abs()).abs();
            /* The original has out-of-bounds access -> index can be 18; array of size 15 */
            if index < 15 { result[index as usize] += filt[x_filt as usize]; }
            x_edge += 1;
        }
        y_edge += x_dim as isize;
    }

    /* if EXPAND and filter is not centered on image edge, mult edge by 2 */
    if f_or_e == EXPAND {
        if x_pos.abs() != mx_pos && x_pos != 0 {
            // loopbound min 0 max 0
            for y_filt in (x_base..filt_sz).step_by(x_dim as usize) {
                result[y_filt as usize] += result[y_filt as usize];
            }
        }
        if y_pos.abs() != my_pos && y_pos != 0 {
            // loopbound min 0 max 0
            for x_filt in y_base..(y_base + x_dim) {
                result[x_filt as usize] += result[x_filt as usize];
            }
        }
    }

}

/*
  Main functions
*/

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn epic_main(
    image: &mut [f32; 4096],
    hi_filter: &mut [f32; FILTER_SIZE],
    lo_filter: &mut [f32; FILTER_SIZE],
    filtertemp: &mut [f32; FILTER_SIZE],
    hi_imagetemp: &mut [f32; X_SIZE * Y_SIZE / 2],
    lo_imagetemp: &mut [f32; X_SIZE * Y_SIZE / 2],
) {
    epic_build_pyr(
        image, X_SIZE, Y_SIZE, NUM_LEVELS,
        hi_filter, lo_filter, FILTER_SIZE,
        filtertemp, hi_imagetemp, lo_imagetemp
    );
}

fn epic_return(image: &mut [f32; 4096]) -> bool {
    let mut checksum: i32 = 0;
    // loopbound min 64 max 64
    for &elem in image.iter().step_by(Y_SIZE + 1) {
        checksum = (checksum as f32 + elem) as i32;
    }
    checksum == 43968
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn epic_log(
    image: &mut [f32; 4096],
    hi_filter: &mut [f32; FILTER_SIZE],
    lo_filter: &mut [f32; FILTER_SIZE],
    filtertemp: &mut [f32; FILTER_SIZE],
    hi_imagetemp: &mut [f32; X_SIZE * Y_SIZE / 2],
    lo_imagetemp: &mut [f32; X_SIZE * Y_SIZE / 2],
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        epic_init(
            black_box(image), black_box(hi_filter), black_box(lo_filter)
        );
        let start = black_box(DWT::cycle_count());
        epic_main(
            black_box(image), black_box(hi_filter),
            black_box(lo_filter), black_box(filtertemp),
            black_box(hi_imagetemp), black_box(lo_imagetemp),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("epic - elapsed: {}", cycles);
    });

    black_box(epic_return(black_box(image)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn epic_std(
    image: &mut [f32; 4096],
    hi_filter: &mut [f32; FILTER_SIZE],
    lo_filter: &mut [f32; FILTER_SIZE],
    filtertemp: &mut [f32; FILTER_SIZE],
    hi_imagetemp: &mut [f32; X_SIZE * Y_SIZE / 2],
    lo_imagetemp: &mut [f32; X_SIZE * Y_SIZE / 2],
) -> bool {
    epic_init(
        black_box(image), black_box(hi_filter), black_box(lo_filter)
    );
    epic_main(
        black_box(image), black_box(hi_filter),
        black_box(lo_filter), black_box(filtertemp),
        black_box(hi_imagetemp), black_box(lo_imagetemp),
    );
    black_box(epic_return(black_box(image)))
}

fn epic() -> bool {
    static mut IMAGE: [f32; 4096] = [0.; 4096];
    static mut HI_FILTER: [f32; FILTER_SIZE] = [0.; FILTER_SIZE];
    static mut LO_FILTER: [f32; FILTER_SIZE] = [0.; FILTER_SIZE];
    static mut FILTERTEMP: [f32; FILTER_SIZE] = [0.; FILTER_SIZE];
    static mut HI_IMAGETEMP: [f32; X_SIZE * Y_SIZE / 2] = [0.; X_SIZE * Y_SIZE / 2];
    static mut LO_IMAGETEMP: [f32; X_SIZE * Y_SIZE / 2] = [0.; X_SIZE * Y_SIZE / 2];
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let image = unsafe { &mut *{ &raw mut IMAGE } };
    let hi_filter = unsafe { &mut *{ &raw mut HI_FILTER } };
    let lo_filter = unsafe { &mut *{ &raw mut LO_FILTER } };
    let filtertemp = unsafe { &mut *{ &raw mut FILTERTEMP } };
    let hi_imagetemp = unsafe { &mut *{ &raw mut HI_IMAGETEMP } };
    let lo_imagetemp = unsafe { &mut *{ &raw mut LO_IMAGETEMP } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(epic_log(
            black_box(image),
            black_box(hi_filter),
            black_box(lo_filter),
            black_box(filtertemp),
            black_box(hi_imagetemp),
            black_box(lo_imagetemp),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(epic_std(
            black_box(image),
            black_box(hi_filter),
            black_box(lo_filter),
            black_box(filtertemp),
            black_box(hi_imagetemp),
            black_box(lo_imagetemp),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(epic());
    
    #[cfg(feature = "log")]
    {
        rprintln!("epic: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(epic());

    #[cfg(feature = "log")]
    std::println!("epic: {}", success);

    std::process::exit(!success as i32);
}
