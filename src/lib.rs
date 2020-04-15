#![no_std]

pub use try_blocks_macros::try_blocks;

#[doc(hidden)]
pub mod _rt {
    use core::task::Poll;

    pub trait Try: sealed::Sealed {
        type Ok_;
        type Error;

        fn into_result(self) -> Result<Self::Ok_, Self::Error>;

        fn from_ok(v: Self::Ok_) -> Self;
        fn from_error(e: Self::Error) -> Self;
    }

    #[inline]
    pub fn into_result<T: Try>(t: T) -> Result<T::Ok_, T::Error> {
        t.into_result()
    }

    #[inline]
    pub fn from_ok<T: Try>(ok: T::Ok_) -> T {
        T::from_ok(ok)
    }

    #[inline]
    pub fn from_error<T: Try>(err: T::Error) -> T {
        T::from_error(err)
    }

    impl<T, E> Try for Result<T, E> {
        type Ok_ = T;
        type Error = E;

        fn into_result(self) -> Result<Self::Ok_, Self::Error> {
            self
        }

        fn from_ok(v: Self::Ok_) -> Self {
            Ok(v)
        }

        fn from_error(e: Self::Error) -> Self {
            Err(e)
        }
    }

    #[derive(Debug)]
    pub struct NoneError(());

    impl<T> Try for Option<T> {
        type Ok_ = T;
        type Error = NoneError;

        fn into_result(self) -> Result<Self::Ok_, Self::Error> {
            self.ok_or_else(|| NoneError(()))
        }

        fn from_ok(v: Self::Ok_) -> Self {
            Some(v)
        }

        fn from_error(_: Self::Error) -> Self {
            None
        }
    }

    impl<T, E> Try for Poll<Result<T, E>> {
        type Ok_ = Poll<T>;
        type Error = E;

        #[inline]
        fn into_result(self) -> Result<Self::Ok_, Self::Error> {
            match self {
                Poll::Ready(Ok(x)) => Ok(Poll::Ready(x)),
                Poll::Ready(Err(e)) => Err(e),
                Poll::Pending => Ok(Poll::Pending),
            }
        }

        #[inline]
        fn from_ok(v: Self::Ok_) -> Self {
            v.map(Ok)
        }

        fn from_error(e: Self::Error) -> Self {
            Poll::Ready(Err(e))
        }
    }

    impl<T, E> Try for Poll<Option<Result<T, E>>> {
        type Ok_ = Poll<Option<T>>;
        type Error = E;

        #[inline]
        fn into_result(self) -> Result<Self::Ok_, Self::Error> {
            match self {
                Poll::Ready(Some(Ok(x))) => Ok(Poll::Ready(Some(x))),
                Poll::Ready(Some(Err(e))) => Err(e),
                Poll::Ready(None) => Ok(Poll::Ready(None)),
                Poll::Pending => Ok(Poll::Pending),
            }
        }

        #[inline]
        fn from_ok(v: Self::Ok_) -> Self {
            v.map(|v| v.map(Ok))
        }

        fn from_error(e: Self::Error) -> Self {
            Poll::Ready(Some(Err(e)))
        }
    }

    mod sealed {
        use core::task::Poll;

        pub trait Sealed {}
        impl<T, E> Sealed for Result<T, E> {}
        impl<T> Sealed for Option<T> {}
        impl<T, E> Sealed for Poll<Result<T, E>> {}
        impl<T, E> Sealed for Poll<Option<Result<T, E>>> {}
    }
}
