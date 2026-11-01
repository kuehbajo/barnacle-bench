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

  Name: cjpeg_transupp

  Author: Thomas G. Lane

  Function: This file contains image transformation routines and other utility
    code used by the jpegtran sample application. These are NOT part of the core
    JPEG library. But we keep these routines separate from jpegtran.c to ease
    the task of maintaining jpegtran-like programs that have other user
    interfaces.

  Source: MediaBench II
          http://euler.slu.edu/~fritts/mediabench (mirror)

  Original name: cjpeg

  Changes: No major functional changes.

  License: See the accompanying README file.

*/

#![cfg_attr(target_arch = "arm", no_std, no_main)]

#[cfg(all(target_arch = "arm", feature = "log"))]
use rtt_target::{rprintln, rtt_init, set_print_channel, ChannelMode};

#[cfg(all(target_arch = "arm", feature = "log"))]
use cortex_m::peripheral::DWT;

#[cfg(target_arch = "arm")]
use panic_halt as _;

use core::hint::black_box;

mod jpeglib;
use jpeglib::JpegCompressStruct;

use crate::jpeglib::{ DCTSIZE, DCTSIZE2 };

/*
  Initialization- and return-value-related functions
*/

struct Random(i8);

impl Random {
    fn init() -> Self {
        Self(0)
    }

    /*
        cjpeg_transupp_RandomInteger generates random integers between -128 and 127.
    */
    fn next(&mut self) -> i8 {
        self.0 = (((self.0 as i32 * 133 + 81) % 256) - 128) as i8;
        self.0
    }
}

impl Iterator for Random {
    type Item = i8;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next())
    }
}

struct CjpegTransuppInput {
    cjpeg_transupp_input: [i8; 256],
    cjpeg_transupp_input2: [i8; 80],
    cjpeg_transupp_input3: [i8; 65],
    cjpeg_transupp_input3_2: [i8; 65],
    cjpeg_transupp_input4: [i8; 64],
    cjpeg_transupp_input5: [i8; 65],
    cjpeg_transupp_input5_2: [i8; 65],
}

impl CjpegTransuppInput {
    const fn new() -> Self {
        CjpegTransuppInput {
            cjpeg_transupp_input: [0; 256],
            cjpeg_transupp_input2: [0; 80],
            cjpeg_transupp_input3: [0; 65],
            cjpeg_transupp_input3_2: [0; 65],
            cjpeg_transupp_input4: [0; 64],
            cjpeg_transupp_input5: [0; 65],
            cjpeg_transupp_input5_2: [0; 65],
        }
    }
}

struct CjpegTransuppOutput {
    cjpeg_transupp_output_data: [i8; 512],
    cjpeg_transupp_output_data2: [i8; 512],
    cjpeg_transupp_output_data3: [i8; 512],
    cjpeg_transupp_output_data4: [i8; 512],
    cjpeg_transupp_output_data5: [i8; 512],
}

impl CjpegTransuppOutput {
    const fn new() -> Self {
        CjpegTransuppOutput {
            cjpeg_transupp_output_data: [0; 512],
            cjpeg_transupp_output_data2: [0; 512],
            cjpeg_transupp_output_data3: [0; 512],
            cjpeg_transupp_output_data4: [0; 512],
            cjpeg_transupp_output_data5: [0; 512],
        }
    }
}

fn cjpeg_transupp_init(
    input: &mut CjpegTransuppInput,
    dstinfo: &mut JpegCompressStruct,
) {
    dstinfo.max_h_samp_factor = 2;
    dstinfo.max_v_samp_factor = 2;
    dstinfo.num_components = 3;

    let mut rand = Random::init();

    // loopbound min 256 max 256
    input.cjpeg_transupp_input.iter_mut().for_each(|x| *x = rand.next());

    // loopbound min 80 max 80
    input.cjpeg_transupp_input2.iter_mut().for_each(|x| *x = rand.next());

    // loopbound min 65 max 65
    input.cjpeg_transupp_input3.iter_mut().for_each(|x| *x = rand.next());

    // loopbound min 65 max 65
    input.cjpeg_transupp_input3_2.iter_mut().for_each(|x| *x = rand.next());

    // loopbound min 64 max 64
    input.cjpeg_transupp_input4.iter_mut().for_each(|x| *x = rand.next());

    // loopbound min 65 max 65
    input.cjpeg_transupp_input5.iter_mut().for_each(|x| *x = rand.next());

    // loopbound min 65 max 65
    input.cjpeg_transupp_input5_2.iter_mut().for_each(|x| *x = rand.next());
}

