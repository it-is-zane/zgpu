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

#[derive(Debug)]
pub enum GpuError {
    AdapterRequestFailed,
    DeviceRequestFailed(wgpu::RequestDeviceError),
}
pub fn get_gpu() -> Result<(wgpu::Instance, wgpu::Device, wgpu::Queue), GpuError> {
    let instance_descriptor = wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    };

    let instance = wgpu::Instance::new(&instance_descriptor);

    let adapter_descriptor = wgpu::RequestAdapterOptionsBase {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    };

    let adapter = insync(async {
        instance
            .request_adapter(&adapter_descriptor)
            .await
            .ok_or(GpuError::AdapterRequestFailed)
    })?;

    let device_descriptor = wgpu::DeviceDescriptor {
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        label: Some("Device"),
        memory_hints: wgpu::MemoryHints::Performance,
    };

    let (device, queue) = insync(async { adapter.request_device(&device_descriptor, None).await })
        .map_err(GpuError::DeviceRequestFailed)?;

    Ok((instance, device, queue))
}
