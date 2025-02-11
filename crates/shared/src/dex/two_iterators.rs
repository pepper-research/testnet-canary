pub enum TwoIterators<X, Y> {
    A(X),
    B(Y),
}

impl<X, Y> Iterator for TwoIterators<X, Y>
where
    X: Iterator<Item = (i64, usize)>,
    Y: Iterator<Item = (i64, usize)>,
{
    type Item = (i64, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            TwoIterators::A(x) => x.next(),
            TwoIterators::B(y) => y.next(),
        }
    }
}
