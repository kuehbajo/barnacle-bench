/*

  Rust Implementation

  Author: kuehbajo
  License: The Rust implementation shares the same license as the original main file.

  THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND,
  EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
  OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
  IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
  DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
  OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR
  THE USE OR OTHER DEALINGS IN THE SOFTWARE.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite.

*/

use core::cmp;

/*
##################
    rijndael_enc_libc.h
##################
*/

pub struct File<'a> {
    pub data: &'a [u8],
    pub cur_pos: usize,
}

/*
##################
    rijndael_enc_libc.c
##################
*/

impl<'a> File<'a> {
    pub const fn new(data: &'a [u8]) -> Self {
        File {
            data,
            cur_pos: 0,
        }
    }

    pub fn read(&mut self, buf: &mut [u8], size: usize, count: usize) -> Result<usize, &'static str> {
        if self.eof() { return Err("EOF"); }
        let bytes_to_read = cmp::min(size * count, self.data.len() - self.cur_pos);
        if bytes_to_read == 0 { return Ok(0); }

        // loopbound min 7220 max 7220
        buf[..bytes_to_read].copy_from_slice(&self.data[self.cur_pos..(self.cur_pos + bytes_to_read)]);
        self.cur_pos += bytes_to_read;
        Ok(bytes_to_read)
    }

    pub fn eof(&self) -> bool {
        self.cur_pos == self.data.len()
    }
}
