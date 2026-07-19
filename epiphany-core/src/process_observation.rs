use anyhow::{Context, Result, bail};
use std::path::PathBuf;

/// The immutable operating-system identity of one process incarnation.
///
/// A PID alone is an address which the OS is free to reuse.  `creation_token`
/// is the native, monotonic-within-boot incarnation token (Windows FILETIME or
/// Linux `/proc/<pid>/stat` starttime), and the executable path prevents a
/// different image from inheriting authority through an erroneous launch.
/// `created_at_rfc3339` is a display projection derived from the native token;
/// its presence or formatting is not a fourth process-identity authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessInstanceIdentity {
    pub process_id: u32,
    pub creation_token: u64,
    pub created_at_rfc3339: Option<String>,
    pub executable_path: PathBuf,
}

fn same_process_incarnation(
    expected: &ProcessInstanceIdentity,
    observed: &ProcessInstanceIdentity,
) -> bool {
    expected.process_id == observed.process_id
        && expected.creation_token == observed.creation_token
        && expected.executable_path == observed.executable_path
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProcessInstanceObservation {
    ExactAlive,
    ExactExited { exit_code: Option<u32> },
    Replaced { observed: ProcessInstanceIdentity },
    Missing,
    Inaccessible,
    Indeterminate { reason: String },
}

/// Compatibility projection for old operator displays.  It is deliberately
/// incapable of calling access denial or PID absence "dead".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpiphanyProcessObservation {
    Alive,
    Missing,
}

impl EpiphanyProcessObservation {
    pub fn label(self) -> &'static str {
        match self {
            Self::Alive => "alive",
            Self::Missing => "missing",
        }
    }
}

pub fn capture_process_instance(process_id: u32) -> Result<ProcessInstanceIdentity> {
    if process_id == 0 {
        bail!("process-instance identity requires a nonzero PID");
    }
    platform::capture(process_id)
}

pub fn observe_process_instance(expected: &ProcessInstanceIdentity) -> ProcessInstanceObservation {
    if expected.process_id == 0 || expected.creation_token == 0 {
        return ProcessInstanceObservation::Indeterminate {
            reason: "invalid expected process-instance identity".to_string(),
        };
    }
    platform::observe(expected)
}

pub fn terminate_process_instance(expected: &ProcessInstanceIdentity) -> Result<()> {
    if observe_process_instance(expected) != ProcessInstanceObservation::ExactAlive {
        bail!("refusing to terminate a process whose exact incarnation is not proven alive");
    }
    platform::terminate(expected)
}

/// Returns a boot-incarnation token only when the platform can prove it.
pub fn native_boot_identity() -> Option<String> {
    platform::boot_identity()
}

