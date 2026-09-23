//! a small bidirectional channel wrapper built on `std::sync::mpsc`.

use std::{
    sync::mpsc::{Receiver, RecvError, RecvTimeoutError, SendError, Sender, TryRecvError, channel},
    time::Duration,
};

/// a bidirectional channel built from a paired sender and receiver.
pub struct Channel<S, R> {
    sender: Sender<S>,
    receiver: Receiver<R>,
}

impl<S, R> Channel<S, R> {
    /// creates a connected pair of channel endpoints.
    pub fn new() -> (Self, Channel<R, S>) {
        let (sender_1, receiver_2) = channel();
        let (sender_2, receiver_1) = channel();
        (
            Self {
                sender: sender_1,
                receiver: receiver_1,
            },
            Channel {
                sender: sender_2,
                receiver: receiver_2,
            },
        )
    }

    /// sends a message to the opposite endpoint.
    pub fn send(&self, message: S) -> Result<(), SendError<S>> {
        self.sender.send(message)
    }

    /// sends a batch of messages to the opposite endpoint.
    pub fn send_batch(&self, messages: impl IntoIterator<Item = S>) -> Result<(), SendError<S>> {
        for message in messages {
            self.send(message)?;
        }
        Ok(())
    }

    /// blocks until a message is received.
    pub fn receive(&self) -> Result<R, RecvError> {
        self.receiver.recv()
    }

    /// waits up to `timeout` for a message to arrive.
    pub fn receive_timeout(&self, timeout: Duration) -> Result<R, RecvTimeoutError> {
        self.receiver.recv_timeout(timeout)
    }

    /// attempts to receive a message without blocking.
    pub fn try_receive(&self) -> Result<R, TryRecvError> {
        self.receiver.try_recv()
    }
}

#[cfg(test)]
mod test {
    use std::{
        sync::mpsc::{RecvTimeoutError, TryRecvError},
        time::Duration,
    };

    use super::Channel;
    type IntChannel = Channel<i32, i32>;

    #[test]
    fn send_receive() {
        let (left, right) = IntChannel::new();

        let handle = std::thread::spawn(move || {
            right.send(42).unwrap();
            assert_eq!(right.receive(), Ok(100));
        });

        assert_eq!(left.receive(), Ok(42));
        assert_eq!(left.send(100), Ok(()));

        handle.join().unwrap();
    }

    #[test]
    fn timeout() {
        let (left, right) = IntChannel::new();

        assert_eq!(
            left.receive_timeout(Duration::from_millis(100)),
            Err(RecvTimeoutError::Timeout)
        );

        assert_eq!(right.send(1), Ok(()));
        assert_eq!(left.receive_timeout(Duration::from_millis(100)), Ok(1));
    }

    #[test]
    fn try_receive() {
        let (left, right) = IntChannel::new();

        assert_eq!(left.try_receive(), Err(TryRecvError::Empty));

        assert_eq!(right.send(1), Ok(()));
        assert_eq!(left.try_receive(), Ok(1));
    }

    #[test]
    fn multiple_messages() {
        let (left, right) = IntChannel::new();

        std::thread::spawn(move || {
            for i in 0..5 {
                assert_eq!(right.send(i), Ok(()));
            }
        });

        for i in 0..5 {
            assert_eq!(left.receive().unwrap(), i);
        }
    }

    #[test]
    fn batch() {
        let (left, right) = IntChannel::new();

        const MESSAGES: [i32; 5] = [0, 1, 2, 3, 4];

        std::thread::spawn(move || {
            assert_eq!(right.send_batch(MESSAGES), Ok(()));
        });

        for i in MESSAGES {
            assert_eq!(left.receive().unwrap(), i);
        }
    }
}