fn cjpeg_transupp_return(
    output: &CjpegTransuppOutput,
) -> bool {
    let mut checksum: i32 = 0;

    // loopbound min 512 max 512
    checksum += output.cjpeg_transupp_output_data.iter().map(|&x| x as i32).sum::<i32>();

    // loopbound min 512 max 512
    checksum += output.cjpeg_transupp_output_data2.iter().map(|&x| x as i32).sum::<i32>();

    // loopbound min 512 max 512
    checksum += output.cjpeg_transupp_output_data3.iter().map(|&x| x as i32).sum::<i32>();

    // loopbound min 512 max 512
    checksum += output.cjpeg_transupp_output_data4.iter().map(|&x| x as i32).sum::<i32>();

    // loopbound min 512 max 512
    checksum += output.cjpeg_transupp_output_data5.iter().map(|&x| x as i32).sum::<i32>();

    checksum - 1624 == 0
}

/*
  Algorithm core functions
*/

/*
  Vertical flip
*/
fn cjpeg_transupp_do_flip_v(
    input: &CjpegTransuppInput,
    output: &mut CjpegTransuppOutput,
    dstinfo: &JpegCompressStruct,
) {
    /*
      We output into a separate array because we can't touch different rows of the
      source virtual array simultaneously. Otherwise, this is a pretty
      straightforward analog of horizontal flip.
      Within a DCT block, vertical mirroring is done by changing the signs of
      odd-numbered rows.
      Partial iMCUs at the bottom edge are copied verbatim.
    */
    let mcu_rows = dstinfo.image_height / (dstinfo.max_v_samp_factor as usize * DCTSIZE);

    let mut compptr_v_samp_factor = 8;
    let mut compptr_height_in_blocks: usize;
    let mut compptr_width_in_blocks: usize = 29;

    // loopbound min 3 max 3
    for _ in 0..dstinfo.num_components {
        let comp_height = mcu_rows * compptr_v_samp_factor;

        compptr_height_in_blocks = 10;
        // loopbound min 2 max 10
        for dst_blk_y in (0..compptr_height_in_blocks).step_by(compptr_v_samp_factor) {
            // loopbound min 1 max 8
            for _offset_y in 0..compptr_v_samp_factor {
                if dst_blk_y < comp_height {
                    /* Row is within the mirrorable area. */
                    // loopbound min 15 max 29
                    for dst_blk_x in 0..compptr_width_in_blocks {
                        let mut src_iter = input.cjpeg_transupp_input.iter();
                        let mut dst_iter = output.cjpeg_transupp_output_data.iter_mut();
                        // loopbound min 4 max 4
                        for _ in (0..DCTSIZE).step_by(2) {
                            // loopbound min 8 max 8
                            for _ in 0..DCTSIZE {
                                let s = src_iter.next().unwrap();
                                let d = dst_iter.next().unwrap();
                                /* copy even row */
                                if dst_blk_x < comp_height {
                                    *d = *s;
                                }
                            }
                            // loopbound min 8 max 8
                            for _ in 0..DCTSIZE {
                                let s = src_iter.next().unwrap();
                                let d = dst_iter.next().unwrap();
                                /* copy odd row with sign change */
                                if dst_blk_x < comp_height {
                                    *d = -*s;
                                }
                            }
                        }
                    }
              }
        }
      }
      compptr_v_samp_factor = 1;
      compptr_width_in_blocks = 15;
    }
}

#[derive(Clone, Copy)]
enum Rotation { Rot90, Rot270, }

