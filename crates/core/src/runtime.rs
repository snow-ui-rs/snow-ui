//! Platform abstraction for running framework-owned asynchronous work.

#[cfg(not(target_arch = "wasm32"))]
pub fn run<F>(future: F)
where
    F: std::future::Future<Output = ()>,
{
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Snow UI async runtime")
        .block_on(future);
}

#[cfg(target_arch = "wasm32")]
pub fn run<F>(future: F)
where
    F: std::future::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn interval<F>(duration: std::time::Duration, callback: F)
where
    F: Fn() + Send + 'static,
{
    std::thread::Builder::new()
        .name("snow-ui-interval-timer".to_string())
        .spawn(move || {
            loop {
                std::thread::sleep(duration);
                callback();
            }
        })
        .expect("failed to start Snow UI interval timer");
}

#[cfg(target_arch = "wasm32")]
pub fn interval<F>(duration: std::time::Duration, callback: F)
where
    F: Fn() + 'static,
{
    let interval = gloo_timers::callback::Interval::new(duration.as_millis() as u32, move || {
        callback();
    });
    interval.forget();
}
