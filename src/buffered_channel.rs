use tokio::{
    select,
    sync::broadcast::{channel as broadcast_channel, error::RecvError, Receiver, Sender},
    task::JoinHandle,
};
use tracing::warn;

pub struct BufferedReceiver<T> {
    receiver: Receiver<T>,
    _handle: JoinHandle<()>,
    _stopper: tokio::sync::oneshot::Sender<()>,
}

impl<T: Clone + Send + 'static> BufferedReceiver<T> {
    fn new(mut receiver: Receiver<T>, buffer_consumer: Receiver<T>) -> Self {
        let (stopper, mut stop_signal) = tokio::sync::oneshot::channel::<()>();
        let task_handle = tokio::spawn(async move {
            loop {
                select! {
                    maybe_item = receiver.recv() => {
                        match maybe_item {
                        Ok(_) => {}
                        Err(RecvError::Closed) => return,
                        Err(RecvError::Lagged(missed_count)) => {
                            warn!("Buffered channel droped: ${missed_count} on support loop")
                        }
                    }
                },
                    _ = &mut stop_signal => return
                }
            }
        });

        Self {
            _handle: task_handle,
            receiver: buffer_consumer,
            _stopper: stopper,
        }
    }

    pub async fn read_next(&mut self) -> Option<T> {
        loop {
            match self.receiver.recv().await {
                Ok(v) => return Some(v),
                Err(RecvError::Closed) => return None,
                Err(RecvError::Lagged(num_lost)) => {
                    warn!("Bufferd channel lost {num_lost} items")
                }
            }
        }
    }
}

pub fn channel<T>(buffer_size: usize) -> (Sender<T>, BufferedReceiver<T>)
where
    T: Clone + Send + 'static,
{
    let (sender, receiver) = broadcast_channel::<T>(buffer_size);
    let receiver2 = sender.subscribe();
    (sender, BufferedReceiver::new(receiver, receiver2))
}

#[cfg(test)]
mod test {
    use tokio::{sync::broadcast::error::SendError, task::yield_now};

    use super::*;

    #[tokio::test]
    async fn test_channel_works() {
        let (sender, mut receiver) = channel::<u32>(10);
        sender.send(1).expect("sent");
        assert_eq!(receiver.read_next().await.unwrap(), 1);
        drop(receiver);
        yield_now().await;
        assert_matches::assert_matches!(sender.send(2), Err(SendError(2)))
    }

    #[tokio::test]
    async fn test_earliest_messages_are_dropped() {
        let (sender, mut receiver) = channel::<u32>(3);
        sender.send(1).expect("sent");
        sender.send(2).expect("sent");
        sender.send(3).expect("sent");
        sender.send(4).expect("sent");
        sender.send(5).expect("sent");
        yield_now().await;
        assert_eq!(receiver.read_next().await.unwrap(), 2);
        assert_eq!(receiver.read_next().await.unwrap(), 3);
        assert_eq!(receiver.read_next().await.unwrap(), 4);
        assert_eq!(receiver.read_next().await.unwrap(), 5);
    }

    #[tokio::test]
    async fn test_receiver_returns_none_when_sender_is_dropped() {
        let (sender, mut receiver) = channel::<u32>(3);
        sender.send(1).expect("sent");
        sender.send(2).expect("sent");
        sender.send(3).expect("sent");
        sender.send(4).expect("sent");
        sender.send(5).expect("sent");
        yield_now().await;
        drop(sender);
        assert_eq!(receiver.read_next().await.unwrap(), 2);
        assert_eq!(receiver.read_next().await.unwrap(), 3);
        assert_eq!(receiver.read_next().await.unwrap(), 4);
        assert_eq!(receiver.read_next().await.unwrap(), 5);
        assert_eq!(receiver.read_next().await, None)
    }
}
