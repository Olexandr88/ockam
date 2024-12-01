use crate::messages::{RoutingNumber, UdpRoutingMessage, UdpTransportMessage};
use crate::workers::pending_messages::{PendingMessage, PendingMessageState};
use core::cmp::min;
use ockam_core::compat::collections::VecDeque;
use tracing::{error, trace};

const MAX_PENDING_MESSAGES_U16: u16 = 5;
const MAX_PENDING_MESSAGES_USIZE: usize = MAX_PENDING_MESSAGES_U16 as usize;

/// Pending routing messages for a certain peer
/// Currently, we only support messages with the right order, which means:
///  1. If a newer routing message is received - the old one is dropped if it wasn't fully assembled
///  2. If a part of a routing message has arrived out of order - the message is fully dropped
pub(crate) struct PeerPendingRoutingMessageStorage {
    // Reusable buffers to avoid excess allocations
    buffer_queue: VecDeque<Vec<u8>>,
    // Oldest routing number we can accept
    oldest_routing_number: RoutingNumber,
    // Messages with following routing numbers:
    // [self.oldest_routing_number, ..., self.oldest_routing_number + MAX_PENDING_MESSAGES_USIZE - 1]
    pending_messages: [PendingMessageState; MAX_PENDING_MESSAGES_USIZE],
}

impl PeerPendingRoutingMessageStorage {
    // Create given the first received message
    pub(crate) fn new(routing_number: RoutingNumber) -> Self {
        Self {
            buffer_queue: Default::default(),
            oldest_routing_number: routing_number,
            pending_messages: Default::default(),
        }
    }

    pub(crate) fn add_transport_message_and_try_assemble(
        &mut self,
        transport_message: UdpTransportMessage<'_>,
    ) -> Option<UdpRoutingMessage<'static>> {
        trace!(
            "Received routing message {}, offset {}",
            transport_message.routing_number,
            transport_message.offset
        );

        // self.oldest_routing_number is the oldest message we can accept,
        // older than that are ignored
        if transport_message.routing_number < self.oldest_routing_number {
            trace!(
                "Dropping routing message: {} because it arrived late. Offset {}",
                transport_message.routing_number,
                transport_message.offset
            );

            return None;
        }

        // We received a newer message
        let diff = transport_message.routing_number - self.oldest_routing_number;

        // Move self.pending_messages if needed and update the diff
        let diff = if diff >= MAX_PENDING_MESSAGES_U16 {
            // We received a much newer message, we need to drop one or few older messages so that
            // this message fits into our self.pending_messages

            // Length of the shift we need to perform on our self.pending_messages array
            let shift = diff - MAX_PENDING_MESSAGES_U16 + 1;

            // Drop the messages that don't fit anymore
            let number_of_messages_to_drop = min(shift, MAX_PENDING_MESSAGES_U16) as usize;

            for i in 0..number_of_messages_to_drop {
                match self.pending_messages[i].take() {
                    PendingMessageState::NotReceived => {
                        trace!(
                            "Discarding old not received routing message {} because a new routing message has arrived: {}",
                            self.oldest_routing_number + (i as u16),
                            transport_message.routing_number
                        );
                    }
                    PendingMessageState::InProgress(pending_message) => {
                        trace!(
                            "Discarding old partially received routing message {} because a new routing message has arrived: {}",
                            self.oldest_routing_number + (i as u16),
                            transport_message.routing_number
                        );

                        // Put the buffer back to reuse in the future
                        let buffer = pending_message.drop_message();
                        self.buffer_queue.push_back(buffer);
                    }
                    PendingMessageState::FullyHandled => {}
                }
            }

            // If we didn't drop all the messages, move the rest to the left
            if shift < MAX_PENDING_MESSAGES_U16 {
                let number_of_messages_to_shift = (MAX_PENDING_MESSAGES_U16 - shift) as usize;
                for i in 0..number_of_messages_to_shift {
                    self.pending_messages[i] = self.pending_messages[i + shift as usize].take();
                }
            }

            self.oldest_routing_number += shift;

            (diff - shift) as usize
        } else {
            diff as usize
        };

        let pending_message_state = self.pending_messages[diff].take();

        let mut pending_message = match pending_message_state {
            PendingMessageState::NotReceived => {
                let buffer = self.buffer_queue.pop_front().unwrap_or_default();

                PendingMessage::new(
                    transport_message.routing_number,
                    transport_message.total,
                    buffer,
                )
            }
            PendingMessageState::InProgress(m) => m,
            PendingMessageState::FullyHandled => {
                // Already send out.
                return None;
            }
        };

        match pending_message.add_transport_message_and_try_assemble(transport_message) {
            Some(routing_message_binary) => {
                let res = match minicbor::decode::<UdpRoutingMessage>(&routing_message_binary) {
                    Ok(routing_message) => Some(routing_message.into_owned()),
                    Err(err) => {
                        error!("Error while decoding UDP message {}", err);
                        None
                    }
                };

                self.pending_messages[diff] = PendingMessageState::FullyHandled;

                res
            }
            None => {
                self.pending_messages[diff] = PendingMessageState::InProgress(pending_message);
                None
            }
        }
    }
}
