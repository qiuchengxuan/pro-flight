rs-flight
=========

Flight controller written in Rust, working in progress

Progress
========

* Implemented
  - [x] USB console serial
  - [x] Barometer/Gyroscope/Accelerometer
  - [x] IMU
  - [x] DMA based OSD/HUD
  - [x] SDCARD read & write
  - [x] Logger
  - [x] YAML based config file
  - [x] Static memory allocator
  - [x] Battery Voltage ADC
  - [x] PWM & ESC
  - [x] SBUS Receiver
  - [x] GNSS UBX Protocol
  - [x] Displacement integral
  - [x] CRC based OSD font check
  - [x] timer based task scheduler
  - [x] software interrupt based event
  - [x] GNSS improved AHRS
* WIP
  - [ ] Complementary filter
* Future
  - [ ] GNSS NMEA Protocol
  - [ ] Blackbox
  - [ ] DMA based SDCARD read & write
  - [ ] Stabilizer
  - [ ] Navigation
  - [ ] EKF filter
  - [ ] Involve async syntax

Memory allocator
================

Fow now it's using alloc-cortex-m as memory allocator,
but there's some alternatives to choose, e.g.

* linked_list_allocator
* static-alloc

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
  
* GINS

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
                           |  IMU  |-------+--->| GINS |
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
  | GINS |------------->| Autopilot |---------------+          |
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
  | GINS |------------------------------------+             +-->| HUD(OSD) |
  +------+                                                      +----------+
  @enduml
  ```
