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

  Name: cjpeg_jpeg6b_wrbmp.c

  Author: Thomas G. Lane.

  Function: This file contains routines to write output images in Microsoft "BMP"
  format (MS Windows 3.x and OS/2 1.x flavors).
  Either 8-bit colormapped or 24-bit full-color format can be written.
  No compression is supported.

  These routines may need modification for non-Unix environments or
  specialized applications.  As they stand, they assume output to
  an ordinary stdio stream.

  Source: Independent JPEG Group's software

  Changes: a brief summary of major functional changes (not formatting)

  License:  See the accompanying README file

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
use input::cjpeg_wrbmp_init_input;

mod cdjpeg;
use cdjpeg::CjpegWrbmpCdjpegProgressMgr;

mod jpeglib;
use jpeglib::{ CjpegWrbmpJpegDecompressStruct, CjpegWrbmpJpegProgressMgr };

mod jmorecfg;
use jmorecfg::cjpeg_wrbmp_getjsample;

/*
  Declaration of global variables
*/
// struct cjpeg_wrbmp_bmp_dest_struct {
//     publ: cjpeg_wrbmp_djpeg_dest_struct, /* public fields */
//     is_os2: cjpeg_wrbmp_boolean,   /* saves the OS2 format request flag */
//     whole_image: cjpeg_wrbmp_jvirt_sarray_ptr, /* needed to reverse row order */
//     data_width: CJPEG_WRBMP_JDIMENSION,  /* JSAMPLEs per row */
//     row_width: CJPEG_WRBMP_JDIMENSION,   /* physical width of one row in the BMP file */
//     pad_bytes: usize,    /* number of padding bytes needed per row */
//     cur_output_row: CJPEG_WRBMP_JDIMENSION,  /* next row# to write to virtual array */
// }

// type cjpeg_wrbmp_bmp_dest_ptr<'a> = &'a cjpeg_wrbmp_bmp_dest_struct;
// extern unsigned char cjpeg_wrbmp_colormap[ 3 ][ 256 ];
// unsigned char cjpeg_wrbmp_output_array[ 6144 ];
// unsigned char *cjpeg_wrbmp_jpeg_stream /*= cjpeg_jpeg6b_wrbmp_output_array*/;
// int cjpeg_wrbmp_checksum;

// struct cjpeg_wrbmp_jpeg_decompress_struct cjpeg_wrbmp_jpeg_dec_1;
// struct cjpeg_wrbmp_jpeg_decompress_struct cjpeg_wrbmp_jpeg_dec_2;
// struct cjpeg_wrbmp_djpeg_dest_struct cjpeg_wrbmp_djpeg_dest;
// cjpeg_wrbmp_bmp_dest_struct    cjpeg_wrbmp_bmp_dest;

// unsigned char cjpeg_wrbmp_colormap[ 3 ][ 256 ];

/*
   Initialization functions
*/
fn cjpeg_wrbmp_init(
    colormap: &mut [[u8; 256]; 3],
    jpeg_dec_1: &mut CjpegWrbmpJpegDecompressStruct,
    jpeg_dec_2: &mut CjpegWrbmpJpegDecompressStruct,
    jpeg_stream: &mut [u8; 6144],
    jpeg_stream_i: &mut usize,
    checksum: &mut i32,
) {
    cjpeg_wrbmp_init_input(colormap);

    jpeg_dec_1.progress                = CjpegWrbmpJpegProgressMgr::default();
    jpeg_dec_1.output_height           = 30;
    jpeg_dec_1.actual_number_of_colors = 256;
    jpeg_dec_1.out_color_components    = 2;

    jpeg_dec_2.progress                = CjpegWrbmpJpegProgressMgr::default();
    jpeg_dec_2.output_height           = 30;
    jpeg_dec_2.actual_number_of_colors = 256;
    jpeg_dec_2.out_color_components    = 3;

    *jpeg_stream = [0; 6144];
    *jpeg_stream_i = 0;

    *checksum = 0;
}

/*
   Calculation functions
*/
fn cjpeg_wrbmp_putc_modified(
    character: i32,
    jpeg_stream: &mut [u8; 6144],
    jpeg_stream_i: &mut usize,
    checksum: &mut i32,
) -> i32 {
    jpeg_stream[*jpeg_stream_i] = character as u8;

    *jpeg_stream_i += 1;

    *checksum += character;

    character
}

fn cjpeg_wrbmp_finish_output_bmp(
    cinfo: &mut CjpegWrbmpJpegDecompressStruct,

) {
    let mut progress: CjpegWrbmpCdjpegProgressMgr = CjpegWrbmpCdjpegProgressMgr {
        publ: Some(&mut cinfo.progress),
        completed_extra_passes: 0,
    };

    // Write the file body from our virtual array
    // loopbound min 30 max 30
    for row in (0..cinfo.output_height).rev() {
        if let Some(publ) = progress.publ.as_mut() {
            publ.pass_counter = (cinfo.output_height - row) as i64;
            publ.pass_limit = cinfo.output_height as i64;
        }
    }

    if progress.publ.is_some() {
        _ = progress.completed_extra_passes + 1;
    }
}

