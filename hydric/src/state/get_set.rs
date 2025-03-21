use std::cmp::PartialEq;

/// Creates a getter-setter from the given getter and setter.
pub fn get_set<'a, T: PartialEq + Clone + 'a, S: Fn(T) + 'a>(
    val: T,
    setter: S,
) -> impl Fn(Option<T>) -> T + use<T, S> {
    move |it| {
        if let Some(it) = it {
            setter(it)
        }
        val.clone()
    }
}
