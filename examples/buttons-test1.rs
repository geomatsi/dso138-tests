#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m as cm;
use hal::gpio::gpiob::{PB12, PB13, PB14, PB15};
use hal::gpio::{Input, PullUp};
use hal::prelude::*;
use hal::stm32::TIM2;
use hal::timer::CountDownTimer;
use hal::timer::Event;
use hal::timer::Timer;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal as hal;

use embedded_hal::digital::v2::InputPin;

use rtic::app;

#[app(device = stm32f1xx_hal::stm32, peripherals = true)]
const APP: () = {
    struct Resources {
        // early resources
        #[init(false)]
        cb1: bool,
        #[init(false)]
        cb2: bool,
        #[init(false)]
        cb3: bool,
        #[init(false)]
        cb4: bool,

        // late resources
        button1: PB12<Input<PullUp>>,
        button2: PB13<Input<PullUp>>,
        button3: PB14<Input<PullUp>>,
        button4: PB15<Input<PullUp>>,
        tmr: CountDownTimer<TIM2>,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();

        let mut rcc = cx.device.RCC.constrain();
        let mut flash = cx.device.FLASH.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(32.mhz())
            .freeze(&mut flash.acr);

        let mut tmr =
            Timer::tim2(cx.device.TIM2, &clocks, &mut rcc.apb1).start_count_down(100.hz());
        tmr.listen(Event::Update);

        /* buttons */
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.apb2);
        let button1 = gpiob.pb12.into_pull_up_input(&mut gpiob.crh);
        let button2 = gpiob.pb13.into_pull_up_input(&mut gpiob.crh);
        let button3 = gpiob.pb14.into_pull_up_input(&mut gpiob.crh);
        let button4 = gpiob.pb15.into_pull_up_input(&mut gpiob.crh);

        /* init late resources */
        init::LateResources {
            button1,
            button2,
            button3,
            button4,
            tmr,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cm::asm::nop();
        }
    }

    #[task(binds = TIM2, resources = [tmr, button1, cb1, button2, cb2, button3, cb3, button4, cb4])]
    fn tim2(cx: tim2::Context) {
        if cx.resources.button1.is_low().unwrap() {
            if !*cx.resources.cb1 {
                *cx.resources.cb1 = true;
                rprintln!("B1: pressed");
            }
        } else if *cx.resources.cb1 {
            *cx.resources.cb1 = false;
            rprintln!("B1: released");
        }

        if cx.resources.button2.is_low().unwrap() {
            if !*cx.resources.cb2 {
                *cx.resources.cb2 = true;
                rprintln!("B2: pressed");
            }
        } else if *cx.resources.cb2 {
            *cx.resources.cb2 = false;
            rprintln!("B2: released");
        }

        if cx.resources.button3.is_low().unwrap() {
            if !*cx.resources.cb3 {
                *cx.resources.cb3 = true;
                rprintln!("B3: pressed");
            }
        } else if *cx.resources.cb3 {
            *cx.resources.cb3 = false;
            rprintln!("B3: released");
        }

        if cx.resources.button4.is_low().unwrap() {
            if !*cx.resources.cb4 {
                *cx.resources.cb4 = true;
                rprintln!("B4: pressed");
            }
        } else if *cx.resources.cb4 {
            *cx.resources.cb4 = false;
            rprintln!("B4: released");
        }

        cx.resources.tmr.clear_update_interrupt_flag();
    }
};
