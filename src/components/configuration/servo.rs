pub fn to_pwm_duty(max_duty: u16, value: i16) -> u16 {
    let base = max_duty / 20; // 0.5ms
    let range = (max_duty / 10) as u32; // 2.0ms
    let unsigned = (value as i32 + i16::MAX as i32 + 1) as u32; // [-32768, 32767] => [0, 65535]
    base + (range * unsigned / u16::MAX as u32) as u16
}

mod test {
    #[test]
    fn test_to_pwm_duty() {
        use super::to_pwm_duty;

        let base = 90;
        assert_eq!(to_pwm_duty(1800, 0), base + 90);
        assert_eq!(to_pwm_duty(1800, -8192), base + 67);
        assert_eq!(to_pwm_duty(1800, 8192), base + 112);
        assert_eq!(to_pwm_duty(1800, -32768), base + 0);
        assert_eq!(to_pwm_duty(1800, 32767), base + 180);
    }
}
