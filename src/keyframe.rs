use num_rational::Rational64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyframeInterpolation {
    Linear,
    Hold,
    Bezier,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ControlPoint {
    pub time_offset: f64,
    pub value_offset: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe<T> {
    pub time: Rational64,
    pub value: T,
    pub interpolation: KeyframeInterpolation,
    pub in_handle: Option<ControlPoint>,
    pub out_handle: Option<ControlPoint>,
}

impl<T> Keyframe<T> {
    pub fn new(time: Rational64, value: T, interpolation: KeyframeInterpolation) -> Self {
        Self {
            time,
            value,
            interpolation,
            in_handle: None,
            out_handle: None,
        }
    }

    pub fn with_handles(
        time: Rational64,
        value: T,
        in_handle: ControlPoint,
        out_handle: ControlPoint,
    ) -> Self {
        Self {
            time,
            value,
            interpolation: KeyframeInterpolation::Bezier,
            in_handle: Some(in_handle),
            out_handle: Some(out_handle),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyframeTrack<T> {
    pub name: String,
    pub default_value: T,
    pub keyframes: Vec<Keyframe<T>>,
}

impl<T: Clone> KeyframeTrack<T> {
    pub fn new(name: impl Into<String>, default_value: T) -> Self {
        Self {
            name: name.into(),
            default_value,
            keyframes: Vec::new(),
        }
    }

    pub fn add_keyframe(&mut self, keyframe: Keyframe<T>) {
        let pos = self
            .keyframes
            .binary_search_by_key(&keyframe.time, |k| k.time);
        match pos {
            Ok(idx) => self.keyframes[idx] = keyframe,
            Err(idx) => self.keyframes.insert(idx, keyframe),
        }
    }

    pub fn is_animated(&self) -> bool {
        !self.keyframes.is_empty()
    }
}

impl KeyframeTrack<f64> {
    pub fn evaluate_at(&self, time: Rational64) -> f64 {
        if self.keyframes.is_empty() {
            return self.default_value;
        }

        if time <= self.keyframes[0].time {
            return self.keyframes[0].value;
        }

        if time >= self.keyframes.last().unwrap().time {
            return self.keyframes.last().unwrap().value;
        }

        // Find surrounding keyframes
        let mut idx = 0;
        while idx < self.keyframes.len() - 1 && self.keyframes[idx + 1].time <= time {
            idx += 1;
        }

        let k1 = &self.keyframes[idx];
        let k2 = &self.keyframes[idx + 1];

        match k1.interpolation {
            KeyframeInterpolation::Hold => k1.value,
            KeyframeInterpolation::Linear => {
                let dt = (k2.time - k1.time).to_f64().unwrap_or(1.0);
                let t = if dt.abs() < 1e-9 {
                    0.0
                } else {
                    ((time - k1.time).to_f64().unwrap_or(0.0) / dt).clamp(0.0, 1.0)
                };
                k1.value + (k2.value - k1.value) * t
            }
            KeyframeInterpolation::Bezier => {
                let dt = (k2.time - k1.time).to_f64().unwrap_or(1.0);
                let t = if dt.abs() < 1e-9 {
                    0.0
                } else {
                    ((time - k1.time).to_f64().unwrap_or(0.0) / dt).clamp(0.0, 1.0)
                };
                // Cubic bezier smoothstep approximation
                let smooth_t = t * t * (3.0 - 2.0 * t);
                k1.value + (k2.value - k1.value) * smooth_t
            }
        }
    }
}

trait ToF64 {
    fn to_f64(&self) -> Option<f64>;
}

impl ToF64 for Rational64 {
    fn to_f64(&self) -> Option<f64> {
        Some(*self.numer() as f64 / *self.denom() as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_linear_interpolation() {
        let mut track = KeyframeTrack::new("opacity", 1.0);
        track.add_keyframe(Keyframe::new(
            Rational64::new(0, 1),
            0.0,
            KeyframeInterpolation::Linear,
        ));
        track.add_keyframe(Keyframe::new(
            Rational64::new(10, 1),
            100.0,
            KeyframeInterpolation::Linear,
        ));

        assert_eq!(track.evaluate_at(Rational64::new(0, 1)), 0.0);
        assert_eq!(track.evaluate_at(Rational64::new(5, 1)), 50.0);
        assert_eq!(track.evaluate_at(Rational64::new(10, 1)), 100.0);
        assert_eq!(track.evaluate_at(Rational64::new(-1, 1)), 0.0);
        assert_eq!(track.evaluate_at(Rational64::new(15, 1)), 100.0);
    }

    #[test]
    fn test_keyframe_hold() {
        let mut track = KeyframeTrack::new("step", 0.0);
        track.add_keyframe(Keyframe::new(
            Rational64::new(0, 1),
            10.0,
            KeyframeInterpolation::Hold,
        ));
        track.add_keyframe(Keyframe::new(
            Rational64::new(5, 1),
            20.0,
            KeyframeInterpolation::Hold,
        ));

        assert_eq!(track.evaluate_at(Rational64::new(2, 1)), 10.0);
        assert_eq!(track.evaluate_at(Rational64::new(5, 1)), 20.0);
    }
}
