use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::thread;
use std::time::Duration;

// ── Sensor paths ────────────────────────────────────────────────────────────
const TZ_PA: &str = "/sys/class/thermal/thermal_zone18/temp";
const TZ_QUIET: &str = "/sys/class/thermal/thermal_zone19/temp";
const TZ_CHARGE: &str = "/sys/class/thermal/thermal_zone20/temp";
const TZ_EMMC: &str = "/sys/class/thermal/thermal_zone21/temp";
const TZ_BATTERY: &str = "/sys/class/thermal/thermal_zone34/temp";
const BAT_SOC_PATH: &str = "/sys/class/power_supply/battery/capacity";
const SCREEN_STATE_NODE: &str = "/sys/class/thermal/thermal_message/screen_state";

// ── Thermal control paths ───────────────────────────────────────────────────
const CPU0_MAX_FREQ: &str = "/sys/devices/system/cpu/cpufreq/policy0/scaling_max_freq";
const CPU4_MAX_FREQ: &str = "/sys/devices/system/cpu/cpufreq/policy4/scaling_max_freq";
const GPU_MAX_FREQ: &str = "/sys/class/kgsl/kgsl-3d0/max_gpuclk";
const CPU2_ONLINE: &str = "/sys/devices/system/cpu/cpu2/online";
const CPU3_ONLINE: &str = "/sys/devices/system/cpu/cpu3/online";
const CPU6_ONLINE: &str = "/sys/devices/system/cpu/cpu6/online";
const BACKLIGHT_PATH: &str = "/sys/class/thermal/thermal_message/thermal_max_brightness";
const TEMP_STATE_PATH: &str = "/sys/class/thermal/thermal_message/temp_state";
const WIFI_LIMIT_PATH: &str = "/sys/class/thermal/thermal_message/wifi_limit";
const BOOST_LIMIT_PATH: &str = "/sys/module/cpu_boost/parameters/input_boost_enabled";

// ── Charging control paths ──────────────────────────────────────────────────
const CHARGE_LIMIT_NODE: &str = "/sys/class/power_supply/battery/charge_control_limit";
const RESTRICT_CHG_NODE: &str = "/sys/class/qcom-battery/restrict_chg";
const RESTRICT_CUR_NODE: &str = "/sys/class/qcom-battery/restrict_cur";

// ── Virtual sensor parameters (Optimized via RF Predictive ML) ──────────────
const WEIGHT_QUIET: i32 = 884;
const WEIGHT_PA: i32 = 75;
const WEIGHT_CHARGE: i32 = 28;
const WEIGHT_EMMC: i32 = 13;
const WEIGHT_BATTERY: i32 = 0;
const WEIGHT_SUM: i32 = 1000;
const COMPENSATION: i32 = 0;

// ── Thermal stepwise/monitor configurations ────────────────────────────────
const CPU0_TRIG: [i32; 4] = [39000, 41000, 45000, 46000];
const CPU0_CLR: [i32; 4] = [37000, 39000, 44000, 45000];
const CPU0_TARGET: [i32; 4] = [1804800, 1516800, 1190400, 691200];
const CPU0_DEFAULT: i32 = 1900800;

const CPU4_TRIG: [i32; 6] = [33000, 35000, 38000, 42000, 44500, 45500];
const CPU4_CLR: [i32; 6] = [31000, 33000, 36000, 40000, 43000, 44500];
const CPU4_TARGET: [i32; 6] = [2592000, 2400000, 2208000, 1766400, 1344000, 806400];
const CPU4_DEFAULT: i32 = 2803200;

const GPU_TRIG: [i32; 3] = [43000, 45000, 46000];
const GPU_CLR: [i32; 3] = [41000, 43000, 45000];
const GPU_FREQS: [i32; 7] = [1260000000, 1114800000, 1025000000, 785000000, 600000000, 465000000, 320000000];
const GPU_TARGET_INDICES: [usize; 3] = [2, 4, 5];

const TSTATE_TRIG: [i32; 4] = [46000, 48000, 51000, 53000];
const TSTATE_CLR: [i32; 4] = [44000, 46000, 50000, 51000];
const TSTATE_TARGET: [i32; 4] = [110100000, 110100004, 112300001, 112520001];

// ── High-Performance I/O Wrapper ────────────────────────────────────────────
struct SysfsNode {
    file: std::fs::File,
    buffer: String,
    path: &'static str,
}

