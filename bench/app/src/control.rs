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

use crate::io::{ControlIo, IoIn, Motor};

pub struct Control {
    pub level_pos: [i32; 16],
    pub one_level: i32,
    pub cnt_valid: bool,
    pub cnt: i32,
    pub lift_level: i32,
    pub load_level: i32,
    pub load_pending: bool,
    pub load_sensor: bool,
    pub cmd: LiftCmd,
    pub tim_motor: i32,
    pub tim_imp: i32,
    pub direction_up: bool,
    pub last_imp: bool,
    pub dbg_cnt: i32,
    pub end_cnt: i32,
}

impl Control {
    pub fn init() -> Self {
        Self {
            level_pos: [
                0, 58, 115, 173, 230, 288, 346, 403, 461, 518, 576, 634, 691, 749, 806, 864,
            ],
            one_level: 0,
            cnt_valid: false,
            cnt: 0,
            lift_level: 0,
            load_level: 0,
            load_pending: false,
            load_sensor: false,
            cmd: LiftCmd::None,
            tim_motor: 0,
            tim_imp: 0,
            direction_up: true,
            last_imp: false,
            dbg_cnt: 0,
            end_cnt: 0,
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum LiftCmd {
    None,
    Top,
    Bottom,
    Up,
    Down,
    Error,
}

pub fn check_cmd(ctrl: &mut Control, ctrlio: &mut ControlIo) {
    if ctrl.load_pending {
        if ctrlio.get_io_in(IoIn::SensBottom) {
            ctrl.cmd = LiftCmd::Top;
        }
    } else if ctrlio.get_io_in(IoIn::GoUp) {
        if !ctrlio.get_io_in(IoIn::SensTop) && ctrl.lift_level != 14 {
            ctrl.cmd = LiftCmd::Up;
        }
    } else if ctrlio.get_io_in(IoIn::GoDown) {
        if !ctrlio.get_io_in(IoIn::SensBottom) && ctrl.lift_level != 1 {
            ctrl.cmd = LiftCmd::Down;
        }
    } else if ctrlio.get_io_in(IoIn::GoLoad) {
        if ctrl.load_level != 0 && ctrl.lift_level < ctrl.load_level {
            ctrl.cmd = LiftCmd::Top;
        } else {
            ctrl.cmd = LiftCmd::Bottom;
        }
        ctrl.load_pending = true;
        ctrl.load_sensor = false;
    } else if ctrlio.get_io_in(IoIn::GoTop) {
        if !ctrlio.get_io_in(IoIn::SensTop) {
            ctrl.cmd = LiftCmd::Top;
        }
    } else if ctrlio.get_io_in(IoIn::GoBottom) && !ctrlio.get_io_in(IoIn::SensBottom) {
        ctrl.cmd = LiftCmd::Bottom;
    }

    if ctrl.cmd != LiftCmd::None {
        ctrl.tim_motor = 50;
    }
}

pub fn do_impulse(ctrl: &mut Control, val: bool, reset: bool, motor: bool) {
    if (val && !ctrl.last_imp) && (motor || ctrl.tim_imp > 0) {
        if ctrl.direction_up {
            ctrl.cnt += 1;
        } else {
            ctrl.cnt -= 1;
        }
    }
    if reset {
        ctrl.cnt = 0;
        ctrl.cnt_valid = true;
    }
    ctrl.last_imp = val;
    if ctrl.tim_imp > 0 {
        ctrl.tim_imp -= 1;
        if ctrl.tim_imp == 0 && ctrl.cmd == LiftCmd::None {
            ctrl.cmd = LiftCmd::None;
        }
    }
}

pub fn do_cmd(ctrl: &mut Control, ctrlio: &mut ControlIo) {
    if ctrl.tim_motor > 0 {
        wait_for_motor_start(ctrl, ctrlio);
    } else {
        let run = check_run(ctrl, ctrlio);
        if ctrlio.get_io_out(Motor::On) && !run {
            /* motor stopped */
            ctrl.cmd = LiftCmd::Error;
            ctrl.tim_imp = 50;
        }
        ctrlio.set_io_out(Motor::On, run);
    }
}

pub fn check_level(ctrl: &mut Control, ctrlio: &mut ControlIo) {
    let middle = ctrl.one_level >> 2;
    if ctrl.cnt_valid {
        // loopbound min 14 max 14
        for i in 1..14 {
            ctrl.lift_level = i as i32;
            if ctrl.cnt < ctrl.level_pos[i] - middle {
                break;
            }
        }
    } else {
        ctrl.lift_level = 0;
    }
    for (i, led) in ctrlio.io_led.iter_mut().enumerate() {
        *led = (i as i32 == ctrl.lift_level - 1) as i32;
    }
}

fn wait_for_motor_start(ctrl: &mut Control, ctrlio: &mut ControlIo) {
    ctrl.tim_motor -= 1;
    ctrl.direction_up = ctrl.cmd == LiftCmd::Up || ctrl.cmd == LiftCmd::Top;
    ctrlio.set_io_out(Motor::Up, ctrl.direction_up);
    if !ctrl.cnt_valid {
        ctrl.cnt = 0; // use relative counter
        if ctrl.cmd == LiftCmd::Up {
            ctrl.end_cnt = ctrl.one_level;
        } else {
            ctrl.end_cnt = -ctrl.one_level;
        }
    } else {
        ctrl.end_cnt = ctrl.cnt;
        let mut new_level = 99;
        if ctrl.cmd == LiftCmd::Up {
            new_level = ctrl.lift_level + 1;
        } else if ctrl.cmd == LiftCmd::Down {
            new_level = ctrl.lift_level - 1;
        }
        new_level -= 1;
        if new_level >= 0 && new_level < 14 {
            ctrl.end_cnt = ctrl.level_pos[new_level as usize];
        }
    }
}

fn check_run(ctrl: &mut Control, ctrlio: &mut ControlIo) -> bool {
    match ctrl.cmd {
        LiftCmd::Up if ctrl.cnt < ctrl.end_cnt - 1 && !ctrlio.get_io_in(IoIn::SensTop) => true,
        LiftCmd::Down if ctrl.cnt > ctrl.end_cnt + 1 && !ctrlio.get_io_in(IoIn::SensBottom) => true,
        LiftCmd::Top => {
            if ctrl.load_pending && ctrlio.get_io_in(IoIn::SensLoad) {
                // we are at lift_load position
                ctrl.load_level = ctrl.lift_level;
                ctrl.load_pending = false;
                return false;
            }
            if !ctrlio.get_io_in(IoIn::SensTop) {
                return true;
            }
            /* safe fallback if lift_load sensor does not work */
            ctrl.load_pending = false;
            false
        }
        LiftCmd::Bottom => {
            if ctrl.load_pending {
                if ctrl.load_sensor && !ctrlio.get_io_in(IoIn::SensLoad) {
                    ctrl.load_sensor = false;
                    ctrl.load_pending = false;
                    ctrl.load_level = ctrl.lift_level;
                    return false;
                }
                ctrl.load_sensor = ctrlio.get_io_in(IoIn::SensLoad);
            }
            if !ctrlio.get_io_in(IoIn::SensBottom) {
                return true;
            }
            false
        }
        _ => false,
    }
}
