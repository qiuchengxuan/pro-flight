pro-flight
=========

Flight control software written in rust, provides more flexibility for customize
and better OSD experience.

Working in progress

Progress
========

**Implemented**

* Component
  - [x] IMU
  - [x] Logger
  - [x] YAML-like config
  - [x] Displacement integral
  - [x] Complementary filter
* IO
  - [x] USB console serial
  - [x] DMA buffer descriptor
* HAL
  - [x] Gyroscope/Accelerometer
* Sensor
  - [x] MPU6000
  - [x] Battery Voltage ADC

**WIP**
  - [ ] Barometer
  - [ ] DMA based OSD/HUD

**Future**
  - [ ] GNSS NMEA Protocol
  - [ ] Magnetometer
  - [ ] QMC5883L
  - [ ] software interrupt based event
  - [ ] SDCARD read & write
  - [ ] PWM & ESC
  - [ ] SBUS Receiver
  - [ ] GNSS UBX Protocol
  - [ ] Stabilizer
  - [ ] DMA based I2C
  - [ ] INS calibration
  - [ ] Mavlink
  - [ ] Setup calibration
  - [ ] Blackbox
  - [ ] DMA based SDCARD read & write
  - [ ] Navigation
  - [ ] EKF filter
  - [ ] Involve async syntax
  - [ ] Camera distortion adaption

data-flow
=========

* Speedometer

  ```plantuml
  @startuml
  ditaa

  +-----------+ Pressure +-----------+       Derivative
  | Barometer |--------->| Altimeter |-----------------+
  +-----------+          +-----------+                 |
                                                       v
  +---------------+ Accel  +-------+            +-------------+
  | Accelerometer |------->|       |            |             |
  +---------------+        |       | Integral   |             |
                           |  IMU  |----------->| Speedometer |
  +-----------+     Gyro   |       |            |             |
  | Gyroscope |----------->|       |            |             |
  +-----------+            +-------+            +-------------+
                                                       ^
  +-----------+                           Velocity     |
  | GNSS      |----------------------------------------+
  +-----------+
  ```

* SINS

  ```plantuml
  @startuml
  ditaa

  +-----------+ Pressure +-----------+       Altitude
  | Barometer |--------->| Altimeter |-------------+
  +-----------+          +-----------+             |
                                                   v
  +---------------+ Accel  +-------+            +------+
  | Accelerometer |------->|       |            |      |
  +---------------+        |       | Integral   |      |
                           |  IMU  |-------+--->| SINS |
  +-----------+     Gyro   |       |       |    |      |
  | Gyroscope |----------->|       |       |    |      |
  +-----------+            +-------+       |    +------+
                                           |       ^
                        +-------------+    |       |
                        | Speedometer |----+       |
                        +-------------+            |
  +-----------+  Position                          |
  | GNSS      |------------------------------------+
  +-----------+
  ```

* Output

  ```plantuml
  @startuml
  ditaa
  +-------------+         +-----+
  | Speedometer |-------->|     |  AoA
  +-------------+         | AoA |----------------+
                     +--->|     |                |
                     |    +-----+                v
  +-----+ Attitude   |                     +-----------+
  | IMU |------------+-------------------->| Stablizer |-------+
  +-----+                                  +-----------+       |
                                                               |
  +------+ Steerpoint   +-----------+                          |
  | SINS |------------->| Autopilot |---------------+          |
  +------+              +-----------+               |          |      +-----------+    +----------+
                                                    |          +----->|           |    |          |
  +----------+ Remote controller +---------------+  +---------------->|   Mixer   |--->|   PWMs   |
  | Receiver |------------------>| Configuration |------------------->|           |    |          |
  +----------+                   +---------------+                    +-----------+    +----------+
  ```

* Telemetry

  ```plantuml
  @startuml
  ditaa

  +-------------+       Speed vector                            +----------+
  | Speedometer |----+------------------------+             +-->| Blackbox |
  +-------------+    |                        |             |   +----------+
                     |     +-----+            v             |
                     +---->|     |  AoA   +-----------+     |
                           | AoA |------->| Telemetry |-----+
                     +---->|     |        +-----------+     |
                     |     +-----+          ^ ^             |
  +-----+            |                      | |             |
  | IMU |------------+----------------------+ |             |
  +-----+    Attitude                         |             |
                                              |             |
  +------+   Postion & Steerpoint             |             |   +----------+
  | SINS |------------------------------------+             +-->| HUD(OSD) |
  +------+                                                      +----------+
  @enduml
  ```
