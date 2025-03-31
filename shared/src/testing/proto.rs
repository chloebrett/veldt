#[cfg(test)]

pub mod proto_testing {
    use std::fmt::Debug;

    pub fn assert_proto_round_trip<T, U>(object: T)
    where
        T: Clone + From<U> + PartialEq + Debug,
        U: From<T>,
    {
        let proto: U = object.clone().into();
        let result: T = proto.into();
        assert_eq!(object, result);
    }
}