impl SysfsNode {
    fn new(path: &'static str, read_only: bool) -> Option<Self> {
        match OpenOptions::new()
            .read(true)
            .write(!read_only)
            .open(path)
        {
            Ok(file) => Some(Self {
                file,
                buffer: String::with_capacity(32),
                path,
            }),
            Err(e) => {
                eprintln!("Warning: Could not open {} ({})", path, e);
                None
            }
        }
    }

    fn read(&mut self) -> Option<i32> {
        self.file.seek(SeekFrom::Start(0)).ok()?;
        self.buffer.clear();
        self.file.read_to_string(&mut self.buffer).ok()?;
        self.buffer.trim().parse().ok()
    }

    fn write(&mut self, value: i32) {
        if let Err(e) = self.file.seek(SeekFrom::Start(0)) {
            eprintln!("Failed to seek {}: {}", self.path, e);
            return;
        }
        if let Err(e) = self.file.set_len(0) {
            eprintln!("Failed to truncate {}: {}", self.path, e);
            return;
        }
        let val_str = format!("{}\n", value);
        if let Err(e) = self.file.write_all(val_str.as_bytes()) {
            eprintln!("Failed to write {} to {}: {}", value, self.path, e);
        }
        if let Err(e) = self.file.sync_all() {
            eprintln!("Failed to sync {}: {}", self.path, e);
        }
    }
}

// ── Helper for optional writes ─────────────────────────────────────────────
macro_rules! write_opt {
    ($node:expr, $value:expr) => {
        if let Some(n) = &mut $node {
            n.write($value);
        }
    };
}

