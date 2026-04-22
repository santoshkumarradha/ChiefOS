//! Kernel principal identity and boot attestation.

pub mod attestation;
pub mod ceremony;
pub mod identity;

pub use attestation::{
    minimal_kernel_bootstrap_grants, BootAttestation, KernelGrantBundle,
    BOOT_ATTESTATION_MAX_AGE_SECS, KERNEL_PRINCIPAL,
};
pub use ceremony::{
    boot_kernel_principal, reset_chief_link, FirstRunCeremonyOutcome, ResetChiefLink,
};
pub use identity::DeviceIdentity;
