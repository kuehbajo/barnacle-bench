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

  Name: h264_dec_ldecode_macroblock.c

  Author: Inge Lille-Langoy et al.

  Function: H.264 decoder

  Source: MediaBench II
          http://euler.slu.edu/~fritts/mediabench (mirror)

  Original name: h264_dec_ldecode_macroblock.c

  Changes: no functional changes

  License: see copyright.txt

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
use input::H264_DEC_LIST_IMG_UV;

/*
##################
    h264_dec.h
##################
*/

#[derive(PartialEq)]
enum SliceType {
    PSlice = 0,
    BSlice,
    SpSlice,
}

impl Default for SliceType {
    fn default() -> Self {
        SliceType::PSlice
    }
}

// image parameters
#[derive(Default)]
struct ImageParameters {
    direct_spatial_mv_pred_flag: i32,
    itype: SliceType,
    width_cr: i32,
    pix_c_y: i32,
    pix_c_x: i32,
    mpr: [[u16; 16]; 16],
    chroma_log2_weight_denom: u32,
    wp_round_chroma: i32,
    apply_weights: u32,
    num_blk8x8_uv: i32,
    mb_cr_size_x: i32,
    mb_cr_size_y: i32,
}

impl ImageParameters {
    const fn new() -> Self {
        ImageParameters {
            direct_spatial_mv_pred_flag: 0,
            itype: SliceType::PSlice,
            width_cr: 0,
            pix_c_y: 0,
            pix_c_x: 0,
            mpr: [[0; 16]; 16],
            chroma_log2_weight_denom: 0,
            wp_round_chroma: 0,
            apply_weights: 0,
            num_blk8x8_uv: 0,
            mb_cr_size_x: 0,
            mb_cr_size_y: 0,
        }
    }
}

/*
  Initialization- and return-value-related functions
*/

fn h264_dec_return(
    img_mpr: &mut [[i8; 7]; 7],
    picture_imguv: &mut [[[i8; 64]; 64]; 2],
) -> bool {
    (img_mpr[0][0] + picture_imguv[0][0][0]) as i16 + 128 != 0
}

fn h264_dec_init(
    mv_array: &mut [[[i8; 2]; 65]; 65],
    list_imguv: &mut [[[i16; 45]; 45]; 2],
    img_m7: &mut [[i32; 16]; 16],
    img: &mut ImageParameters,
) {
    let bitmask: i8 = 0;

    /*
        Apply volatile XOR-bitmask to entire input array.
    */
    // loopbound min 8450 max 8450
    for el in mv_array.iter_mut().flatten().flatten() {
        *el ^= bitmask;
    }

    // loopbound min 4050 max 4050
    for el in list_imguv.iter_mut().flatten().flatten() {
        *el ^= bitmask as i16;
    }

    // // loopbound min 256 max 256
    for el in img_m7.iter_mut().flatten() {
        *el ^= bitmask as i32;
    };

    img.mb_cr_size_x = 8;
    img.mb_cr_size_y = 8;
    img.num_blk8x8_uv = 2;
    img.pix_c_x = 256;
    img.pix_c_y = 256;
    img.width_cr = 352;
    img.apply_weights = 0;
    img.direct_spatial_mv_pred_flag = 1;
    img.itype = SliceType::BSlice;
    img.wp_round_chroma = 0;
    img.chroma_log2_weight_denom = 0;
}


/*
  Algorithm core functions
*/

