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

macro_rules! impl_lpf {
    ($type:ty, $aux_type:ty) => {
        impl LPF<$type> {
            pub fn new(sample_rate: f32, freq: f32) -> Self {
                let rc = 1.0 / (2.0 * core::f32::consts::PI * freq);
                let alpha = ((1.0 / (1.0 + rc * sample_rate)) * 1000.0) as $type;
                Self { alpha, value: 0 }
            }

            pub fn filter(&mut self, sample: $type) -> $type {
                let alpha = self.alpha as $aux_type;
                let value = self.value as $aux_type;
                self.value =
                    (((1000 - alpha) * value + alpha * sample as $aux_type) / 1000) as $type;
                self.value
            }
        }
    };
}

impl_lpf!(u16, u32);
impl_lpf!(i16, i32);

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
