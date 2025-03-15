// #![no_std] // todo
// #![no_main] // todo

use std::{
    fmt::Debug,
    fs::File,
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

use cpal::{
    FromSample, Sample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use hound::{WavSpec, WavWriter};

use chrono::Local;

#[derive(Default)]
struct RingBuff<T> {
    index: usize,
    saturated: bool,
    pub capacity: usize,
    contents: Vec<T>,
}

// impl<'a, T> Iterator<'a, T> for RingBuff<'a, T> {

// // returns an iterator over the properly arranged items. is no longer correct once something is pushed
//     fn (&'a self) -> impl Iterator<Item = &'a T> {
//         self.contents[..self.index].iter().chain(self.contents[self.index..].iter())
//     }
// }

impl<T: Clone + Default> RingBuff<T> {
    fn new<const CAP: usize>() -> Self { // I don't really like this syntax. subject to change
        Self {
            capacity: CAP,
            contents: {
                let mut vec = Vec::<T>::with_capacity(CAP);
                vec.resize(CAP, T::default());
                vec
            },
            ..Default::default()
        }
    }
}

impl<T: Clone> RingBuff<T> {
    fn vectorize(&self) -> Vec<T> {
        [
            self.contents[..self.index]
                .iter()
                .clone()
                .map(T::to_owned)
                .collect::<Vec<T>>(),
            self.saturated
                .then(|| {
                    self.contents[self.index..]
                        .iter()
                        .clone()
                        .map(T::to_owned)
                        .collect()
                })
                .unwrap_or(Vec::new()),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<T>>()
    }

    fn push(&mut self, value: T) {
        self.contents[self.index] = value;
        if self.index == self.capacity - 1 {
            self.index = 0;
        } else {
            self.index += 1;
        }
    }
}

fn write_input_data<T, U>(data: &Vec<T>, config: &cpal::StreamConfig)
where
    T: Sample<Float = f32>,
    U: Sample + hound::Sample + FromSample<T>,
{
    let path = std::env::current_dir()
        .unwrap()
        .join(format!("{}.wav", Local::now().format("%Y-%m-%d_%H:%M:%S")));
    File::create(&path).unwrap();

    let spec = WavSpec {
        channels: config.channels,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
        sample_rate: config.sample_rate.0,
    };
    let mut writer = WavWriter::create(&path, spec).unwrap();
    for &sample in data {
        writer
            .write_sample(sample.mul_amp(0.25f32).to_sample::<U>())
            .ok();
    }
}

const BUFF_LEN: usize = 1 << 25;

fn main() {
    let (tx, rx): (Sender<f32>, Receiver<f32>) = mpsc::channel();

    //TODO create another channel for signalling termination to child threads
    let buff = Arc::new(Mutex::new(RingBuff::<f32>::new::<BUFF_LEN>()));
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

    let _read_handle = thread::spawn(move || {
        let stream = device
            .build_input_stream(
                &config_clone,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // let mut lock = buff_clone
                    //     .lock()
                    //     .unwrap();

                    // lock.append(&mut VecDeque::from_iter(data.to_owned()));
                    // write_input_data::<f32, f32>(data, &&writer_clone);
                    // println!("data!");
                    for &sample in data {
                        match tx.send(sample) {
                            Ok(_) => {}
                            Err(_) => panic!("Exiting due to send-channel error..."),
                        }
                    }
                },
                move |err| {
                    eprintln!("An error has been detected during recording: {err:?}");
                },
                None,
            )
            .unwrap();
        stream.play().expect("Unable to record...");
        thread::park();
    });

    let _write_handle = thread::spawn(move || {
        loop {
            let data = match rx.recv() {
                Ok(d) => d,
                Err(_) => {
                    panic!("Exiting due to recv-channel error...")
                }
            };

            let mut lock = buff_clone.lock().unwrap();
            lock.push(data);
            // println!("{}", lock.capacity);
        }
    });

    let mut line = String::new();
    loop {
        let _ = std::io::stdin().read_line(&mut line);
        if line == "q" {
            break;
        }

        println!("Saving data...");
        let lock = buff.lock().unwrap();
        write_input_data::<f32, f32>(&lock.vectorize(), &config);
    }

    //TODO send signals to children to exit
}

fn print(data: impl Debug) {
    println!("{data:?}");
}