#[inline]
fn cjpeg_wrbmp_write_colormap_helper(
    characters: (i32, i32, i32),
    map_entry_size: i32,
    jpeg_stream: &mut [u8; 6144],
    jpeg_stream_i: &mut usize,
    checksum: &mut i32
) {
    cjpeg_wrbmp_putc_modified(
        characters.0, jpeg_stream, jpeg_stream_i, checksum,
    );
    cjpeg_wrbmp_putc_modified(
        characters.1, jpeg_stream, jpeg_stream_i, checksum
    );
    cjpeg_wrbmp_putc_modified(
        characters.2, jpeg_stream, jpeg_stream_i, checksum
    );

    if map_entry_size == 4 {
        cjpeg_wrbmp_putc_modified(0, jpeg_stream, jpeg_stream_i, checksum);
    }
}

fn cjpeg_wrbmp_write_colormap(
    cinfo: &mut CjpegWrbmpJpegDecompressStruct,
    map_colors: i32,
    map_entry_size: i32,
    c_map: i32,
    colormap: &mut [[u8; 256]; 3],
    jpeg_stream: &mut [u8; 6144],
    jpeg_stream_i: &mut usize,
    checksum: &mut i32,
) {
    let num_colors: i32 = cinfo.actual_number_of_colors;
    let mut i: i32 = 0;

    if c_map != 0 {
        if cinfo.out_color_components == 3 {
            // Normal case with RGB colormap
            // loopbound min 256 max 256
            while i < num_colors {
                let characters = (
                    cjpeg_wrbmp_getjsample(colormap[2][i as usize]),
                    cjpeg_wrbmp_getjsample(colormap[1][i as usize]),
                    cjpeg_wrbmp_getjsample(colormap[0][i as usize]),
                );
                cjpeg_wrbmp_write_colormap_helper(
                    characters, map_entry_size, jpeg_stream, jpeg_stream_i, checksum
                );
                i += 1;
            }
        } else {
            // Grayscale colormap (only happens with grayscale quantization)
            // loopbound min 256 max 256
           while i < num_colors {
                let characters = (
                    cjpeg_wrbmp_getjsample(colormap[2][i as usize]),
                    cjpeg_wrbmp_getjsample(colormap[1][i as usize]),
                    cjpeg_wrbmp_getjsample(colormap[0][i as usize]),
                );
                cjpeg_wrbmp_write_colormap_helper(
                    characters, map_entry_size, jpeg_stream, jpeg_stream_i, checksum
                );
                i += 1;
            }
        }
    } else {
        // If no colormap, must be grayscale data.  Generate a linear "map".
        // loopbound min 256 max 256
        while i < 256 {
            cjpeg_wrbmp_write_colormap_helper(
                (i, i, i), map_entry_size, jpeg_stream, jpeg_stream_i, checksum
            );
            i += 1;
        }
    }

    // Pad colormap with zeros to ensure specified number of colormap entries.
    // loopbound min 512 max 512
    while i < map_colors {
        cjpeg_wrbmp_write_colormap_helper(
            (0, 0, 0), map_entry_size, jpeg_stream, jpeg_stream_i, checksum
        );
        i += 1;
    }
}

// entrypoint
#[inline(never)]
#[unsafe(no_mangle)]
fn cjpeg_wrbmp_main(
    colormap: &mut [[u8; 256]; 3],
    jpeg_dec_1: &mut CjpegWrbmpJpegDecompressStruct,
    jpeg_dec_2: &mut CjpegWrbmpJpegDecompressStruct,
    jpeg_stream: &mut [u8; 6144],
    jpeg_stream_i: &mut usize,
    checksum: &mut i32,
) {
    cjpeg_wrbmp_finish_output_bmp(jpeg_dec_1);
    cjpeg_wrbmp_write_colormap(
        jpeg_dec_1, 768, 4, 1,
        colormap, jpeg_stream, jpeg_stream_i, checksum,
    );

    cjpeg_wrbmp_finish_output_bmp(jpeg_dec_2);
    cjpeg_wrbmp_write_colormap(
        jpeg_dec_2, 768, 4, 1,
        colormap, jpeg_stream, jpeg_stream_i, checksum
    );
}

fn cjpeg_wrbmp_return(checksum: &mut i32) -> bool {
    (*checksum - 209330) == 0
}

#[cfg(all(target_arch = "arm", feature = "log"))]
const RUNS: usize = 2;

