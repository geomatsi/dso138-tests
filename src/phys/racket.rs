use core::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy, Default)]
pub struct Racket<N>
where
    N: Sub<Output = N> + Div<Output = N> + Mul<Output = N> + Add<Output = N> + Neg<Output = N>,
    N: Default + Copy + Clone + PartialOrd,
{
    cx: N,
    cy: N,
    hw: N,
    hh: N,
}

impl<N> Racket<N>
where
    N: Sub<Output = N> + Div<Output = N> + Mul<Output = N> + Add<Output = N> + Neg<Output = N>,
    N: Default + Copy + Clone + PartialOrd,
{
    pub fn new(cx: N, cy: N, hw: N, hh: N) -> Racket<N> {
        Racket { cx, cy, hw, hh }
    }

    pub fn step(&mut self, dx: N) {
        self.cx = self.cx + dx;
    }

    pub fn get_cx(&self) -> N {
        self.cx
    }

    pub fn get_cy(&self) -> N {
        self.cy
    }

    pub fn get_hw(&self) -> N {
        self.hw
    }

    pub fn get_hh(&self) -> N {
        self.hh
    }

    pub fn bounce(r: &mut Racket<N>, xmin: N, xmax: N) -> bool {
        let mut res = false;

        if r.cx >= xmax {
            r.cx = xmax;
            res = true;
        }

        if r.cx <= xmin {
            r.cx = xmin;
            res = true;
        }

        res
    }
}
