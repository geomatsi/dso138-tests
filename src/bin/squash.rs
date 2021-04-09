#![deny(warnings)]
#![no_main]
#![no_std]

use cortex_m as cm;
use display_interface_parallel_gpio::PGPIO8BitInterface;
use dso138_tests::hw::delay_timer::DelayTimer;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::style::PrimitiveStyle;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use hal::gpio::gpioa::PA15;
use hal::gpio::gpiob::{PB0, PB1, PB2, PB3, PB4, PB5, PB6, PB7};
use hal::gpio::gpiob::{PB11, PB12, PB13, PB14, PB15};
use hal::gpio::gpioc::{PC14, PC15};
use hal::gpio::{Input, Output, PullUp, PushPull};
use hal::prelude::*;
use hal::stm32::TIM3;
use hal::timer::CountDownTimer;
use hal::timer::Event;
use hal::timer::Timer;
use ili9341::{Ili9341, Orientation};
use panic_rtt_target as _;
use rtic::app;
use rtic::cyccnt::Instant;
use rtic::cyccnt::U32Ext;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal as hal;

type DisplayType = Ili9341<
    PGPIO8BitInterface<
        PB0<Output<PushPull>>,
        PB1<Output<PushPull>>,
        PB2<Output<PushPull>>,
        PB3<Output<PushPull>>,
        PB4<Output<PushPull>>,
        PB5<Output<PushPull>>,
        PB6<Output<PushPull>>,
        PB7<Output<PushPull>>,
        PC14<Output<PushPull>>,
        PC15<Output<PushPull>>,
    >,
    PB11<Output<PushPull>>,
>;

/* cpu sysclk: 72 MHz (no external quartz) */

const PROC_PERIOD: u32 = 72_0000; /* 10 msec */

