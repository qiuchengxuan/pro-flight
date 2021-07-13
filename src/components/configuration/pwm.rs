pub fn to_motor_pwm_duty(max_duty: u16, rate: u16, value: i16) -> u16 {
    let duty_per_ms = max_duty as u32 * rate as u32 / 1000;
    let throttle = (value as i32 + i16::MAX as i32 + 1) as u32;
    (duty_per_ms + duty_per_ms * throttle / u16::MAX as u32) as u16
}

fn angle_to_axis(angle: i8) -> i32 {
    angle as i32 * i16::MAX as i32 / 90
}

pub fn to_servo_pwm_duty(max_duty: u16, value: i16, min: i8, max: i8, reversed: bool) -> u16 {
    let base = max_duty / 40; // 0.5ms
    let range = (max_duty / 10) as u32; // 2.0ms
    let offset = angle_to_axis((max + min) / 2);
    let mut value = if reversed { -value } else { value } as i32 + offset;
    let max = angle_to_axis(max);
    if value > max {
        value = max;
    }
    let min = angle_to_axis(min);
    if value < min {
        value = min;
    }
    let signed = value as i32 + i16::MAX as i32 + 1; // [-32767, 32767] => [1, 65535]
    base + (range * (signed as u32) / u16::MAX as u32) as u16
}

mod test {
    #[test]
    fn test_to_motor_pwm_duty() {
        use super::to_motor_pwm_duty;

        let max_duty = 20000;
        assert_eq!(to_motor_pwm_duty(max_duty, 400, -32768), 8000);
        assert_eq!(to_motor_pwm_duty(max_duty, 400, 0), 12000);
        assert_eq!(to_motor_pwm_duty(max_duty, 400, 32767), 16000);
    }

    #[test]
    fn test_to_servo_pwm_duty() {
        use super::to_servo_pwm_duty;

        let max_duty = 180 * 10;
        let center = max_duty / 40 + max_duty / 20; // 0.5ms + 1.0ms
        assert_eq!(to_servo_pwm_duty(max_duty, 0, -90, 90, false), center);
        assert_eq!(to_servo_pwm_duty(max_duty, -32768, -90, 90, false), center - 90);
        assert_eq!(to_servo_pwm_duty(max_duty, 32767, -90, 90, false), center + 90);
        assert_eq!(to_servo_pwm_duty(max_duty, -8192, -90, 90, false), center - 23);
        assert_eq!(to_servo_pwm_duty(max_duty, 8192, -90, 90, false), center + 22);
    }
}
