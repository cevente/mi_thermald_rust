#!/system/bin/sh
# Thermal & Power ML Data Collection Script
# Saves data to /sdcard/ThermalML/

# ==============================================================================
# 1. SYSFS PATHS (Aligned with your Rust implementation)
# ==============================================================================
TZ_PA="/sys/class/thermal/thermal_zone18/temp"
TZ_QUIET="/sys/class/thermal/thermal_zone19/temp"
TZ_CHARGE="/sys/class/thermal/thermal_zone20/temp"
TZ_EMMC="/sys/class/thermal/thermal_zone21/temp"
TZ_BATTERY="/sys/class/thermal/thermal_zone34/temp"

BAT_SOC="/sys/class/power_supply/battery/capacity"
SCREEN_STATE="/sys/class/thermal/thermal_message/screen_state"

# For ML, tracking Current Frequency is more useful than Max Frequency
CPU0_FREQ="/sys/devices/system/cpu/cpufreq/policy0/scaling_cur_freq"
CPU4_FREQ="/sys/devices/system/cpu/cpufreq/policy4/scaling_cur_freq"
GPU_FREQ="/sys/class/kgsl/kgsl-3d0/gpuclk"

CPU2_ON="/sys/devices/system/cpu/cpu2/online"
CPU3_ON="/sys/devices/system/cpu/cpu3/online"
CPU6_ON="/sys/devices/system/cpu/cpu6/online"

CHG_LIMIT="/sys/class/power_supply/battery/charge_control_limit"
RESTRICT_CHG="/sys/class/qcom-battery/restrict_chg"
RESTRICT_CUR="/sys/class/qcom-battery/restrict_cur"

# ==============================================================================
# 2. HELPER FUNCTIONS
# ==============================================================================
# Efficiently reads a sysfs node without spawning subshells
read_node() {
    if [ -r "$1" ]; then
        read -r val < "$1"
        echo "${val:-0}" # Return value, or 0 if empty
    else
        echo "0" # Return 0 if file unreadable/missing
    fi
}

# ==============================================================================
# 3. INTERACTIVE SCENARIO MENU
# ==============================================================================
echo "========================================================"
echo "  Thermal ML Data Collector - Scenario Labeling"
echo "========================================================"
echo "Select the current scenario to label the ML dataset:"
echo "  1) Normal Usage (Scrolling, UI navigation)"
echo "  2) Gaming (Heavy CPU/GPU load)"
echo "  3) Video / YouTube (Media decoding load)"
echo "  4) Screen Off (Idle / Sleep)"
echo "  5) Screen Off (Charging)"
echo "  6) Screen On  (Charging + Usage)"
echo "  7) Custom..."
echo "========================================================"
printf "Enter choice [1-7]: "
read choice

case $choice in
    1) LABEL="normal_usage" ;;
    2) LABEL="gaming" ;;
    3) LABEL="video_playback" ;;
    4) LABEL="screen_off_idle" ;;
    5) LABEL="screen_off_charging" ;;
    6) LABEL="screen_on_charging" ;;
    7) 
        printf "Enter custom label (no spaces): "
        read LABEL
        ;;
    *) 
        LABEL="unlabeled"
        ;;
esac

# ==============================================================================
# 4. CSV SETUP
# ==============================================================================
OUT_DIR="/sdcard/ThermalML"
mkdir -p "$OUT_DIR"

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
FILE_NAME="${OUT_DIR}/thermal_dataset_${LABEL}_${TIMESTAMP}.csv"

# Write the CSV Header
echo "timestamp_sec,scenario,t_pa,t_quiet,t_charge,t_emmc,t_battery,soc,screen_state,cpu0_freq,cpu4_freq,gpu_freq,cpu2_on,cpu3_on,cpu6_on,chg_limit,restrict_chg,restrict_cur" > "$FILE_NAME"

echo "Started capturing data for: $LABEL"
echo "Saving to: $FILE_NAME"
echo "Press [CTRL+C] to stop recording."
echo "--------------------------------------------------------"

# ==============================================================================
# 5. DATA COLLECTION LOOP
# ==============================================================================
while true; do
    # Get current epoch time for time-series ML
    TS=$(date +%s)

    # Read Temperatures
    t_pa=$(read_node "$TZ_PA")
    t_quiet=$(read_node "$TZ_QUIET")
    t_charge=$(read_node "$TZ_CHARGE")
    t_emmc=$(read_node "$TZ_EMMC")
    t_bat=$(read_node "$TZ_BATTERY")

    # Read Battery & Screen
    soc=$(read_node "$BAT_SOC")
    screen=$(read_node "$SCREEN_STATE")

    # Read Frequencies
    cpu0_f=$(read_node "$CPU0_FREQ")
    cpu4_f=$(read_node "$CPU4_FREQ")
    gpu_f=$(read_node "$GPU_FREQ")

    # Read Core states
    c2=$(read_node "$CPU2_ON")
    c3=$(read_node "$CPU3_ON")
    c6=$(read_node "$CPU6_ON")

    # Read Charging state
    chg_lim=$(read_node "$CHG_LIMIT")
    r_chg=$(read_node "$RESTRICT_CHG")
    r_cur=$(read_node "$RESTRICT_CUR")

    # Construct CSV row
    ROW="${TS},${LABEL},${t_pa},${t_quiet},${t_charge},${t_emmc},${t_bat},${soc},${screen},${cpu0_f},${cpu4_f},${gpu_f},${c2},${c3},${c6},${chg_lim},${r_chg},${r_cur}"

    # Append to file
    echo "$ROW" >> "$FILE_NAME"

    # Print a tiny visual indicator to the console so you know it's working
    printf "\r[LOG] %s | T_Bat: %.1f°C | SoC: %s%% | CPU4: %s Hz  " "$(date +%H:%M:%S)" "$((t_bat / 1000))" "$soc" "$cpu4_f"

    # Sleep for 2 seconds (Matches the Rust loop `dt`)
    sleep 2
done
