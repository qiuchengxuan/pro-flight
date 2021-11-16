PID tuning
==========

PID configuration looks like follows:

```yaml
pids:
  roll:
    max-rate: 30
    kp: 1.0
    ki: 1.0
    kd: 2.0
  pitch:
    max-rate: 10
    kp: 5.8
    ki: 5.0
    kd: 2.2
  yaw:
    max-rate: 10
    kp: 7.0
    ki: 4.5
    kd: 2.0
```

* max-rate

  Maximum rate (degrees) when input is 100%
  
* kp

  Multiplier that convert error (degrees) to servo percentage
  
  Assuming your aircraft maximum roll rate is 270 degrees, 100% / 270 = 0.37, 
  therefore pids.roll.kp should be 0.37
