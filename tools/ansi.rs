use std::sync::OnceLock;

fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("NO_COLOR").is_none())
}

fn rgb(r: u8, g: u8, b: u8, text: &str) -> String {
    if !enabled() {
        return text.to_string();
    }
    format!("\x1b[38;2;{r};{g};{b}m{text}\x1b[0m")
}

pub fn id(text: &str) -> String {
    rgb(180, 190, 254, text)
}

pub fn dim(text: &str) -> String {
    rgb(108, 112, 134, text)
}

pub fn bold(text: &str) -> String {
    if !enabled() {
        return text.to_string();
    }
    format!("\x1b[1m{text}\x1b[0m")
}
