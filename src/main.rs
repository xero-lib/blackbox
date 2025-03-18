// #![no_std] // todo
// #![no_main] // todo

mod macros;
use macros::break_if;

mod ringbuff;
use ringbuff::RingBuff;

mod file_io;
use file_io::write_input_data;

use std::{
    io, sync::{Arc, Mutex}, thread
};

use ringbuf::{
    StaticRb,
    traits::{Consumer, Observer, Producer, Split},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const BUFF_LEN: usize = 1 << 25;

fn main() -> Result<(), io::Error>{
    let static_rb = StaticRb::<f32, 2048>::default();
    let (mut tx, mut rx) = static_rb.split();

    //TODO create another channel for signalling termination to child threads
    let buff = Arc::new(Mutex::new(RingBuff::<f32>::new::<BUFF_LEN>())); // probably faster/more efficient to use ringbuf crate without Arc<Mutex>
    let host = cpal::host_from_id(cpal::HostId::Jack).unwrap_or(cpal::default_host());
    let device = host
        .default_input_device()
        .expect("No input devices found.");

    let config = device
        .supported_input_configs()
        .expect("No audio device configs found. Are there any connected?")
        .next()
        .expect("No audio configurations avaiable for default device...")
        .with_max_sample_rate()
        .config();

    let buff_clone = buff.clone();
    let config_clone = config.clone();

    let write_handle = thread::spawn(move || {
        let mut data = Vec::<f32>::new();
        data.resize(2048, 0.0);
        loop {
            if rx.is_empty() {
                thread::park();
            }

            let num_bytes = rx.pop_slice(&mut data);
            let mut lock = buff_clone.lock().unwrap();
            lock.push_slice(&data[..num_bytes]);
        }
    });

    // cpal has its own thread, but looking to transition away from libraries, so leaving it in its own thread for easier refactoring later
    let _read_handle = thread::spawn(move || {
        let stream = device
            .build_input_stream(
                &config_clone,
                move |data: &[f32], _| {
                    write_handle.thread().unpark();
                    tx.push_slice(data);
                },
                |err| eprintln!("An error has been detected during recording: {err:?}"),
                None,
            )
            .unwrap();

        stream.play().expect("Unable to record...");
        println!("Recording started...");
        thread::park();
    });

    let mut line = String::new();
    loop {
        let _ = std::io::stdin().read_line(&mut line);

        break_if!(line.trim() == "q");

        println!("Saving data...");
        let lock = buff.lock().unwrap();
        write_input_data::<f32, f32>(&lock.vectorize(), &config);
    }

    //TODO send signals to children to exit

    Ok(())
}
