use std::ops::Sub;

use alkahest::alkahest;
use tokio::time::Instant;

#[derive(Debug, Clone)]
#[alkahest(Formula, Serialize, Deserialize)]
pub struct StandingTransform {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f64,
}

impl Default for StandingTransform {
    fn default() -> Self {
        Self {
            x: 0f64,
            y: 0f64,
            z: 0f64,
            yaw: 0f64,
        }
    }
}

pub struct StandingTransformEncoder {
    target: StandingTransform,
    checked: bool,
    last_sent: StandingTransform,
    last_sent_at: Instant,
}

impl StandingTransform {
    pub fn decode(&self) -> (f64, f64, f64, f64, f64, f64, f64) {
        let xx = self.yaw.abs() - 1f64;
        let xz = (1f64 - xx.powi(2)).sqrt() * self.yaw.signum();
        let zx = -xz;
        let zz = xx;
        (self.x, self.y, self.z, xx, xz, zx, zz)
    }
}

impl StandingTransformEncoder {
    pub fn new() -> Self {
        Self {
            target: StandingTransform::default(),
            checked: false,
            last_sent: StandingTransform::default(),
            last_sent_at: Instant::now(),
        }
    }
}

impl Default for StandingTransformEncoder {
    fn default() -> Self {
        Self::new()
    }
}
impl StandingTransformEncoder {
    pub fn push(&mut self, target: StandingTransform) {
        self.target = target;
        self.checked = false;
    }
    pub fn payload(&mut self) -> Option<StandingTransform> {
        self.checked = true;
        let elapse = Instant::now().sub(self.last_sent_at).as_millis();
        if elapse < 50 {
            return None;
        }
        let elapse = match u32::try_from(elapse) {
            Ok(elapse) => f64::from(elapse),
            Err(_) => return None,
        };

        let threshold = 0.3f64 / elapse.ln_1p();
        if (self.target.x.sub(self.last_sent.x).powi(2)
            + self.target.y.sub(self.last_sent.y).powi(2)
            + self.target.z.sub(self.last_sent.z).powi(2)
            > threshold.powi(2))
            || (self.target.yaw.sub(self.last_sent.yaw).abs() > threshold)
        {
            self.last_sent_at = Instant::now();
            self.last_sent = self.target.clone();
            Some(self.target.clone())
        } else {
            None
        }
    }
}
