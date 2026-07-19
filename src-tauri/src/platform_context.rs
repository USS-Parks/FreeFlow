use crate::contracts::PlatformContext;
use crate::settings::{AppCategory, AppContextProfile, AppSettings};
use serde::Serialize;
use specta::Type;
use tauri::AppHandle;

const CONTEXT_WINDOW_CHARS: usize = 16;
const SELECTED_TEXT_CAPTURE_CHARS: i32 = 12_001;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ContextAccessStatus {
    Enabled,
    Disabled,
    ProfileDisabled,
    DeniedApplication,
    RemoteApplication,
    SecureField,
    SecurityUnknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Type)]
pub struct ContextDiagnostics {
    pub application_id: Option<String>,
    pub window_title: Option<String>,
    pub category: AppCategory,
    pub status: ContextAccessStatus,
    pub captured_characters: usize,
}

pub fn capture_active_target(settings: &AppSettings) -> PlatformContext {
    capture_platform_target(settings, false)
}

/// Reads selected text only for an explicit transform action. Ordinary target
/// capture never reads selection contents.
pub fn capture_selected_target(settings: &AppSettings) -> PlatformContext {
    capture_platform_target(settings, true)
}

pub fn selected_text_block_reason(
    settings: &AppSettings,
    context: &PlatformContext,
) -> Option<&'static str> {
    if !context.secure_field_known {
        Some("target_security_unknown")
    } else if context.secure_field {
        Some("secure_field")
    } else if is_remote_application(context.application_id.as_deref()) {
        Some("remote_application")
    } else if is_denied_application(settings, context.application_id.as_deref()) {
        Some("denied_application")
    } else if context.target_id.is_none() {
        Some("target_identity_unavailable")
    } else if context
        .selected_text
        .as_deref()
        .is_none_or(|text| text.trim().is_empty())
    {
        Some("no_selection")
    } else {
        None
    }
}

pub fn classify_application(application_id: Option<&str>) -> AppCategory {
    let id = application_id.unwrap_or_default().to_ascii_lowercase();
    let stem = id.strip_suffix(".exe").unwrap_or(&id);
    if ["outlook", "olk", "mail", "thunderbird", "spark"].contains(&stem) {
        AppCategory::Email
    } else if [
        "slack", "teams", "ms-teams", "discord", "whatsapp", "signal",
    ]
    .contains(&stem)
    {
        AppCategory::Messaging
    } else if ["winword", "pages", "notion", "libreoffice", "soffice"].contains(&stem) {
        AppCategory::Document
    } else if [
        "code",
        "cursor",
        "devenv",
        "xcode",
        "idea",
        "webstorm",
        "pycharm",
        "sublime_text",
    ]
    .contains(&stem)
    {
        AppCategory::Code
    } else if [
        "windowsterminal",
        "terminal",
        "iterm2",
        "powershell",
        "pwsh",
        "cmd",
        "wezterm-gui",
        "alacritty",
    ]
    .contains(&stem)
    {
        AppCategory::Terminal
    } else {
        AppCategory::Other
    }
}

pub fn profile_for_application(
    settings: &AppSettings,
    application_id: Option<&str>,
) -> AppContextProfile {
    settings.app_context_profile(classify_application(application_id))
}

