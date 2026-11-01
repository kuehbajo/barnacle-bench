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

  Name: cdjpeg.h

  Author: Thomas G. Lane.

  This file is part of the Independent JPEG Group's software.
  For conditions of distribution and use, see the accompanying README file.

  This file contains common declarations for the sample applications
  cjpeg and djpeg.  It is NOT used by the core JPEG library.

  Source: Independent JPEG Group's software

  Changes: no major functional changes

  License:  See the accompanying README file

*/

use crate::jpeglib::CjpegWrbmpJpegProgressMgr;

/*

  This program is part of the TACLeBench benchmark suite.
  Version V 1.x

  Name: cderror.h

  Author: Thomas G. Lane.

  This file is part of the Independent JPEG Group's software.
  For conditions of distribution and use, see the accompanying README file.

  This file defines the error and message codes for the cjpeg/djpeg
  applications.  These strings are not needed as part of the JPEG library
  proper.
  Edit this file to add new codes, or to translate the message strings to
  some other language.

  Source: Independent JPEG Group's software

  Changes: no major functional changes

  License:  See the accompanying README file

*/

/*
   cjpeg/djpeg may need to perform extra passes to convert to or from
   the source/destination file format.  The JPEG library does not know
   about these passes, but we'd like them to be counted by the progress
   monitor.  We use an expanded progress monitor object to hold the
   additional pass count.
*/

pub struct CjpegWrbmpCdjpegProgressMgr<'a> {
    pub publ: Option<&'a mut CjpegWrbmpJpegProgressMgr>, /* fields known to JPEG library */
    pub completed_extra_passes: i32, /* extra passes completed */
}

