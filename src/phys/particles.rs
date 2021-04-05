use num_traits::Num;

#[derive(Debug, Clone, Copy)]
pub enum ParticleColor {
    GREEN,
    RED,
    BLUE,
    YELLOW,
    WHITE,
}

impl Default for ParticleColor {
    fn default() -> ParticleColor {
        ParticleColor::GREEN
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Particle<N>
where
    N: Default + Copy + Clone + Num + PartialOrd,
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
    N: Default + Copy + Clone + Num + PartialOrd,
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
    pub fn collide(p: &mut Particle<N>, q: &mut Particle<N>) -> bool {
        let dx = p.px - q.px;
        let dy = p.py - q.py;
        let rr = dx * dx + dy * dy;
        let dx1 = (p.px + p.vx) - (q.px + q.vx);
        let dy1 = (p.py + p.vy) - (q.py + q.vy);

        if rr > (p.r + p.r) * (p.r + p.r) {
            return false;
        }

        if rr < dx1 * dx1 + dy1 * dy1 {
            return false;
        }

        if N::is_zero(&rr) {
            p.m = true;
            q.m = true;
            return true;
        }

        let nvx: N = p.vx + (dx * dx * (q.vx - p.vx) + dx * dy * (q.vy - p.vy)) / rr;
        let nvy: N = p.vy + (dy * dy * (q.vy - p.vy) + dx * dy * (q.vx - p.vx)) / rr;

        let nwx: N = q.vx + (dx * dx * (p.vx - q.vx) + dx * dy * (p.vy - q.vy)) / rr;
        let nwy: N = q.vy + (dy * dy * (p.vy - q.vy) + dx * dy * (p.vx - q.vx)) / rr;

        p.vx = nvx;
        p.vy = nvy;
        q.vx = nwx;
        q.vy = nwy;

        p.m = true;
        q.m = true;

        true
    }

    // bounce from the walls
    pub fn bounce(p: &mut Particle<N>, w: N, h: N) -> bool {
        let mut res = false;

        if p.px >= w {
            p.vx = N::zero() - p.vx;
            p.px = w;
            res = true;
        }

        if p.px <= N::zero() {
            p.vx = N::zero() - p.vx;
            p.px = N::zero();
            res = true;
        }

        if p.py >= h {
            p.vy = N::zero() - p.vy;
            p.py = h;
            res = true;
        }

        if p.py <= N::zero() {
            p.vy = N::zero() - p.vy;
            p.py = N::zero();
            res = true;
        }

        p.m = res;
        res
    }
}