fn normalized_application_id(application_id: Option<&str>) -> String {
    application_id
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

fn is_denied_application(settings: &AppSettings, application_id: Option<&str>) -> bool {
    let id = normalized_application_id(application_id);
    !id.is_empty()
        && settings
            .app_context_denylist
            .iter()
            .any(|denied| denied.trim().eq_ignore_ascii_case(&id))
}

fn is_remote_application(application_id: Option<&str>) -> bool {
    matches!(
        normalized_application_id(application_id).as_str(),
        "mstsc.exe"
            | "mstsc"
            | "wfica32.exe"
            | "wfica32"
            | "screen sharing.app"
            | "screensharingagent"
    )
}

fn profile_allows_context(settings: &AppSettings, application_id: Option<&str>) -> bool {
    settings.app_context_enabled
        && profile_for_application(settings, application_id).surrounding_text_enabled
        && !is_denied_application(settings, application_id)
        && !is_remote_application(application_id)
}

pub fn context_access_status(
    settings: &AppSettings,
    context: &PlatformContext,
) -> ContextAccessStatus {
    if !settings.app_context_enabled {
        ContextAccessStatus::Disabled
    } else if !context.secure_field_known {
        ContextAccessStatus::SecurityUnknown
    } else if context.secure_field {
        ContextAccessStatus::SecureField
    } else if is_remote_application(context.application_id.as_deref()) {
        ContextAccessStatus::RemoteApplication
    } else if is_denied_application(settings, context.application_id.as_deref()) {
        ContextAccessStatus::DeniedApplication
    } else if !profile_for_application(settings, context.application_id.as_deref())
        .surrounding_text_enabled
    {
        ContextAccessStatus::ProfileDisabled
    } else {
        ContextAccessStatus::Enabled
    }
}

pub fn diagnostics(settings: &AppSettings, context: &PlatformContext) -> ContextDiagnostics {
    let status = context_access_status(settings, context);
    let exposes_context = status == ContextAccessStatus::Enabled;
    ContextDiagnostics {
        application_id: context.application_id.clone(),
        window_title: exposes_context
            .then(|| context.window_title.clone())
            .flatten(),
        category: classify_application(context.application_id.as_deref()),
        status,
        captured_characters: if exposes_context {
            context
                .preceding_text
                .as_deref()
                .map_or(0, |text| text.chars().count())
        } else {
            0
        },
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_context_diagnostics(app: AppHandle) -> ContextDiagnostics {
    let settings = crate::settings::get_settings(&app);
    let context = capture_active_target(&settings);
    diagnostics(&settings, &context)
}

pub fn same_target(left: &PlatformContext, right: &PlatformContext) -> bool {
    match (&left.target_id, &right.target_id) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

#[cfg(target_os = "windows")]
fn capture_platform_target(settings: &AppSettings, read_selected_text: bool) -> PlatformContext {
    use std::path::Path;
    use windows::core::{BOOL, PWSTR};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::System::Ole::{
        SafeArrayDestroy, SafeArrayGetElement, SafeArrayGetLBound, SafeArrayGetUBound,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTextPattern2,
        TextPatternRangeEndpoint_Start, TextUnit_Character, UIA_TextPattern2Id,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    };

    unsafe fn process_name(process_id: u32) -> Option<String> {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).ok()?;
        let mut buffer = vec![0_u16; 32_768];
        let mut length = buffer.len() as u32;
        let result = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        );
        let _ = CloseHandle(process);
        result.ok()?;
        let path = String::from_utf16_lossy(&buffer[..length as usize]);
        Path::new(&path)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
    }

    unsafe fn runtime_id(element: &IUIAutomationElement) -> Option<String> {
        let array = element.GetRuntimeId().ok()?;
        if array.is_null() {
            return None;
        }
        let result = (|| {
            let lower = SafeArrayGetLBound(array, 1).ok()?;
            let upper = SafeArrayGetUBound(array, 1).ok()?;
            let mut values = Vec::with_capacity((upper - lower + 1).max(0) as usize);
            for index in lower..=upper {
                let mut value = 0_i32;
                SafeArrayGetElement(
                    array,
                    &index,
                    (&mut value as *mut i32).cast::<std::ffi::c_void>(),
                )
                .ok()?;
                values.push(value.to_string());
            }
            (!values.is_empty()).then(|| values.join("."))
        })();
        let _ = SafeArrayDestroy(array);
        result
    }

    unsafe fn focused_accessibility(
        read_preceding_text: bool,
        read_selected_text: bool,
    ) -> Option<(bool, String, Option<String>, Option<String>)> {
        let initialized = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let result = (|| {
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;
            let element = automation.GetFocusedElement().ok()?;
            let secure = element.CurrentIsPassword().ok()?.as_bool();
            let runtime_id = runtime_id(&element);
            if secure {
                return Some((true, String::new(), runtime_id, None));
            }

            let pattern: Option<IUIAutomationTextPattern2> =
                element.GetCurrentPatternAs(UIA_TextPattern2Id).ok();

            let selected_text = (|| {
                if !read_selected_text {
                    return None;
                }
                let pattern = pattern.as_ref()?;
                let ranges = pattern.GetSelection().ok()?;
                if ranges.Length().ok()? != 1 {
                    return None;
                }
                let range = ranges.GetElement(0).ok()?;
                let text = range.GetText(SELECTED_TEXT_CAPTURE_CHARS).ok()?.to_string();
                (!text.trim().is_empty()).then_some(text)
            })();

            let preceding = (|| {
                if !read_preceding_text {
                    return None;
                }
                let pattern = pattern.as_ref()?;
                let mut active = BOOL(0);
                let caret = pattern.GetCaretRange(&mut active).ok()?;
                if !active.as_bool() {
                    return None;
                }
                let range = caret.Clone().ok()?;
                let moved = range
                    .MoveEndpointByUnit(
                        TextPatternRangeEndpoint_Start,
                        TextUnit_Character,
                        -(CONTEXT_WINDOW_CHARS as i32),
                    )
                    .ok()?;
                if moved == 0 {
                    None
                } else {
                    range
                        .GetText(CONTEXT_WINDOW_CHARS as i32)
                        .ok()
                        .map(|text| text.to_string())
                }
            })()
            .unwrap_or_default();
            Some((false, preceding, runtime_id, selected_text))
        })();
        if initialized {
            CoUninitialize();
        }
        result
    }

    unsafe {
        let window = GetForegroundWindow();
        if window.0.is_null() {
            return PlatformContext::default();
        }

        let title_length = GetWindowTextLengthW(window);
        let window_title = if title_length > 0 {
            let mut buffer = vec![0_u16; title_length as usize + 1];
            let copied = GetWindowTextW(window, &mut buffer);
            (copied > 0).then(|| String::from_utf16_lossy(&buffer[..copied as usize]))
        } else {
            None
        };

        let mut process_id = 0_u32;
        GetWindowThreadProcessId(window, Some(&mut process_id));
        let application_id = process_name(process_id);
        let accessibility = focused_accessibility(
            profile_allows_context(settings, application_id.as_deref()),
            read_selected_text,
        );
        let secure_field = accessibility
            .as_ref()
            .map(|(secure, _, _, _)| *secure)
            .unwrap_or(false);
        let preceding_text = accessibility
            .as_ref()
            .and_then(|(secure, text, _, _)| (!*secure && !text.is_empty()).then(|| text.clone()));
        let target_id = accessibility
            .as_ref()
            .and_then(|(_, _, runtime_id, _)| runtime_id.as_ref())
            .map(|runtime_id| format!("windows:{process_id}:{runtime_id}"));
        let selected_text = accessibility
            .as_ref()
            .and_then(|(_, _, _, selected)| selected.clone());

        PlatformContext {
            application_id,
            window_title,
            selected_text,
            secure_field,
            secure_field_known: accessibility.is_some(),
            target_id,
            preceding_text,
        }
    }
}

#[cfg(target_os = "macos")]
fn capture_platform_target(settings: &AppSettings, read_selected_text: bool) -> PlatformContext {
    use core_foundation::base::{CFRange, CFType, CFTypeRef, TCFType};
    use core_foundation::string::{CFString, CFStringRef};
    use std::ffi::c_void;

    type AXUIElementRef = *const c_void;
    type AXValueRef = *const c_void;

    const AX_ERROR_SUCCESS: i32 = 0;
    const AX_VALUE_CF_RANGE_TYPE: i32 = 4;

    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn AXUIElementCreateSystemWide() -> AXUIElementRef;
        fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> i32;
        fn AXUIElementCopyParameterizedAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            parameter: CFTypeRef,
            value: *mut CFTypeRef,
        ) -> i32;
        fn AXUIElementGetPid(element: AXUIElementRef, pid: *mut i32) -> i32;
        fn AXValueCreate(value_type: i32, value: *const c_void) -> AXValueRef;
        fn AXValueGetValue(value: AXValueRef, value_type: i32, value: *mut c_void) -> bool;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFHash(value: CFTypeRef) -> usize;
    }

    #[link(name = "proc")]
    unsafe extern "C" {
        fn proc_pidpath(pid: i32, buffer: *mut c_void, buffer_size: u32) -> i32;
    }

    unsafe fn copy_attribute(element: AXUIElementRef, name: &str) -> Option<CFType> {
        let attribute = CFString::new(name);
        let mut value: CFTypeRef = std::ptr::null();
        (AXUIElementCopyAttributeValue(element, attribute.as_concrete_TypeRef(), &mut value)
            == AX_ERROR_SUCCESS
            && !value.is_null())
        .then(|| CFType::wrap_under_create_rule(value))
    }

    fn as_string(value: &CFType) -> Option<String> {
        value.downcast::<CFString>().map(|value| value.to_string())
    }

    unsafe fn process_name(pid: i32) -> Option<String> {
        let mut buffer = vec![0_u8; 4096];
        let length = proc_pidpath(
            pid,
            buffer.as_mut_ptr().cast::<c_void>(),
            buffer.len() as u32,
        );
        if length <= 0 {
            return None;
        }
        let path = String::from_utf8_lossy(&buffer[..length as usize]);
        std::path::Path::new(path.as_ref())
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
    }

    unsafe fn preceding_text(element: AXUIElementRef) -> Option<String> {
        let selected_range = copy_attribute(element, "AXSelectedTextRange")?;
        let mut range = CFRange {
            location: 0,
            length: 0,
        };
        if !AXValueGetValue(
            selected_range.as_CFTypeRef(),
            AX_VALUE_CF_RANGE_TYPE,
            (&mut range as *mut CFRange).cast::<c_void>(),
        ) || range.location <= 0
        {
            return None;
        }
        let length = usize::min(range.location as usize, CONTEXT_WINDOW_CHARS) as isize;
        let requested = CFRange {
            location: range.location - length,
            length,
        };
        let parameter = AXValueCreate(
            AX_VALUE_CF_RANGE_TYPE,
            (&requested as *const CFRange).cast::<c_void>(),
        );
        if parameter.is_null() {
            return None;
        }
        let parameter = CFType::wrap_under_create_rule(parameter);
        let attribute = CFString::new("AXStringForRange");
        let mut value: CFTypeRef = std::ptr::null();
        let copied = AXUIElementCopyParameterizedAttributeValue(
            element,
            attribute.as_concrete_TypeRef(),
            parameter.as_CFTypeRef(),
            &mut value,
        );
        if copied != AX_ERROR_SUCCESS || value.is_null() {
            return None;
        }
        as_string(&CFType::wrap_under_create_rule(value))
    }

    unsafe {
        let system = AXUIElementCreateSystemWide();
        if system.is_null() {
            return PlatformContext::default();
        }
        let system = CFType::wrap_under_create_rule(system);
        let focused = match copy_attribute(system.as_CFTypeRef(), "AXFocusedUIElement") {
            Some(value) => value,
            None => return PlatformContext::default(),
        };
        let focused_ref = focused.as_CFTypeRef();
        let role = copy_attribute(focused_ref, "AXRole").and_then(|value| as_string(&value));
        let subrole = copy_attribute(focused_ref, "AXSubrole").and_then(|value| as_string(&value));
        let secure_field_known = role.is_some();
        let secure_field = role.as_deref() == Some("AXSecureTextField")
            || subrole.as_deref() == Some("AXSecureTextField");

        let mut pid = 0_i32;
        let pid_known = AXUIElementGetPid(focused_ref, &mut pid) == AX_ERROR_SUCCESS;
        let window_title = copy_attribute(focused_ref, "AXWindow")
            .and_then(|window| copy_attribute(window.as_CFTypeRef(), "AXTitle"))
            .and_then(|value| as_string(&value));

        let application_id = pid_known.then(|| process_name(pid)).flatten();
        let read_preceding_text = profile_allows_context(settings, application_id.as_deref());

        PlatformContext {
            application_id,
            window_title,
            selected_text: (read_selected_text && secure_field_known && !secure_field)
                .then(|| copy_attribute(focused_ref, "AXSelectedText"))
                .flatten()
                .and_then(|value| as_string(&value))
                .filter(|text| !text.trim().is_empty()),
            secure_field,
            secure_field_known,
            // Core Foundation hashes AX elements by their underlying
            // accessibility identity. Including it prevents a focus change to
            // another control in the same process from passing the target gate.
            target_id: pid_known.then(|| format!("macos:{pid}:{}", CFHash(focused_ref))),
            preceding_text: (read_preceding_text && secure_field_known && !secure_field)
                .then(|| preceding_text(focused_ref))
                .flatten(),
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn capture_platform_target(_settings: &AppSettings, _read_selected_text: bool) -> PlatformContext {
    PlatformContext::default()
}

#[cfg(test)]
mod tests {
    use super::{
        classify_application, context_access_status, diagnostics, profile_for_application,
        same_target, selected_text_block_reason, ContextAccessStatus,
    };
    use crate::contracts::PlatformContext;
    use crate::settings::{get_default_settings, AppBoundaryStyle, AppCategory};

    #[test]
    fn target_match_requires_stable_explicit_identity() {
        let mut left = PlatformContext::default();
        let mut right = PlatformContext::default();
        assert!(!same_target(&left, &right));
        left.target_id = Some("target-1".into());
        right.target_id = Some("target-1".into());
        assert!(same_target(&left, &right));
        right.target_id = Some("target-2".into());
        assert!(!same_target(&left, &right));
    }

    #[test]
    fn process_names_classify_without_window_content() {
        assert_eq!(
            classify_application(Some("OUTLOOK.EXE")),
            AppCategory::Email
        );
        assert_eq!(
            classify_application(Some("Slack.exe")),
            AppCategory::Messaging
        );
        assert_eq!(
            classify_application(Some("WINWORD.EXE")),
            AppCategory::Document
        );
        assert_eq!(classify_application(Some("Code.exe")), AppCategory::Code);
        assert_eq!(
            classify_application(Some("WindowsTerminal.exe")),
            AppCategory::Terminal
        );
        assert_eq!(
            classify_application(Some("browser.exe")),
            AppCategory::Other
        );
    }

    #[test]
    fn context_is_off_by_default_and_profiles_are_deterministic() {
        let settings = get_default_settings();
        let context = PlatformContext {
            application_id: Some("outlook.exe".into()),
            secure_field_known: true,
            ..PlatformContext::default()
        };
        assert_eq!(
            context_access_status(&settings, &context),
            ContextAccessStatus::Disabled
        );
        assert_eq!(
            profile_for_application(&settings, Some("code.exe")).boundary_style,
            AppBoundaryStyle::Literal
        );
    }

    #[test]
    fn secure_denied_remote_and_profile_disabled_paths_expose_no_text() {
        let mut settings = get_default_settings();
        settings.app_context_enabled = true;
        settings.app_context_denylist = vec!["secret.exe".into()];

        for (application, secure, known, expected) in [
            ("mail.exe", true, true, ContextAccessStatus::SecureField),
            (
                "mail.exe",
                false,
                false,
                ContextAccessStatus::SecurityUnknown,
            ),
            (
                "secret.exe",
                false,
                true,
                ContextAccessStatus::DeniedApplication,
            ),
            (
                "mstsc.exe",
                false,
                true,
                ContextAccessStatus::RemoteApplication,
            ),
            (
                "code.exe",
                false,
                true,
                ContextAccessStatus::ProfileDisabled,
            ),
        ] {
            let context = PlatformContext {
                application_id: Some(application.into()),
                window_title: Some("private title".into()),
                secure_field: secure,
                secure_field_known: known,
                preceding_text: Some("private text".into()),
                ..PlatformContext::default()
            };
            assert_eq!(context_access_status(&settings, &context), expected);
            let result = diagnostics(&settings, &context);
            assert_eq!(result.captured_characters, 0);
            assert_eq!(result.window_title, None);
        }
    }

    #[test]
    fn enabled_context_diagnostics_report_only_character_count() {
        let mut settings = get_default_settings();
        settings.app_context_enabled = true;
        let context = PlatformContext {
            application_id: Some("winword.exe".into()),
            window_title: Some("Document".into()),
            secure_field_known: true,
            preceding_text: Some("prior text".into()),
            ..PlatformContext::default()
        };
        let result = diagnostics(&settings, &context);
        assert_eq!(result.status, ContextAccessStatus::Enabled);
        assert_eq!(result.category, AppCategory::Document);
        assert_eq!(result.captured_characters, 10);
        assert_eq!(result.window_title.as_deref(), Some("Document"));
    }

    #[test]
    fn explicit_selection_capture_still_fails_closed_for_sensitive_targets() {
        let mut settings = get_default_settings();
        settings.app_context_denylist = vec!["denied.exe".into()];
        let base = PlatformContext {
            application_id: Some("editor.exe".into()),
            selected_text: Some("selected text".into()),
            secure_field_known: true,
            target_id: Some("target".into()),
            ..PlatformContext::default()
        };
        assert_eq!(selected_text_block_reason(&settings, &base), None);

        let mut secure = base.clone();
        secure.secure_field = true;
        assert_eq!(
            selected_text_block_reason(&settings, &secure),
            Some("secure_field")
        );

        let mut denied = base.clone();
        denied.application_id = Some("denied.exe".into());
        assert_eq!(
            selected_text_block_reason(&settings, &denied),
            Some("denied_application")
        );

        let mut remote = base;
        remote.application_id = Some("mstsc.exe".into());
        assert_eq!(
            selected_text_block_reason(&settings, &remote),
            Some("remote_application")
        );
    }
}