#[inline]
fn transpose_and_mirror_block(
    src: &[i8],
    dst: &mut [i8],
    rot: Rotation,
) {
    // loopbound min 8 max 8
    for i in 0..DCTSIZE {
        // loopbound min 8 max 8
        for j in 0..DCTSIZE {
            let val = src[i * DCTSIZE + j];
            let flip = match rot {
                Rotation::Rot90  => (i & 1) == 1,
                Rotation::Rot270 => (j & 1) == 1,
            };
            dst[j * DCTSIZE + i] = if flip { -val } else { val };
        }
    }
}

fn cjpeg_transupp_do_rot(
    src: (&[i8], Option<&[i8]>),
    dst: &mut [i8],
    rot: Rotation,
    dstinfo: &JpegCompressStruct,
) {
    let mcu_lim: usize = match rot {
        /*
          Because of the horizontal mirror step, we can't process partial iMCUs at the
          (output) right edge properly. They just get transposed and not mirrored.
        */
        Rotation::Rot90 => { dstinfo.image_width / (dstinfo.max_h_samp_factor as usize * DCTSIZE) }, // MCU_cols
        /*
          Because of the horizontal mirror step, we can't process partial iMCUs at the
          (output) bottom edge properly. They just get transposed and not mirrored.
        */
        Rotation::Rot270 => { dstinfo.image_height / (dstinfo.max_v_samp_factor as usize * DCTSIZE) }, // MCU_rows
    };

    let mut compptr_h_samp_factor: usize = 2;
    let mut compptr_v_samp_factor: usize = 8;
    let mut compptr_height_in_blocks: usize = 29;
    let mut compptr_width_in_blocks: usize = 19;

    // loopbound min 3 max 3
    for _ci in 0..dstinfo.num_components {
        let comp_lim = match rot {
            Rotation::Rot90 => mcu_lim * compptr_h_samp_factor, // comp_width
            Rotation::Rot270 => mcu_lim * compptr_v_samp_factor, // comp_height
        };

        // loopbound min 4 max 15
        for dst_blk_y in (0..compptr_height_in_blocks).step_by(compptr_v_samp_factor) {
            // loopbound min 1 max 8
            for _offset_y in 0..compptr_v_samp_factor {
                // loopbound min 10 max 10
                for dst_blk_x in (0..compptr_width_in_blocks).step_by(compptr_h_samp_factor) {
                    // loopbound min 1 max 2
                    for _offset_x in 0..compptr_h_samp_factor {
                        let mirror = match rot {
                            Rotation::Rot90 =>
                                dst_blk_x < comp_lim,
                            Rotation::Rot270 =>
                                dst_blk_y < comp_lim,
                        };
                        let src: &[i8] = if !mirror && !src.1.is_none() { src.1.unwrap() } else { src.0 };
                        if mirror {
                            transpose_and_mirror_block(src, dst, rot);
                        } else {
                            // loopbound min 8 max 8
                            let mut i = 0;
                            while i < DCTSIZE {
                                i += 1;
                            }
                            // loopbound min 8 max 8
                            for j in 0..DCTSIZE {
                                if dst_blk_y < comp_lim {
                                    dst[j * DCTSIZE + i] = src[i * DCTSIZE + j];
                                }

                            }
                        }
                    }
                }
            }
        }
        compptr_h_samp_factor = 1;
        compptr_v_samp_factor = 1;
        compptr_height_in_blocks = 15;
        compptr_width_in_blocks = 10;
    }
}

/*
  90 degree rotation is equivalent to
   1. Transposing the image;
   2. Horizontal mirroring.
  These two steps are merged into a single processing routine.
*/
fn cjpeg_transupp_do_rot_90(
    input: &CjpegTransuppInput,
    output: &mut CjpegTransuppOutput,
    dstinfo: &JpegCompressStruct
) {
    cjpeg_transupp_do_rot(
        (&input.cjpeg_transupp_input2, None),
        &mut output.cjpeg_transupp_output_data2,
        Rotation::Rot90,
        dstinfo,
    );
}

