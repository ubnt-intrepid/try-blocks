/*!
An attribute-style macro that emulates unstable `try` blocks.

# Usage

```
# fn main() {}
# use try_blocks::try_blocks;
# use std::io::Read as _;
use std::io;
use std::net::TcpStream;

#[try_blocks]
fn fallible() {
    let result: io::Result<Vec<u8>> = try {
        let mut stream = TcpStream::connect("127.0.0.1:8000")?;

        let mut buf = vec![0u8; 1024];
        stream.read_exact(&mut buf[..])?;

        buf // tail expr is automatically wrapped
    };

    match result {
        Ok(data) => { /* ... */ },
        Err(err) => {
            eprintln!("caught an error: {}", err);
            std::process::exit(101);
        }
    }
}
```


```
# fn main() {}
# use try_blocks::try_blocks;
use std::io;
use async_std::net::TcpStream;

#[try_blocks]
async fn fallible_async() {
    let result: io::Result<Vec<u8>> = try {
        let mut stream = TcpStream::connect("127.0.0.1:8000").await?;

        let mut buf = vec![0u8; 1024];
        stream.read_exact(&mut buf[..]).await?;

        buf
    };

    match result {
        Ok(data) => { /* ... */ },
        Err(err) => {
            eprintln!("caught an error: {}", err);
            std::process::exit(101);
        }
    }
}

# mod async_std { pub mod net {
# use std::io;
# pub struct TcpStream;
# impl TcpStream {
#     pub async fn connect(_: &str) -> io::Result<Self> { unimplemented!() }
#     pub async fn read_exact(&mut self, _: &mut [u8]) -> io::Result<()> { unimplemented!() }
# }
# } }
```

!*/

#![no_std]

/// Expand `try` blocks.
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