#[app(device = stm32f1xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
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
        #[init(120)]
        pos: i32,

        // late resources
        display: DisplayType,
        button1: PB12<Input<PullUp>>,
        button2: PB13<Input<PullUp>>,
        button3: PB14<Input<PullUp>>,
        button4: PB15<Input<PullUp>>,
        led: PA15<Output<PushPull>>,
        btmr: CountDownTimer<TIM3>,
    }

    #[init(schedule = [proc_task])]
    fn init(mut cx: init::Context) -> init::LateResources {
        rtt_init_print!();

        let mut rcc = cx.device.RCC.constrain();
        let mut flash = cx.device.FLASH.constrain();
        let mut afio = cx.device.AFIO.constrain(&mut rcc.apb2);

        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(32.mhz())
            .freeze(&mut flash.acr);

        /* enable monotonic timer */
        cx.core.DCB.enable_trace();
        cx.core.DWT.enable_cycle_counter();

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = cx.device.GPIOC.split(&mut rcc.apb2);

        let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

        let mut btmr =
            Timer::tim3(cx.device.TIM3, &clocks, &mut rcc.apb1).start_count_down(5000.hz());
        btmr.listen(Event::Update);

        let dtmr = Timer::tim2(cx.device.TIM2, &clocks, &mut rcc.apb1)
            .start_master(1000.khz(), hal::pac::tim2::cr2::MMS_A::RESET);

        /* buttons */

        let button1 = gpiob.pb12.into_pull_up_input(&mut gpiob.crh);
        let button2 = gpiob.pb13.into_pull_up_input(&mut gpiob.crh);
        let button3 = gpiob.pb14.into_pull_up_input(&mut gpiob.crh);
        let button4 = gpiob.pb15.into_pull_up_input(&mut gpiob.crh);

        /* led */

        let led = pa15.into_push_pull_output(&mut gpioa.crh);

        /* display */

        let mut delay = DelayTimer::new(dtmr);

        let p0 = gpiob.pb0.into_push_pull_output(&mut gpiob.crl);
        let p1 = gpiob.pb1.into_push_pull_output(&mut gpiob.crl);
        let p2 = gpiob.pb2.into_push_pull_output(&mut gpiob.crl);
        let p3 = pb3.into_push_pull_output(&mut gpiob.crl);
        let p4 = pb4.into_push_pull_output(&mut gpiob.crl);
        let p5 = gpiob.pb5.into_push_pull_output(&mut gpiob.crl);
        let p6 = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);
        let p7 = gpiob.pb7.into_push_pull_output(&mut gpiob.crl);

        let mut ncs = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
        let mut nrd = gpiob.pb10.into_push_pull_output(&mut gpiob.crh);

        let nreset = gpiob.pb11.into_push_pull_output(&mut gpiob.crh);
        let nwr = gpioc.pc15.into_push_pull_output(&mut gpioc.crh);
        let rs = gpioc.pc14.into_push_pull_output(&mut gpioc.crh);

        ncs.set_low().unwrap();
        nrd.set_high().unwrap();

        let pio8bit = PGPIO8BitInterface::new(p0, p1, p2, p3, p4, p5, p6, p7, rs, nwr);
        let mut display = Ili9341::new(pio8bit, nreset, &mut delay).unwrap();

        display.set_orientation(Orientation::Portrait).unwrap();

        /* clear screen */

        let bg = PrimitiveStyle::with_fill(Rgb565::BLACK);

        Rectangle::new(
            Point::new(0, 0),
            Point::new(display.width() as i32, display.height() as i32),
        )
        .into_styled(bg)
        .draw(&mut display)
        .unwrap();

        cx.schedule.proc_task(Instant::now()).unwrap();

        /* init late resources */
        init::LateResources {
            display,
            button1,
            button2,
            button3,
            button4,
            led,
            btmr,
        }
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            cm::asm::nop();
        }
    }

    #[task(binds = TIM3, resources = [btmr, button1, cb1, button2, cb2, button3, cb3, button4, cb4])]
    fn tim3(cx: tim3::Context) {
        if cx.resources.button1.is_low().unwrap() {
            if !*cx.resources.cb1 {
                *cx.resources.cb1 = true;
            }
        } else if *cx.resources.cb1 {
            *cx.resources.cb1 = false;
        }

        if cx.resources.button2.is_low().unwrap() {
            if !*cx.resources.cb2 {
                *cx.resources.cb2 = true;
            }
        } else if *cx.resources.cb2 {
            *cx.resources.cb2 = false;
        }

        if cx.resources.button3.is_low().unwrap() {
            if !*cx.resources.cb3 {
                *cx.resources.cb3 = true;
            }
        } else if *cx.resources.cb3 {
            *cx.resources.cb3 = false;
        }

        if cx.resources.button4.is_low().unwrap() {
            if !*cx.resources.cb4 {
                *cx.resources.cb4 = true;
            }
        } else if *cx.resources.cb4 {
            *cx.resources.cb4 = false;
        }

        cx.resources.btmr.clear_update_interrupt_flag();
    }

    #[task(schedule = [proc_task], resources = [display, cb1, cb4, pos])]
    fn proc_task(cx: proc_task::Context) {
        let width = cx.resources.display.width() as i32;
        let display = cx.resources.display;
        let old = *cx.resources.pos;

        let new = match (*cx.resources.cb1, *cx.resources.cb4) {
            (true, false) => {
                *cx.resources.pos += 5;
                if *cx.resources.pos > width {
                    *cx.resources.pos = width;
                }
                *cx.resources.pos
            }
            (false, true) => {
                *cx.resources.pos -= 5;
                if *cx.resources.pos < 0 {
                    *cx.resources.pos = 0;
                }
                *cx.resources.pos
            }
            _ => *cx.resources.pos,
        };

        if old != new {
            let ground = PrimitiveStyle::with_fill(Rgb565::BLACK);
            let color = PrimitiveStyle::with_fill(Rgb565::GREEN);

            rprintln!("draw: {} -> {}", old, new);

            Rectangle::new(Point::new(old - 20, 0), Point::new(old + 20, 10))
                .into_styled(ground)
                .draw(display)
                .unwrap();
            Rectangle::new(Point::new(new - 20, 0), Point::new(new + 20, 10))
                .into_styled(color)
                .draw(display)
                .unwrap();
        }

        cx.schedule
            .proc_task(cx.scheduled + PROC_PERIOD.cycles())
            .unwrap();
    }

    // needed for RTIC timer queue and task management
    extern "C" {
        fn EXTI2();
    }
};
