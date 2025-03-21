use std::cmp::PartialEq;

/// Creates a getter-setter from the given getter and setter.
/// Careful: don't use for floats! Use get_set_float instead.
pub fn get_set<'a, T: PartialEq + Clone + 'a, S: Fn(T) + 'a>(
    val: T,
    setter: S,
) -> impl Fn(Option<T>) -> T + use<T, S> {
    move |it| {
        if let Some(it) = it {
            if it != val {
                setter(it);
            }
        }
        val.clone()
    }
}
