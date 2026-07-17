# mi_thermald

mi_thermald is a Rust-based thermal and charging management daemon for Android, reverse-engineered from Xiaomi's mi_thermald (decompiled in Ghidra) and tailored for sapphire/sapphiren devices. Unlike the original implementation, this version eliminates dependency on *.conf config files, instead embedding all thermal policies directly in the binary. It replaces standard thermal HALs to provide granular, efficient control over device temperatures, CPU frequency, GPU clocks, and battery charging limits.

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
WEIGHT_BATTERY: 0    // 0.0%  - Battery (disabled)
```

# Temperature Thresholds

```
CPU0: 39°C → 41°C → 45°C → 46°C
CPU4: 33°C → 35°C → 38°C → 42°C → 44.5°C → 45.5°C
GPU: 43°C → 45°C → 46°C
```

# Throttling Levels

· CPU0 (Little cores): 4 levels (1.9GHz → 691MHz)
· CPU4 (Big cores): 6 levels (2.8GHz → 806MHz)
· GPU: 3 levels (1.26GHz → 320MHz)

# Charging Control

· Screen on: Restricts charging to prevent overheating
· Screen off: Temperature-based current limiting (1.0A - 1.5A)
· Battery protection: Disables charging above 39°C

# Hotplug Management

· Disables cores 2,3,6 at 49°C or when battery < 6%
· Automatically re-enables when conditions normalize

# Monitoring

Real-time status output every 2 seconds:

```
V:42°C | B:36°C | C0:L2 C4:L3 G:L1 TS:L0 BL:255 W:0 Bs:0 HP:cb SOC:78% R:N
```

# Status Fields

· V: Virtual temperature
· B: Battery temperature
· C0/C4: CPU throttling levels
· G: GPU throttling level
· TS: Temperature state level
· BL: Backlight brightness (160=dimmed, 255=full)
· W: WiFi throttle state (0=off, 1=limited)
· Bs: Boost state (0=enabled, 1=disabled)
· HP: Hotplug status (C=CCC thermal, B=BCL battery)
· SOC: Battery percentage
· R: Charging restriction state (Y=active, N=inactive)

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
