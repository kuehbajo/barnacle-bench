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

  Name: jpeglib.h

  Author: Thomas G. Lane.

  This file is part of the Independent JPEG Group's software.
  For conditions of distribution and use, see the accompanying README file.

  This file defines the application interface for the JPEG library.
  Most applications using the library need only include this file,
  and perhaps jerror.h if they want to know the exact error codes.

  Source: Independent JPEG Group's software

  Changes: no major functional changes

  License:  See the accompanying README file
*/

use crate::jmorecfg::CjpegWrbmpJdimension;

/* Routines that are to be used by both halves of the library are declared
   to receive a pointer to this structure.  There are no actual instances of
   jpeg_common_struct, only of jpeg_compress_struct and jpeg_decompress_struct.
*/

/* Master record for a decompression instance */

pub struct CjpegWrbmpJpegDecompressStruct {
   pub progress: CjpegWrbmpJpegProgressMgr, /* Progress monitor, or NULL if none */
   pub output_height: CjpegWrbmpJdimension, /* scaled image height */
   pub out_color_components: i32, /* # of color components in out_color_space */
   pub actual_number_of_colors: i32,  /* number of entries in use */
}

impl CjpegWrbmpJpegDecompressStruct {
   pub const fn new() -> Self {
      CjpegWrbmpJpegDecompressStruct {
         progress: CjpegWrbmpJpegProgressMgr::new(),
         output_height: 0,
         out_color_components: 0,
         actual_number_of_colors: 0,
      }
   }
}

/* Progress monitor object */

pub struct CjpegWrbmpJpegProgressMgr {
   pub pass_counter: i64,    /* work units completed in this pass */
   pub pass_limit: i64,    /* total number of work units in this pass */
}

impl Default for CjpegWrbmpJpegProgressMgr {
   fn default() -> Self {
      CjpegWrbmpJpegProgressMgr {
         pass_counter: 0,
         pass_limit: 0,
      }
   }
}

impl CjpegWrbmpJpegProgressMgr {
   const fn new() -> Self {
      CjpegWrbmpJpegProgressMgr {
         pass_counter: 0,
         pass_limit: 0,
      }
   }
}
