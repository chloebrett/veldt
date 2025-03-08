use leptos::prelude::*;
use leptos::reactive::wrappers::write::SignalSetter;
use std::marker::{Send, Sync};
use thaw_utils::Model;

/// Helper to turn a subfield of a struct wrapped in an RwSignal
/// into a model that Thaw can use. Note: inner fields must also be
/// wrapped in RwSignals.
pub fn subfield<T: 'static, F: Send + Sync>(
    signal: &RwSignal<T, LocalStorage>,
    getter: impl Fn(&T) -> RwSignal<F>,
    setter: impl Fn(F) + Send + Sync + 'static,
) -> Model<F> {
    let get: Signal<F> = signal.with(getter).into();
    let set: SignalSetter<F> = SignalSetter::map(setter);
    (get, set).into()
}