pub fn native_process_executable_path(process_id: u32) -> Result<Option<PathBuf>> {
    match capture_process_instance(process_id) {
        Ok(identity) => Ok(Some(identity.executable_path)),
        Err(error) if platform::is_missing_error(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn observe_native_process(process_id: u32) -> Result<EpiphanyProcessObservation> {
    // This legacy API has no expected incarnation and therefore cannot prove
    // termination. It remains useful only as a conservative liveness display.
    match capture_process_instance(process_id) {
        Ok(_) => Ok(EpiphanyProcessObservation::Alive),
        Err(error) if platform::is_missing_error(&error) => Ok(EpiphanyProcessObservation::Missing),
        Err(error) => Err(error),
    }
}

#[cfg(windows)]
mod platform {
    use super::*;
    use chrono::{DateTime, SecondsFormat, Utc};
    use std::os::windows::ffi::OsStringExt;
    use windows_sys::Win32::Foundation::{
        CloseHandle, ERROR_ACCESS_DENIED, ERROR_INVALID_PARAMETER, ERROR_NO_MORE_FILES, FILETIME,
        GetLastError, INVALID_HANDLE_VALUE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT,
    };
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        PROCESS_TERMINATE, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
    };

    #[repr(C)]
    struct SystemTimeOfDayInformation {
        boot_time: i64,
        current_time: i64,
        time_zone_bias: i64,
        current_time_zone_id: u32,
        reserved: u32,
        boot_time_bias: u64,
        sleep_time_bias: u64,
    }

    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn NtQuerySystemInformation(
            system_information_class: u32,
            system_information: *mut core::ffi::c_void,
            system_information_length: u32,
            return_length: *mut u32,
        ) -> i32;
    }

    const WINDOWS_TO_UNIX_100NS: u64 = 116_444_736_000_000_000;
    const SYNCHRONIZE_ACCESS: u32 = 0x0010_0000;

    struct OwnedHandle(windows_sys::Win32::Foundation::HANDLE);
    impl Drop for OwnedHandle {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    fn filetime_token(value: FILETIME) -> u64 {
        ((value.dwHighDateTime as u64) << 32) | value.dwLowDateTime as u64
    }

    fn rfc3339_from_filetime(token: u64) -> Result<String> {
        let unix_100ns = token
            .checked_sub(WINDOWS_TO_UNIX_100NS)
            .context("process creation FILETIME predates Unix epoch")?;
        let seconds = (unix_100ns / 10_000_000) as i64;
        let nanos = ((unix_100ns % 10_000_000) * 100) as u32;
        let value = DateTime::<Utc>::from_timestamp(seconds, nanos)
            .context("process creation FILETIME is out of range")?;
        Ok(value.to_rfc3339_opts(SecondsFormat::Nanos, true))
    }

    fn pid_in_snapshot(process_id: u32) -> Result<bool> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return Err(std::io::Error::last_os_error()).context("create process snapshot");
            }
            let snapshot = OwnedHandle(snapshot);
            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
            if Process32FirstW(snapshot.0, &mut entry) == 0 {
                return Err(std::io::Error::last_os_error()).context("read process snapshot");
            }
            loop {
                if entry.th32ProcessID == process_id {
                    return Ok(true);
                }
                if Process32NextW(snapshot.0, &mut entry) == 0 {
                    let error = GetLastError();
                    if error == ERROR_NO_MORE_FILES {
                        return Ok(false);
                    }
                    return Err(std::io::Error::from_raw_os_error(error as i32))
                        .context("advance process snapshot");
                }
            }
        }
    }

    fn open(process_id: u32) -> Result<OwnedHandle> {
        unsafe {
            let handle = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE_ACCESS,
                0,
                process_id,
            );
            if handle.is_null() {
                return Err(std::io::Error::last_os_error())
                    .with_context(|| format!("open process {process_id}"));
            }
            Ok(OwnedHandle(handle))
        }
    }

    fn identity_from_handle(
        process_id: u32,
        handle: &OwnedHandle,
    ) -> Result<ProcessInstanceIdentity> {
        unsafe {
            let mut creation: FILETIME = std::mem::zeroed();
            let mut exit: FILETIME = std::mem::zeroed();
            let mut kernel: FILETIME = std::mem::zeroed();
            let mut user: FILETIME = std::mem::zeroed();
            if GetProcessTimes(handle.0, &mut creation, &mut exit, &mut kernel, &mut user) == 0 {
                return Err(std::io::Error::last_os_error()).context("query process creation time");
            }
            let creation_token = filetime_token(creation);
            if creation_token == 0 {
                bail!("operating system returned a zero process creation token");
            }
            let mut buffer = vec![0_u16; 32_768];
            let mut length = buffer.len() as u32;
            if QueryFullProcessImageNameW(handle.0, 0, buffer.as_mut_ptr(), &mut length) == 0 {
                return Err(std::io::Error::last_os_error())
                    .context("query process executable path");
            }
            buffer.truncate(length as usize);
            let raw = PathBuf::from(std::ffi::OsString::from_wide(&buffer));
            let executable_path = std::fs::canonicalize(&raw)
                .with_context(|| format!("canonicalize process executable {}", raw.display()))?;
            Ok(ProcessInstanceIdentity {
                process_id,
                creation_token,
                created_at_rfc3339: Some(rfc3339_from_filetime(creation_token)?),
                executable_path,
            })
        }
    }

    pub(super) fn capture(process_id: u32) -> Result<ProcessInstanceIdentity> {
        identity_from_handle(process_id, &open(process_id)?)
    }

    pub(super) fn observe(expected: &ProcessInstanceIdentity) -> ProcessInstanceObservation {
        let present = match pid_in_snapshot(expected.process_id) {
            Ok(value) => value,
            Err(error) => {
                return ProcessInstanceObservation::Indeterminate {
                    reason: error.to_string(),
                };
            }
        };
        if !present {
            return ProcessInstanceObservation::Missing;
        }
        let handle = match open(expected.process_id) {
            Ok(handle) => handle,
            Err(error) => {
                return if error
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(|e| e.raw_os_error() == Some(ERROR_ACCESS_DENIED as i32))
                {
                    ProcessInstanceObservation::Inaccessible
                } else {
                    ProcessInstanceObservation::Indeterminate {
                        reason: error.to_string(),
                    }
                };
            }
        };
        let observed = match identity_from_handle(expected.process_id, &handle) {
            Ok(observed) if !same_process_incarnation(expected, &observed) => {
                return ProcessInstanceObservation::Replaced { observed };
            }
            Ok(observed) => observed,
            Err(error) => {
                return ProcessInstanceObservation::Indeterminate {
                    reason: error.to_string(),
                };
            }
        };
        debug_assert!(same_process_incarnation(expected, &observed));
        unsafe {
            match WaitForSingleObject(handle.0, 0) {
                WAIT_OBJECT_0 => {
                    let mut code = 0;
                    let exit_code = (GetExitCodeProcess(handle.0, &mut code) != 0).then_some(code);
                    ProcessInstanceObservation::ExactExited { exit_code }
                }
                WAIT_TIMEOUT => ProcessInstanceObservation::ExactAlive,
                WAIT_FAILED => ProcessInstanceObservation::Indeterminate {
                    reason: format!(
                        "wait for process failed with Windows error {}",
                        GetLastError()
                    ),
                },
                other => ProcessInstanceObservation::Indeterminate {
                    reason: format!("unexpected process wait result {other}"),
                },
            }
        }
    }

    pub(super) fn terminate(expected: &ProcessInstanceIdentity) -> Result<()> {
        let raw = unsafe {
            OpenProcess(
                PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION,
                0,
                expected.process_id,
            )
        };
        if raw.is_null() {
            bail!("failed to open exact process for termination");
        }
        let handle = OwnedHandle(raw);
        if identity_from_handle(expected.process_id, &handle)? != *expected {
            bail!("process incarnation changed before termination actuator acquired its handle");
        }
        let ok = unsafe { TerminateProcess(handle.0, 1) };
        if ok == 0 {
            bail!("failed to terminate exact process");
        }
        Ok(())
    }

    pub(super) fn boot_identity() -> Option<String> {
        let mut information = SystemTimeOfDayInformation {
            boot_time: 0,
            current_time: 0,
            time_zone_bias: 0,
            current_time_zone_id: 0,
            reserved: 0,
            boot_time_bias: 0,
            sleep_time_bias: 0,
        };
        let status = unsafe {
            NtQuerySystemInformation(
                3,
                (&mut information as *mut SystemTimeOfDayInformation).cast(),
                std::mem::size_of::<SystemTimeOfDayInformation>() as u32,
                std::ptr::null_mut(),
            )
        };
        (status >= 0 && information.boot_time > 0)
            .then(|| format!("windows-kernel-boot-filetime:{}", information.boot_time))
    }

    pub(super) fn is_missing_error(error: &anyhow::Error) -> bool {
        error.chain().any(|source| {
            source.downcast_ref::<std::io::Error>().is_some_and(|e| {
            matches!(e.raw_os_error(), Some(code) if code == ERROR_INVALID_PARAMETER as i32)
        })
        })
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use chrono::{DateTime, SecondsFormat, Utc};

    fn stat(process_id: u32) -> Result<(u64, char)> {
        let raw = std::fs::read_to_string(format!("/proc/{process_id}/stat"))
            .with_context(|| format!("read /proc/{process_id}/stat"))?;
        let close = raw
            .rfind(") ")
            .context("malformed proc stat command field")?;
        let fields: Vec<&str> = raw[close + 2..].split_whitespace().collect();
        let state = fields
            .first()
            .and_then(|value| value.chars().next())
            .context("missing proc state")?;
        // starttime is field 22; `fields[0]` is original field 3.
        let starttime = fields.get(19).context("missing proc starttime")?.parse()?;
        Ok((starttime, state))
    }

    fn created_at_rfc3339(starttime: u64) -> Result<String> {
        let ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
        if ticks_per_second <= 0 {
            bail!("operating system returned an invalid process clock frequency");
        }
        let boot_seconds = std::fs::read_to_string("/proc/stat")?
            .lines()
            .find_map(|line| line.strip_prefix("btime "))
            .context("proc stat has no boot time")?
            .parse::<i64>()?;
        let ticks_per_second = ticks_per_second as u64;
        let seconds = boot_seconds
            .checked_add((starttime / ticks_per_second) as i64)
            .context("process creation time overflow")?;
        let nanos = ((starttime % ticks_per_second) * 1_000_000_000 / ticks_per_second) as u32;
        let created = DateTime::<Utc>::from_timestamp(seconds, nanos)
            .context("process creation time is out of range")?;
        Ok(created.to_rfc3339_opts(SecondsFormat::Nanos, true))
    }

    pub(super) fn capture(process_id: u32) -> Result<ProcessInstanceIdentity> {
        let (creation_token, _) = stat(process_id)?;
        if creation_token == 0 {
            bail!("operating system returned a zero process creation token");
        }
        let executable_path = std::fs::canonicalize(format!("/proc/{process_id}/exe"))
            .with_context(|| format!("read process {process_id} executable"))?;
        Ok(ProcessInstanceIdentity {
            process_id,
            creation_token,
            created_at_rfc3339: Some(created_at_rfc3339(creation_token)?),
            executable_path,
        })
    }

    pub(super) fn observe(expected: &ProcessInstanceIdentity) -> ProcessInstanceObservation {
        let (creation_token, state) = match stat(expected.process_id) {
            Ok(observed) => observed,
            Err(error) if is_missing_error(&error) => return ProcessInstanceObservation::Missing,
            Err(error) => {
                return ProcessInstanceObservation::Indeterminate {
                    reason: error.to_string(),
                };
            }
        };
        if creation_token == expected.creation_token && matches!(state, 'Z' | 'X') {
            return ProcessInstanceObservation::ExactExited { exit_code: None };
        }
        match capture(expected.process_id) {
            Ok(observed) if !same_process_incarnation(expected, &observed) => {
                ProcessInstanceObservation::Replaced { observed }
            }
            Ok(_) => ProcessInstanceObservation::ExactAlive,
            Err(error) => ProcessInstanceObservation::Indeterminate {
                reason: error.to_string(),
            },
        }
    }

    pub(super) fn terminate(expected: &ProcessInstanceIdentity) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            let pidfd = unsafe {
                libc::syscall(libc::SYS_pidfd_open, expected.process_id as i32, 0) as i32
            };
            if pidfd < 0 {
                return Err(std::io::Error::last_os_error())
                    .context("pidfd_open exact coordinator");
            }
            if observe(expected) != ProcessInstanceObservation::ExactAlive {
                unsafe { libc::close(pidfd) };
                bail!("process incarnation changed before pidfd termination");
            }
            let result = unsafe {
                libc::syscall(
                    libc::SYS_pidfd_send_signal,
                    pidfd,
                    libc::SIGTERM,
                    std::ptr::null::<libc::siginfo_t>(),
                    0,
                )
            };
            unsafe { libc::close(pidfd) };
            if result != 0 {
                return Err(std::io::Error::last_os_error())
                    .context("pidfd_send_signal exact coordinator");
            }
            return Ok(());
        }
        #[cfg(not(target_os = "linux"))]
        bail!(
            "exact inherited process termination requires a native incarnation handle on this Unix platform"
        )
    }

    pub(super) fn boot_identity() -> Option<String> {
        std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub(super) fn is_missing_error(error: &anyhow::Error) -> bool {
        error.chain().any(|source| {
            source
                .downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        })
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
mod platform {
    use super::*;
    pub(super) fn capture(_: u32) -> Result<ProcessInstanceIdentity> {
        bail!("native process-instance observation is unsupported on this platform")
    }
    pub(super) fn observe(_: &ProcessInstanceIdentity) -> ProcessInstanceObservation {
        ProcessInstanceObservation::Indeterminate {
            reason: "native process-instance observation is unsupported on this platform"
                .to_string(),
        }
    }
    pub(super) fn terminate(_: &ProcessInstanceIdentity) -> Result<()> {
        bail!("exact process termination is unsupported on this platform")
    }
    pub(super) fn boot_identity() -> Option<String> {
        None
    }
    pub(super) fn is_missing_error(_: &anyhow::Error) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_pid() {
        assert!(capture_process_instance(0).is_err());
    }

    #[test]
    fn native_boot_identity_is_available_and_stable() {
        let first = native_boot_identity().expect("native boot identity unavailable");
        let second = native_boot_identity().expect("native boot identity disappeared");
        assert_eq!(first, second);
    }

    #[test]
    fn live_current_process_has_stable_exact_identity() {
        let identity = capture_process_instance(std::process::id()).unwrap();
        assert_ne!(identity.creation_token, 0);
        assert!(identity.created_at_rfc3339.is_some());
        assert!(identity.executable_path.is_absolute());
        assert_eq!(
            observe_process_instance(&identity),
            ProcessInstanceObservation::ExactAlive
        );
    }

    #[test]
    fn derived_creation_time_is_not_process_identity_authority() {
        let mut identity = capture_process_instance(std::process::id()).unwrap();
        identity.created_at_rfc3339 = None;
        assert_eq!(
            observe_process_instance(&identity),
            ProcessInstanceObservation::ExactAlive
        );
    }

    #[test]
    fn forged_incarnation_is_replaced_not_alive() {
        let mut identity = capture_process_instance(std::process::id()).unwrap();
        identity.creation_token = identity.creation_token.saturating_add(1);
        assert!(matches!(
            observe_process_instance(&identity),
            ProcessInstanceObservation::Replaced { .. }
        ));
    }

    #[test]
    fn child_exit_is_observed_for_the_exact_instance() {
        let mut child = if cfg!(windows) {
            std::process::Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "Start-Sleep -Milliseconds 500; exit 17",
                ])
                .spawn()
                .unwrap()
        } else {
            std::process::Command::new("sh")
                .args(["-c", "exit 17"])
                .spawn()
                .unwrap()
        };
        let identity = capture_process_instance(child.id()).unwrap();
        let _ = child.wait().unwrap();
        let observed = observe_process_instance(&identity);
        assert!(matches!(
            observed,
            ProcessInstanceObservation::ExactExited {
                exit_code: Some(17)
            } | ProcessInstanceObservation::Missing
        ));
    }
}
