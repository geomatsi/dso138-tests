#![no_main]
#![no_std]

use cortex_m as cm;
use cortex_m_rt as rt;
use display_interface_parallel_gpio::PGPIO8BitInterface;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Rectangle, Triangle};
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

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let mut rng = WyRng::default();

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

    display.set_orientation(Orientation::Portrait).unwrap();

    let bg = PrimitiveStyle::with_fill(Rgb565::BLACK);

    Rectangle::new(
        Point::new(0, 0),
        Point::new(display.width() as i32, display.height() as i32),
    )
    .into_styled(bg)
    .draw(&mut display)
    .unwrap();

    loop {
        let mut n: [u8; 4] = [0; 4];
        rng.fill_bytes(&mut n);

        let color = PrimitiveStyle::with_fill(Rgb565::new(n[0], n[1], n[2]));

        match n[3] & 0b111 {
            0..=3 => {
                let mut b: [u8; 4] = [0; 4];
                rng.fill_bytes(&mut b);

                let p1 = Point::new(b[0] as i32, b[1] as i32);
                let p2 = Point::new(b[0] as i32 + b[2] as i32, b[1] as i32 + b[3] as i32);

                rprintln!("rectangle: ({}, {}), ({}, {})", p1.x, p1.y, p2.x, p2.y);

                Rectangle::new(p1, p2)
                    .into_styled(color)
                    .draw(&mut display)
                    .unwrap();
            }
            4..=5 => {
                let mut b: [u8; 6] = [0; 6];
                rng.fill_bytes(&mut b);

                let p1 = Point::new(b[0] as i32, b[1] as i32);
                let p2 = Point::new(b[0] as i32 + b[2] as i32, b[1] as i32 + b[3] as i32);
                let p3 = Point::new(b[0] as i32 + b[4] as i32, b[1] as i32 + b[5] as i32);

                rprintln!(
                    "triangle: ({}, {}), ({}, {}), ({}, {})",
                    p1.x,
                    p1.y,
                    p2.x,
                    p2.y,
                    p3.x,
                    p3.y
                );

                Triangle::new(p1, p2, p3)
                    .into_styled(color)
                    .draw(&mut display)
                    .unwrap();
            }
            6..=7 => {
                let mut b: [u8; 3] = [0; 3];
                rng.fill_bytes(&mut b);

                let p = Point::new(b[0] as i32, b[1] as i32);
                let r: u32 = (b[2] >> 1) as u32;

                rprintln!("circle: ({}, {}), {}", p.x, p.y, r);

                Circle::new(p, r)
                    .into_styled(color)
                    .draw(&mut display)
                    .unwrap();
            }
            _ => {}
        }

        led.toggle().unwrap();
    }
}
