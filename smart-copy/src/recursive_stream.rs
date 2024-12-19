use std::task::Poll;

use futures::{future::BoxFuture, FutureExt, Stream};

pub enum RecursiveItem<'a, T> {
    Item(T),
    Future(BoxFuture<'a, Vec<RecursiveItem<'a, T>>>),
}

/// Polls every future in parallel
pub struct RecursiveStream2<'a, T> {
    queue: Vec<BoxFuture<'a, Vec<RecursiveItem<'a, T>>>>,
}

impl<'a, T> RecursiveStream2<'a, T> {
    pub fn new(future: BoxFuture<'a, Vec<RecursiveItem<'a, T>>>) -> Self {
        Self {
            queue: vec![future],
        }
    }
}

impl<'a, T> Stream for RecursiveStream2<'a, T> {
    type Item = Vec<T>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.queue.is_empty() {
            return Poll::Ready(None);
        }
        let mut i = 0;
        let self_mut = self.get_mut();
        let mut items = vec![];
        loop {
            if i == self_mut.queue.len() {
                break;
            }
            match self_mut.queue[i].poll_unpin(cx) {
                Poll::Ready(items_and_futures) => {
                    let mut futures = vec![];
                    for item_or_future in items_and_futures {
                        match item_or_future {
                            RecursiveItem::Item(item) => {
                                items.push(item);
                            }
                            RecursiveItem::Future(future) => {
                                futures.push(future);
                            }
                        }
                    }
                    self_mut.queue.splice(i..i + 1, futures);
                }
                Poll::Pending => i += 1,
            }
        }
        if !items.is_empty() {
            Poll::Ready(Some(items))
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use super::*;
    use futures::{stream, FutureExt, StreamExt};

    #[tokio::test]
    async fn concept_is_useful() {
        let mut vec = stream::iter(vec![vec![1, 2, 3], vec![4, 5]])
            .flat_map_unordered(None, |vec| stream::iter(vec))
            .collect::<Vec<_>>()
            .await;
        vec.sort();
        assert_eq!(vec, vec![1, 2, 3, 4, 5])
    }

    #[tokio::test]
    async fn simple() {
        let mut vec = RecursiveStream2::new(
            async {
                let mut vec = Vec::new();
                vec.push(RecursiveItem::Item(1));
                vec.push(RecursiveItem::Item(2));
                vec.push(RecursiveItem::Future(
                    async {
                        let mut vec = Vec::new();
                        vec.push(RecursiveItem::Item(3));
                        vec.push(RecursiveItem::Item(4));
                        vec
                    }
                    .boxed(),
                ));
                vec
            }
            .boxed(),
        )
        .flat_map_unordered(None, |vec| stream::iter(vec))
        .collect::<Vec<_>>()
        .await;
        vec.sort();
        assert_eq!(vec, vec![1, 2, 3, 4])
    }

    #[tokio::test]
    async fn recursive_function() {
        fn async_fn(
            depth: usize,
            count: u32,
            path: Option<Vec<u32>>,
        ) -> impl Future<Output = Vec<RecursiveItem<'static, Vec<u32>>>> + Send {
            async move {
                let mut vec = Vec::default();
                for i in 0..count {
                    let path = {
                        let mut path = path.clone().unwrap_or_default();
                        path.push(i);
                        path
                    };
                    vec.push(match depth {
                        1 => RecursiveItem::Item(path),
                        depth => RecursiveItem::Future({
                            let boxed_future: BoxFuture<
                                'static,
                                Vec<RecursiveItem<'static, Vec<u32>>>,
                            > = Box::pin(async_fn(depth - 1, count, Some(path)));
                            boxed_future
                        }),
                    });
                }
                vec
            }
        }
        let mut vec = RecursiveStream2::new(async_fn(3, 2, None).boxed())
            .flat_map_unordered(None, |vec| stream::iter(vec))
            .collect::<Vec<_>>()
            .await;
        vec.sort();
        assert_eq!(
            vec,
            vec![
                vec![0, 0, 0],
                vec![0, 0, 1],
                vec![0, 1, 0],
                vec![0, 1, 1],
                vec![1, 0, 0],
                vec![1, 0, 1],
                vec![1, 1, 0],
                vec![1, 1, 1]
            ]
        )
    }
}
