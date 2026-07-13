use anyhow::Result;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpiphanyProcessObservation {
    Alive,
    Dead,
    Missing,
}

#[cfg(windows)]
pub fn native_process_executable_path(process_id: u32) -> Result<Option<PathBuf>> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
    };
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
        if handle.is_null() {
            return Ok(None);
        }
        let mut buffer = vec![0_u16; 32_768];
        let mut length = buffer.len() as u32;
        let read = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length);
        CloseHandle(handle);
        if read == 0 {
            return Ok(None);
        }
        buffer.truncate(length as usize);
        Ok(Some(PathBuf::from(String::from_utf16(&buffer)?)))
    }
}

#[cfg(unix)]
pub fn native_process_executable_path(process_id: u32) -> Result<Option<PathBuf>> {
    let path = PathBuf::from(format!("/proc/{process_id}/exe"));
    match std::fs::read_link(path) {
        Ok(path) => Ok(Some(path)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

impl EpiphanyProcessObservation {
    pub fn label(self) -> &'static str {
        match self {
            Self::Alive => "alive",
            Self::Dead => "dead",
            Self::Missing => "missing",
        }
    }
}

#[cfg(windows)]
pub fn observe_native_process(process_id: u32) -> Result<EpiphanyProcessObservation> {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    const STILL_ACTIVE: u32 = 259;
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
        if handle.is_null() {
            return Ok(EpiphanyProcessObservation::Dead);
        }
        let mut exit_code = 0_u32;
        let read = GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle);
        if read == 0 {
            anyhow::bail!("failed to inspect managed service process {process_id}");
        }
        Ok(if exit_code == STILL_ACTIVE {
            EpiphanyProcessObservation::Alive
        } else {
            EpiphanyProcessObservation::Dead
        })
    }
}

#[cfg(unix)]
pub fn observe_native_process(process_id: u32) -> Result<EpiphanyProcessObservation> {
    let result = unsafe { libc::kill(process_id as i32, 0) };
    if result == 0 {
        Ok(EpiphanyProcessObservation::Alive)
    } else {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ESRCH) {
            Ok(EpiphanyProcessObservation::Dead)
        } else if error.raw_os_error() == Some(libc::EPERM) {
            Ok(EpiphanyProcessObservation::Alive)
        } else {
            Err(error.into())
        }
    }
}
