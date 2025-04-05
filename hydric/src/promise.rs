use poll_promise::Promise;

pub type AsyncResult<T, E> = Option<Promise<Result<T, E>>>;

pub fn poll<T, E, F>(input: &mut AsyncResult<T, E>, mut if_ready: F)
where
    T: Send,
    E: Send,
    F: FnMut(&T),
{
    let promise_ref = input.as_ref();
    let mut should_clear = false;
    if let Some(promise) = promise_ref {
        if let Some(Ok(result)) = promise.ready() {
            should_clear = true;

            if_ready(result);
        }
    }
    if should_clear {
        // Clear the promise.
        *input = None;
    }
}
