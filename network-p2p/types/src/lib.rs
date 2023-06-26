// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libp2p::futures::channel::oneshot;
use std::borrow::Cow;
use std::fmt;

pub mod multi_address_with_peer_id;
pub mod network_state;
pub mod peer_id;

pub use libp2p::core::{identity, multiaddr, Multiaddr, PeerId, PublicKey};
pub use libp2p::request_response::{InboundFailure, OutboundFailure};
pub use libp2p::{build_multiaddr, multihash};
pub use multi_address_with_peer_id::{parse_addr, parse_str_addr, MultiaddrWithPeerId};
pub use sc_peerset::{ReputationChange, BANNED_THRESHOLD};

/// Build memory protocol Multiaddr by port
pub fn memory_addr(port: u64) -> Multiaddr {
    build_multiaddr!(Memory(port))
}

/// Generate a random memory protocol Multiaddr
pub fn random_memory_addr() -> Multiaddr {
    memory_addr(rand::random::<u64>())
}

/// Check the address is a memory protocol Multiaddr.
pub fn is_memory_addr(addr: &Multiaddr) -> bool {
    addr.iter()
        .any(|protocol| matches!(protocol, libp2p::core::multiaddr::Protocol::Memory(_)))
}

/// Error that can be generated by `parse_str_addr`.
#[derive(Debug)]
pub enum ParseErr {
    /// Error while parsing the multiaddress.
    MultiaddrParse(multiaddr::Error),
    /// Multihash of the peer ID is invalid.
    InvalidPeerId,
    /// The peer ID is missing from the address.
    PeerIdMissing,
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErr::MultiaddrParse(err) => write!(f, "{}", err),
            ParseErr::InvalidPeerId => write!(f, "Peer id at the end of the address is invalid"),
            ParseErr::PeerIdMissing => write!(f, "Peer id is missing from the address"),
        }
    }
}

impl std::error::Error for ParseErr {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ParseErr::MultiaddrParse(err) => Some(err),
            ParseErr::InvalidPeerId => None,
            ParseErr::PeerIdMissing => None,
        }
    }
}

impl From<multiaddr::Error> for ParseErr {
    fn from(err: multiaddr::Error) -> ParseErr {
        ParseErr::MultiaddrParse(err)
    }
}

/// Error in a request.
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum RequestFailure {
    /// We are not currently connected to the requested peer.
    NotConnected,
    /// Given protocol hasn't been registered.
    UnknownProtocol,
    /// Remote has closed the substream before answering, thereby signaling that it considers the
    /// request as valid, but refused to answer it.
    Refused,
    /// The remote replied, but the local node is no longer interested in the response.
    Obsolete,
    /// Problem on the network.
    #[display(fmt = "Problem on the network: {:?}", _0)]
    Network(#[error(ignore)] OutboundFailure),
}

/// Error when processing a request sent by a remote.
#[derive(Debug, derive_more::Display, derive_more::Error)]
pub enum ResponseFailure {
    /// Problem on the network.
    #[display(fmt = "Problem on the network: {:?}", _0)]
    Network(#[error(ignore)] InboundFailure),
}

/// Response for an incoming request to be send by a request protocol handler.
#[derive(Debug)]
pub struct OutgoingResponse {
    /// The payload of the response.
    ///
    /// `Err(())` if none is available e.g. due an error while handling the request.
    pub result: Result<Vec<u8>, ()>,
    /// Reputation changes accrued while handling the request. To be applied to the reputation of
    /// the peer sending the request.
    pub reputation_changes: Vec<ReputationChange>,
}

/// A single request received by a peer on a request-response protocol.
#[derive(Debug)]
pub struct IncomingRequest {
    /// Who sent the request.
    pub peer: PeerId,

    /// Request sent by the remote. Will always be smaller than
    /// [`ProtocolConfig::max_request_size`].
    pub payload: Vec<u8>,

    /// Channel to send back the response.
    ///
    /// There are two ways to indicate that handling the request failed:
    ///
    /// 1. Drop `pending_response` and thus not changing the reputation of the peer.
    ///
    /// 2. Sending an `Err(())` via `pending_response`, optionally including reputation changes for
    /// the given peer.
    pub pending_response: oneshot::Sender<OutgoingResponse>,
}

#[derive(Debug)]
pub struct ProtocolRequest {
    pub protocol: Cow<'static, str>,
    pub request: IncomingRequest,
}

/// When sending a request, what to do on a disconnected recipient.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IfDisconnected {
    /// Try to connect to the peer.
    TryConnect,
    /// Just fail if the destination is not yet connected.
    ImmediateError,
}

/// Convenience functions for `IfDisconnected`.
impl IfDisconnected {
    /// Shall we connect to a disconnected peer?
    pub fn should_connect(self) -> bool {
        match self {
            Self::TryConnect => true,
            Self::ImmediateError => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_address() {
        let addr = random_memory_addr();
        assert!(is_memory_addr(&addr));
    }
}
