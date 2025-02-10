pub fn map_vec<I, O>(input: Vec<I>) -> Vec<O>
    where I : Into<O> {
    input.into_iter().map(|elem| elem.into()).collect()
}
