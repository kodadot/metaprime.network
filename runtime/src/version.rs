use crate::RUNTIME_API_VERSIONS;
use sp_runtime::create_runtime_str;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("kodadot-parachain"),
    impl_name: create_runtime_str!("kodadot-parachain"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
    NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}
