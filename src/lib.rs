#[macro_use]
extern crate futures;
extern crate tokio;
extern crate crossbeam;

pub mod api;
mod utils;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{ElementLink, Element, AsyncElementLink, AsyncElement};
    use crate::utils::test::packet_generators::{ immediate_stream, LinearIntervalGenerator };
    use crate::utils::test::packet_collectors::ExhaustiveDrain;
    use core::time;

    use futures::future::lazy;

    struct IdentityElement {
        id: i32
    }

    impl Element for IdentityElement {
        type Input = i32;
        type Output = i32;

        fn process(&mut self, packet: Self::Input) -> Self::Output {
            println!("Got packet {} in element {}", packet, self.id);
            packet
        }
    }

    /// One Synchronous Element, sourced with an interval yield
    /// 
    /// This test creates one Sync element, and uses the LinearIntervalGenerator to test whether
    /// the element responds correctly to an upstream source providing a series of valid packets,
    /// interleaved with Async::NotReady values, finalized by a Async::Ready(None)
    #[test]
    fn one_sync_element_interval_yield() {
        let packet_generator = LinearIntervalGenerator::new(time::Duration::from_millis(100), 10);

        let elem1 = IdentityElement { id: 0 };
        let elem2 = IdentityElement { id: 1 };

        // core_elem1 to! core_elem2

        let elem1_link = ElementLink::new(Box::new(packet_generator), elem1);
        let elem2_link = ElementLink::new(Box::new(elem1_link), elem2);

        let consumer = ExhaustiveDrain::new(1, Box::new(elem2_link));

        tokio::run(consumer);
    }


    struct AsyncIdentityElement {
        id: i32
    }

    impl AsyncElement for AsyncIdentityElement {
        type Input = i32;
        type Output = i32;

        fn process(&mut self, packet: Self::Input) -> Self::Output {
            println!("AsyncElement #{} got packet {}", self.id, packet);
            packet
        }
    }


    #[test]
    fn one_async_element_immediate_yield() {
        let default_channel_size = 10;
        let packet_generator = immediate_stream(0..=20);


        let elem0 = AsyncIdentityElement { id: 0 };

        let elem0_link = AsyncElementLink::new(Box::new(packet_generator), elem0, default_channel_size);

        let elem0_drain = elem0_link.consumer;
        let elem0_consumer = ExhaustiveDrain::new(1, Box::new(elem0_link.provider));

        tokio::run(lazy (|| {
            tokio::spawn(elem0_drain);
            tokio::spawn(elem0_consumer);
            Ok(())
        }));
    }

    #[test]
    fn two_async_elements_immediate_yield() {
        let default_channel_size = 10;
        let packet_generator = immediate_stream(0..=20);

        let elem0 = AsyncIdentityElement { id: 0 };
        let elem1 = AsyncIdentityElement { id: 1 };

        let elem0_link = AsyncElementLink::new(Box::new(packet_generator), elem0, default_channel_size);
        let elem1_link = AsyncElementLink::new(Box::new(elem0_link.provider), elem1, default_channel_size);

        let elem0_drain = elem0_link.consumer;
        let elem1_drain = elem1_link.consumer;

        let elem1_consumer = ExhaustiveDrain::new(1, Box::new(elem1_link.provider));

        tokio::run(lazy (|| {
            tokio::spawn(elem0_drain);
            tokio::spawn(elem1_drain);
            tokio::spawn(elem1_consumer);
            Ok(())
        }));
    }

    #[test]
    fn series_sync_and_async_immediate_yield() {
        let default_channel_size = 10;
        let packet_generator = immediate_stream(0..=20);

        let elem0 = IdentityElement { id: 0 };
        let elem1 = AsyncIdentityElement { id: 1 };
        let elem2 = IdentityElement { id: 2 };
        let elem3 = AsyncIdentityElement { id: 3 };

        let elem0_link = ElementLink::new(Box::new(packet_generator), elem0);
        let elem1_link = AsyncElementLink::new(Box::new(elem0_link), elem1, default_channel_size);
        let elem2_link = ElementLink::new(Box::new(elem1_link.provider), elem2);
        let elem3_link = AsyncElementLink::new(Box::new(elem2_link), elem3, default_channel_size);

        let elem1_drain = elem1_link.consumer;
        let elem3_drain = elem3_link.consumer;

        let elem3_consumer = ExhaustiveDrain::new(0, Box::new(elem3_link.provider));

        tokio::run(lazy (|| {
            tokio::spawn(elem1_drain);
            tokio::spawn(elem3_drain); 
            tokio::spawn(elem3_consumer);
            Ok(())
        }));
    }

        #[test]
    fn one_async_element_interval_yield() {
        let default_channel_size = 10;
        let packet_generator = LinearIntervalGenerator::new(time::Duration::from_millis(100), 20);

        let elem0 = AsyncIdentityElement { id: 0 };

        let elem0_link = AsyncElementLink::new(Box::new(packet_generator), elem0, default_channel_size);

        let elem0_drain = elem0_link.consumer;
        let elem0_consumer = ExhaustiveDrain::new(0, Box::new(elem0_link.provider));

        tokio::run(lazy (|| {
            tokio::spawn(elem0_drain);
            tokio::spawn(elem0_consumer);
            Ok(())
        }));
    }

    #[test]
    fn two_async_elements_interval_yield() {
        let default_channel_size = 10;
        let packet_generator = LinearIntervalGenerator::new(time::Duration::from_millis(100), 20);

        let elem0 = AsyncIdentityElement { id: 0 };
        let elem1 = AsyncIdentityElement { id: 1 };

        let elem0_link = AsyncElementLink::new(Box::new(packet_generator), elem0, default_channel_size);
        let elem1_link = AsyncElementLink::new(Box::new(elem0_link.provider), elem1, default_channel_size);

        let elem0_drain = elem0_link.consumer;
        let elem1_drain = elem1_link.consumer;

        let elem1_consumer = ExhaustiveDrain::new(0, Box::new(elem1_link.provider));

        tokio::run(lazy (|| {
            tokio::spawn(elem0_drain);
            tokio::spawn(elem1_drain);
            tokio::spawn(elem1_consumer);
            Ok(())
        }));
    }

    #[test]
    fn series_sync_and_async_interval_yield() {
        let default_channel_size = 10;
        let packet_generator = LinearIntervalGenerator::new(time::Duration::from_millis(100), 20);

        let elem0 = IdentityElement { id: 0 };
        let elem1 = AsyncIdentityElement { id: 1 };
        let elem2 = IdentityElement { id: 2 };
        let elem3 = AsyncIdentityElement { id: 3 };

        let elem0_link = ElementLink::new(Box::new(packet_generator), elem0);
        let elem1_link = AsyncElementLink::new(Box::new(elem0_link), elem1, default_channel_size);
        let elem2_link = ElementLink::new(Box::new(elem1_link.provider), elem2);
        let elem3_link = AsyncElementLink::new(Box::new(elem2_link), elem3, default_channel_size);

        let elem1_drain = elem1_link.consumer;
        let elem3_drain = elem3_link.consumer;

        let elem3_consumer = ExhaustiveDrain::new(2, Box::new(elem3_link.provider));

        tokio::run(lazy (|| {
            tokio::spawn(elem1_drain);
            tokio::spawn(elem3_drain); 
            tokio::spawn(elem3_consumer);
            Ok(())
        }));
    }
}
