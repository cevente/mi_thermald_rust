# mi_thermald

mi_thermald is a high-performance, ML-mi_thermald is a Rust-based thermal and charging management daemon for Android, reverse-engineered from Xiaomi's mi_thermald (decompiled in Ghidra) and tailored for sapphire/sapphiren devices. Unlike the original implementation, this version eliminates dependency on *.conf config files, instead embedding all thermal policies directly in the binary. It replaces standard thermal HALs to provide granular, efficient control over device temperatures, CPU frequency, GPU clocks, and battery charging limits.

# Features

# ML-Weighted Thermal Management

Uses machine learning-derived importance scores (e.g., t_quiet driving 88% of thermal behavior) to calculate accurate virtual temperatures.

# High-Performance I/O

Utilizes cached file descriptors via SysfsNode, eliminating the performance overhead of repeatedly opening/closing sysfs nodes.

# Resource Efficiency

Written in Rust to provide memory safety and zero-cost abstractions, avoiding the common segmentation faults and memory leaks found in C++ thermal engines.

# Proactive Protection

Automatically throttles CPU/GPU and limits charging when critical temperature thresholds are reached, with specialized handling for screen-on vs. screen-off states.

# Configuration

# ML Weights

```
WEIGHT_QUIET: 884    // 88.4% - Primary thermal indicator
WEIGHT_PA: 75        // 7.5%  - Power amplifier
WEIGHT_CHARGE: 28    // 2.8%  - Charging IC
WEIGHT_EMMC: 13      // 1.3%  - Storage
```

# Temperature Thresholds

```
THERMAL_SPIKE_THRESHOLD: 1500    // 1.5°C delta
PROACTIVE_CPU_TEMP_BOOST: 3000   // 3°C proactive offset
PROACTIVE_CHG_TEMP_THRESHOLD: 35000  // 35°C
```

# Throttling Levels

· CPU0 (Little cores): 4 levels (1.9GHz → 691MHz)
· CPU4 (Big cores): 6 levels (2.8GHz → 806MHz)
· GPU: 3 levels (1.26GHz → 320MHz)

# Monitoring

Real-time status output every 2 seconds:

```
V:42°C | B:36°C | Δ:+0.5°C | C0:L2 C4:L3 G:L1 SOC:78%
```

# Status Fields

· V: Virtual temperature
· B: Battery temperature
· Δ: Temperature rate of change
· C0/C4: CPU throttling levels
· G: GPU throttling level
· SOC: Battery percentage

# Safety Features

· Graceful degradation if sensors fail
· Proactive throttling prevents thermal runaway
· Battery protection at high temperatures
· Emergency core hotplugging at critical thresholds

# License

# MIT License

# Copyright (c) 2026 Cedric Loste

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

# Disclaimer

USE AT YOUR OWN RISK: This software directly manipulates system controls and can potentially cause hardware damage if misconfigured. Always test thoroughly on your specific hardware. The author assumes no responsibility for any damage caused by this software.