/*
  270 degree rotation is equivalent to
    1. Horizontal mirroring;
    2. Transposing the image.
  These two steps are merged into a single processing routine.
*/
fn cjpeg_transupp_do_rot_270(
    input: &CjpegTransuppInput,
    output: &mut CjpegTransuppOutput,
    dstinfo: &JpegCompressStruct,
) {
    cjpeg_transupp_do_rot(
          (&input.cjpeg_transupp_input3, Some(&input.cjpeg_transupp_input3_2)),
          &mut output.cjpeg_transupp_output_data3,
          Rotation::Rot270,
          dstinfo, 
      );
}

/*
  180 degree rotation is equivalent to
    1. Vertical mirroring;
    2. Horizontal mirroring.
  These two steps are merged into a single processing routine.
*/
fn cjpeg_transupp_do_rot_180(
  src: &[i8],
  dst: &mut [i8],
  dstinfo: &mut JpegCompressStruct,
) {
    let mut compptr_h_samp_factor: usize = 2;
    let mut compptr_v_samp_factor: usize = 8;
    let mut compptr_width_in_blocks: usize = 29;
    let mut compptr_height_in_blocks: usize = 19;

    let mcu_cols: usize = dstinfo.image_width / ( dstinfo.max_h_samp_factor as usize * DCTSIZE );
    let mcu_rows: usize = dstinfo.image_height / ( dstinfo.max_v_samp_factor as usize * DCTSIZE );

    // loopbound min 3 max 3
    for _act in 0..dstinfo.num_components {
        let comp_width  = mcu_cols * compptr_h_samp_factor;
        let comp_height = mcu_rows * compptr_v_samp_factor;
        // loopbound min 3 max 10
        for dst_blk_y in (0..compptr_height_in_blocks).step_by(compptr_v_samp_factor) {
            // loopbound min 1 max 8
            for _ in 0..compptr_v_samp_factor {
                if dst_blk_y < comp_height {
                    /* Row is within the mirrorable area. */

                    /* Process the blocks that can be mirrored both ways. */
                    // loopbound min 14 max 28
                    for dst_blk_x in 0..comp_width {
                        let mut index: usize = 0;

                        // loopbound min 4 max 4
                        for i in 0..DCTSIZE {
                            /* For even row, negate every odd column. */
                            if i & 1 == 0 {
                                // loopbound min 4 max 4
                                for _ in (0..DCTSIZE).step_by(2) {
                                    if dst_blk_x < comp_height {
                                        dst[index] = src[index];
                                        index += 1;
                                        dst[index] = -src[index];
                                        index += 1;
                                    }
                                }
                            /* For odd row, negate every even column. */
                            } else {
                                // loopbound min 4 max 4
                                for _ in (0..DCTSIZE).step_by(2) {
                                    if dst_blk_x < comp_height {
                                        dst[index] = src[index];
                                        index += 1;
                                        dst[index] = -src[index];
                                        index += 1;
                                    }
                                }
                            }
                        }
                    }

                    /* Any remaining right-edge blocks are only mirrored vertically. */
                    // loopbound min 1 max 1
                    for _dst_blk_x in comp_width..compptr_width_in_blocks {
                        let mut index = 0;
                        // loopbound min 4 max 4
                        for _ in (0..DCTSIZE).step_by(2) {
                            // loopbound min 8 max 8
                            for _ in 0..DCTSIZE {
                                dst[index] = src[index];
                                index += 1;
                            }
                            // loopbound min 8 max 8
                            for _ in 0..DCTSIZE {
                                dst[index] = src[index];
                                index += 1;
                            }
                        }
                    }
                } else {
                    /* Remaining rows are just mirrored horizontally. */
                    let mut dst_blk_x = 0;
                    let mut it: usize = 0;
                    /* Process the blocks that can be mirrored. */
                    // loopbound min 14 max 14
                    loop {
                        let mut index: usize = 0;
                        // loopbound min 32 max 32
                        while it < DCTSIZE2 {
                            dst[index] = src[index];
                            index += 1;
                            dst[index] = -src[index];
                            index += 1;
                            it += 2;
                        }
                        dst_blk_x += 1;
                        if dst_blk_x >= comp_width { break; }
                    }

                    /* Any remaining right-edge blocks are only copied. */
                    // loopbound min 1 max 1
                    while dst_blk_x < compptr_width_in_blocks {
                        let mut index = 0;
                        // loopbound min 1 max 1
                        loop {
                            dst[index] = src[index];
                            index += 1;
                            it += 1;
                            if it >= DCTSIZE2 { break; }
                        }
                        dst_blk_x += 1;
                    }
                }
            }
        }
        compptr_h_samp_factor = 1;
        compptr_v_samp_factor = 1;
        compptr_width_in_blocks = 15;
        compptr_height_in_blocks = 10;
    }
}


