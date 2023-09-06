use minicbor::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// Timestamp in seconds (UTC)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode, Serialize, Deserialize)]
#[rustfmt::skip]
#[cbor(transparent)]
#[serde(transparent)]
pub struct TimestampInSeconds(#[n(0)] pub u64);
