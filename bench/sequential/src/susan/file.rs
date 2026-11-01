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

  C version: wccfile.c

*/

use core::cmp;

#[allow(unused)]
pub enum SeekOrigin { SET, CUR, END, }

pub struct File<'a> {
    pub data: &'a [i8],
    pub cur_pos: usize,
}

impl<'a> File<'a> {
    pub const fn new(data: &'a [i8]) -> Self {
        File {
            data,
            cur_pos: 0,
        }
    }

    pub fn getc(&mut self) -> Result<i8, &'static str> {
        if self.eof() { Err("EOF") }
        else {
            let res = self.data[self.cur_pos];
            self.cur_pos += 1;
            Ok(res)
        }
    }
    pub fn read(&mut self, buf: &mut [i8], count: usize) -> Result<usize, &'static str> {
        if self.eof() { return Err("EOF"); }
        let bytes_to_read = cmp::min(count, self.data.len() - self.cur_pos);
        if bytes_to_read == 0 { return Ok(0); }

        // loopbound min 7220 max 7220
        buf[..bytes_to_read].copy_from_slice(&self.data[self.cur_pos..(self.cur_pos + bytes_to_read)]);
        self.cur_pos += bytes_to_read;
        Ok(bytes_to_read)
    }

    pub fn read_line(&mut self, buf: &mut [i8], num: usize) -> Result<usize, &'static str> {
        if self.eof() { return Err("EOF"); }

        let mut pos = 0;
        // loopbound min 57 max 57
        while pos < num && self.cur_pos < self.data.len() {
            buf[pos] = self.data[self.cur_pos];
            if buf[pos] == b'\n' as i8 { break; }
            self.cur_pos += 1;
            pos += 1;
        }
        Ok(pos)
    }

    pub fn seek(&mut self, offset: isize, origin: SeekOrigin) -> Result<(), &'static str> {
        let new_pos = match origin {
            SeekOrigin::SET => offset as usize,
            SeekOrigin::CUR => (self.cur_pos as isize + offset) as usize,
            SeekOrigin::END => (self.data.len() as isize + offset) as usize,
        };
        if new_pos >= self.data.len() {
            return Err("Out of Bounds");
        }

        self.cur_pos = new_pos;
        Ok(())
    }

    pub fn eof(&self) -> bool {
        self.cur_pos == self.data.len()
    }
}
