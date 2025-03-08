// #![no_std] // todo
// #![no_main] // todo

use std::{
    collections::VecDeque,
    fmt::Debug,
    sync::{Arc, Mutex, mpsc::{self, Sender, Receiver}},
    fs::File,
    io::BufWriter,
    thread,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait}, Sample, FromSample
};

use hound::{WavSpec, WavWriter};

fn write_input_data<T, U>(data: &[T], writer: &Arc<Mutex<WavWriter<BufWriter<File>>>>)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    let mut writer = writer.lock().unwrap();
    for &sample in data {
        writer.write_sample(sample.to_sample::<U>()).ok();
    }
}

const BUFF_LEN: usize = 2_usize.pow(24);

fn main() {
    //!TODO change from Vec to just f32 and loop
    let (tx, rx): (Sender<f32>, Receiver<f32>) = mpsc::channel();
    let buff = Arc::new(Mutex::new(VecDeque::<f32>::with_capacity(BUFF_LEN)));
    let host = cpal::host_from_id(cpal::HostId::Jack).unwrap_or(cpal::default_host());
    let device = host
        .default_input_device()
        .expect("No input devices found.");
    print(device.supported_input_configs().unwrap().next());

    let config = device
        .supported_input_configs()
        .expect("No audio device configs found. Are there any connected?")
        // .find(|cfg| cfg.sample_format() == SampleFormat::I32)
        .next()
        .unwrap()
        .with_max_sample_rate()
        .config();

    let spec = WavSpec { channels: config.channels, bits_per_sample: 32, sample_format: hound::SampleFormat::Float, sample_rate: config.sample_rate.0};
    let writer = Arc::new(Mutex::new(WavWriter::create("output.wav", spec).unwrap()));

    let buff_clone = buff.clone();
    // let writer_clone = writer.clone();

    
    let read_handle = thread::spawn(move || {
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // let mut lock = buff_clone
                //     .lock()
                //     .unwrap();

                // lock.append(&mut VecDeque::from_iter(data.to_owned()));
                // write_input_data::<f32, f32>(data, &&writer_clone);
                // println!("data!");
                for point in data {
                    tx.send(*point).unwrap();
                }
            },
            move |err| {
                eprintln!("Failed to instantiate: {err:?}");
            },
            None,
        ).unwrap();
        stream.play().expect("Unable to record...");
        thread::park();
    });

    let write_handle = thread::spawn(move|| {
        loop {
            let data = match rx.recv() {
                Ok(d) => d,
                Err(e) => { eprintln!("{e:#?}"); continue; }
            };

            let mut lock = buff_clone.lock().unwrap();
            lock.push_back(data);
        }
    });

    loop {
        let _ = std::io::stdin().read_line(&mut String::new());
        let mut lock = buff.lock().unwrap();
        println!("Saving data...");
        write_input_data::<f32, f32>(lock.make_contiguous(), &writer);
    }

}

fn print(data: impl Debug) {
    println!("{data:?}");
}
