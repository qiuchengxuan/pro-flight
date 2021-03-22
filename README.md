pro-flight
=========

Flight controller based on drone-os, working in progress

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
