use std::cmp::PartialEq;

/// Creates a getter-setter from the given getter and setter.
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

/// Creates a getter-setter from the given getter and setter.
/// Allows the setter to be FnMut.
pub fn get_set_mut<'a, T: PartialEq + Clone + 'a, S: FnMut(T) + 'a>(
    val: T,
    mut setter: S,
) -> impl FnMut(Option<T>) -> T + use<T, S> {
    move |it| {
        if let Some(it) = it {
            if it != val {
                setter(it);
            }
        }
        val.clone()
    }
}
