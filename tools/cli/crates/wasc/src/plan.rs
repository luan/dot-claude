use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Plan {
    pub name: String,
    pub path: PathBuf,
    pub title: String,
    pub mod_time: SystemTime,
    pub size: u64,
}

pub fn list_plans() -> Vec<Plan> {
    let Some(home) = dirs::home_dir() else {
        return Vec::new();
    };
    let dir = home.join(".claude").join("plans");
    let Ok(entries) = fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut plans: Vec<Plan> = entries
        .flatten()
        .filter(|e| !e.path().is_dir() && e.path().extension().is_some_and(|ext| ext == "md"))
        .filter_map(|e| {
            let info = e.metadata().ok()?;
            let name = e.path().file_stem()?.to_string_lossy().to_string();
            let title = extract_title(&e.path());
            Some(Plan {
                name,
                path: e.path(),
                title,
                mod_time: info.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                size: info.len(),
            })
        })
        .collect();

    plans.sort_by_key(|a| std::cmp::Reverse(a.mod_time));
    plans
}

pub fn load_content(path: &PathBuf) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| format!("Error loading plan: {e}"))
}

fn extract_title(path: &PathBuf) -> String {
    let Ok(f) = fs::File::open(path) else {
        return String::new();
    };
    let reader = BufReader::new(f);
    for line in reader.lines().map_while(Result::ok) {
        let trimmed = line.trim().to_string();
        if let Some(title) = trimmed.strip_prefix("# ") {
            return title.to_string();
        }
    }
    String::new()
}

pub fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes}B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    }
}

pub fn format_date(time: SystemTime) -> String {
    let duration = time
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs() as i64;

    // Simple date formatting without chrono
    let days = secs / 86400;
    let mut y = 1970i32;
    let mut remaining_days = days;

    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }

    let months_days: &[i64] = if is_leap(y) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 0;
    for (i, &md) in months_days.iter().enumerate() {
        if remaining_days < md {
            m = i;
            break;
        }
        remaining_days -= md;
    }

    let d = remaining_days + 1;
    let month_names = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    // Get current year for comparison
    let now_secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    let now_days = now_secs / 86400;
    let mut current_year = 1970i32;
    let mut rd = now_days;
    loop {
        let diy = if is_leap(current_year) { 366 } else { 365 };
        if rd < diy {
            break;
        }
        rd -= diy;
        current_year += 1;
    }

    if y != current_year {
        format!("{} {y}", month_names[m])
    } else {
        format!("{} {d:02}", month_names[m])
    }
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
