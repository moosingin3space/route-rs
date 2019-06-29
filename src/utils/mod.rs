use crate::api::ElementStream;
use futures::{Stream, Async, Poll, Future};
use tokio::timer::Interval;
use std::time::Duration;
use std::fmt::Debug;

// TODO: move generators and consumers into their own modules

/*
Packet Generators
*/

#[allow(dead_code)]
pub struct LinearIntervalGenerator {
    interval: Interval,
    iterations: usize,
    seq_num: i32
}

#[allow(dead_code)]
impl LinearIntervalGenerator {
    pub fn new(duration: Duration, iterations: usize) -> Self {
        LinearIntervalGenerator {
            interval: Interval::new_interval(duration),
            iterations,
            seq_num: 0
        }
    }
}

#[allow(dead_code)]
impl Stream for LinearIntervalGenerator {
    type Item = i32;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, ()> {
        try_ready!(self.interval.poll().map_err(|_| ()));
        if self.seq_num as usize > self.iterations {
            Ok(Async::Ready(None))
        } else {
            self.seq_num += 1;
            Ok(Async::Ready(Some(self.seq_num)))
        }
    }
}

/*
Element Consumers
*/

#[allow(dead_code)]
pub struct ExhaustiveConsumer<T: Debug> {
    stream: ElementStream<T>
}

#[allow(dead_code)]
impl<T: Debug> ExhaustiveConsumer<T> {
    pub fn new(stream: ElementStream<T>) -> Self {
        ExhaustiveConsumer { stream }
    }
}

#[allow(dead_code)]
impl <T: Debug> Future for ExhaustiveConsumer<T> {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        println!("Consumer poll");

        loop {
            match try_ready!(self.stream.poll()) {
                Some(value) => {
                    println!("Consumer received packet: {:?}", value);
                },
                None => {
                    println!("Consumer received none. End of packet stream");
                    return Ok(Async::Ready(()))
                }
            }
        }
    }
}