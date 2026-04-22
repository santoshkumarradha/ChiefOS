//! Device capability probe — RAM, CPU, GPU, architecture detection.

use serde::{Deserialize, Serialize};

/// Device architecture enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arch {
    AppleSilicon,
    X86_64,
    Aarch64Linux,
}

/// Device capability probe result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapability {
    pub ram_gb: u32,
    pub cpu_cores: u32,
    pub has_gpu: bool,
    pub arch: Arch,
}

impl DeviceCapability {
    /// Probe current device capabilities.
    ///
    /// Returns:
    /// - RAM in GB (parsed from /proc/meminfo or system calls)
    /// - CPU core count
    /// - GPU presence (heuristic: Apple Silicon always true, Linux check sysfs)
    /// - Architecture (platform-specific)
    pub fn probe() -> Self {
        let (ram_gb, cpu_cores, arch, has_gpu) = Self::probe_platform();
        DeviceCapability {
            ram_gb,
            cpu_cores,
            has_gpu,
            arch,
        }
    }

    /// Can this device run a tier locally?
    /// Per ADR-0013: Fast needs 4GB+, Deep needs 14GB+ (GPU-preferred for Deep).
    pub fn can_run_tier_locally(&self, tier: &chief_sdk::Tier) -> bool {
        match tier {
            chief_sdk::Tier::Fast => {
                // Fast (Qwen 2.5 3B Q4) needs ~4GB + headroom
                self.ram_gb >= 6
            }
            chief_sdk::Tier::Deep => {
                // Deep (Qwen 14B Q4) needs ~14GB + headroom, prefers GPU
                // On Apple Silicon 16GB is minimum; on x86 16GB also tight without GPU
                if self.has_gpu {
                    self.ram_gb >= 12 // With GPU, 12GB can work
                } else {
                    self.ram_gb >= 24 // Without GPU, need more
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn probe_platform() -> (u32, u32, Arch, bool) {
        // macOS: use sysctl to get RAM and CPU count
        let ram_bytes = Self::sysctl_u64("hw.memsize").unwrap_or(0);
        let ram_gb = (ram_bytes / (1024 * 1024 * 1024)) as u32;

        let cpu_cores = num_cpus::get() as u32;

        // Detect Apple Silicon
        let arch = if std::env::consts::ARCH == "aarch64" {
            Arch::AppleSilicon
        } else {
            Arch::X86_64
        };

        // Apple Silicon always has GPU
        let has_gpu = arch == Arch::AppleSilicon;

        (ram_gb, cpu_cores, arch, has_gpu)
    }

    #[cfg(target_os = "linux")]
    fn probe_platform() -> (u32, u32, Arch, bool) {
        // Linux: parse /proc/meminfo and check architecture
        let ram_gb = Self::meminfo_ram_gb().unwrap_or(0);
        let cpu_cores = num_cpus::get() as u32;

        let arch = if std::env::consts::ARCH == "aarch64" {
            Arch::Aarch64Linux
        } else {
            Arch::X86_64
        };

        // Check for GPU (heuristic: /proc/driver/nvidia or /dev/dri)
        let has_gpu = std::path::Path::new("/proc/driver/nvidia").exists()
            || std::path::Path::new("/dev/dri").exists();

        (ram_gb, cpu_cores, arch, has_gpu)
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    fn probe_platform() -> (u32, u32, Arch, bool) {
        // Fallback for other platforms
        let ram_gb = 8; // Conservative default
        let cpu_cores = num_cpus::get() as u32;
        let arch = if std::env::consts::ARCH == "aarch64" {
            Arch::Aarch64Linux
        } else {
            Arch::X86_64
        };
        let has_gpu = false;
        (ram_gb, cpu_cores, arch, has_gpu)
    }

    #[cfg(target_os = "macos")]
    fn sysctl_u64(name: &str) -> Option<u64> {
        use std::process::Command;
        let output = Command::new("sysctl").arg("-n").arg(name).output().ok()?;

        if output.status.success() {
            let val_str = String::from_utf8(output.stdout).ok()?;
            val_str.trim().parse().ok()
        } else {
            None
        }
    }

    #[cfg(target_os = "linux")]
    fn meminfo_ram_gb() -> Option<u32> {
        let content = std::fs::read_to_string("/proc/meminfo").ok()?;
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return Some((kb / (1024 * 1024)) as u32);
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chief_sdk::Tier;

    #[test]
    fn can_run_tier_locally_fast_minimum() {
        let dev = DeviceCapability {
            ram_gb: 6,
            cpu_cores: 4,
            has_gpu: false,
            arch: Arch::X86_64,
        };
        assert!(dev.can_run_tier_locally(&Tier::Fast));
    }

    #[test]
    fn can_run_tier_locally_fast_insufficient() {
        let dev = DeviceCapability {
            ram_gb: 4,
            cpu_cores: 4,
            has_gpu: false,
            arch: Arch::X86_64,
        };
        assert!(!dev.can_run_tier_locally(&Tier::Fast));
    }

    #[test]
    fn can_run_tier_locally_deep_with_gpu() {
        let dev = DeviceCapability {
            ram_gb: 16,
            cpu_cores: 8,
            has_gpu: true,
            arch: Arch::AppleSilicon,
        };
        assert!(dev.can_run_tier_locally(&Tier::Deep));
    }

    #[test]
    fn can_run_tier_locally_deep_without_gpu() {
        let dev = DeviceCapability {
            ram_gb: 16,
            cpu_cores: 8,
            has_gpu: false,
            arch: Arch::X86_64,
        };
        // 16GB without GPU is insufficient for Deep
        assert!(!dev.can_run_tier_locally(&Tier::Deep));
    }

    #[test]
    fn can_run_tier_locally_deep_plenty_ram() {
        let dev = DeviceCapability {
            ram_gb: 32,
            cpu_cores: 16,
            has_gpu: false,
            arch: Arch::X86_64,
        };
        assert!(dev.can_run_tier_locally(&Tier::Deep));
    }
}