fn h264_dec_decode_one_macroblock(
    mv_array: &mut [[[i8; 2]; 65]; 65],
    list_imguv: &mut [[[i16; 45]; 45]; 2],
    img_m7: &mut [[i32; 16]; 16],
    img_mpr: &mut [[i8; 7]; 7],
    picture_imguv: &mut [[[i8; 64]; 64]; 2],
    img: &mut ImageParameters,
  ) {
    let max_y_cr = 287;
    let curr_mb_field = false;
    let dec_picture_ref_idx = true;
    let active_pps_weighted_pred_flag = false;
    let active_pps_weighted_bipred_idc = false;
    let smb = false;
    let residue_transform_flag = false;

    let mut j1: i32;

    let f1_x = 64 / img.mb_cr_size_x;
    let f2_x = f1_x - 1;
    let f1_y = 64 / img.mb_cr_size_y;
    let f2_y = f1_y - 1;
    let f3 = f1_x * f1_y;
    let f4 = f3 >> 1;

    let mut fw_pred = 0;
    let mut bw_pred = 0;

    let mut fw_refframe;
    let mut bw_refframe;
    let mut fw_ref_idx = -1;
    let mut bw_ref_idx = -1;

    let mut mb_nr = 0;

    // loopbound min 2 max 2
    for uv in 0..2 {
        let intra_prediction = false;

        // loopbound min 1 max 1
        for _ in 0..(img.num_blk8x8_uv / 2) {
            // loopbound min 4 max 4
            for _ in 0..4 {
                let ioff = 0;
                let joff = 0;
                let j4 = img.pix_c_y + joff;
                let i4 = img.pix_c_x + ioff;

                let mv_mode = 1;
                let pred_dir = -1;

                if !intra_prediction {
                    if pred_dir != 2 {
                        // loopbound min 4 max 4
                        for jj in 0..4 {
                            let jf = (((j4 + jj) / (img.mb_cr_size_y / 4)) % 64) as usize;
                            // loopbound min 4 max 4
                            for ii in 0..4 {
                                let ifx = (((i4 + ii) / (img.mb_cr_size_x / 4)) % 64) as usize;
                                let i1 = (i4 + ii) * f1_x + mv_array[jf][ifx][0] as i32;
                                if !curr_mb_field {
                                    j1 = (j4 + jj) * f1_y + mv_array[jf][ifx][1] as i32;
                                } else if mb_nr % 2 == 0 {
                                    j1 = ((img.pix_c_y / 2) + jj + joff) * f1_y + mv_array[jf][ifx][1] as i32;
                                    mb_nr += 1;
                                } else {
                                    j1 = ((img.pix_c_y - img.mb_cr_size_y) / 2 + jj + joff) * f1_y
                                        + mv_array[jf][ifx][1] as i32;
                                    mb_nr += 1;
                                }
                                let ii0 = (i1 / f1_x).clamp(0, img.width_cr - 1) % 45;
                                let jj0 = (j1 / f1_y).clamp(0, max_y_cr) % 45;
                                let ii1 = ((i1 + f2_x) / f1_x).clamp(0, img.width_cr - 1) % 45;
                                let jj1 = ((j1 + f2_y) / f1_y).clamp(0, max_y_cr) % 45;

                                let if1 = i1 & f2_x;
                                let jf1 = j1 & f2_y;
                                let if0 = f1_x - if1;
                                let jf0 = f1_y - jf1;

                                if img.apply_weights != 0 {
                                    img_mpr[(ii + ioff) as usize][(jj + joff) as usize]
                                      = ((if0 * jf0 * list_imguv[uv][jj0 as usize][ii0 as usize] as i32
                                        + if1 * jf0 * list_imguv[uv][jj0 as usize][ii1 as usize] as i32
                                        + if0 * jf1 * list_imguv[uv][jj1 as usize][ii0 as usize] as i32
                                        + if1 * jf1 * list_imguv[uv][jj1 as usize][ii1 as usize] as i32
                                        + f4) / f3) as i8;
                                }
                }
              }
            } else {
              // loopbound min 4 max 4
              for jj in 0..4 {
                  let jf = ((j4 + jj) / (img.mb_cr_size_y / 4)) as usize;
                  // loopbound min 4 max 4
                  for ii in 0..4 {
                      let ifx = (((i4 + ii) / (img.mb_cr_size_x / 4)) % 64) as usize;
                      let mut direct_pdir = 2;
                      if mv_mode == 0 && img.direct_spatial_mv_pred_flag != 0 {
                          if dec_picture_ref_idx {
                              fw_refframe = 0;
                              fw_ref_idx = fw_refframe;
                          }
                          if dec_picture_ref_idx {
                            bw_refframe = 0;
                            bw_ref_idx = bw_refframe;
                          }

                          if !dec_picture_ref_idx {
                              direct_pdir = 0;
                          } else if !dec_picture_ref_idx {
                              direct_pdir = 1;
                          }

                          if direct_pdir == 0 || direct_pdir == 2 {
                              let i1 = (img.pix_c_x + ii + ioff) * f1_x + mv_array[jf as usize][ifx][0] as i32;

                              if !curr_mb_field {
                                j1 = (img.pix_c_y + jj + joff) * f1_y +
                                    mv_array[jf][ifx][1] as i32;
                              } else if mb_nr % 2 == 0 {
                                  j1 = (img.pix_c_y / 2 + jj + joff) *
                                      f1_y + mv_array[jf][ifx][1] as i32;
                              } else {
                                  j1 = ((img.pix_c_y - img.mb_cr_size_y)
                                        / 2 + jj + joff) * f1_y
                                      + mv_array[jf][ifx][1] as i32;
                              }
                              let ii0 = (i1 / f1_x).clamp(0, img.width_cr - 1) % 45;
                              let jj0 = (j1 / f1_y).clamp(0, max_y_cr) % 45;
                              let ii1 = ((i1 + f2_x) / f1_x).clamp(0, img.width_cr - 1) % 45;
                              let jj1 = ((j1 + f2_y) / f1_y).clamp(0, max_y_cr) % 45;

                              let if1 = i1 & f2_x;
                              let jf1 = j1 & f2_y;
                              let if0 = f1_x - if1;
                              let jf0 = f1_y - jf1;

                              fw_pred
                                  = (if0 * jf0 * list_imguv[uv][jj0 as usize][ii0 as usize] as i32
                                  + if1 * jf0 * list_imguv[uv][jj0 as usize][ii1 as usize] as i32
                                  + if0 * jf1 * list_imguv[uv][jj1 as usize][ii0 as usize] as i32
                                  + if1 * jf1 * list_imguv[uv][jj1 as usize][ii1 as usize] as i32
                                  + f4) / f3;
                            }
                        if direct_pdir == 1 || direct_pdir == 2 {
                            let i1 = (img.pix_c_x + ii + ioff) * f1_x + mv_array[jf][ifx][0] as i32;

                            if !curr_mb_field {
                                j1 = (img.pix_c_y + jj + joff) * f1_y + mv_array[jf][ifx][1] as i32;
                            } else if mb_nr % 2 == 0 {
                                j1 = (img.pix_c_y / 2 + jj + joff) * f1_y + mv_array[jf][ifx][1] as i32;
                            } else {
                                j1 = ((img.pix_c_y - img.mb_cr_size_y) / 2 + jj + joff) * f1_y 
                                    + mv_array[jf][ifx][1] as i32;
                            }
                            let ii0 = (i1 / f1_x).clamp(0, img.width_cr - 1) % 45;
                            let jj0 = (j1 / f1_y).clamp(0, max_y_cr) % 45;
                            let ii1 = ((i1 + f2_x) / f1_x).clamp(0, img.width_cr - 1) % 45;
                            let jj1 = ((j1 + f2_y) / f1_y).clamp(0, max_y_cr) % 45;

                            let if1 = i1 & f2_x;
                            let jf1 = j1 & f2_y;
                            let if0 = f1_x - if1;
                            let jf0 = f1_y - jf1;

                            bw_pred
                                = (if0 * jf0 * list_imguv[uv][jj0 as usize][ii0 as usize] as i32
                                + if1 * jf0 * list_imguv[uv][jj0 as usize][ii1 as usize] as i32
                                + if0 * jf1 * list_imguv[uv][jj1 as usize][ii0 as usize] as i32
                                + if1 * jf1 * list_imguv[uv][jj1 as usize][ii1 as usize] as i32
                                + f4) / f3;
                        }
                    } else {
                      fw_refframe = 0;
                      bw_refframe = 0;

                      fw_ref_idx = fw_refframe;
                      bw_ref_idx = bw_refframe;

                      let i1 = (img.pix_c_x + ii + ioff) * f1_x + mv_array[jf][ifx][0] as i32;
                      if !curr_mb_field {
                          j1 = (img.pix_c_y + jj + joff) * f1_y + mv_array[jf][ifx][1] as i32;
                      } else if mb_nr % 2 == 0 {
                          j1 = (img.pix_c_y / 2 + jj + joff) * f1_y + mv_array[jf][ifx][1] as i32;
                      } else {
                        j1 = ((img.pix_c_y - img.mb_cr_size_y) / 2 + jj + joff) * f1_y
                            + mv_array[jf][ifx][1] as i32;
                      }
                      let ii0 = (i1 / f1_x).clamp(0, img.width_cr - 1) % 45;
                      let jj0 = (j1 / f1_y).clamp(0, max_y_cr) % 45;
                      let ii1 = ((i1 + f2_x) / f1_x).clamp(0, img.width_cr - 1) % 45;
                      let jj1 = ((j1 + f2_y) / f1_y).clamp(0, max_y_cr) % 45;

                      let if1 = i1 & f2_x;
                      let jf1 = j1 & f2_y;
                      let if0 = f1_x - if1;
                      let jf0 = f1_y - jf1;

                      let _
                          = (if0 * jf0 * list_imguv[uv][jj0 as usize][ii0 as usize] as i32
                          + if1 * jf0 * list_imguv[uv][jj0 as usize][ii1 as usize] as i32
                          + if0 * jf1 * list_imguv[uv][jj1 as usize][ii0 as usize] as i32
                          + if1 * jf1 * list_imguv[uv][jj1 as usize][ii1 as usize] as i32
                          + f4) / f3;

                      let i1 = (img.pix_c_x + ii + ioff) * f1_x + mv_array[jf][ifx][0] as i32;

                      if !curr_mb_field {
                        j1 = (img.pix_c_y + jj + joff) * f1_y + mv_array[jf][ifx][1] as i32;
                      } else if mb_nr % 2 == 0 {
                          j1 = (img.pix_c_y / 2 + jj + joff) * f1_y + mv_array[jf][ifx][1] as i32;
                      } else {
                          j1 = ((img.pix_c_y - img.mb_cr_size_y) / 2 + jj + joff) * f1_y
                              + mv_array[jf][ifx][1] as i32;
                      }
                      let ii0 = (i1 / f1_x).clamp(0, img.width_cr - 1) % 45;
                      let jj0 = (j1 / f1_y).clamp(0, max_y_cr) % 45;
                      let ii1 = ((i1 + f2_x) / f1_x).clamp(0, img.width_cr - 1) % 45;
                      let jj1 = ((j1 + f2_y) / f1_y).clamp(0, max_y_cr) % 45;

                      let if1 = i1 & f2_x;
                      let jf1 = j1 & f2_y;
                      let if0 = f1_x - if1;
                      let jf0 = f1_y - jf1;

                      bw_pred
                          = (if0 * jf0 * list_imguv[uv][jj0 as usize][ii0 as usize] as i32
                          + if1 * jf0 * list_imguv[uv][jj0 as usize][ii1 as usize] as i32
                          + if0 * jf1 * list_imguv[uv][jj1 as usize][ii0 as usize] as i32
                          + if1 * jf1 * list_imguv[uv][jj1 as usize][ii1 as usize] as i32
                          + f4) / f3;
                  }

                  let cond1 = img.itype == SliceType::PSlice || img.itype == SliceType::SpSlice;
                  let cond2 = active_pps_weighted_pred_flag && cond1;
                  let cond3 = active_pps_weighted_bipred_idc && (img.itype == SliceType::BSlice);
                  if img.apply_weights != 0 {
                      if (cond2 || cond3) && curr_mb_field {
                          _ = fw_ref_idx >> 1;
                          _ = bw_ref_idx >> 1;
                      }

                    if img.direct_spatial_mv_pred_flag != 0 && direct_pdir == 1 {
                        let cond = (img.wp_round_chroma >> img.chroma_log2_weight_denom) < 0;
                        let val = img.wp_round_chroma >> img.chroma_log2_weight_denom;
                        img.mpr[(ii + ioff) as usize][(jj + joff) as usize] = if cond { 0 } else { val as u16 };
                    } else if img.direct_spatial_mv_pred_flag != 0 && direct_pdir == 0 {
                        let cond = ((img.wp_round_chroma >> img.chroma_log2_weight_denom)) < 0;
                        let val = (img.wp_round_chroma) >> img.chroma_log2_weight_denom;
                        img.mpr[(ii + ioff) as usize][(jj + joff) as usize]
                            = if cond { 0 } else { val as u16 };
                    } else {
                        let alpha_fw = 0;
                        let alpha_bw = 0;

                        let cond = ((alpha_fw * fw_pred + alpha_bw * bw_pred
                              + (1 << img.chroma_log2_weight_denom)) >> (img.chroma_log2_weight_denom + 1)) < 0;
                        let val = (alpha_fw * fw_pred + alpha_bw * bw_pred
                              + (1 << img.chroma_log2_weight_denom)) >> (img.chroma_log2_weight_denom + 1);
                        img.mpr[(ii + ioff) as usize][(jj + joff) as usize]
                            = if cond { 0 } else { val as u16 };
                    }
                  } else if img.direct_spatial_mv_pred_flag != 0 && direct_pdir == 1 {
                      img.mpr[(ii + ioff) as usize][(jj + joff) as usize] = bw_pred as u16;
                  } else if img.direct_spatial_mv_pred_flag != 0 && direct_pdir == 0 {
                      img.mpr[(ii + ioff) as usize][(jj + joff) as usize] = fw_pred as u16;
                  } else {
                      img.mpr[(ii + ioff) as usize][(jj + joff) as usize] = ((fw_pred + bw_pred + 1) / 2) as u16;
                      
                  }
                  }
              }
            }
          }

          if !smb {
              // loopbound min 4 max 4
              for ii in 0..4 {
                  // loopbound min 4 max 4
                  for jj in 0..4 {
                      if !residue_transform_flag {
                          picture_imguv[uv][((j4 + jj) % 64) as usize][((i4 + ii) % 54) as usize]
                              = img_m7[ii as usize][jj as usize] as i8;
                      }
                  }
              }
          }
        }
      }

    if smb {
        // loopbound min 2 max 2
        for j in 4..6 {
            let joff = (j - 4) * 4;
            let j4 = img.pix_c_y + joff;
            // loopbound min 2 max 2
            for i in 0..2 {
                let ioff = i * 4;
                let i4 = img.pix_c_x + ioff;
                // loopbound min 4 max 4
                for ii in 0..4 {
                    // loopbound min 4 max 4
                    for jj in 0..4 {
                        picture_imguv[uv][((j4 + jj) % 64) as usize][((i4 + ii) % 54) as usize]
                            = img_m7[ii as usize][jj as usize] as i8;
                    }
                }
            }
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
fn h264_dec_main(
    mv_array: &mut [[[i8; 2]; 65]; 65],
    list_imguv: &mut [[[i16; 45]; 45]; 2],
    img_m7: &mut [[i32; 16]; 16],
    img_mpr: &mut [[i8; 7]; 7],
    picture_imguv: &mut [[[i8; 64]; 64]; 2],
    img: &mut ImageParameters,
) {
    h264_dec_decode_one_macroblock(
        mv_array, list_imguv, img_m7, img_mpr, picture_imguv, img
    );
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn h264_dec_log(
    mv_array: &mut [[[i8; 2]; 65]; 65],
    list_imguv: &mut [[[i16; 45]; 45]; 2],
    img_m7: &mut [[i32; 16]; 16],
    img_mpr: &mut [[i8; 7]; 7],
    picture_imguv: &mut [[[i8; 64]; 64]; 2],
    img: &mut ImageParameters,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        h264_dec_init(
            black_box(mv_array), black_box(list_imguv),
            black_box(img_m7), black_box(img),
        );
        let start = black_box(DWT::cycle_count());
        h264_dec_main(
            black_box(mv_array), black_box(list_imguv),
            black_box(img_m7), black_box(img_mpr),
            black_box(picture_imguv), black_box(img),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("h264_dec - elapsed: {}", cycles);
    });

    black_box(h264_dec_return(
        black_box(img_mpr), black_box(picture_imguv),
    ))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn h264_dec_std(
    mv_array: &mut [[[i8; 2]; 65]; 65],
    list_imguv: &mut [[[i16; 45]; 45]; 2],
    img_m7: &mut [[i32; 16]; 16],
    img_mpr: &mut [[i8; 7]; 7],
    picture_imguv: &mut [[[i8; 64]; 64]; 2],
    img: &mut ImageParameters,
) -> bool {
    h264_dec_init(
        black_box(mv_array), black_box(list_imguv),
        black_box(img_m7), black_box(img),
    );
    h264_dec_main(
        black_box(mv_array), black_box(list_imguv),
        black_box(img_m7), black_box(img_mpr),
        black_box(picture_imguv), black_box(img),
    );
    black_box(h264_dec_return(
        black_box(img_mpr), black_box(picture_imguv),
    ))
}

fn h264_dec() -> bool {
    static mut MV_ARRAY: [[[i8; 2]; 65]; 65] = [[[0; 2]; 65]; 65];
    static mut LIST_IMGUV: [[[i16; 45]; 45]; 2] = H264_DEC_LIST_IMG_UV;
    static mut IMG_M7: [[i32; 16]; 16] = [[0; 16]; 16];
    static mut IMG_MPR: [[i8; 7]; 7] = [[0; 7]; 7];
    static mut PICTURE_IMGUV: [[[i8; 64]; 64]; 2] = [[[0; 64]; 64]; 2];
    static mut IMG: ImageParameters = ImageParameters::new();
    
    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let mv_array = unsafe { &mut *{ &raw mut MV_ARRAY } };
    let list_imguv = unsafe { &mut *{ &raw mut LIST_IMGUV } };
    let img_m7 = unsafe { &mut *{ &raw mut IMG_M7 } };
    let img_mpr = unsafe { &mut *{ &raw mut IMG_MPR } };
    let picture_imguv = unsafe { &mut *{ &raw mut PICTURE_IMGUV } };
    let img = unsafe { &mut *{ &raw mut IMG } };
    

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(h264_dec_log(
            black_box(mv_array), black_box(list_imguv),
            black_box(img_m7), black_box(img_mpr),
            black_box(picture_imguv), black_box(img),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(h264_dec_std(
            black_box(mv_array), black_box(list_imguv),
            black_box(img_m7), black_box(img_mpr),
            black_box(picture_imguv), black_box(img),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(h264_dec());
    
    #[cfg(feature = "log")]
    {
        rprintln!("h264_dec: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(h264_dec());

    #[cfg(feature = "log")]
    std::println!("h264_dec: {}", success);

    std::process::exit(!success as i32);
}
