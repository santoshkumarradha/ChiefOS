//! First-run binding ceremony and Reset Chief affordance.

use crate::kernel_principal::attestation::{minimal_kernel_bootstrap_grants, BootAttestation};
use crate::kernel_principal::identity::{device_key_path, DeviceIdentity};
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{self, IsTerminal, Write};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResetChiefLink {
    pub label: String,
    pub href: String,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct FirstRunCeremonyOutcome {
    pub first_run: bool,
    pub identity: DeviceIdentity,
    pub attestation: BootAttestation,
    pub reset_chief: ResetChiefLink,
}

pub fn boot_kernel_principal(chief_home: impl AsRef<Path>) -> Result<FirstRunCeremonyOutcome> {
    let chief_home = chief_home.as_ref();
    let key_path = device_key_path(chief_home);
    let first_run = !key_path.exists();

    if first_run {
        prompt_bind_device()?;
    }

    let identity = DeviceIdentity::load_or_create(&key_path)?;
    let attestation =
        BootAttestation::sign(&identity, minimal_kernel_bootstrap_grants(chief_home))?;
    attestation
        .append_jsonl(chief_home)
        .context("append first boot attestation")?;

    Ok(FirstRunCeremonyOutcome {
        first_run,
        identity,
        attestation,
        reset_chief: reset_chief_link(),
    })
}

pub fn reset_chief_link() -> ResetChiefLink {
    ResetChiefLink {
        label: "Reset Chief".to_string(),
        href: "chief://ceremony/reset-chief".to_string(),
        reason: "Kernel grants are rooted in device identity; reset wipes and re-binds the device"
            .to_string(),
    }
}

fn prompt_bind_device() -> Result<()> {
    if !io::stdin().is_terminal() {
        return Ok(());
    }

    print!("Bind this device to you? [Y/n] ");
    io::stdout().flush().context("flush ceremony prompt")?;

    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .context("read ceremony prompt")?;
    let normalized = answer.trim().to_ascii_lowercase();
    if normalized.is_empty() || normalized == "y" || normalized == "yes" {
        Ok(())
    } else {
        Err(anyhow!("device binding ceremony declined"))
    }
}
