use parking::Parker;
use std::{
    cell::RefCell,
    future::Future,
    io,
    os::fd::RawFd,
    pin::Pin,
    task::{Context, Poll, Waker},
};

pub enum Iotype {
    WRITE,
    READ,
}

pub struct Iotask {
    pub inner: Box<Inner>,
}

pub struct Inner {
    pub id: u64,
    pub fd: RawFd,
    pub buf: RefCell<Vec<u8>>,
    pub _type: Iotype,
    pub finish: bool,
}

impl Iotask {
    pub fn new(id: u64, fd: RawFd, buf: RefCell<Vec<u8>>, _type: Iotype) -> Self {
        Iotask {
            inner: Box::new(Inner {
                id,
                fd,
                buf,
                _type,
                finish: false,
            }),
        }
    }
}

impl Future for Iotask {
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.inner.finish {
            Poll::Ready(Ok(1))
        } else {
            Poll::Pending
        }
    }
}

macro_rules! pin {
    ($($x:ident),* $(,)?) => {
        $(
            let mut $x = $x;
            #[allow(unused_mut)]
            let mut $x = unsafe {
                core::pin::Pin::new_unchecked(&mut $x)
            };
        )*
    }
}

pub fn block_on<F>(future: impl Future<Output = F>) -> F {
    pin!(future);
    fn parker_and_waker() -> (Parker, Waker) {
        let parker = Parker::new();
        let unparker = parker.unparker();
        let waker = Waker::from(unparker);
        (parker, waker)
    }

    let (parker, waker) = parker_and_waker();
    let cx = &mut Context::from_waker(&waker);

    loop {
        match future.as_mut().poll(cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => parker.park(),
        }
    }
}

pub async fn yield_now() {
    YieldNow { is_ready: false }.await
}

struct YieldNow {
    is_ready: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.is_ready {
            return Poll::Ready(());
        }

        self.is_ready = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
