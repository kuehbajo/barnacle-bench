/*

  Rust Implementation

  Author: David Heim
  License: The Rust implementation shares the same license (GPL 3) as its origin.

  ------------------------------------------------------

  The corresponding C version is part of the TACLeBench benchmark suite
  containing the following source and license information.

  Version V 2.0

  Name: lift

  Author: Martin Schoeberl, Benedikt Huber

  Function: Lift Controler

  Source: C-Port from http://www.jopdesign.com/doc/jembench.pdf

  Changes: no major functional changes

  License: GPL version 3 or later

*/

#[derive(Default)]
pub struct SimulatedIo {
    pub input: i32,
    pub output: i32,
    pub led: i32,
    pub adc1: i32,
    pub adc2: i32,
    pub adc3: i32,
}

#[derive(Default)]
pub struct ControlIo {
    pub io_in: [i32; 10],
    pub io_out: [i32; 4],
    pub io_analog: [i32; 4],
    pub io_led: [i32; 16],
    pub dly1: i32,
    pub dly2: i32,
}

pub enum IoIn {
    SensImpuls = 0,
    SensTop = 1,
    SensBottom = 2,
    SensLoad = 3,
    GoUp = 4,
    GoDown = 5,
    GoTop = 6,
    GoBottom = 7,
    GoLoad = 8,
}

pub enum Motor {
    On = 0,
    Up = 1,
}

impl ControlIo {
    pub fn get_io_in(&self, index: IoIn) -> bool {
        self.io_in[index as usize] != 0
    }

    pub fn get_io_out(&self, index: Motor) -> bool {
        self.io_out[index as usize] != 0
    }

    pub fn set_io_out(&mut self, index: Motor, val: bool) {
        self.io_out[index as usize] = val as i32;
    }
}
