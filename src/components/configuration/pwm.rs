pub fn to_motor_pwm_duty(max_duty: u16, value: i16) -> u16 {
    let base = max_duty / 20; // 1.0ms
    let range = (max_duty / 20) as u32; // 1.0ms
    let throttle = (value as i32 + i16::MAX as i32 + 1) as u32;
    base + (range * throttle / u16::MAX as u32) as u16
}

pub fn to_servo_pwm_duty(max_duty: u16, value: i16, angle: i8) -> u16 {
    let base = max_duty / 40; // 0.5ms
    let range = (max_duty / 10) as u32; // 2.0ms
    let offset = angle as i32 * i16::MAX as i32 / 90;
    let mut signed = value as i32 + i16::MAX as i32 + 1 - offset; // [-32768, 32767] => [0, 65535]
    if signed > u16::MAX as i32 {
        signed = u16::MAX as i32;
    } else if signed < 0 {
        signed = 0;
    }
    base + (range * (signed as u32) / u16::MAX as u32) as u16
}

mod test {
    #[test]
    fn test_to_servo_pwm_duty() {
        use super::to_servo_pwm_duty;

        let max_duty = 180 * 10;
        let center = max_duty / 40 + max_duty / 20; // 0.5ms + 1.0ms
        assert_eq!(to_servo_pwm_duty(max_duty, 0, 0), center);
        assert_eq!(to_servo_pwm_duty(max_duty, -32768, 0), center - 90);
        assert_eq!(to_servo_pwm_duty(max_duty, 32767, 0), center + 90);
        assert_eq!(to_servo_pwm_duty(max_duty, -8192, 0), center - 23);
        assert_eq!(to_servo_pwm_duty(max_duty, 8192, 0), center + 22);
    }
}
