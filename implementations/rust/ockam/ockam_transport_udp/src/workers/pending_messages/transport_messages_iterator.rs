use crate::messages::{
    RoutingNumber, UdpRoutingMessage, UdpTransportMessage, CURRENT_VERSION, MAX_PAYLOAD_SIZE,
};
use crate::MAX_MESSAGE_SIZE;
use ockam_core::LocalMessage;
use ockam_transport_core::TransportError;
use tracing::trace;

pub(crate) struct TransportMessagesIterator {
    current_routing_number: RoutingNumber,
    offset: u16,
    total: u16,
    data: Vec<u8>,
}

impl TransportMessagesIterator {
    pub(crate) fn new(
        current_routing_number: RoutingNumber,
        local_message: LocalMessage,
    ) -> ockam_core::Result<Self> {
        let routing_message = UdpRoutingMessage::from(local_message);

        let routing_message = ockam_core::cbor_encode_preallocate(routing_message)?;

        if routing_message.len() > MAX_MESSAGE_SIZE {
            return Err(TransportError::MessageLengthExceeded)?;
        }

        let total = routing_message.len() / MAX_PAYLOAD_SIZE + 1;

        let total: u16 = total
            .try_into()
            .map_err(|_| TransportError::MessageLengthExceeded)?;

        Ok(Self {
            current_routing_number,
            offset: 0,
            total,
            data: routing_message,
        })
    }
}

impl Iterator for TransportMessagesIterator {
    type Item = ockam_core::Result<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.offset == self.total {
            return None;
        }

        let data_offset_begin = (self.offset as usize) * MAX_PAYLOAD_SIZE;
        let data_offset_end = if self.offset + 1 == self.total {
            self.data.len()
        } else {
            data_offset_begin + MAX_PAYLOAD_SIZE
        };

        let part = UdpTransportMessage::new(
            CURRENT_VERSION,
            self.current_routing_number,
            self.offset,
            self.total,
            &self.data[data_offset_begin..data_offset_end],
        );

        trace!(
            "Sending Routing Message {}. Offset {}",
            self.current_routing_number,
            part.offset
        );

        // TODO: Avoid allocation
        match ockam_core::cbor_encode_preallocate(part) {
            Ok(res) => {
                self.offset += 1;
                Some(Ok(res))
            }
            Err(err) => Some(Err(err)),
        }
    }
}
