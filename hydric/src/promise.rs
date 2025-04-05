use poll_promise::Promise;

pub type AsyncResult<T, E> = Option<Promise<Result<T, E>>>;

/// Spawns a promise that performs the given async closure.
pub fn spawn<T, E, F>(input: &mut AsyncResult<T, E>, closure: F)
where
    F: Future<Output = Result<T, E>> + 'static,
    T: Send + 'static,
    E: Send + 'static,
{
    *input = Some(Promise::spawn_local(closure));
}

/// Polls a promise that might be finished. If it's finished, the callback is called and the
/// promise reference is cleared.
pub fn poll<T, E, F>(input: &mut AsyncResult<T, E>, mut callback: F)
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
            callback(result);
        }
    }
    if should_clear {
        // Clear the promise.
        *input = None;
    }
}
