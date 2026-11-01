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

  Name: jpeglib

  Author: Thomas G. Lane

  Function: This file defines the application interface for the JPEG library.
    Most applications using the library need only include this file, and perhaps
    jerror.h if they want to know the exact error codes.

  Source: MediaBench II
          http://euler.slu.edu/~fritts/mediabench (mirror)

  Original name: cjpeg

  Changes: No major functional changes.

  License: See the accompanying README file.

*/

/*
  Various constants determining the sizes of things.
  All of these are specified by the JPEG standard, so don't change them if you
  want to be compatible.
*/

/* The basic DCT block is 8x8 samples */
pub const DCTSIZE: usize = 8;

/* DCTSIZE squared; # of elements in a block */
pub const DCTSIZE2: usize = 64;

/*
  Master record for a compression instance
*/

pub struct JpegCompressStruct {
    /* input image width */
    pub image_width: usize,

    /* input image height */
    pub image_height: usize,

    /* # of color components in JPEG image */
    pub num_components: usize,

    /* largest h_samp_factor */
    pub max_h_samp_factor: i32,

    /* largest v_samp_factor */
    pub max_v_samp_factor: i32,
}

impl JpegCompressStruct {
    pub const fn new() -> Self {
        JpegCompressStruct {
            image_width: 0,
            image_height: 0,
            num_components: 0,
            max_h_samp_factor: 0,
            max_v_samp_factor: 0,
        }
    }
}
