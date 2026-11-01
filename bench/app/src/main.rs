#![no_main]
#![no_std]

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

  Original name: run_lift.c

  Changes: no major functional changes

  License: GPL version 3 or later

*/

mod control;
mod io;

use crate::{
    app::{ctrl_loop, get_vals, idle, set_vals},
    control::LiftCmd,
    io::{IoIn, Motor},
};
use core::hint::black_box;
use cortex_m_rt::exception;
use panic_halt as _;
use rtic::Mutex;
use rtic::mutex_prelude::TupleExt02;

#[cfg(feature = "log")]
use rtt_target::rprintln;

#[rtic::app(device = xmc4500, dispatchers = [CAN0_0])]
mod app {
    #[cfg(feature = "log")]
    use cortex_m::peripheral::DWT;
    #[cfg(feature = "log")]
    use rtt_target::{rprintln, rtt_init_print};

    use crate::{
        control::Control,
        io::{ControlIo, SimulatedIo},
        task_ctrl_loop, task_get_vals, task_idle, task_set_vals,
    };

    #[shared]
    struct Shared {
        simio: SimulatedIo,
        ctrlio: ControlIo,
        checksum: i32,
    }

    #[local]
    struct Local {
        ctrl: Control,
        start_cycles: u32,
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
        #[cfg(feature = "log")]
        rtt_init_print!();
        (
            Shared {
                simio: SimulatedIo::default(),
                ctrlio: ControlIo::default(),
                checksum: 0,
            },
            Local {
                ctrl: Control::init(),
                #[cfg(feature = "log")]
                start_cycles: DWT::cycle_count(),
                #[cfg(not(feature = "log"))]
                start_cycles: 0,
            },
        )
    }

    #[idle(shared = [simio, checksum],  local = [start_cycles])]
    fn idle(mut c: idle::Context) -> ! {
        task_idle(&mut c);
        #[cfg(feature = "log")]
        {
            let cycles = DWT::cycle_count() - *c.local.start_cycles;
            c.shared.checksum.lock(|&mut c| {
                rprintln!("{}", cycles);
                rprintln!("*STOP*{}", 1 - (c == 4005888) as i32);
            });
        }
        loop {}
    }

    #[task(priority = 1, shared = [simio, ctrlio])]
    async fn get_vals(c: get_vals::Context) {
        #[cfg(feature = "log_task")]
        let start_cycles = DWT::cycle_count();
        task_get_vals(c);
        #[cfg(feature = "log_task")]
        rprintln!("get_vals: {}", DWT::cycle_count() - start_cycles);
    }

    #[task(priority = 1, shared = [ctrlio], local = [ctrl])]
    async fn ctrl_loop(c: ctrl_loop::Context) {
        #[cfg(feature = "log_task")]
        let start_cycles = DWT::cycle_count();
        task_ctrl_loop(c);
        #[cfg(feature = "log_task")]
        rprintln!("ctrl_loop: {}", DWT::cycle_count() - start_cycles);
    }

    #[task(priority = 1, shared = [simio, ctrlio, checksum])]
    async fn set_vals(c: set_vals::Context) {
        #[cfg(feature = "log_task")]
        let start_cycles = DWT::cycle_count();
        task_set_vals(c);
        #[cfg(feature = "log_task")]
        rprintln!("set_vals: {}", DWT::cycle_count() - start_cycles);
    }
}

#[inline(never)]
fn task_idle(c: &mut idle::Context) {
    for _ in 0..=1000 {
        c.shared.simio.lock(|simio| {
            /* zero input stimulus */
            simio.input = black_box(0);
            simio.adc1 = black_box(0);
            simio.adc2 = black_box(0);
            simio.adc3 = black_box(0);
        });

        /* run lift_controller */
        get_vals::spawn().unwrap();
        ctrl_loop::spawn().unwrap();
        set_vals::spawn().unwrap();
    }
}

#[inline(never)]
fn task_get_vals(c: get_vals::Context) {
    let get_vals::SharedResources {
        mut simio,
        mut ctrlio,
        ..
    } = c.shared;

    let mut in0 = simio.lock(|simio| simio.input);
    ctrlio.lock(|ctrlio| {
        let mut in1 = ctrlio.dly1;
        let mut in2 = ctrlio.dly2;

        ctrlio.dly2 = ctrlio.dly1;
        ctrlio.dly1 = in0;
        /* majority voting for input values
        delays input value change by one period */
        // loopbound min 10 max 10
        for i in ctrlio.io_in.iter_mut() {
            *i = (((in0 & 1) + (in1 & 1) + (in2 & 1)) > 1) as i32;
            in0 >>= 1;
            in1 >>= 1;
            in2 >>= 1;
        }
        simio.lock(|simio| {
            ctrlio.io_analog[0] = simio.adc1;
            ctrlio.io_analog[1] = simio.adc2;
            ctrlio.io_analog[2] = simio.adc3;
        })
    });
}

#[inline(never)]
fn task_ctrl_loop(c: ctrl_loop::Context) {
    let ctrl = c.local.ctrl;
    let mut ctrlio = c.shared.ctrlio;

    ctrlio.lock(|ctrlio| {
        match ctrl.cmd {
            LiftCmd::None => {
                control::check_cmd(ctrl, ctrlio);
            }
            _ => {
                control::do_impulse(
                    ctrl,
                    ctrlio.get_io_in(IoIn::SensImpuls),
                    ctrlio.get_io_in(IoIn::SensBottom),
                    ctrlio.get_io_out(Motor::On),
                );
                control::do_cmd(ctrl, ctrlio);
            }
        }
        control::check_level(ctrl, ctrlio);
        ctrlio.io_led[13] = ((ctrl.dbg_cnt & 0x80) != 0) as i32;
    });
    ctrl.dbg_cnt += 1;
}

#[inline(never)]
fn task_set_vals(c: set_vals::Context) {
    let set_vals::SharedResources {
        simio,
        ctrlio,
        mut checksum,
        ..
    } = c.shared;

    let mut val = 0;
    (ctrlio, simio).lock(|ctrlio, simio| {
        for &o in ctrlio.io_out.iter().rev() {
            val <<= 1;
            val |= if o != 0 { 1 } else { 0 }
        }
        simio.output = val;
        for &o in ctrlio.io_led.iter().rev() {
            val <<= 1;
            val |= if o != 0 { 1 } else { 0 }
        }
        simio.led = val;
    });
    checksum.lock(|c| *c += val)
}

#[exception]
unsafe fn DefaultHandler(_irqn: i16) {
    #[cfg(feature = "log")]
    rprintln!("IRQn = {}", _irqn);
    panic!()
}
