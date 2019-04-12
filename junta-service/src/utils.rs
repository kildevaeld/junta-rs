use futures::prelude::*;

pub enum OneOfTwo<I, E, F1: Future<Item = I, Error = E>, F2: Future<Item = I, Error = E>> {
    Future1(F1),
    Future2(F2),
}

pub struct OneOfTwoFuture<I, E, F1: Future<Item = I, Error = E>, F2: Future<Item = I, Error = E>> {
    inner: OneOfTwo<I, E, F1, F2>,
}

impl<I, E, F1: Future<Item = I, Error = E>, F2: Future<Item = I, Error = E>>
    OneOfTwoFuture<I, E, F1, F2>
{
    pub fn new(inner: OneOfTwo<I, E, F1, F2>) -> OneOfTwoFuture<I, E, F1, F2> {
        OneOfTwoFuture { inner }
    }
}

impl<I, E, F1: Future<Item = I, Error = E>, F2: Future<Item = I, Error = E>> Future
    for OneOfTwoFuture<I, E, F1, F2>
where
    F1: Future<Item = I, Error = E>,
    F2: Future<Item = I, Error = E>,
{
    type Item = I;
    type Error = E;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match &mut self.inner {
            OneOfTwo::Future1(fut) => fut.poll(),
            OneOfTwo::Future2(fut) => fut.poll(),
        }
    }
}

pub enum OneOfTree<
    I,
    E,
    F1: Future<Item = I, Error = E>,
    F2: Future<Item = I, Error = E>,
    F3: Future<Item = I, Error = E>,
> {
    Future1(F1),
    Future2(F2),
    Future3(F3),
}

pub struct OneOfTreeFuture<
    I,
    E,
    F1: Future<Item = I, Error = E>,
    F2: Future<Item = I, Error = E>,
    F3: Future<Item = I, Error = E>,
> {
    inner: OneOfTree<I, E, F1, F2, F3>,
}

impl<
        I,
        E,
        F1: Future<Item = I, Error = E>,
        F2: Future<Item = I, Error = E>,
        F3: Future<Item = I, Error = E>,
    > OneOfTreeFuture<I, E, F1, F2, F3>
{
    pub fn new(inner: OneOfTree<I, E, F1, F2, F3>) -> OneOfTreeFuture<I, E, F1, F2, F3> {
        OneOfTreeFuture { inner }
    }
}

impl<I, E, F1: Future, F2: Future, F3: Future> Future for OneOfTreeFuture<I, E, F1, F2, F3>
where
    F1: Future<Item = I, Error = E>,
    F2: Future<Item = I, Error = E>,
    F3: Future<Item = I, Error = E>,
{
    type Item = I;
    type Error = E;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match &mut self.inner {
            OneOfTree::Future1(fut) => fut.poll(),
            OneOfTree::Future2(fut) => fut.poll(),
            OneOfTree::Future3(fut) => fut.poll(),
        }
    }
}

// pub enum OneOfFour<
//     I,
//     E,
//     F1: Future<Item = I, Error = E>,
//     F2: Future<Item = I, Error = E>,
//     F3: Future<Item = I, Error = E>,
//     F4: Future<Item = I, Error = E>,
// > {
//     Future1(F1),
//     Future2(F2),
//     Future3(F3),
//     Future4(F4),
// }

// pub struct OneOfFourFuture<I, E, F1: Future, F2: Future, F3: Future, F4: Future> {
//     inner: OneOfFour<I, E, F1, F2, F3, F4>,
// }

// impl<I, E, F1: Future, F2: Future, F3: Future, F4: Future> OneOfFourFuture<I, E, F1, F2, F3, F4> {
//     pub fn new(inner: OneOfFour<I, E, F1, F2, F3, F4>) -> OneOfFourFuture<I, E, F1, F2, F3, F4> {
//         OneOfFourFuture { inner }
//     }
// }

// impl<I, E, F1: Future, F2: Future, F3: Future, F4: Future> Future
//     for OneOfFourFuture<I, E, F1, F2, F3, F4>
// where
//     F1: Future<Item = I, Error = E>,
//     F2: Future<Item = I, Error = E>,
//     F3: Future<Item = I, Error = E>,
//     F4: Future<Item = I, Error = E>,
// {
//     type Item = I;
//     type Error = E;
//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         match &mut self.inner {
//             OneOfFour::Future1(fut) => fut.poll(),
//             OneOfFour::Future2(fut) => fut.poll(),
//             OneOfFour::Future3(fut) => fut.poll(),
//             OneOfFour::Future4(fut) => fut.poll(),
//         }
//     }
// }
