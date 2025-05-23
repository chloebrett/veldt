use poll_promise::Promise;
use std::fmt::Debug;

// Async states represent outgoing requests that may or may not have finished.
// The Option<Promise<Result<..., ()>>> format is for the following reasons:
// * The Option is None if no request has been made.
// * The Promise is pending if the request is in progress, and resolved if it has succeeded or
// failed.
// * The Result contains the result of the promise - Ok(something) if succeeded and
// Err(something) if it failed. Currently the errors are just unit, i.e. ().
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
    E: Send + Debug,
    F: FnMut(&T),
{
    let promise_ref = input.as_ref();
    let mut should_clear = false;
    if let Some(promise) = promise_ref {
        match promise.ready() {
            Some(Ok(result)) => {
                should_clear = true;
                callback(result);
            }
            Some(Err(err)) => {
                should_clear = true;
                log::error!("Error from promise: {:?}", err);
            }
            _ => {}
        }
    }
    if should_clear {
        // Clear the promise.
        *input = None;
    }
}