/*
  Transverse transpose is equivalent to
    1. 180 degree rotation;
    2. Transposition;
  or
    1. Horizontal mirroring;
    2. Transposition;
    3. Horizontal mirroring.
  These steps are merged into a single processing routine.
*/
fn cjpeg_transupp_do_transverse(
    src1: &[i8], src2: &[i8],
    dst: &mut [i8],
    dstinfo: &JpegCompressStruct,
) {
    let mut compptr_h_samp_factor: usize = 2;
    let mut compptr_v_samp_factor: usize = 8;
    let mut compptr_height_in_blocks: usize = 29;
    let mut compptr_width_in_blocks: usize = 19;

    let mcu_cols = dstinfo.image_width / ( dstinfo.max_h_samp_factor as usize * DCTSIZE );
    let mcu_rows = dstinfo.image_height / ( dstinfo.max_v_samp_factor as usize * DCTSIZE );

  // loopbound min 3 max 3
  for _ in 0..dstinfo.num_components {
    let comp_width  = mcu_cols * compptr_h_samp_factor;
    let comp_height = mcu_rows * compptr_v_samp_factor;

    // loopbound min 4 max 15
    for dst_blk_y in (0..compptr_height_in_blocks).step_by(compptr_v_samp_factor) {
      let mut offset_y = 0;
      // loopbound min 1 max 8
      loop {
        let mut dst_blk_x = 0;
        // loopbound min 5 max 10
        loop {
          // loopbound min 1 max 2
          for _ in 0..compptr_h_samp_factor {
            if dst_blk_y < comp_height {

              if dst_blk_x < comp_width {
                /* Block is within the mirrorable area. */

                // loopbound min 4 max 4
                let mut i = 0;
                while i < DCTSIZE {
                  let mut j = 0;
                  // loopbound min 4 max 4
                  while j < DCTSIZE {
                    if dst_blk_y < comp_width {
                        dst[ j * DCTSIZE + i ] = src1[ i * DCTSIZE + j ];
                    }
                    j += 1;
                    dst[ j * DCTSIZE + i ] = -src1[ i * DCTSIZE + j ];
                    j += 1;
                  }
                  i += 1;
                  // loopbound min 4 max 4
                  j = 0;
                  while j < DCTSIZE {
                    if dst_blk_y < comp_width {
                      dst[ j * DCTSIZE + i ] = -src1[ i * DCTSIZE + j ]; 
                    }
                    j += 1;
                    dst[ j * DCTSIZE + i ] = src1[ i * DCTSIZE + j ];
                    j += 1;
                  }
                  i += 1;
                }
              } else {
                /* Right-edge blocks are mirrored in y only */
                // loopbound min 8 max 8
                for i in 0..DCTSIZE {
                  let mut j = 0;
                  // loopbound min 4 max 4
                  while j < DCTSIZE {
                    if dst_blk_y < comp_width {
                      dst[ j * DCTSIZE + i ] = src1[ i * DCTSIZE + j ];
                    }
                    j += 1;
                    dst[ j * DCTSIZE + i ] = -src1[ i * DCTSIZE + j ];
                    j += 1;
                  }
                }
              }
            } else {
              if dst_blk_x < comp_width {
                /* Bottom-edge blocks are mirrored in x only */

                // loopbound min 4 max 4
                let mut i = 0;
                while i < DCTSIZE {
                  // loopbound min 8 max 8
                  for j in 0..DCTSIZE {
                    dst[ j * DCTSIZE + i ] = src2[ i * DCTSIZE + j ];
                  }
                  i += 1;
                  // loopbound min 8 max 8
                  for j in 0..DCTSIZE {
                    dst[ j * DCTSIZE + i ] = -src2[ i * DCTSIZE + j ];
                  }
                  i += 1;
                }
              } else {
                /* At lower right corner, just transpose, no mirroring */
                // loopbound min 8 max 8
                for i in 0..DCTSIZE {
                  // loopbound min 8 max 8
                  for j in 0..DCTSIZE {
                    dst[ j * DCTSIZE + i ] = src2[ i * DCTSIZE + j ];
                  }
                }
              }
            }
            dst_blk_x += compptr_h_samp_factor;
          }
          if dst_blk_x >= compptr_width_in_blocks { break; }
        }
        offset_y += 1;
        if offset_y >= compptr_v_samp_factor { break; }
      }
    }
    compptr_h_samp_factor = 1;
    compptr_v_samp_factor = 1;
    compptr_height_in_blocks = 15;
    compptr_width_in_blocks  = 10;
  }
    
}

