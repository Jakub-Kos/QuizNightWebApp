//! Analytic unit-step response of a damped spring.
#[derive(Clone, Copy, Debug)]
pub struct Spring {
    pub stiffness: f64,
    pub damping: f64,
    pub mass: f64,
}
impl Spring {
    pub fn new(stiffness: f64, damping: f64, mass: f64) -> Self {
        assert!(stiffness > 0.0 && damping > 0.0 && mass > 0.0);
        Self {
            stiffness,
            damping,
            mass,
        }
    }
    pub fn from_bounce(bounce: f64, duration: f64) -> Self {
        assert!((0.0..1.0).contains(&bounce) && duration > 0.0);
        let k = (std::f64::consts::TAU / duration).powi(2);
        Self::new(k, 2.0 * (1.0 - bounce) * k.sqrt(), 1.0)
    }
    /// Framer Motion's spring for `transition={{ type: "spring", duration, bounce }}` (and for
    /// `bounce` alone, with its default duration of 0.8 s): `findSpring` in motion-dom, which picks
    /// the stiffness whose envelope has decayed to 0.001 after `duration` seconds.
    pub fn from_duration(bounce: f64, duration: f64) -> Self {
        const SAFE_MIN: f64 = 0.001;
        let ratio = (1.0 - bounce).clamp(0.05, 1.0);
        let duration = duration.clamp(0.01, 10.0);
        let angular = move |w: f64| w * (1.0 - ratio * ratio).sqrt();
        let envelope = move |w: f64| {
            if ratio < 1.0 {
                let decay = w * ratio;
                SAFE_MIN - decay / angular(w) * (-decay * duration).exp()
            } else {
                -SAFE_MIN + (-w * duration).exp() * (w * duration + 1.0)
            }
        };
        let derivative = move |w: f64| {
            if ratio < 1.0 {
                let decay = w * ratio;
                let e = ratio * ratio * w * w * duration;
                let factor = if -envelope(w) + SAFE_MIN > 0.0 {
                    -1.0
                } else {
                    1.0
                };
                // motion-dom takes the angular frequency of w² here; kept for the same result
                factor * (-e * (-decay * duration).exp()) / angular(w * w)
            } else {
                (-w * duration).exp() * -w * duration * duration
            }
        };
        let mut w = 5.0 / duration;
        for _ in 1..12 {
            w -= envelope(w) / derivative(w);
        }
        if !w.is_finite() {
            return Self::new(100.0, 10.0, 1.0);
        }
        let stiffness = w * w;
        Self::new(stiffness, ratio * 2.0 * stiffness.sqrt(), 1.0)
    }
    pub fn position(self, t: f64) -> f64 {
        let w = (self.stiffness / self.mass).sqrt();
        let a = self.damping / (2.0 * self.mass);
        let d = a * a - w * w;
        if d.abs() < 1e-9 {
            1.0 - (-a * t).exp() * (1.0 + a * t)
        } else if d < 0.0 {
            let b = (-d).sqrt();
            1.0 - (-a * t).exp() * ((b * t).cos() + a / b * (b * t).sin())
        } else {
            let b = d.sqrt();
            let r1 = -a + b;
            let r2 = -a - b;
            1.0 + (r2 * (r1 * t).exp() - r1 * (r2 * t).exp()) / (r1 - r2)
        }
    }
    pub fn settle_time(self) -> f64 {
        let w = (self.stiffness / self.mass).sqrt();
        let a = self.damping / (2.0 * self.mass);
        let rate = if a > w { a - (a * a - w * w).sqrt() } else { a };
        let mut t = 10.0 / rate;
        while (self.position(t) - 1.0).abs() > 0.0001 {
            t *= 1.2;
        }
        t
    }
    pub fn to_css_linear(self) -> String {
        let duration = self.settle_time();
        let n = ((duration * (self.stiffness / self.mass).sqrt() * 24.0).ceil() as usize)
            .clamp(64, 4096);
        format!(
            "linear({})",
            (0..=n)
                .map(|i| format!(
                    "{:.6} {:.6}%",
                    if i == n {
                        1.0
                    } else {
                        self.position(duration * i as f64 / n as f64)
                    },
                    100.0 * i as f64 / n as f64
                ))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn response() {
        for b in [0.0, 0.25, 0.7] {
            let s = Spring::from_bounce(b, 1.2);
            assert!(s.position(0.0).abs() < 1e-9);
            assert!((s.position(s.settle_time()) - 1.0).abs() < 0.001);
            let overshoot = (0..1000).any(|i| s.position(i as f64 / 100.0) > 1.00001);
            assert_eq!(overshoot, b > 0.0);
            let css = s.to_css_linear();
            assert!(css.starts_with("linear("));
            let mut last = -1.0;
            for stop in css
                .trim_start_matches("linear(")
                .trim_end_matches(')')
                .split(',')
            {
                let p = stop
                    .split_whitespace()
                    .nth(1)
                    .unwrap()
                    .trim_end_matches('%')
                    .parse::<f64>()
                    .unwrap();
                assert!(p > last);
                last = p;
            }
        }
    }
    #[test]
    fn matches_framer_find_spring() {
        // findSpring({ duration, bounce }) from motion-dom 12, run in Node
        for (bounce, duration, stiffness, damping) in [
            (0.2, 0.8, 126.40214521096279, 17.988593378584795),
            (0.4, 0.8, 190.2142763013069, 16.55018301632589),
            (0.3, 0.6, 268.94068049682403, 22.95917537225096),
            (0.0, 1.0, 85.25592442711776, 18.466826952903173),
        ] {
            let s = Spring::from_duration(bounce, duration);
            assert!((s.stiffness - stiffness).abs() < 1e-6, "{bounce} {duration}");
            assert!((s.damping - damping).abs() < 1e-6, "{bounce} {duration}");
        }
    }
    #[test]
    fn numerical() {
        for damping in [4.0, 20.0, 30.0] {
            let s = Spring::new(100.0, damping, 1.0);
            let (mut x, mut v) = (0.0, 0.0);
            let dt = 0.00001;
            for i in 1..100001 {
                v += (100.0 * (1.0 - x) - damping * v) * dt;
                x += v * dt;
                if i % 1000 == 0 {
                    assert!((x - s.position(i as f64 * dt)).abs() < 0.0001);
                }
            }
        }
    }
}
