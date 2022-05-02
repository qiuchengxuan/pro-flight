Flight control law protection
=============================

* bank angle restriction
  - When bank angle within ±33° without further control input, 
    FCS will maintain current bank angle.
  - When bank angle beyond ±33° and witin ±`max-roll`° without roll input, 
    FCS will reduce bank angle to ±33°
  - When bank angle exceed ±`max-roll`°, FCS will reduce bank angle within ±`max-roll`°
    even with roll input.
* pitch angle restriction
  - When pitch angle exceed `max-pitch`° or `min-pitch`°, 
    FCS will reduce pitch angle within `max-pitch`°-`min-pitch`° even with pitch input
  - Without pitch input, FCS will try to maintain 1g acceleration.
