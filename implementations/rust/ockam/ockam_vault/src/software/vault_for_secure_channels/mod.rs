use cfg_if::cfg_if;

#[cfg(not(any(feature = "aws-lc", feature = "rust-crypto",)))]
compile_error! {"One feature must be enabled: \"aws-lc\" or \"rust-crypto\""}

#[cfg(any(
    feature = "OCKAM_XX_25519_AES128_GCM_SHA256",
    feature = "OCKAM_XX_25519_AES256_GCM_SHA256",
    not(feature = "disable_default_noise_protocol")
))]
cfg_if! {
    if #[cfg(feature = "aws-lc")] {
        mod aes_aws_lc;
        use aes_aws_lc::make_aes;
    } else {
        mod aes_rs;
        use aes_rs::make_aes;
    }
}

mod common;
mod types;
mod vault_for_encryption_at_rest;
#[allow(clippy::module_inception)]
mod vault_for_secure_channels;

pub use types::*;
pub use vault_for_encryption_at_rest::*;
pub use vault_for_secure_channels::*;
