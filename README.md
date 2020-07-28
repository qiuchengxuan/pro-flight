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
* WIP
  - [ ] Software interrupt based task scheduler
  - [ ] Complementary filter
* Future
  - [ ] GNSS NMEA Protocol
  - [ ] Blackbox
  - [ ] DMA based SDCARD read & write
  - [ ] Stabilizer
  - [ ] Navigation
  - [ ] EKF filter

data-flow
=========

```plantuml
@startuml
ditaa

+-----------+         +-----------+  Altitude                                        +----------+
| Barometer |-------->| Altimeter |-------------+-------------------------+       +->| Blackbox |
+-----------+         +-----------+             |                         |       |  +----------+
                                                v                         |       |
+------+ Position                        +-------------+                  |       |
| GNSS |---------------------+-------+-->| Speedometer |----------------+ |       |
+------+                     |       |   +-------------+                | |       |
                             v       |      ^                           | |       |
+---------------+        +-------+   |      |  Quaternion &             | |       |
| Accelerometer |------->|       |   |      |  Acceleration             | |       |
+---------------+        |  IMU  |----------+-------------------------+ | |       |
                   +---->|       |   |      |                         | | |       |
+---------------+  |     +-------+   v      v                         v v v       |
| Gyroscope     |--+         |     +------------+         Waypoint  +-----------+ |  +----------+
+---------------+            |     | Navigation |------------+----->| Telemetry |-+->| HUD(OSD) |
                        Gyro |     +------------+            |      +-----------+    +----------+
                             v                               v        ^
                       +------------+                     +-----+     |
                       | Stabilizer |                     | EAC |     |
                       +------------+                     +-----+     |
                             |                      +--------|--------+
                             +-----------------------------+ |      +-----------+    +----------+
                                                    |      | +----->|           |    |          |
+----------+ Remote controller +---------------+    |      +------->|   Mixer   |--->|   PWMs   |
| Receiver |------------------>| Configuration |----+-------------->|           |    |          |
+----------+                   +---------------+                    +-----------+    +----------+
@enduml
```
