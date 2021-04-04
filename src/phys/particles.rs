#[derive(Debug, Clone, Copy)]
pub enum ParticleColor {
    GREEN,
    RED,
    BLUE,
    YELLOW,
    WHITE,
}

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    px: i32,
    py: i32,
    vx: i32,
    vy: i32,
    dt: i32,
    r: i32,
    m: bool,
    c: ParticleColor,
}

impl Default for Particle {
    fn default() -> Particle {
        Particle {
            px: 0,
            py: 0,
            vx: 0,
            vy: 0,
            dt: 1,
            r: 5,
            m: false,
            c: ParticleColor::GREEN,
        }
    }
}

impl Particle {
    pub fn new(px: i32, py: i32, vx: i32, vy: i32, r: i32, dt: i32, c: ParticleColor) -> Particle {
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

    pub fn get_x(&self) -> i32 {
        self.px
    }

    pub fn get_y(&self) -> i32 {
        self.py
    }

    pub fn get_r(&self) -> i32 {
        self.r
    }

    pub fn get_color(&self) -> ParticleColor {
        self.c
    }

    pub fn set_color(&mut self, c: ParticleColor) {
        self.c = c;
    }

    pub fn step(&mut self) {
        self.px += self.vx * self.dt;
        self.py += self.vy * self.dt;
        self.m = false;
    }

    pub fn reflect(&mut self, cx: i32, cy: i32) {
        self.vx *= cx;
        self.vy *= cy;
    }

    pub fn energy(&self) -> u64 {
        (self.vx * self.vx + self.vy * self.vy) as u64
    }

    pub fn collided(&self) -> bool {
        self.m
    }

    // perfectly elastic collision of two equal round particles
    pub fn collide(p: &mut Particle, q: &mut Particle) -> bool {
        let dx = p.px - q.px;
        let dy = p.py - q.py;
        let rr = dx * dx + dy * dy;
        let dx1 = (p.px + p.vx) - (q.px + q.vx);
        let dy1 = (p.py + p.vy) - (q.py + q.vy);

        if rr == 0 {
            p.m = true;
            q.m = true;
            return true;
        }

        if dx * dx + dy * dy > 4 * p.r * p.r {
            return false;
        }

        if dx * dx + dy * dy < dx1 * dx1 + dy1 * dy1 {
            return false;
        }

        let nvx: i32 = p.vx + (dx * dx * (q.vx - p.vx) + dx * dy * (q.vy - p.vy)) / rr;
        let nvy: i32 = p.vy + (dy * dy * (q.vy - p.vy) + dx * dy * (q.vx - p.vx)) / rr;

        let nwx: i32 = q.vx + (dx * dx * (p.vx - q.vx) + dx * dy * (p.vy - q.vy)) / rr;
        let nwy: i32 = q.vy + (dy * dy * (p.vy - q.vy) + dx * dy * (p.vx - q.vx)) / rr;

        p.vx = nvx;
        p.vy = nvy;
        q.vx = nwx;
        q.vy = nwy;

        p.m = true;
        q.m = true;

        true
    }

    // bounce from the walls
    pub fn bounce(p: &mut Particle, w: i32, h: i32) -> bool {
        let mut res = false;

        if p.px >= w {
            p.reflect(-1, 1);
            p.px = w;
            res = true;
        }

        if p.px <= 0 {
            p.reflect(-1, 1);
            p.px = 0;
            res = true;
        }

        if p.py >= h {
            p.reflect(1, -1);
            p.py = h;
            res = true;
        }

        if p.py <= 0 {
            p.reflect(1, -1);
            p.py = 0;
            res = true;
        }

        p.m = res;
        res
    }
}
