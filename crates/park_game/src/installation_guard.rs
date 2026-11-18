//! Lets the Windows installer detect a running game before replacing its files.

#[cfg(windows)]
pub(super) struct InstallationGuard(windows::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl InstallationGuard {
    pub(super) fn acquire() -> windows::core::Result<Self> {
        use windows::{Win32::System::Threading::CreateMutexW, core::w};

        // Each instance retains a handle until exit. Parallel parks remain usable.
        unsafe {
            CreateMutexW(
                None,
                false,
                w!("Local\\Parkbound.Game.34245805-94B2-4E62-9306-1B56003A3F55"),
            )
            .map(Self)
        }
    }
}

#[cfg(windows)]
impl Drop for InstallationGuard {
    fn drop(&mut self) {
        // This handle was created above and is owned solely by this guard.
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(self.0) };
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use windows::{
        Win32::System::Threading::{OpenMutexW, SYNCHRONIZATION_SYNCHRONIZE},
        core::w,
    };

    #[test]
    fn running_instances_expose_the_installer_mutex() {
        let first = InstallationGuard::acquire().unwrap();
        let second = InstallationGuard::acquire().unwrap();
        drop(first);
        let observed = unsafe {
            OpenMutexW(
                SYNCHRONIZATION_SYNCHRONIZE,
                false,
                w!("Local\\Parkbound.Game.34245805-94B2-4E62-9306-1B56003A3F55"),
            )
        }
        .unwrap();
        unsafe { windows::Win32::Foundation::CloseHandle(observed).unwrap() };
        drop(second);
    }
}
