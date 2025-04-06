pub fn insync<T>(mut future: impl std::future::Future<Output = T>) -> T {
    const VTABLE: std::task::RawWakerVTable =
        std::task::RawWakerVTable::new(|_| RAW_WAKER, |_| {}, |_| {}, |_| {});
    const RAW_WAKER: std::task::RawWaker = std::task::RawWaker::new(std::ptr::null(), &VTABLE);

    let mut future = std::pin::pin!(future);
    let waker = unsafe { std::task::Waker::from_raw(RAW_WAKER) };
    let mut cx = std::task::Context::from_waker(&waker);

    loop {
        match future.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(val) => break val,
            std::task::Poll::Pending => continue,
        }
    }
}

pub unsafe fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    core::slice::from_raw_parts((p as *const T) as *const u8, core::mem::size_of::<T>())
}
