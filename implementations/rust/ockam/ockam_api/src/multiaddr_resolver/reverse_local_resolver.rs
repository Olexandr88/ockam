use ockam_core::{Address, Result, Route, LOCAL};
use ockam_multiaddr::proto::Service;
use ockam_multiaddr::MultiAddr;

use crate::error::ApiError;

pub struct ReverseLocalResolver;

impl ReverseLocalResolver {
    /// Try to convert an Ockam Route into a MultiAddr.
    pub fn resolve_route(r: &Route) -> Result<MultiAddr> {
        let mut ma = MultiAddr::default();
        for a in r.iter() {
            ma.try_extend(&Self::resolve_address(a)?)?
        }
        Ok(ma)
    }

    /// Try to convert an Ockam Address to a MultiAddr.
    pub fn resolve_address(a: &Address) -> Result<MultiAddr> {
        let mut ma = MultiAddr::default();
        match a.transport_type() {
            LOCAL => ma.push_back(Service::new(a.address()))?,
            other => {
                error!(target: "ockam_api", transport = %other, "unsupported transport type");
                return Err(ApiError::core(format!("unknown transport type: {other}")));
            }
        }
        Ok(ma)
    }
}
