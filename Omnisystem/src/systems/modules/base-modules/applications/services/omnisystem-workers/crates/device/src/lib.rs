/// Device Workers - 16+ Hardware Operations

pub mod battery;
pub mod thermal;
pub mod display;
pub mod audio;
pub mod input;
pub mod sensor;
pub mod gpu;
pub mod camera;
pub mod microphone;
pub mod bluetooth;
pub mod usb;
pub mod accelerator;
pub mod power;
pub mod fan;
pub mod keyboard;
pub mod vibration;

pub use battery::BatteryWorker;
pub use thermal::ThermalWorker;
pub use display::DisplayWorker;
pub use audio::AudioWorker;
pub use input::InputWorker;
pub use sensor::SensorWorker;
pub use gpu::GPUWorker;
pub use camera::CameraWorker;
pub use microphone::MicrophoneWorker;
pub use bluetooth::BluetoothWorker;
pub use usb::USBWorker;
pub use accelerator::AcceleratorWorker;
pub use power::PowerWorker;
pub use fan::FanWorker;
pub use keyboard::KeyboardWorker;
pub use vibration::VibrationWorker;
