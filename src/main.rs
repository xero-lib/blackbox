// #![no_std] // todo
// #![no_main] // todo

mod ringbuff;
use ringbuff::RingBuff;

mod file_io;
use file_io::write_input_data;

use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const BUFF_LEN: usize = 1 << 25;

fn main() {
    let (tx, rx): (Sender<f32>, Receiver<f32>) = mpsc::channel();

    //TODO create another channel for signalling termination to child threads
    let buff = Arc::new(Mutex::new(RingBuff::<f32>::new::<BUFF_LEN>()));
    let host = cpal::host_from_id(cpal::HostId::Jack).unwrap_or(cpal::default_host());
    let device = host
        .default_input_device()
        .expect("No input devices found.");

    let config = device
        .supported_input_configs()
        .expect("No audio device configs found. Are there any connected?")
        // .find(|cfg| cfg.sample_format() == SampleFormat::f32)
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
                    for &sample in data {
                        match tx.send(sample) {
                            Err(e) => panic!("Exiting due to send-channel error...\n{e:#}"),
                            _ => ()
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
        println!("Recording started...");
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

