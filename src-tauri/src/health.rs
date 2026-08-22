//! Performance Monitor (Phase 11): a cheap system-resource snapshot for the Performance dock
//! panel — total CPU utilisation, system memory load, and this process's USER/GDI object counts
//! (the classic WebView/desktop handle-leak signals). Petrel's Process Health Monitor shows the
//! same style of gauges.
//!
//! Windows-only metrics; on any other target every field is `None` so the panel shows "n/a".
//! Every Win32 call is fully error-checked and returns `None` on failure — a missing metric
//! never panics or crashes the poll.

/// A single resource reading. Percentages are 0..100; `*_count` carries the raw object counts
/// for the tooltip.
#[derive(serde::Serialize, Default)]
pub(crate) struct HealthSnapshot {
    /// System memory in use, 0..100 (`MEMORYSTATUSEX.dwMemoryLoad`).
    pub mem_system: Option<f32>,
    /// Total physical memory, MB (`MEMORYSTATUSEX.ullTotalPhys`). Constant for the session, so
    /// the panel does not plot it - it is here for the diagnostic report, where "85% memory in
    /// use" means something very different on an 8 GB field laptop and a 64 GB workstation.
    pub mem_total_mb: Option<u64>,
    /// Total CPU utilisation across all cores, 0..100, over the interval since the previous
    /// snapshot (`GetSystemTimes` busy-vs-idle delta). `None` on the first snapshot (no baseline).
    pub cpu_load: Option<f32>,
    /// This process's USER objects as % of the 10,000 per-process ceiling.
    pub user_objects: Option<f32>,
    /// This process's GDI objects as % of the 10,000 per-process ceiling.
    pub gdi_objects: Option<f32>,
    pub user_count: Option<u32>,
    pub gdi_count: Option<u32>,
}

/// Windows' default per-process ceiling for BOTH USER and GDI objects. Approaching it is the
/// signature of a handle leak (the app stops drawing / creating windows near the limit).
#[cfg(windows)]
const OBJ_LIMIT: f32 = 10_000.0;

#[cfg(windows)]
pub(crate) fn snapshot() -> HealthSnapshot {
    let user = gui_count(true);
    let gdi = gui_count(false);
    let (mem_system, mem_total_mb) = mem_status();
    HealthSnapshot {
        mem_system,
        mem_total_mb,
        cpu_load: cpu_load(),
        user_objects: user.map(|c| c as f32 / OBJ_LIMIT * 100.0),
        gdi_objects: gdi.map(|c| c as f32 / OBJ_LIMIT * 100.0),
        user_count: user,
        gdi_count: gdi,
    }
}

#[cfg(not(windows))]
pub(crate) fn snapshot() -> HealthSnapshot {
    HealthSnapshot::default()
}

/// Memory load and total physical memory from ONE `GlobalMemoryStatusEx` call - the struct
/// carries both, and asking twice could return two readings taken a moment apart.
#[cfg(windows)]
fn mem_status() -> (Option<f32>, Option<u64>) {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut s = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    if unsafe { GlobalMemoryStatusEx(&mut s) }.is_err() {
        return (None, None);
    }
    (Some(s.dwMemoryLoad as f32), Some(s.ullTotalPhys / (1024 * 1024)))
}

/// USER or GDI object count for THIS process. `GetGuiResources` returns 0 on error, and a
/// live GUI process always has some objects, so 0 is treated as "unavailable".
#[cfg(windows)]
fn gui_count(user: bool) -> Option<u32> {
    use windows::Win32::System::Threading::{
        GetCurrentProcess, GetGuiResources, GR_GDIOBJECTS, GR_USEROBJECTS,
    };
    let flag = if user { GR_USEROBJECTS } else { GR_GDIOBJECTS };
    let n = unsafe { GetGuiResources(GetCurrentProcess(), flag) };
    (n > 0).then_some(n)
}

/// Total system CPU utilisation (all cores), 0..100, over the interval since the previous call.
/// `GetSystemTimes` reports cumulative idle/kernel/user tick counters; CPU% is the busy fraction
/// of the delta between two reads. A process-static baseline holds the previous counters, so the
/// panel's fixed poll cadence defines the averaging window. The first call has no baseline and
/// returns `None` (the gauge shows "n/a" for one tick, then live values).
#[cfg(windows)]
fn cpu_load() -> Option<f32> {
    use std::sync::Mutex;
    use windows::Win32::Foundation::FILETIME;
    use windows::Win32::System::Threading::GetSystemTimes;

    fn ticks(f: FILETIME) -> u64 {
        ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64
    }
    // (idle, total) counters from the previous snapshot. `Mutex::new` is const, so no lazy-init
    // machinery is needed; poll contention is nil (one caller on a ~1.5 s timer).
    static PREV: Mutex<Option<(u64, u64)>> = Mutex::new(None);

    let (mut idle, mut kernel, mut user) =
        (FILETIME::default(), FILETIME::default(), FILETIME::default());
    unsafe { GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)) }.ok()?;
    let idle_now = ticks(idle);
    // Kernel time already includes idle time, so total = kernel + user and busy = total - idle.
    let total_now = ticks(kernel) + ticks(user);

    let mut prev = PREV.lock().ok()?;
    let out = match *prev {
        Some((idle_prev, total_prev)) if total_now > total_prev => {
            let d_total = (total_now - total_prev) as f64;
            let d_idle = idle_now.saturating_sub(idle_prev) as f64;
            Some((100.0 * (1.0 - d_idle / d_total)).clamp(0.0, 100.0) as f32)
        }
        _ => None, // first sample (or a counter reset): establish the baseline, report nothing yet
    };
    *prev = Some((idle_now, total_now));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_never_panics_and_percentages_are_sane() {
        let s = snapshot();
        for p in [s.mem_system, s.cpu_load, s.user_objects, s.gdi_objects].into_iter().flatten() {
            assert!(p >= 0.0 && p <= 1000.0, "percentage out of any sane range: {p}");
        }
    }
}
