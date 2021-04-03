#![no_main]
#![no_std]

use cortex_m as cm;
use cortex_m_rt as rt;
use display_interface_parallel_gpio::PGPIO8BitInterface;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Rectangle};
use embedded_graphics::style::PrimitiveStyle;
use embedded_hal::digital::v2::OutputPin;
use hal::delay::Delay;
use hal::prelude::*;
use ili9341::{Ili9341, Orientation};
use panic_rtt_target as _;
use rand_core::RngCore;
use rt::entry;
use rtt_target::{rprintln, rtt_init_print};
use stm32f1xx_hal as hal;
use wyhash::WyRng;

const PNUM: usize = 20;

struct Particle {
    px: i32,
    py: i32,
    vx: i32,
    vy: i32,
    pr: i32,
    dt: i32,
}

impl Default for Particle {
    fn default() -> Particle {
        Particle {
            px: 0,
            py: 0,
            vx: 0,
            vy: 0,
            pr: 5,
            dt: 1,
        }
    }
}

impl Particle {
    fn new(px: i32, py: i32, vx: i32, vy: i32, pr: i32, dt: i32) -> Particle {
        Particle {
            px,
            py,
            vx,
            vy,
            pr,
            dt,
        }
    }

    fn step(&mut self) {
        self.px += self.vx * self.dt;
        self.py += self.vy * self.dt;
    }

    fn rebound(&mut self, cx: i32, cy: i32) {
        self.vx *= cx;
        self.vy *= cy;
    }

    fn area(&self) -> (Point, Point) {
        (
            Point::new(self.px - self.pr, self.py - self.pr),
            Point::new(self.px + self.pr, self.py + self.pr),
        )
    }
}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let dp = hal::stm32::Peripherals::take().unwrap();
    let cp = cm::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(72.mhz())
        .pclk1(32.mhz())
        .freeze(&mut flash.acr);

    let mut delay = Delay::new(cp.SYST, clocks);

    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.apb2);

    let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

    let mut led = pa15.into_push_pull_output(&mut gpioa.crh);

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
    let h = display.height() as i32;
    let w = display.width() as i32;

    display.set_orientation(Orientation::Portrait).unwrap();

    let fc = PrimitiveStyle::with_fill(Rgb565::BLACK);
    let pc = PrimitiveStyle::with_fill(Rgb565::GREEN);

    // black screen
    Rectangle::new(Point::new(0, 0), Point::new(w, h))
        .into_styled(fc)
        .draw(&mut display)
        .unwrap();

    let mut ens: [Particle; PNUM] = Default::default();
    let mut rng = WyRng::default();
    let mut b: [u8; 4] = [0; 4];

    for i in 0..PNUM {
        rng.fill_bytes(&mut b);
        ens[i].px = (b[0] >> 1) as i32;
        ens[i].py = (b[1] >> 1) as i32;
        ens[i].vx = ((b[2] & 0xF) + 1) as i32;
        ens[i].vy = ((b[3] & 0xF) + 1) as i32;
    }

    loop {
        rprintln!("step");

        for p in &mut ens {
            Rectangle::new(p.area().0, p.area().1)
                .into_styled(fc)
                .draw(&mut display)
                .unwrap();

            p.step();

            if p.px > w || p.px < 0 {
                p.rebound(-1, 1)
            }

            if p.py > h || p.py < 0 {
                p.rebound(1, -1)
            }

            Circle::new(Point::new(p.px, p.py), p.pr as u32)
                .into_styled(pc)
                .draw(&mut display)
                .unwrap();
        }

        delay.delay_ms(10u16);
        led.toggle().unwrap();
    }
}
