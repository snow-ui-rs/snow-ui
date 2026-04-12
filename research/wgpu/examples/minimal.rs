use wgpu::{Backends, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, PowerPreference, RequestAdapterOptions};

async fn run() {
    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        ..Default::default()
    });

    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .expect("Failed to find a suitable GPU adapter");

    let adapter_info = adapter.get_info();
    println!("Adapter: {} ({:?})", adapter_info.name, adapter_info.backend);

    let (device, queue) = adapter
        .request_device(
            &DeviceDescriptor {
                label: Some("Research WGPU Device"),
                features: Features::empty(),
                limits: Limits::default(),
            },
            None,
        )
        .await
        .expect("Failed to create device");

    println!("Device created: {:?}", device.features());
    println!("Queue created successfully.");

    drop(queue);
    drop(device);
}

fn main() {
    pollster::block_on(run());
}
