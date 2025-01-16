// /// An enum that represents two different iterator types
// ///
// /// This enum can hold one of two different iterators, each yielding pairs of `i64` and `usize`
// pub enum TwoIterators<X, Y> {
//     /// Variant A holding an iterator of type X
//     A(X),
//     /// Variant B holding an iterator of type Y
//     B(Y),
// }

// impl<X, Y> Iterator for TwoIterators<X, Y>
// where
//     X: Iterator<Item=(i64, usize)>,
//     Y: Iterator<Item=(i64, usize)>,
// {
//     /// The type of item this iterator produces
//     type Item = (i64, usize);

//     /// Advances the iterator and returns the next value
//     /// - Returns `Some((i64, usize))` if there is a next item
//     /// - Returns `None` if the iteration is complete
//     fn next(&mut self) -> Option<Self::Item> {
//         match self {
//             TwoIterators::A(x) => x.next(),
//             TwoIterators::B(y) => y.next(),
//         }
//     }
// }
