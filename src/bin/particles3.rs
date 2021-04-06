#![no_main]
#![no_std]

use cortex_m as cm;
use cortex_m_rt as rt;
use display_interface_parallel_gpio::PGPIO8BitInterface;
use dso138_tests::phys::particles::{Particle, ParticleColor};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Rectangle};
use embedded_graphics::style::PrimitiveStyle;
use embedded_hal::digital::v2::OutputPin;
use fixed::{types::extra::U12, FixedI32};
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

// colliding particles: fixed point arithmetic
type T = FixedI32<U12>;

fn get_color(p: &Particle<T>) -> PrimitiveStyle<Rgb565> {
    match p.get_color() {
        ParticleColor::Green => PrimitiveStyle::with_fill(Rgb565::GREEN),
        ParticleColor::Red => PrimitiveStyle::with_fill(Rgb565::RED),
        ParticleColor::Blue => PrimitiveStyle::with_fill(Rgb565::BLUE),
        ParticleColor::Yellow => PrimitiveStyle::with_fill(Rgb565::YELLOW),
        ParticleColor::White => PrimitiveStyle::with_fill(Rgb565::WHITE),
    }
}

fn area(p: &Particle<T>) -> (Point, Point) {
    (
        Point::new(
            (p.get_x() - p.get_r()).to_num::<i32>(),
            (p.get_y() - p.get_r()).to_num::<i32>(),
        ),
        Point::new(
            (p.get_x() + p.get_r()).to_num::<i32>(),
            (p.get_y() + p.get_r()).to_num::<i32>(),
        ),
    )
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
    let fc = PrimitiveStyle::with_fill(Rgb565::BLACK);
    let h = T::from_num(display.height());
    let w = T::from_num(display.width());

    display.set_orientation(Orientation::Portrait).unwrap();

    // black screen
    Rectangle::new(
        Point::new(0, 0),
        Point::new(w.to_num::<i32>(), h.to_num::<i32>()),
    )
    .into_styled(fc)
    .draw(&mut display)
    .unwrap();

    let mut ens: [Particle<T>; PNUM] = Default::default();
    let mut rng = WyRng::default();
    let mut rnd: [u8; 4] = [0; 4];

    // randomize particles position and velocity
    for p in ens.iter_mut() {
        rng.fill_bytes(&mut rnd);
        *p = Particle::new(
            T::from_num(rnd[0] >> 1),
            T::from_num(rnd[1] >> 1),
            T::from_num((rnd[2] & 0xF) + 1),
            T::from_num((rnd[3] & 0xF) + 1),
            T::from_num(6.0),
            T::from_num(0.1),
            ParticleColor::Green,
        );
    }

    // customize several particles colors to make their motion easier to see
    ens[0].set_color(ParticleColor::Red);
    ens[1].set_color(ParticleColor::Blue);
    ens[2].set_color(ParticleColor::Yellow);
    ens[3].set_color(ParticleColor::White);

    let mut collisions: u64 = 0;

    loop {
        let mut energy: T = T::from_num(0);

        for p in ens.iter_mut().take(PNUM) {
            energy += p.energy();
        }

        rprintln!("energy: {} collisions: {}", energy, collisions);

        for i in 0..PNUM {
            let (head, tail) = ens.split_at_mut(i + 1);
            let p = &mut head[i];

            if p.collided() {
                continue;
            }

            if Particle::bounce(p, T::from_num(0), w, T::from_num(0), h) {
                continue;
            }

            for q in tail {
                if q.collided() {
                    continue;
                }

                if Particle::collide(p, q) {
                    collisions += 1;
                    break;
                }
            }
        }

        for p in &mut ens {
            Rectangle::new(area(p).0, area(p).1)
                .into_styled(fc)
                .draw(&mut display)
                .unwrap();

            p.step();

            Circle::new(
                Point::new(p.get_x().to_num::<i32>(), p.get_y().to_num::<i32>()),
                p.get_r().to_num::<u32>(),
            )
            .into_styled(get_color(p))
            .draw(&mut display)
            .unwrap();
        }

        led.toggle().unwrap();
    }
}
