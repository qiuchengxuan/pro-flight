pub struct LPF<T> {
    alpha: T,
    value: T,
}

impl LPF<f32> {
    pub fn new(sample_rate: f32, freq: f32) -> Self {
        let rc = 1.0 / (2.0 * core::f32::consts::PI * freq);
        let alpha = 1.0 / (1.0 + rc * sample_rate);
        Self { alpha, value: 0.0 }
    }

    pub fn filter(&mut self, sample: f32) -> f32 {
        self.value = (1.0 - self.alpha) * self.value + self.alpha * sample;
        self.value
    }
}

impl LPF<u16> {
    pub fn new(sample_rate: f32, freq: f32) -> Self {
        let rc = 1.0 / (2.0 * core::f32::consts::PI * freq);
        let alpha = ((1.0 / (1.0 + rc * sample_rate)) * 1000.0) as u16;
        Self { alpha: alpha, value: 0 }
    }

    pub fn filter(&mut self, sample: u16) -> u16 {
        let alpha = self.alpha as usize;
        let value = self.value as usize;
        self.value = (((1000 - alpha) * value + alpha * sample as usize) / 1000) as u16;
        self.value
    }
}

mod test {
    #[test]
    fn test_lpf() {
        use super::LPF;

        let mut lpf = LPF::<f32>::new(10.0, 1.0);
        lpf.value = 3.335;
        let value0 = lpf.filter(3.295);
        let value1 = lpf.filter(3.295);
        assert!(3.295 < value1 && value1 < value0);
        let value2 = lpf.filter(3.295);
        assert!(3.295 < value2 && value2 < value1);
    }
}
