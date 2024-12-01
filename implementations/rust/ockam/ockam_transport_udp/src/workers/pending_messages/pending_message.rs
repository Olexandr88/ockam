use crate::messages::{RoutingNumber, UdpTransportMessage};
use ockam_core::compat::collections::HashSet;
use std::mem;
use tracing::{trace, warn};

pub(crate) struct PendingMessage {
    routing_number: RoutingNumber,
    total: u16,
    // TODO: I think usually total number is low, so we can use bitmap or vec here
    not_received_parts: HashSet<u16>,
    binary: Vec<u8>,
    last_part: Option<Vec<u8>>,
}

impl PendingMessage {
    pub(crate) fn new(routing_number: RoutingNumber, total: u16, binary: Vec<u8>) -> Self {
        let mut not_received_parts = HashSet::<u16>::default();

        for i in 0..total {
            not_received_parts.insert(i);
        }

        Self {
            routing_number,
            total,
            not_received_parts,
            binary,
            last_part: None,
        }
    }

    fn merge_last_part_if_needed(&mut self) {
        let last_part = if let Some(last_part) = self.last_part.take() {
            last_part
        } else {
            return;
        };

        let data_offset_begin = self.binary.len();
        let data_offset_end = data_offset_begin + last_part.len();

        self.binary.resize(data_offset_end, 0);

        self.binary[data_offset_begin..data_offset_end].copy_from_slice(&last_part);
    }

    pub(crate) fn add_transport_message_and_try_assemble(
        &mut self,
        transport_message: UdpTransportMessage<'_>,
    ) -> Option<Vec<u8>> {
        if self.total != transport_message.total {
            warn!("Received invalid message for Routing number: {}. Original total number: {}, received total number: {}", self.routing_number, self.total,transport_message.total);
            return None;
        }

        if self.total <= transport_message.offset {
            warn!(
                "Received invalid message for Routing number: {}",
                self.routing_number
            );
            return None;
        }

        if !self.not_received_parts.remove(&transport_message.offset) {
            warn!(
                "Received duplicate message for Routing number: {}. Offset: {}",
                self.routing_number, transport_message.offset
            );
            return None;
        }

        if transport_message.offset + 1 != self.total {
            // Not the last part
            let data_offset_begin =
                transport_message.offset as usize * transport_message.payload.len();
            let data_offset_end = data_offset_begin + transport_message.payload.len();

            // FIXME: Add some sanity checks
            if self.binary.len() < data_offset_end {
                self.binary.resize(data_offset_end, 0);
            }

            trace!(
                "Filling message data. Routing number: {}. Offset: {}. Data offset: {}..{}",
                self.routing_number,
                transport_message.offset,
                data_offset_begin,
                data_offset_end
            );
            self.binary[data_offset_begin..data_offset_end]
                .copy_from_slice(&transport_message.payload);
        } else {
            // Last part
            if self.last_part.is_some() {
                return None;
            }

            self.last_part = Some(transport_message.payload.into_owned());
        }

        if self.not_received_parts.is_empty() {
            self.merge_last_part_if_needed();

            // We will return the clone here, but we'll reuse the original buffer later
            Some(self.binary.clone())
        } else {
            None
        }
    }

    pub(crate) fn drop_message(mut self) -> Vec<u8> {
        self.binary.clear();
        self.binary
    }
}

pub(crate) enum PendingMessageState {
    NotReceived,
    InProgress(PendingMessage),
    FullyHandled,
}

impl Default for PendingMessageState {
    fn default() -> Self {
        Self::NotReceived
    }
}

impl PendingMessageState {
    pub(crate) fn take(&mut self) -> Self {
        mem::replace(self, Self::NotReceived)
    }
}
