use std::path::{Path, PathBuf};
use std::process::Command;

use image::{ImageBuffer, Rgba, RgbaImage};

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return PathBuf::from(home).join(rest);
    }
    PathBuf::from(path)
}

fn default_icons_dir() -> Option<PathBuf> {
    let dir = expand_tilde("~/.claude/icons");
    // Only return Some if tilde was actually expanded
    if dir.is_absolute() { Some(dir) } else { None }
}

pub fn sanitize_session(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn icon_filename(session: &str, symbol: &str, color_hex: &str) -> String {
    let safe = sanitize_session(session);
    let color = color_hex.trim_start_matches('#');
    format!("{safe}-{symbol}-{color}.png")
}

fn cache_path_in(dir: &Path, session: &str, symbol: &str, color_hex: &str) -> PathBuf {
    dir.join(icon_filename(session, symbol, color_hex))
}

fn cleanup_stale_in(dir: &Path, session: &str, symbol: &str) {
    let safe = sanitize_session(session);
    let prefix = format!("{safe}-{symbol}-");
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with(&prefix) && name_str.ends_with(".png") {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

fn parse_hex_color(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}

fn draw_circle(img: &mut RgbaImage, [cr, cg, cb]: [u8; 3]) {
    let size = img.width();
    let cx = size as f32 / 2.0;
    let cy = size as f32 / 2.0;
    let radius: f32 = 56.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= radius * radius {
                img.put_pixel(x, y, Rgba([cr, cg, cb, 255]));
            }
        }
    }
}

/// Draw a line between two points using Bresenham's algorithm.
fn draw_line(
    img: &mut RgbaImage,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    color: Rgba<u8>,
    thickness: i32,
) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx: i32 = if x0 < x1 { 1 } else { -1 };
    let sy: i32 = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;
    let w = img.width() as i32;
    let h = img.height() as i32;

    loop {
        for ty in -thickness..=thickness {
            for tx in -thickness..=thickness {
                let px = x + tx;
                let py = y + ty;
                if px >= 0 && py >= 0 && px < w && py < h {
                    img.put_pixel(px as u32, py as u32, color);
                }
            }
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}

fn draw_filled_rect(img: &mut RgbaImage, x: i32, y: i32, w: i32, h: i32, color: Rgba<u8>) {
    let iw = img.width() as i32;
    let ih = img.height() as i32;
    for py in y.max(0)..=(y + h - 1).min(ih - 1) {
        for px in x.max(0)..=(x + w - 1).min(iw - 1) {
            img.put_pixel(px as u32, py as u32, color);
        }
    }
}

fn draw_filled_circle_overlay(img: &mut RgbaImage, cx: i32, cy: i32, radius: i32, color: Rgba<u8>) {
    let iw = img.width() as i32;
    let ih = img.height() as i32;
    for y in (cy - radius).max(0)..=(cy + radius).min(ih - 1) {
        for x in (cx - radius).max(0)..=(cx + radius).min(iw - 1) {
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= radius * radius {
                img.put_pixel(x as u32, y as u32, color);
            }
        }
    }
}

fn draw_symbol(img: &mut RgbaImage, symbol: &str) {
    // Semi-transparent black, matching gen-circle.swift alpha 0.5
    let color = Rgba([0u8, 0u8, 0u8, 128u8]);
    let thick = 5;

    match symbol {
        "check" => {
            draw_line(img, 36, 66, 54, 88, color, thick);
            draw_line(img, 54, 88, 92, 40, color, thick);
        }
        "lock" => {
            // Lock body
            draw_filled_rect(img, 38, 60, 52, 42, color);
            // Shackle: two vertical bars + top bar
            draw_filled_rect(img, 44, 40, 10, 24, color);
            draw_filled_rect(img, 74, 40, 10, 24, color);
            draw_filled_rect(img, 44, 36, 40, 12, color);
            // Hollow middle of shackle
            draw_filled_rect(img, 54, 40, 20, 20, Rgba([0, 0, 0, 0]));
        }
        "chat" => {
            // Speech bubble body
            draw_filled_rect(img, 24, 28, 80, 58, color);
            // Triangle pointer
            draw_filled_rect(img, 28, 86, 20, 8, color);
            draw_filled_rect(img, 28, 94, 14, 6, color);
            draw_filled_rect(img, 28, 100, 8, 6, color);
        }
        "question" => {
            // Top arc of question mark
            draw_filled_rect(img, 46, 30, 36, 10, color);
            // Right side going down
            draw_filled_rect(img, 72, 30, 10, 28, color);
            // Bottom curve
            draw_filled_rect(img, 46, 50, 36, 10, color);
            // Stem
            draw_filled_rect(img, 59, 60, 10, 20, color);
            // Dot
            draw_filled_circle_overlay(img, 64, 96, 7, color);
        }
        _ => {}
    }
}

fn generate_in(dir: &Path, color_hex: &str, symbol: &str, session: &str) -> Option<PathBuf> {
    let path = cache_path_in(dir, session, symbol, color_hex);

    if path.exists() {
        return Some(path);
    }

    cleanup_stale_in(dir, session, symbol);
    std::fs::create_dir_all(dir).ok()?;

    let [r, g, b] = parse_hex_color(color_hex)?;
    let mut img: RgbaImage = ImageBuffer::new(128, 128);
    draw_circle(&mut img, [r, g, b]);
    draw_symbol(&mut img, symbol);
    img.save(&path).ok()?;
    Some(path)
}

pub fn generate(color_hex: &str, symbol: &str, session: &str) -> Option<PathBuf> {
    let dir = default_icons_dir()?;
    generate_in(&dir, color_hex, symbol, session)
}

pub fn tmux_session_color(session: &str) -> Option<String> {
    let out = Command::new("tmux")
        .args(["show-option", "-t", session, "-qv", "@session_color"])
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    let color = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if color.is_empty() { None } else { Some(color) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_icons_dir() -> (TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("icons");
        std::fs::create_dir_all(&dir).unwrap();
        (tmp, dir)
    }

    #[test]
    fn sanitize_session_keeps_safe_chars() {
        assert_eq!(sanitize_session("my-session_01"), "my-session_01");
    }

    #[test]
    fn sanitize_session_replaces_unsafe_chars() {
        assert_eq!(sanitize_session("my.session/path"), "my_session_path");
    }

    #[test]
    fn sanitize_session_replaces_spaces() {
        assert_eq!(sanitize_session("my session"), "my_session");
    }

    #[test]
    fn icon_filename_strips_hash_from_color() {
        let name = icon_filename("sess", "lock", "#aabbcc");
        assert_eq!(name, "sess-lock-aabbcc.png");
        assert!(!name.contains('#'));
    }

    #[test]
    fn icon_filename_without_hash_same_as_with() {
        assert_eq!(
            icon_filename("sess", "check", "#ff0000"),
            icon_filename("sess", "check", "ff0000"),
        );
    }

    #[test]
    fn icon_filename_sanitizes_session_name() {
        let name = icon_filename("my.session", "check", "ff0000");
        assert_eq!(name, "my_session-check-ff0000.png");
    }

    #[test]
    fn cache_path_in_contains_correct_filename() {
        let dir = PathBuf::from("/tmp/icons");
        let path = cache_path_in(&dir, "my-session", "check", "#ff0000");
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        assert_eq!(name, "my-session-check-ff0000.png");
    }

    #[test]
    fn cache_path_in_is_inside_given_dir() {
        let dir = PathBuf::from("/tmp/icons");
        let path = cache_path_in(&dir, "sess", "check", "ff0000");
        assert_eq!(path.parent().unwrap(), dir.as_path());
    }

    #[test]
    fn parse_hex_color_valid() {
        assert_eq!(parse_hex_color("#ff8040"), Some([255, 128, 64]));
        assert_eq!(parse_hex_color("ff8040"), Some([255, 128, 64]));
    }

    #[test]
    fn parse_hex_color_invalid_length() {
        assert_eq!(parse_hex_color("#fff"), None);
        assert_eq!(parse_hex_color(""), None);
    }

    #[test]
    fn parse_hex_color_invalid_chars() {
        assert_eq!(parse_hex_color("zzzzzz"), None);
    }

    #[test]
    fn generate_in_produces_128x128_png() {
        let (_tmp, dir) = make_icons_dir();
        let result = generate_in(&dir, "#3c8dbc", "check", "test-session");
        assert!(result.is_some(), "generate_in returned None");
        let path = result.unwrap();
        assert!(path.exists(), "PNG file not created at {path:?}");
        let img = image::open(&path).unwrap();
        assert_eq!(img.width(), 128);
        assert_eq!(img.height(), 128);
    }

    #[test]
    fn generate_in_circle_has_correct_color_at_top() {
        let (_tmp, dir) = make_icons_dir();
        let result = generate_in(&dir, "#ff0000", "check", "color-test");
        assert!(result.is_some());
        let path = result.unwrap();
        let img = image::open(&path).unwrap().into_rgba8();
        // Sample well inside circle but far from the check symbol (upper portion)
        let pixel = img.get_pixel(64, 20);
        assert_eq!(pixel[0], 255, "red channel at (64,20): {:?}", pixel);
        assert_eq!(pixel[1], 0, "green channel at (64,20): {:?}", pixel);
        assert_eq!(pixel[2], 0, "blue channel at (64,20): {:?}", pixel);
        assert_eq!(
            pixel[3], 255,
            "alpha at (64,20) should be opaque: {:?}",
            pixel
        );
    }

    #[test]
    fn generate_in_corners_are_transparent() {
        let (_tmp, dir) = make_icons_dir();
        let result = generate_in(&dir, "#00ff00", "check", "corner-test");
        assert!(result.is_some());
        let path = result.unwrap();
        let img = image::open(&path).unwrap().into_rgba8();
        let corner = img.get_pixel(0, 0);
        assert_eq!(
            corner[3], 0,
            "corner should be transparent, alpha={}",
            corner[3]
        );
    }

    #[test]
    fn generate_in_returns_same_path_on_second_call() {
        let (_tmp, dir) = make_icons_dir();
        let first = generate_in(&dir, "#0000ff", "lock", "cache-test").unwrap();
        let second = generate_in(&dir, "#0000ff", "lock", "cache-test").unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn generate_in_cache_hit_does_not_regenerate() {
        let (_tmp, dir) = make_icons_dir();
        let first = generate_in(&dir, "#0000ff", "lock", "regen-test").unwrap();
        let mtime_before = std::fs::metadata(&first).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let second = generate_in(&dir, "#0000ff", "lock", "regen-test").unwrap();
        let mtime_after = std::fs::metadata(&second).unwrap().modified().unwrap();
        assert_eq!(
            mtime_before, mtime_after,
            "file should not be re-written on cache hit"
        );
    }

    #[test]
    fn cleanup_stale_in_removes_old_color_variants() {
        let (_tmp, dir) = make_icons_dir();
        let stale1 = dir.join("mysess-check-aaaaaa.png");
        let stale2 = dir.join("mysess-check-bbbbbb.png");
        let keeper = dir.join("mysess-lock-aaaaaa.png");
        std::fs::write(&stale1, b"fake").unwrap();
        std::fs::write(&stale2, b"fake").unwrap();
        std::fs::write(&keeper, b"fake").unwrap();

        cleanup_stale_in(&dir, "mysess", "check");

        assert!(!stale1.exists(), "stale1 should be removed");
        assert!(!stale2.exists(), "stale2 should be removed");
        assert!(keeper.exists(), "keeper (different symbol) should remain");
    }

    #[test]
    fn generate_in_invalid_color_returns_none() {
        let (_tmp, dir) = make_icons_dir();
        let result = generate_in(&dir, "notacolor", "check", "err-session");
        assert!(result.is_none());
    }

    #[test]
    fn generate_in_creates_dir_if_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("new/icons");
        assert!(!dir.exists());
        generate_in(&dir, "#123456", "chat", "mkdir-test");
        assert!(dir.exists(), "icons dir should have been created");
    }

    #[test]
    fn generate_in_all_symbols_produce_files() {
        let (_tmp, dir) = make_icons_dir();
        for symbol in ["check", "lock", "chat", "question"] {
            let session = format!("sym-{symbol}");
            let result = generate_in(&dir, "#3c8dbc", symbol, &session);
            assert!(
                result.is_some(),
                "generate_in returned None for symbol={symbol}"
            );
            assert!(result.unwrap().exists(), "PNG missing for symbol={symbol}");
        }
    }

    #[test]
    fn generate_in_cleanup_removes_stale_before_writing_new() {
        let (_tmp, dir) = make_icons_dir();
        let stale = dir.join("sess-check-ff0000.png");
        std::fs::write(&stale, b"old").unwrap();

        let result = generate_in(&dir, "#00ff00", "check", "sess").unwrap();
        assert!(!stale.exists(), "stale file should have been cleaned up");
        assert!(result.exists(), "new file should exist");
    }
}
