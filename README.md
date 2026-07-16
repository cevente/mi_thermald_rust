# mi_thermald
 ​mi_thermald is a high-performance, ML-optimized thermal and charging management daemon written in Rust for Android. It replaces standard thermal HALs to provide granular, efficient control over device temperatures, CPU frequency, GPU clocks, and battery charging limits.  
 
# ​Features

# ​ML-Weighted Thermal Management:
Uses machine learning-derived importance scores (e.g., t_quiet driving 88% of thermal behavior) to calculate accurate virtual temperatures. 

# ​High-Performance I/O:
Utilizes cached file descriptors via SysfsNode, eliminating the performance overhead of repeatedly opening/closing sysfs nodes.  

# ​Resource Efficiency:
Written in Rust to provide memory safety and zero-cost abstractions, avoiding the common segmentation faults and memory leaks found in C++ thermal engines.  

# ​Proactive Protection:
Automatically throttles CPU/GPU and limits charging when critical temperature thresholds are reached, with specialized handling for screen-on vs. screen-off states.  
