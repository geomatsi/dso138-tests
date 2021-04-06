use core::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy)]
pub enum ParticleColor {
    Green,
    Red,
    Blue,
    Yellow,
    White,
}

impl Default for ParticleColor {
    fn default() -> ParticleColor {
        ParticleColor::Green
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Particle<N>
where
    N: Sub<Output = N> + Div<Output = N> + Mul<Output = N> + Add<Output = N> + Neg<Output = N>,
    N: Default + Copy + Clone + PartialOrd,
{
    px: N,
    py: N,
    vx: N,
    vy: N,
    dt: N,
    r: N,
    m: bool,
    c: ParticleColor,
}

impl<N> Particle<N>
where
    N: Sub<Output = N> + Div<Output = N> + Mul<Output = N> + Add<Output = N> + Neg<Output = N>,
    N: Default + Copy + Clone + PartialOrd,
{
    pub fn new(px: N, py: N, vx: N, vy: N, r: N, dt: N, c: ParticleColor) -> Particle<N> {
        Particle {
            px,
            py,
            vx,
            vy,
            dt,
            r,
            m: false,
            c,
        }
    }

    pub fn get_x(&self) -> N {
        self.px
    }

    pub fn get_y(&self) -> N {
        self.py
    }

    pub fn get_r(&self) -> N {
        self.r
    }

    pub fn get_color(&self) -> ParticleColor {
        self.c
    }

    pub fn set_color(&mut self, c: ParticleColor) {
        self.c = c;
    }

    pub fn step(&mut self) {
        self.px = self.px + self.vx * self.dt;
        self.py = self.py + self.vy * self.dt;
        self.m = false;
    }

    pub fn energy(&self) -> N {
        self.vx * self.vx + self.vy * self.vy
    }

    pub fn collided(&self) -> bool {
        self.m
    }

    // perfectly elastic collision of two equal round particles
    #[allow(clippy::suspicious_operation_groupings)]
    pub fn collide(p: &mut Particle<N>, q: &mut Particle<N>) -> bool {
        let dx = p.px - q.px;
        let dy = p.py - q.py;
        let dx1 = (p.px + p.dt * p.vx) - (q.px + q.dt * q.vx);
        let dy1 = (p.py + p.dt * p.vy) - (q.py + q.dt * q.vy);

        if dx * dx + dy * dy > (p.r + p.r) * (p.r + p.r) {
            return false;
        }

        if dx * dx + dy * dy < dx1 * dx1 + dy1 * dy1 {
            return false;
        }

        let (nvx, nvy, nwx, nwy) = if dx * dx + dy * dy < p.r * p.r {
            // 'mutual exchange' approximation: particles are too close due to discrete time
            (q.vx, q.vy, p.vx, q.vy)
        } else {
            // precise calculation of two round colliding particles
            (
                p.vx + (dx * dx * (q.vx - p.vx) + dx * dy * (q.vy - p.vy)) / (dx * dx + dy * dy),
                p.vy + (dy * dy * (q.vy - p.vy) + dx * dy * (q.vx - p.vx)) / (dx * dx + dy * dy),
                q.vx + (dx * dx * (p.vx - q.vx) + dx * dy * (p.vy - q.vy)) / (dx * dx + dy * dy),
                q.vy + (dy * dy * (p.vy - q.vy) + dx * dy * (p.vx - q.vx)) / (dx * dx + dy * dy),
            )
        };

        p.vx = nvx;
        p.vy = nvy;
        q.vx = nwx;
        q.vy = nwy;

        p.m = true;
        q.m = true;

        true
    }

    // bounce from the walls
    pub fn bounce(p: &mut Particle<N>, wmin: N, wmax: N, hmin: N, hmax: N) -> bool {
        let mut res = false;

        if p.px >= wmax {
            p.vx = p.vx.neg();
            p.px = wmax;
            res = true;
        }

        if p.px <= wmin {
            p.vx = p.vx.neg();
            p.px = wmin;
            res = true;
        }

        if p.py >= hmax {
            p.vy = p.vy.neg();
            p.py = hmax;
            res = true;
        }

        if p.py <= hmin {
            p.vy = p.vy.neg();
            p.py = hmin;
            res = true;
        }

        p.m = res;
        res
    }
}
