// #![no_std] // todo
// #![no_main] // todo

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait}, SampleFormat
};

const BUFF_LEN: usize = 2_usize.pow(24);

fn main() {
    let mut buff = Arc::new(Mutex::new(VecDeque::<i32>::with_capacity(BUFF_LEN)));
    let mut index = Arc::new(Mutex::new(0_usize));

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("No input devices found.");

    let config = device
        .supported_input_configs()
        .expect("No audio device configs found. Are there any connected?")
        .find(|cfg| cfg.sample_format() == SampleFormat::I32)
        .unwrap()
        .with_max_sample_rate()
        .config();

    let buff_clone = buff.clone();

    let stream = device.build_input_stream(
        &config,
        move |data: &[i32], _: &cpal::InputCallbackInfo| {
            buff_clone
                .lock()
                .unwrap()
                .append(&mut VecDeque::from_iter(data.to_owned()));
            buff_clone.clear_poison();
        },
        move |err| {
            eprintln!("Ah hell {err:?}");
        },
        None,
    );

    stream.unwrap().play().unwrap();
    loop {}
    // loop {
    //     println!("{:?}", buff);
    // }
}
