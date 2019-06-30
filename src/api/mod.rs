use futures::{Future, Stream, Async, Poll};
use std::collections::VecDeque;

pub type ElementStream<Input> = Box<dyn Stream<Item = Input, Error = ()> + Send>;

pub trait Element {
    type Input: Sized;
    type Output: Sized;

    fn process(&mut self, packet: Self::Input) -> Self::Output;
}

pub struct ElementLink<E: Element> {
    input_stream: ElementStream<E::Input>,
    element: E
}

impl<E: Element> ElementLink<E> {
    pub fn new(input_stream: ElementStream<E::Input>, element: E) -> Self {
        ElementLink {
            input_stream,
            element
        }
    }
}

impl<E: Element> Stream for ElementLink<E> {
    type Item = E::Output;
    type Error = ();

    /*
    4 cases: Async::Ready(Some), Async::Ready(None), Async::NotReady, Err

    Async::Ready(Some): We have a packet ready to process from the upstream element. It's passed to
    our core's process function for... processing

    Async::Ready(None): The input_stream doesn't have anymore input. Semantically, it's like an
    iterator has exhausted it's input. We should return "Ok(Async::Ready(None))" to signify to our
    downstream components that there's no more input to process. Our Elements should rarely
    return "Async::Ready(None)" since it will effectively kill the Stream chain.

    Async::NotReady: There is more input for us to process, but we can't make any more progress right
    now. The contract for Streams asks us to register with a Reactor so we will be woken up again by
    an Executor, but we will be relying on Tokio to do that for us. This case is handled by the
    "try_ready!" macro, which will automatically return "Ok(Async::NotReady)" if the input stream
    gives us NotReady.

    Err: is also handled by the "try_ready!" macro.
    */
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let input_packet_option: Option<E::Input> = try_ready!(self.input_stream.poll());
        match input_packet_option {
            None => Ok(Async::Ready(None)),
            Some(input_packet) => {
                let output_packet: E::Output = self.element.process(input_packet);
                Ok(Async::Ready(Some(output_packet)))
            },
        }
    }
}

pub trait AsyncElement {
    type Input: Sized;
    type Output: Sized;

    fn process(&mut self, packet: Self::Input) -> Self::Output;
}

pub struct AsyncElementLink<E: AsyncElement> {
    input_stream: ElementStream<E::Input>,
    output_queue: VecDeque<E::Output>,
    queue_capacity: usize,
    element: E
}

impl<E: AsyncElement> AsyncElementLink<E> {
    pub fn new(input_stream: ElementStream<E::Input>, element: E, queue_capacity: usize) -> Self {
        let output_queue: VecDeque<E::Output> = VecDeque::with_capacity(queue_capacity);
        AsyncElementLink {
            input_stream,
            output_queue,
            queue_capacity,
            element
        }
    }
}
/*
AsyncElementLink has both Stream and Future because it
needs one to hand to Tokio, the Future, and another to hand
to whatever element is after it, the Stream. 
*/
impl<E: AsyncElement> Stream for AsyncElementLink<E> {
    type Item = E::Output;
    type Error = ();

    /*
    4 cases: Async::Ready(Some), Async::Ready(None), Async::NotReady, Err

    Async::Ready(Some): We have a packet in the queue that is ready to be returned, pop it and
    return

    Async::Ready(None): This is never returned

    Async::NotReady: There are no more packets in the queue for us to provide to the output.

    Err: is also handled by the "try_ready!" macro.
    */
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let output_packet_option = self.output_queue.pop_front();
        match output_packet_option {
            None => { return Ok(Async::NotReady) },
            Some(output_packet) => {
                Ok(Async::Ready(Some(output_packet)))
            },
        }
    }
}

impl<E: AsyncElement> Future for AsyncElementLink<E> {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        println!("Aync Element Poll");
        loop {
            /* Check for space in the queue. */
            if self.output_queue.len() >= self.queue_capacity {
                return Ok(Async::NotReady)
            }
            match try_ready!(self.input_stream.poll()) {
                Some(input_packet) => {
                    /* Got a packet, push onto queue*/
                    let output_packet: E::Output = self.element.process(input_packet);
                    self.output_queue.push_back(output_packet);
                },
                None => {
                    println!("Consumer received none. End of packet stream");
                    return Ok(Async::Ready(()))
                }
            }
        }
    }
}