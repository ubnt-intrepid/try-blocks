pub use try_blocks_macros::try_blocks;

#[doc(hidden)]
pub mod _rt {
    pub trait Try {
        type Ok_;
        type Error;

        fn into_result(self) -> Result<Self::Ok_, Self::Error>;

        fn from_ok(v: Self::Ok_) -> Self;
        fn from_error(e: Self::Error) -> Self;
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
}
