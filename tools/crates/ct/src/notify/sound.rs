pub fn sound_file_for_type(notification_type: Option<&str>) -> &'static str {
    match notification_type {
        Some("permission_prompt") | Some("idle_prompt") | Some("elicitation_dialog") => {
            "/usr/share/sounds/freedesktop/stereo/dialog-warning.oga"
        }
        _ => "/usr/share/sounds/freedesktop/stereo/complete.oga",
    }
}

#[cfg(target_os = "linux")]
pub fn play(notification_type: Option<&str>) {
    use std::process::Command;
    let file = sound_file_for_type(notification_type);
    let _ = Command::new("paplay").arg(file).spawn();
}

#[cfg(target_os = "macos")]
pub fn play(_notification_type: Option<&str>) {
    // macOS sound is handled by grrr via --sound flag in macos.rs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_prompt_maps_to_dialog_warning() {
        assert_eq!(
            sound_file_for_type(Some("permission_prompt")),
            "/usr/share/sounds/freedesktop/stereo/dialog-warning.oga"
        );
    }

    #[test]
    fn idle_prompt_maps_to_dialog_warning() {
        assert_eq!(
            sound_file_for_type(Some("idle_prompt")),
            "/usr/share/sounds/freedesktop/stereo/dialog-warning.oga"
        );
    }

    #[test]
    fn elicitation_dialog_maps_to_dialog_warning() {
        assert_eq!(
            sound_file_for_type(Some("elicitation_dialog")),
            "/usr/share/sounds/freedesktop/stereo/dialog-warning.oga"
        );
    }

    #[test]
    fn unknown_type_maps_to_complete() {
        assert_eq!(
            sound_file_for_type(Some("something_else")),
            "/usr/share/sounds/freedesktop/stereo/complete.oga"
        );
    }

    #[test]
    fn none_type_maps_to_complete() {
        assert_eq!(
            sound_file_for_type(None),
            "/usr/share/sounds/freedesktop/stereo/complete.oga"
        );
    }
}
