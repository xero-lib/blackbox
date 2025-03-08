// #![no_std] // todo
// #![no_main] // todo

use std::{
    collections::VecDeque, fmt::Debug, fs::File, io::BufWriter, sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex}, thread
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait}, Sample, FromSample
};

use hound::{WavSpec, WavWriter};

use chrono::Local;

fn write_input_data<T, U>(data: &[T], config: &cpal::StreamConfig)
where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    let path = std::env::current_dir().unwrap().join(format!("{}.wav", Local::now().format("%Y-%m-%d_%H:%M:%S")));
    std::fs::File::create(&path).unwrap();

    let spec = WavSpec { channels: config.channels, bits_per_sample: 32, sample_format: hound::SampleFormat::Float, sample_rate: config.sample_rate.0};
    let mut writer = WavWriter::create(&path, spec).unwrap();
    for &sample in data {
        writer.write_sample(sample.to_sample::<U>()).ok();
    }
}

const BUFF_LEN: usize = 2_usize.pow(24);

fn main() {
    let (tx, rx): (Sender<f32>, Receiver<f32>) = mpsc::channel();

    //TODO create another channel for signalling termination to child threads

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


    let buff_clone = buff.clone();
    let config_clone = config.clone();
    // let writer_clone = writer.clone();

    
    let read_handle = thread::spawn(move || {
        let stream = device.build_input_stream(
            &config_clone,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                // let mut lock = buff_clone
                //     .lock()
                //     .unwrap();

                // lock.append(&mut VecDeque::from_iter(data.to_owned()));
                // write_input_data::<f32, f32>(data, &&writer_clone);
                // println!("data!");
                for point in data {
                    match tx.send(*point) {
                        Ok(_) => {},
                        Err(_) => panic!("Exiting due to send channel error...")
                    }
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
                Err(_) => { panic!("Exiting due to recv channel error...") }
            };

            let mut lock = buff_clone.lock().unwrap();
            lock.push_back(data);
        }
    });

    let mut line = String::new();
    loop {
        let _ = std::io::stdin().read_line(&mut line);
        if line == "q" { break }
        let mut lock = buff.lock().unwrap();
        println!("Saving data...");
        write_input_data::<f32, f32>(lock.make_contiguous(), &config);
    }

    //TODO send signals to children to exit
}

fn print(data: impl Debug) {
    println!("{data:?}");
}
