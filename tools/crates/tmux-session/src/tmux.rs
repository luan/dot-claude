use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn home() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap())
}

pub fn tmux(args: &[&str]) -> String {
    String::from_utf8_lossy(
        &Command::new("tmux")
            .args(args)
            .stderr(Stdio::null())
            .output()
            .expect("failed to run tmux")
            .stdout,
    )
    .trim()
    .to_string()
}

/// Like `tmux()` but does NOT trim the output. Use for batch commands where
/// leading/trailing newlines encode meaningful empty fields (e.g. unset options
/// producing an empty first line in a multi-command batch).
pub fn tmux_lines(args: &[&str]) -> String {
    String::from_utf8_lossy(
        &Command::new("tmux")
            .args(args)
            .stderr(Stdio::null())
            .output()
            .expect("failed to run tmux")
            .stdout,
    )
    .to_string()
}

pub struct TmuxState {
    pub current: String,
    pub alive: HashSet<String>,
    pub attn: HashMap<String, String>,
}

pub fn query_state() -> TmuxState {
    let raw = tmux(&[
        "display-message",
        "-p",
        "#S",
        ";",
        "list-sessions",
        "-F",
        "#{session_name}\t#{@attention}",
    ]);
    let mut lines = raw.lines();
    let current = lines.next().unwrap_or("").to_string();
    let mut alive = HashSet::new();
    let mut attn = HashMap::new();
    for line in lines {
        let (name, val) = line.split_once('\t').unwrap_or((line, ""));
        if !name.is_empty() {
            alive.insert(name.to_string());
            if !val.is_empty() {
                attn.insert(name.to_string(), val.to_string());
            }
        }
    }
    TmuxState {
        current,
        alive,
        attn,
    }
}

#[derive(Clone)]
pub struct WindowInfo {
    pub index: usize,
    pub name: String,
    pub active: bool,
    pub zoomed: bool,
}

pub fn query_windows_all() -> HashMap<String, Vec<WindowInfo>> {
    let raw = tmux(&[
        "list-windows",
        "-a",
        "-F",
        "#{session_name}\t#{window_index}\t#{window_name}\t#{?window_active,1,0}\t#{?window_zoomed_flag,1,0}",
    ]);
    let mut map: HashMap<String, Vec<WindowInfo>> = HashMap::new();
    for line in raw.lines().filter(|l| !l.is_empty()) {
        let parts: Vec<&str> = line.splitn(5, '\t').collect();
        if parts.len() >= 5 {
            let session = parts[0].to_string();
            map.entry(session).or_default().push(WindowInfo {
                index: parts[1].parse().unwrap_or(0),
                name: parts[2].to_string(),
                active: parts[3] == "1",
                zoomed: parts[4] == "1",
            });
        }
    }
    map
}

pub fn query_windows() -> Vec<WindowInfo> {
    let raw = tmux(&[
        "list-windows",
        "-F",
        "#{window_index}\t#{window_name}\t#{?window_active,1,0}\t#{?window_zoomed_flag,1,0}",
    ]);
    raw.lines()
        .filter(|l| !l.is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(4, '\t');
            let index = parts.next()?.parse().ok()?;
            let name = parts.next()?.to_string();
            let active = parts.next() == Some("1");
            let zoomed = parts.next() == Some("1");
            Some(WindowInfo {
                index,
                name,
                active,
                zoomed,
            })
        })
        .collect()
}

pub struct SystemInfo {
    pub cpu_load: f32,
    pub mem_pct: u32,
    pub battery_pct: Option<u32>,
    pub battery_state: BatteryState,
    pub battery_time: String,
    pub caffeinated: bool,
    pub clock: String,
}

pub enum BatteryState {
    Charging,
    Discharging,
    Charged,
    NoBattery,
}

pub fn query_system_info() -> SystemInfo {
    // CPU: 1-minute load average from /proc/loadavg
    let cpu_load: f32 = std::fs::read_to_string("/proc/loadavg")
        .ok()
        .as_deref()
        .and_then(|s| s.split_whitespace().next().and_then(|t| t.parse().ok()))
        .unwrap_or(0.0);

    // Memory from /proc/meminfo
    let mem_pct = parse_memory();

    // Battery from /sys/class/power_supply/
    let (battery_pct, battery_state, battery_time) = parse_battery();

    // Clock
    let clock = chrono::Local::now().format("%H:%M").to_string();

    SystemInfo {
        cpu_load,
        mem_pct,
        battery_pct,
        battery_state,
        battery_time,
        caffeinated: false,
        clock,
    }
}

fn parse_memory() -> u32 {
    let raw = match std::fs::read_to_string("/proc/meminfo") {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let mut total: Option<u64> = None;
    let mut available: Option<u64> = None;
    for line in raw.lines() {
        if line.starts_with("MemTotal:") {
            total = line.split_whitespace().nth(1).and_then(|v| v.parse().ok());
        } else if line.starts_with("MemAvailable:") {
            available = line.split_whitespace().nth(1).and_then(|v| v.parse().ok());
        }
        if total.is_some() && available.is_some() {
            break;
        }
    }
    match (total, available) {
        (Some(t), Some(a)) if t > 0 => ((t - a) * 100 / t) as u32,
        _ => 0,
    }
}

fn parse_battery() -> (Option<u32>, BatteryState, String) {
    let dir = match std::fs::read_dir("/sys/class/power_supply") {
        Ok(d) => d,
        Err(_) => return (None, BatteryState::NoBattery, String::new()),
    };
    for entry in dir.flatten() {
        let path = entry.path();
        let type_content = match std::fs::read_to_string(path.join("type")) {
            Ok(s) => s,
            Err(_) => continue,
        };
        if !type_content.trim().eq_ignore_ascii_case("Battery") {
            continue;
        }
        let pct: u32 = std::fs::read_to_string(path.join("capacity"))
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        let status = std::fs::read_to_string(path.join("status"))
            .unwrap_or_default();
        let state = match status.trim() {
            "Charging" => BatteryState::Charging,
            "Full" => BatteryState::Charged,
            _ => BatteryState::Discharging,
        };
        return (Some(pct), state, String::new());
    }
    (None, BatteryState::NoBattery, String::new())
}

pub fn git_toplevel(dir: &str) -> Option<String> {
    Command::new("git")
        .args(["-C", dir, "rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
