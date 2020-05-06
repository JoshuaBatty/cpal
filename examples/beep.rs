extern crate anyhow;
extern crate cpal;
extern crate winit;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use winit::{
    event::{Event, KeyboardInput, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

fn main() -> Result<(), anyhow::Error> {
    let event_loop = EventLoop::new();
    let _window = WindowBuilder::new()
        .with_title("Press R to start cpal.")
        .build(&event_loop)
        .unwrap();

    event_loop.run(move |event, _, _| {
        use winit::event::{ElementState, VirtualKeyCode};
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(virtual_code),
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => match virtual_code {
                    VirtualKeyCode::R => {
                        // Uncommenting lines 36 and 47 allows cpal to launch
                        // on it's own thread. This solves the crash.
                        //std::thread::spawn(move|| {
                            let host = cpal::default_host();
                            let device = host
                                .default_output_device()
                                .expect("failed to find a default output device");
                            let config = device.default_output_config().unwrap();
                            match config.sample_format() {
                                cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()).unwrap(),
                                cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()).unwrap(),
                                cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()).unwrap(),
                            }
                        //});
                    }
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        }
    });

    Ok(())
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
        sample_clock = (sample_clock + 1.0) % sample_rate;
        (sample_clock * 440.0 * 2.0 * 3.141592 / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value)
        },
        err_fn,
    )?;
    stream.play()?;

    std::thread::sleep(std::time::Duration::from_millis(1000));

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