#[cfg(all(target_arch = "arm", feature = "log"))]
fn cjpeg_wrbmp_log(
    colormap: &mut [[u8; 256]; 3],
    jpeg_dec_1: &mut CjpegWrbmpJpegDecompressStruct,
    jpeg_dec_2: &mut CjpegWrbmpJpegDecompressStruct,
    jpeg_stream: &mut [u8; 6144],
    jpeg_stream_i: &mut usize,
    checksum: &mut i32,
) -> bool {
    let mut run_cycles: [u32; RUNS] = [0; RUNS];

    let channels = rtt_init! {
        up: {
            0: {size: 1024, mode: ChannelMode::BlockIfFull,}
        }
    };
    set_print_channel(channels.up.0);

    run_cycles.iter_mut().for_each(|cycles| {
        cjpeg_wrbmp_init(
            black_box(colormap),
            black_box(jpeg_dec_1),
            black_box(jpeg_dec_2),
            black_box(jpeg_stream),
            black_box(jpeg_stream_i),
            black_box(checksum),
        );
        let start = black_box(DWT::cycle_count());
        cjpeg_wrbmp_main(
            black_box(colormap),
            black_box(jpeg_dec_1),
            black_box(jpeg_dec_2),
            black_box(jpeg_stream),
            black_box(jpeg_stream_i),
            black_box(checksum),
        );
        *cycles = black_box(DWT::cycle_count() - start);
    });

    run_cycles.iter().for_each(|cycles| {
        rprintln!("cjpeg_wrbmp - elapsed: {}", cycles);
    });

    black_box(cjpeg_wrbmp_return(black_box(checksum)))
}

#[cfg(any(unix, windows, not(feature = "log")))]
fn cjpeg_wrbmp_std(
    colormap: &mut [[u8; 256]; 3],
    jpeg_dec_1: &mut CjpegWrbmpJpegDecompressStruct,
    jpeg_dec_2: &mut CjpegWrbmpJpegDecompressStruct,
    jpeg_stream: &mut [u8; 6144],
    jpeg_stream_i: &mut usize,
    checksum: &mut i32,
) -> bool {
    cjpeg_wrbmp_init(
        black_box(colormap),
        black_box(jpeg_dec_1),
        black_box(jpeg_dec_2),
        black_box(jpeg_stream),
        black_box(jpeg_stream_i),
        black_box(checksum),
    );
    cjpeg_wrbmp_main(
        black_box(colormap),
        black_box(jpeg_dec_1),
        black_box(jpeg_dec_2),
        black_box(jpeg_stream),
        black_box(jpeg_stream_i),
        black_box(checksum),
    );
    black_box(cjpeg_wrbmp_return(black_box(checksum)))
}

fn cjpeg_wrbmp() -> bool {
    static mut COLORMAP: [[u8; 256]; 3] = [[0; 256]; 3];
    static mut JPEG_DEC_1: CjpegWrbmpJpegDecompressStruct = CjpegWrbmpJpegDecompressStruct::new();
    static mut JPEG_DEC_2: CjpegWrbmpJpegDecompressStruct = CjpegWrbmpJpegDecompressStruct::new();
    static mut JPEG_STREAM: [u8; 6144] = [0; 6144];
    static mut JPEG_STREAM_I: usize = 0;
    static mut CHECKSUM: i32 = 0;

    // SAFETY: This is the only place we ever reference these and this function is never called again.
    let colormap = unsafe { &mut *{ &raw mut COLORMAP } };
    let jpeg_dec_1 = unsafe { &mut *{ &raw mut JPEG_DEC_1 } };
    let jpeg_dec_2 = unsafe { &mut *{ &raw mut JPEG_DEC_2 } };
    let jpeg_stream = unsafe { &mut *{ &raw mut JPEG_STREAM } };
    let jpeg_stream_i = unsafe { &mut *{ &raw mut JPEG_STREAM_I } };
    let checksum = unsafe { &mut *{ &raw mut CHECKSUM } };

    #[cfg(all(target_arch = "arm", feature = "log"))]
    {
        black_box(cjpeg_wrbmp_log(
            black_box(colormap),
            black_box(jpeg_dec_1),
            black_box(jpeg_dec_2),
            black_box(jpeg_stream),
            black_box(jpeg_stream_i),
            black_box(checksum),
        ))
    }

    #[cfg(any(unix, windows, not(feature = "log")))]
    {
        black_box(cjpeg_wrbmp_std(
            black_box(colormap),
            black_box(jpeg_dec_1),
            black_box(jpeg_dec_2),
            black_box(jpeg_stream),
            black_box(jpeg_stream_i),
            black_box(checksum),
        ))
    }
}

#[unsafe(no_mangle)]
#[cfg(target_arch = "arm")]
pub extern "C" fn main() {
    #[cfg_attr(not(feature = "log"), expect(unused_variables))]
    let success = black_box(cjpeg_wrbmp());
    
    #[cfg(feature = "log")]
    {
        rprintln!("cjpeg_wrbmp: {}", success);
        rprintln!("*STOP*{}", 1 - success as i32);
    }
}

#[cfg(any(unix, windows))]
fn main() {
    let success = black_box(cjpeg_wrbmp());

    #[cfg(feature = "log")]
    std::println!("cjpeg_wrbmp: {}", success);

    std::process::exit(!success as i32);
}