/*
  Main functions
*/

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn cjpeg_transupp_main(
    input: &CjpegTransuppInput,
    output: &mut CjpegTransuppOutput,
    dstinfo: &mut JpegCompressStruct,
) {
    dstinfo.image_width = 227;
    dstinfo.image_height = 149;

    cjpeg_transupp_do_flip_v(
        input, output, dstinfo,
    );

    dstinfo.image_width = 149;
    dstinfo.image_height = 227;

    cjpeg_transupp_do_rot_90(
        input, output, dstinfo,
    );
    cjpeg_transupp_do_rot_270(
        input, output, dstinfo,
    );

    dstinfo.image_width = 227;
    dstinfo.image_height = 149;

    cjpeg_transupp_do_rot_180(
        &input.cjpeg_transupp_input4, &mut output.cjpeg_transupp_output_data4, dstinfo,
    );

    dstinfo.image_width = 149;
    dstinfo.image_height = 227;

    cjpeg_transupp_do_transverse(
      &input.cjpeg_transupp_input5, &input.cjpeg_transupp_input5_2,
      &mut output.cjpeg_transupp_output_data5, dstinfo,
    );
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn cjpeg_transupp_log(
    input: &mut CjpegTransuppInput,
    output: &mut CjpegTransuppOutput,
    dstinfo: &mut JpegCompressStruct,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        cjpeg_transupp_init(
            black_box(input), black_box(dstinfo),
        );
        let start = black_box(DWT::cycle_count());
        cjpeg_transupp_main(
            black_box(input), black_box(output), black_box(dstinfo),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("cjpeg_transupp - elapsed: {}", cycles);
    });

    black_box(cjpeg_transupp_return(black_box(output)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn cjpeg_transupp_std(
    input: &mut CjpegTransuppInput,
    output: &mut CjpegTransuppOutput,
    dstinfo: &mut JpegCompressStruct,
) -> bool {
    cjpeg_transupp_init(
        black_box(input), black_box(dstinfo),
    );
    cjpeg_transupp_main(
        black_box(input), black_box(output), black_box(dstinfo),
    );
    black_box(cjpeg_transupp_return(black_box(output)))
}

fn cjpeg_transupp() -> bool {
    static mut INPUT: CjpegTransuppInput = CjpegTransuppInput::new();
    static mut OUTPUT: CjpegTransuppOutput = CjpegTransuppOutput::new();
    static mut DSTINFO: JpegCompressStruct = JpegCompressStruct::new();
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let input = unsafe { &mut *{ &raw mut INPUT } };
    /* Output arrays replace writing of results into a file. */
    let output = unsafe { &mut *{ &raw mut OUTPUT } };
    let dstinfo = unsafe { &mut *{ &raw mut DSTINFO } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(cjpeg_transupp_log(
            black_box(input), black_box(output), black_box(dstinfo),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(cjpeg_transupp_std(
            black_box(input), black_box(output), black_box(dstinfo),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(cjpeg_transupp());
    
    #[cfg(feature = "log")]
    {
        rprintln!("cjpeg_transupp: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(cjpeg_transupp());

    #[cfg(feature = "log")]
    std::println!("cjpeg_transupp: {}", success);

    std::process::exit(!success as i32);
}
