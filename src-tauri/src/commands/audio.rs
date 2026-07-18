use crate::audio_feedback;
use crate::audio_toolkit::audio::{list_input_devices, list_output_devices};
use crate::managers::audio::{AudioRecordingManager, MicrophoneMode};
use crate::settings::{get_settings, write_settings};
use crate::TranscriptionCoordinator;
use log::warn;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

#[cfg(target_os = "windows")]
use winreg::{
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
    RegKey, HKEY,
};

#[derive(Serialize, Type)]
pub struct CustomSounds {
    start: bool,
    stop: bool,
}

fn custom_sound_exists(app: &AppHandle, sound_type: &str) -> bool {
    crate::portable::resolve_app_data(app, &format!("custom_{}.wav", sound_type))
        .is_ok_and(|path| path.exists())
}

#[tauri::command]
#[specta::specta]
pub fn check_custom_sounds(app: AppHandle) -> CustomSounds {
    CustomSounds {
        start: custom_sound_exists(&app, "start"),
        stop: custom_sound_exists(&app, "stop"),
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AudioDevice {
    pub index: String,
    pub name: String,
    pub is_default: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum MicrophoneDiagnosticStatus {
    Ready,
    PermissionDenied,
    NoInputDevice,
    SelectedDeviceMissing,
    EnumerationFailed,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct MicrophoneDiagnostics {
    pub status: MicrophoneDiagnosticStatus,
    pub requested_device: String,
    pub resolved_device: Option<String>,
    pub available_devices: Vec<AudioDevice>,
    pub stream_open: bool,
    pub recording: bool,
    pub detail: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum PermissionAccess {
    Allowed,
    Denied,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct WindowsMicrophonePermissionStatus {
    pub supported: bool,
    pub overall_access: PermissionAccess,
    pub device_access: PermissionAccess,
    pub app_access: PermissionAccess,
    pub desktop_app_access: PermissionAccess,
}

#[cfg(target_os = "windows")]
fn read_registry_permission_access(root_hkey: HKEY, path: &str) -> PermissionAccess {
    let root = RegKey::predef(root_hkey);
    let Ok(key) = root.open_subkey(path) else {
        return PermissionAccess::Unknown;
    };

    let Ok(value) = key.get_value::<String, _>("Value") else {
        return PermissionAccess::Unknown;
    };

    match value.to_ascii_lowercase().as_str() {
        "allow" => PermissionAccess::Allowed,
        "deny" => PermissionAccess::Denied,
        _ => PermissionAccess::Unknown,
    }
}

#[cfg(target_os = "windows")]
fn get_windows_microphone_permission_status_impl() -> WindowsMicrophonePermissionStatus {
    const MICROPHONE_PATH: &str =
        "Software\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone";
    const DESKTOP_APPS_PATH: &str =
        "Software\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone\\NonPackaged";

    let device_access = read_registry_permission_access(HKEY_LOCAL_MACHINE, MICROPHONE_PATH);
    let app_access = read_registry_permission_access(HKEY_CURRENT_USER, MICROPHONE_PATH);
    let desktop_app_access = read_registry_permission_access(HKEY_CURRENT_USER, DESKTOP_APPS_PATH);

    let overall_access = if [device_access, app_access, desktop_app_access]
        .into_iter()
        .any(|access| access == PermissionAccess::Denied)
    {
        PermissionAccess::Denied
    } else if [device_access, app_access, desktop_app_access]
        .into_iter()
        .all(|access| access == PermissionAccess::Allowed)
    {
        PermissionAccess::Allowed
    } else {
        PermissionAccess::Unknown
    };

    WindowsMicrophonePermissionStatus {
        supported: true,
        overall_access,
        device_access,
        app_access,
        desktop_app_access,
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_windows_microphone_permission_status() -> WindowsMicrophonePermissionStatus {
    #[cfg(target_os = "windows")]
    {
        get_windows_microphone_permission_status_impl()
    }

    #[cfg(not(target_os = "windows"))]
    {
        WindowsMicrophonePermissionStatus {
            supported: false,
            overall_access: PermissionAccess::Unknown,
            device_access: PermissionAccess::Unknown,
            app_access: PermissionAccess::Unknown,
            desktop_app_access: PermissionAccess::Unknown,
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn open_microphone_privacy_settings() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("cmd")
            .args(["/C", "start", "", "ms-settings:privacy-microphone"])
            .spawn()
            .map_err(|e| format!("Failed to open Windows microphone privacy settings: {}", e))?;
        return Ok(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Opening microphone privacy settings is only supported on Windows".to_string())
    }
}

#[tauri::command]
#[specta::specta]
pub fn update_microphone_mode(app: AppHandle, always_on: bool) -> Result<(), String> {
    // Update settings
    let mut settings = get_settings(&app);
    settings.always_on_microphone = always_on;
    write_settings(&app, settings);

    // Update the audio manager mode
    let rm = app.state::<Arc<AudioRecordingManager>>();
    let new_mode = if always_on {
        MicrophoneMode::AlwaysOn
    } else {
        MicrophoneMode::OnDemand
    };

    rm.update_mode(new_mode)
        .map_err(|e| format!("Failed to update microphone mode: {}", e))
}

#[tauri::command]
#[specta::specta]
pub fn get_microphone_mode(app: AppHandle) -> Result<bool, String> {
    let settings = get_settings(&app);
    Ok(settings.always_on_microphone)
}

#[tauri::command]
#[specta::specta]
pub fn get_available_microphones() -> Result<Vec<AudioDevice>, String> {
    let devices =
        list_input_devices().map_err(|e| format!("Failed to list audio devices: {}", e))?;

    let mut result = vec![AudioDevice {
        index: "default".to_string(),
        name: "Default".to_string(),
        is_default: true,
    }];

    result.extend(devices.into_iter().map(|d| AudioDevice {
        index: d.index,
        name: d.name,
        is_default: false, // The explicit default is handled separately
    }));

    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub fn get_microphone_diagnostics(app: AppHandle) -> MicrophoneDiagnostics {
    let manager = app.state::<Arc<AudioRecordingManager>>();
    let settings = get_settings(&app);
    let requested_device = settings
        .selected_microphone
        .clone()
        .unwrap_or_else(|| "default".to_string());
    let permissions = get_windows_microphone_permission_status();

    let (mut status, available_devices, resolved_device, mut detail) = match list_input_devices() {
        Ok(devices) if devices.is_empty() => (
            MicrophoneDiagnosticStatus::NoInputDevice,
            Vec::new(),
            None,
            Some("No audio input device was detected".to_string()),
        ),
        Ok(devices) => {
            let available: Vec<AudioDevice> = devices
                .iter()
                .map(|device| AudioDevice {
                    index: device.index.clone(),
                    name: device.name.clone(),
                    is_default: device.is_default,
                })
                .collect();
            match &settings.selected_microphone {
                    Some(selected) => match devices.iter().find(|device| &device.name == selected) {
                        Some(device) => (
                            MicrophoneDiagnosticStatus::Ready,
                            available,
                            Some(device.name.clone()),
                            None,
                        ),
                        None => (
                            MicrophoneDiagnosticStatus::SelectedDeviceMissing,
                            available,
                            None,
                            Some(format!(
                                "Selected microphone '{selected}' is not available; it may be disconnected"
                            )),
                        ),
                    },
                    None => (
                        MicrophoneDiagnosticStatus::Ready,
                        available,
                        devices
                            .iter()
                            .find(|device| device.is_default)
                            .map(|device| device.name.clone()),
                        None,
                    ),
                }
        }
        Err(error) => (
            MicrophoneDiagnosticStatus::EnumerationFailed,
            Vec::new(),
            None,
            Some(format!("Failed to list audio devices: {error}")),
        ),
    };

    if permissions.supported && permissions.overall_access == PermissionAccess::Denied {
        status = MicrophoneDiagnosticStatus::PermissionDenied;
        detail = Some("Microphone access is denied by Windows privacy settings".to_string());
    }

    MicrophoneDiagnostics {
        status,
        requested_device,
        resolved_device,
        available_devices,
        stream_open: manager.is_stream_open(),
        recording: manager.is_recording(),
        detail,
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_dictation_state(
    app: AppHandle,
) -> crate::transcription_coordinator::DictationStateEvent {
    app.state::<TranscriptionCoordinator>().state()
}

fn validated_microphone_selection(device_name: &str) -> Result<Option<String>, String> {
    if device_name == "default" {
        return Ok(None);
    }
    let devices =
        list_input_devices().map_err(|error| format!("Failed to list audio devices: {error}"))?;
    if devices.iter().any(|device| device.name == device_name) {
        Ok(Some(device_name.to_string()))
    } else {
        Err(format!(
            "Selected microphone '{device_name}' is not available; reconnect it and try again"
        ))
    }
}

#[tauri::command]
#[specta::specta]
pub fn set_selected_microphone(app: AppHandle, device_name: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    let previous_selection = settings.selected_microphone.clone();
    settings.selected_microphone = validated_microphone_selection(&device_name)?;
    write_settings(&app, settings);

    // Update the audio manager to use the new device
    let rm = app.state::<Arc<AudioRecordingManager>>();
    if let Err(error) = rm.update_selected_device() {
        let mut rollback = get_settings(&app);
        rollback.selected_microphone = previous_selection;
        write_settings(&app, rollback);
        let _ = rm.update_selected_device();
        return Err(format!("Failed to update selected device: {error}"));
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_selected_microphone(app: AppHandle) -> Result<String, String> {
    let settings = get_settings(&app);
    Ok(settings
        .selected_microphone
        .unwrap_or_else(|| "default".to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn get_available_output_devices() -> Result<Vec<AudioDevice>, String> {
    let devices =
        list_output_devices().map_err(|e| format!("Failed to list output devices: {}", e))?;

    let mut result = vec![AudioDevice {
        index: "default".to_string(),
        name: "Default".to_string(),
        is_default: true,
    }];

    result.extend(devices.into_iter().map(|d| AudioDevice {
        index: d.index,
        name: d.name,
        is_default: false, // The explicit default is handled separately
    }));

    Ok(result)
}

#[tauri::command]
#[specta::specta]
pub fn set_selected_output_device(app: AppHandle, device_name: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.selected_output_device = if device_name == "default" {
        None
    } else {
        Some(device_name)
    };
    write_settings(&app, settings);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_selected_output_device(app: AppHandle) -> Result<String, String> {
    let settings = get_settings(&app);
    Ok(settings
        .selected_output_device
        .unwrap_or_else(|| "default".to_string()))
}

#[tauri::command]
#[specta::specta]
pub async fn play_test_sound(app: AppHandle, sound_type: String) {
    let sound = match sound_type.as_str() {
        "start" => audio_feedback::SoundType::Start,
        "stop" => audio_feedback::SoundType::Stop,
        _ => {
            warn!("Unknown sound type: {}", sound_type);
            return;
        }
    };
    audio_feedback::play_test_sound(&app, sound);
}

#[tauri::command]
#[specta::specta]
pub fn set_clamshell_microphone(app: AppHandle, device_name: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    let previous_selection = settings.clamshell_microphone.clone();
    settings.clamshell_microphone = validated_microphone_selection(&device_name)?;
    write_settings(&app, settings);
    let manager = app.state::<Arc<AudioRecordingManager>>();
    if let Err(error) = manager.update_selected_device() {
        let mut rollback = get_settings(&app);
        rollback.clamshell_microphone = previous_selection;
        write_settings(&app, rollback);
        let _ = manager.update_selected_device();
        return Err(format!("Failed to update clamshell microphone: {error}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MicrophoneDiagnosticStatus, PermissionAccess};

    #[test]
    fn diagnostic_status_and_permission_values_serialize_stably() {
        assert_eq!(
            serde_json::to_string(&MicrophoneDiagnosticStatus::SelectedDeviceMissing)
                .expect("status serialization"),
            "\"selected_device_missing\""
        );
        assert_eq!(
            serde_json::to_string(&PermissionAccess::Denied).expect("permission serialization"),
            "\"denied\""
        );
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_clamshell_microphone(app: AppHandle) -> Result<String, String> {
    let settings = get_settings(&app);
    Ok(settings
        .clamshell_microphone
        .unwrap_or_else(|| "default".to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn is_recording(app: AppHandle) -> bool {
    let audio_manager = app.state::<Arc<AudioRecordingManager>>();
    audio_manager.is_recording()
}
