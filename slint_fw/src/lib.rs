use futures_util::{StreamExt as _, stream::FusedStream};
use std::fmt::Debug;
use thiserror::Error;

pub mod nav;

/// A handle to slint's property.
///
/// [`PropertyHandle`] does NOT implement [`Clone`] because a property SHOULD NOT be mutated from
/// multiple places (you CAN but SHOULDN'T).
pub struct PropertyHandle<T> {
    getter: Box<dyn Fn() -> T>,
    setter: Box<dyn Fn(T)>,
}

impl<T> PropertyHandle<T> {
    pub fn get(&self) -> T {
        (self.getter)()
    }

    pub fn set(&self, val: T) {
        (self.setter)(val)
    }

    /// Binds `input_stream` to adopter i.e. watches `input_stream` and sets latest value
    /// via [`setter`][Self::setter].
    ///
    /// Once property are bound to stream, [`PropertyHandle`] is consumed and cannot be changed manually.
    pub async fn bind<S>(self, mut input_stream: S) -> Result<(), StreamTerminated>
    where
        S: FusedStream<Item = T> + Unpin,
    {
        loop {
            if let Some(val) = input_stream.next().await {
                (self.setter)(val);
            } else {
                return Err(StreamTerminated(()));
            }
        }
    }
}

/// Returned from [`PropertyHandle::bind()`]
#[derive(Debug, Error)]
#[error("stream has been terminated")]
pub struct StreamTerminated(());

impl<T> Debug for PropertyHandle<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PropertyHandle")
            .field("value", &(self.getter)())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use tokio::sync::watch;
    use tokio_stream::StreamExt;
    use tokio_stream::wrappers::WatchStream;

    #[test]
    fn property_handle_bind_returns_err_when_stream_ends() {
        let val = Cell::new(0);
        let prop = PropertyHandle {
            getter: Box::new(|| val.get()),
            setter: Box::new(|v| {
                val.set(v);
            }),
        };

        let (tx, rx) = watch::channel(0);
        slint::spawn_local(async move {
            prop.bind(WatchStream::new(rx).fuse())
                .await
                .expect_err("should return error");
        })
        .unwrap();
        tx.send(1).unwrap();
        assert_eq!(val.get(), 1);
        tx.send(2).unwrap();
        assert_eq!(val.get(), 2);
        drop(tx);
    }
}
