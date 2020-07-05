data flow
=========

```mermaid
graph LR
  subgraph Input
    Receiver
    MotionSensor
    GNSS
  end

  PostionRing{{Ring<Position>}}
  AccelGyroRing{{Ring<Accel, Gyro>}}
  GNSS --> |10*12B|PostionRing
  MotionSensor --> |16*32B|AccelGyroRing
  MotionSensor --> Stabilizer

  PostionRing --> Navigation
  PostionRing --> GNSS-Speed

  AccelGyroRing --> IMU
  PostionRing --> IMU

  AccelQuatRing{{Ring<Accel, Quaternion>}}
  IMU --> |16*32B|AccelQuatRing
  AccelQuatRing --> Navigation
  AccelQuatRing --> SpeedIntegrator

  GNSS-Speed --> SpeedIntegrator

  IMU --> Autopilot
  SpeedIntegrator --> Autopilot
  Navigation --> Autopilot

  Receiver --> FlightControl
  Stabilizer --> FlightControl
  IMU --> EAC
  Autopilot --> EAC
  EAC --> FlightControl

  IMU --> TelemetryUnit
  SpeedIntegrator --> TelemetryUnit
  Navigation --> TelemetryUnit

  subgraph Output
	PWMs
    OSD
    BlackBox
  end

  FlightControl --> PWMs
  FlightControl --> BlackBox
  TelemetryUnit --> OSD
  TelemetryUnit --> BlackBox
```
