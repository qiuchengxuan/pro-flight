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
* WIP
  - [ ] Software interrupt based task scheduler
  - [ ] Speed integral
* Future
  - [ ] GNSS based heading correction for IMU
  - [ ] GNSS NMEA Protocol
  - [ ] Blackbox
  - [ ] DMA based SDCARD read & write
  - [ ] Stabilizer
  - [ ] Navigation
  - [ ] CRC based OSD font check

data-flow
=========

```plantuml
@startuml
ditaa

+-----------+         +-----------+  Altitude & V/S                                  +----------+
| Barometer |-------->| Altimeter |---------------------------------------+       +->| Blackbox |
+-----------+         +-----------+                                       |       |  +----------+
                                                                          |       |
+---------------+        +-------+                                        |       |
| Accelerometer |------->|       |  Quaternion & Calibrated Acceleration  |       |
+---------------+        |  IMU  |----------+-------------------------+   |       |
                   +---->|       |          |                         |   |       |
+---------------+  |     +-------+          v                         v   v       |
| Gyroscope     |--+         |            +------------+  Waypoint  +-----------+ |  +----------+
+---------------+    +------------------->| Navigation |-----+----->| Telemetry |-+->| HUD(OSD) |
                     |       |            +------------+     |      +-----------+    +----------+
+------+             |       v Calibrated Gyro               v        ^
| GNSS |-------------+ +------------+                     +-----+     |
+------+    Position   | Stabilizer |                     | EAC |     |
                       +------------+                     +-----+     |
                             |                      +--------|--------+
                             +-----------------------------+ |      +-----------+    +----------+
                                                    |      | +----->|           |    |          |
+----------+ Remote controller +---------------+    |      +------->|   Mixer   |--->|   PWMs   |
| Receiver |------------------>| Configuration |----+-------------->|           |    |          |
+----------+                   +---------------+                    +-----------+    +----------+
@enduml
```
