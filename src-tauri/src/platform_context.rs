use crate::contracts::PlatformContext;

const CONTEXT_WINDOW_CHARS: usize = 16;

pub fn capture_active_target() -> PlatformContext {
    capture_platform_target()
}

pub fn same_target(left: &PlatformContext, right: &PlatformContext) -> bool {
    match (&left.target_id, &right.target_id) {
        (Some(left), Some(right)) => left == right,
        _ => false,
    }
}

#[cfg(target_os = "windows")]
fn capture_platform_target() -> PlatformContext {
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

    unsafe fn focused_accessibility() -> Option<(bool, String, Option<String>)> {
        let initialized = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let result = (|| {
            let automation: IUIAutomation =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok()?;
            let element = automation.GetFocusedElement().ok()?;
            let secure = element.CurrentIsPassword().ok()?.as_bool();
            let runtime_id = runtime_id(&element);
            if secure {
                return Some((true, String::new(), runtime_id));
            }

            let preceding = (|| {
                let pattern: IUIAutomationTextPattern2 =
                    element.GetCurrentPatternAs(UIA_TextPattern2Id).ok()?;
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
            Some((false, preceding, runtime_id))
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
        let accessibility = focused_accessibility();
        let secure_field = accessibility
            .as_ref()
            .map(|(secure, _, _)| *secure)
            .unwrap_or(false);
        let preceding_text = accessibility
            .as_ref()
            .and_then(|(secure, text, _)| (!*secure && !text.is_empty()).then(|| text.clone()));
        let target_id = accessibility
            .as_ref()
            .and_then(|(_, _, runtime_id)| runtime_id.as_ref())
            .map(|runtime_id| format!("windows:{process_id}:{runtime_id}"));

        PlatformContext {
            application_id: process_name(process_id),
            window_title,
            selected_text: None,
            secure_field,
            secure_field_known: accessibility.is_some(),
            target_id,
            preceding_text,
        }
    }
}

#[cfg(target_os = "macos")]
fn capture_platform_target() -> PlatformContext {
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

        PlatformContext {
            application_id: pid_known.then(|| process_name(pid)).flatten(),
            window_title,
            selected_text: None,
            secure_field,
            secure_field_known,
            // Core Foundation hashes AX elements by their underlying
            // accessibility identity. Including it prevents a focus change to
            // another control in the same process from passing the target gate.
            target_id: pid_known.then(|| format!("macos:{pid}:{}", CFHash(focused_ref))),
            preceding_text: (secure_field_known && !secure_field)
                .then(|| preceding_text(focused_ref))
                .flatten(),
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn capture_platform_target() -> PlatformContext {
    PlatformContext::default()
}

#[cfg(test)]
mod tests {
    use super::same_target;
    use crate::contracts::PlatformContext;

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
}