// ── Main ────────────────────────────────────────────────────────────────────
fn main() {
    println!("============================================================");
    println!(" Unified Thermal & Charging Daemon (Rust)");
    println!(" ML-Optimized weights | Proactive I/O Cached Mode");
    println!("============================================================");

    // Initialise cached sensor nodes
    let mut node_t_pa = SysfsNode::new(TZ_PA, true);
    let mut node_t_quiet = SysfsNode::new(TZ_QUIET, true);
    let mut node_t_charge = SysfsNode::new(TZ_CHARGE, true);
    let mut node_t_emmc = SysfsNode::new(TZ_EMMC, true);
    let mut node_t_battery = SysfsNode::new(TZ_BATTERY, true);
    let mut node_soc = SysfsNode::new(BAT_SOC_PATH, true);
    let mut node_screen = SysfsNode::new(SCREEN_STATE_NODE, true);

    // Initialise cached control nodes
    let mut node_cpu0 = SysfsNode::new(CPU0_MAX_FREQ, false);
    let mut node_cpu4 = SysfsNode::new(CPU4_MAX_FREQ, false);
    let mut node_gpu = SysfsNode::new(GPU_MAX_FREQ, false);
    let mut node_cpu2_on = SysfsNode::new(CPU2_ONLINE, false);
    let mut node_cpu3_on = SysfsNode::new(CPU3_ONLINE, false);
    let mut node_cpu6_on = SysfsNode::new(CPU6_ONLINE, false);
    let mut node_backlight = SysfsNode::new(BACKLIGHT_PATH, false);
    let mut node_tstate = SysfsNode::new(TEMP_STATE_PATH, false);
    let mut node_wifi = SysfsNode::new(WIFI_LIMIT_PATH, false);
    let mut node_boost = SysfsNode::new(BOOST_LIMIT_PATH, false);
    let mut node_chg_limit = SysfsNode::new(CHARGE_LIMIT_NODE, false);
    let mut node_res_chg = SysfsNode::new(RESTRICT_CHG_NODE, false);
    let mut node_res_cur = SysfsNode::new(RESTRICT_CUR_NODE, false);

    // Initial charging state
    write_opt!(node_res_chg, 0);
    write_opt!(node_res_cur, 1000000);

    // State tracking
    let mut state_cpu0: usize = 0;
    let mut state_cpu4: usize = 0;
    let mut state_gpu: usize = 0;
    let mut state_tstate: usize = 0;
    let mut state_backlight: i32 = 255;
    let mut state_wifi: i32 = 0;
    let mut state_boost: i32 = 0;
    let mut state_ccc_hotplug: bool = false;
    let mut state_bcl_hotplug: bool = false;

    let mut restricted: bool = false;
    let mut prev_screen_state: i32 = 0;

    // ── Runtime loop ──────────────────────────────────────────────────────
    loop {
        // ── Read sensors ──────────────────────────────────────────────────
        let t_pa = node_t_pa.as_mut().and_then(|n| n.read()).unwrap_or(0);
        let t_quiet = node_t_quiet.as_mut().and_then(|n| n.read()).unwrap_or(0);
        let t_charge = node_t_charge.as_mut().and_then(|n| n.read()).unwrap_or(0);
        let t_emmc = node_t_emmc.as_mut().and_then(|n| n.read()).unwrap_or(0);
        let t_battery = node_t_battery.as_mut().and_then(|n| n.read()).unwrap_or(0);
        let soc = node_soc.as_mut().and_then(|n| n.read()).unwrap_or(100);
        let screen_state = node_screen.as_mut().and_then(|n| n.read()).unwrap_or(0);

        // ── Virtual temperature ──────────────────────────────────────────
        let virtual_temp = (WEIGHT_QUIET * t_quiet
            + WEIGHT_PA * t_pa
            + WEIGHT_CHARGE * t_charge
            + WEIGHT_EMMC * t_emmc
            + WEIGHT_BATTERY * t_battery)
            / WEIGHT_SUM
            + COMPENSATION;
        let virtual_c = virtual_temp / 1000;
        let batt_temp = t_battery / 1000;

        // ── CPU0 ──────────────────────────────────────────────────────────
        for i in (0..4).rev() {
            if virtual_temp >= CPU0_TRIG[i] && state_cpu0 <= i {
                state_cpu0 = i + 1;
                println!("[CPU0] Up -> L{} ({}Hz)", state_cpu0, CPU0_TARGET[i]);
                write_opt!(node_cpu0, CPU0_TARGET[i]);
                break;
            }
        }
        for i in 0..4 {
            if virtual_temp <= CPU0_CLR[i] && state_cpu0 > i {
                state_cpu0 = i;
                let val = if i == 0 { CPU0_DEFAULT } else { CPU0_TARGET[i - 1] };
                println!("[CPU0] Down -> L{} ({}Hz)", state_cpu0, val);
                write_opt!(node_cpu0, val);
                break;
            }
        }

        // ── CPU4 ──────────────────────────────────────────────────────────
        for i in (0..6).rev() {
            if virtual_temp >= CPU4_TRIG[i] && state_cpu4 <= i {
                state_cpu4 = i + 1;
                println!("[CPU4] Up -> L{} ({}Hz)", state_cpu4, CPU4_TARGET[i]);
                write_opt!(node_cpu4, CPU4_TARGET[i]);
                break;
            }
        }
        for i in 0..6 {
            if virtual_temp <= CPU4_CLR[i] && state_cpu4 > i {
                state_cpu4 = i;
                let val = if i == 0 { CPU4_DEFAULT } else { CPU4_TARGET[i - 1] };
                println!("[CPU4] Down -> L{} ({}Hz)", state_cpu4, val);
                write_opt!(node_cpu4, val);
                break;
            }
        }

        // ── GPU ───────────────────────────────────────────────────────────
        let mut new_gpu = state_gpu;
        for i in (0..3).rev() {
            if virtual_temp >= GPU_TRIG[i] && state_gpu <= i {
                new_gpu = i + 1;
                break;
            }
        }
        if new_gpu <= state_gpu {
            for i in 0..3 {
                if virtual_temp <= GPU_CLR[i] && state_gpu > i {
                    new_gpu = i;
                    break;
                }
            }
        }
        if new_gpu != state_gpu {
            state_gpu = new_gpu;
            if state_gpu == 0 {
                println!("[GPU] Restored to {}Hz", GPU_FREQS[0]);
                write_opt!(node_gpu, GPU_FREQS[0]);
            } else {
                let freq = GPU_FREQS[GPU_TARGET_INDICES[state_gpu - 1]];
                println!("[GPU] Throttled L{} -> {}Hz", state_gpu, freq);
                write_opt!(node_gpu, freq);
            }
        }

        // ── Temp State ────────────────────────────────────────────────────
        let mut new_tstate = state_tstate;
        for i in (0..4).rev() {
            if virtual_temp >= TSTATE_TRIG[i] && state_tstate <= i {
                new_tstate = i + 1;
                break;
            }
        }
        if new_tstate <= state_tstate {
            for i in 0..4 {
                if virtual_temp <= TSTATE_CLR[i] && state_tstate > i {
                    new_tstate = i;
                    break;
                }
            }
        }
        if new_tstate != state_tstate {
            state_tstate = new_tstate;
            if state_tstate == 0 {
                println!("[TEMP_STATE] Cleared");
                write_opt!(node_tstate, 0);
            } else {
                let val = TSTATE_TARGET[state_tstate - 1];
                println!("[TEMP_STATE] Set to {}", val);
                write_opt!(node_tstate, val);
            }
        }

        // ── Backlight ─────────────────────────────────────────────────────
        if virtual_temp >= 51000 && state_backlight == 255 {
            state_backlight = 160;
            println!("[BACKLIGHT] Limited to 160");
            write_opt!(node_backlight, 160);
        } else if virtual_temp <= 49000 && state_backlight == 160 {
            state_backlight = 255;
            println!("[BACKLIGHT] Restored to full");
            write_opt!(node_backlight, 255);
        }

        // ── WiFi ──────────────────────────────────────────────────────────
        if virtual_temp >= 46000 && state_wifi == 0 {
            state_wifi = 1;
            println!("[WIFI] Limited");
            write_opt!(node_wifi, 1);
        } else if virtual_temp <= 44000 && state_wifi == 1 {
            state_wifi = 0;
            println!("[WIFI] Limit removed");
            write_opt!(node_wifi, 0);
        }

        // ── Boost ─────────────────────────────────────────────────────────
        if virtual_temp >= 48000 && state_boost == 0 {
            state_boost = 1;
            println!("[BOOST] Disabled");
            write_opt!(node_boost, 0);
        } else if virtual_temp <= 46000 && state_boost == 1 {
            state_boost = 0;
            println!("[BOOST] Re-enabled");
            write_opt!(node_boost, 1);
        }

        // ── Hotplug ──────────────────────────────────────────────────────
        if virtual_temp >= 49000 && !state_ccc_hotplug {
            state_ccc_hotplug = true;
            println!("[HOTPLUG-CCC] 49°C: disabling cores 2,3,6");
        } else if virtual_temp <= 47000 && state_ccc_hotplug {
            state_ccc_hotplug = false;
            println!("[HOTPLUG-CCC] Cooled: enabling cores 2,3,6");
        }

        if soc <= 5 && !state_bcl_hotplug {
            state_bcl_hotplug = true;
            println!("[HOTPLUG-BCL] Battery <=5%: disabling cores 2,3,6");
        } else if soc >= 6 && state_bcl_hotplug {
            state_bcl_hotplug = false;
            println!("[HOTPLUG-BCL] Battery >5%: enabling cores 2,3,6");
        }

        let offline = state_ccc_hotplug || state_bcl_hotplug;
        let core_val = if offline { 0 } else { 1 };
        write_opt!(node_cpu2_on, core_val);
        write_opt!(node_cpu3_on, core_val);
        write_opt!(node_cpu6_on, core_val);

        // ── Charging Control ─────────────────────────────────────────────
        if screen_state == 1 {
            write_opt!(node_chg_limit, 0);
            write_opt!(node_res_chg, 1);
            write_opt!(node_res_cur, if batt_temp >= 37 { 1000000 } else { 1500000 });
            restricted = false;
        } else {
            if prev_screen_state == 1 {
                write_opt!(node_res_chg, 0);
                write_opt!(node_res_cur, 1000000);
                restricted = false;
            }

            if batt_temp >= 39 {
                if !restricted {
                    println!("[CHG] Battery temp >= 39°C: entering restricted mode");
                    restricted = true;
                }
                write_opt!(node_chg_limit, 0);
                write_opt!(node_res_chg, 1);
                write_opt!(node_res_cur, 1500000);
            } else if restricted {
                if batt_temp <= 35 {
                    println!("[CHG] Battery cooled to {}°C - exiting restricted mode", batt_temp);
                    restricted = false;
                    write_opt!(node_res_chg, 0);
                    write_opt!(node_res_cur, 1000000);
                } else {
                    write_opt!(node_chg_limit, 0);
                }
            }

            if !restricted {
                write_opt!(node_res_chg, 0);
                write_opt!(node_res_cur, 1000000);

                if (30..=38).contains(&batt_temp) {
                    write_opt!(node_chg_limit, batt_temp - 30);
                } else if batt_temp < 30 {
                    write_opt!(node_chg_limit, 0);
                }
            }
        }
        prev_screen_state = screen_state;

        // ── Status ────────────────────────────────────────────────────────
        println!(
            "V:{}°C | B:{}°C | C0:L{} C4:L{} G:L{} TS:L{} BL:{} W:{} Bs:{} HP:{}{} SOC:{}% R:{}",
            virtual_c,
            batt_temp,
            state_cpu0,
            state_cpu4,
            state_gpu,
            state_tstate,
            state_backlight,
            state_wifi,
            state_boost,
            if state_ccc_hotplug { "C" } else { "c" },
            if state_bcl_hotplug { "B" } else { "b" },
            soc,
            if restricted { "Y" } else { "N" }
        );

        thread::sleep(Duration::from_secs(2));
    }
}
