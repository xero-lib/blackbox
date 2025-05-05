mod ringbuff;
use ringbuff::RingBuff;

mod macros;

mod file_io;
use file_io::write_input_data;
// use slint::{quit_event_loop, CloseRequestResponse};

use std::{
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread,
};

use ringbuf::{
    traits::{Consumer, Observer, Producer, Split},
    StaticRb,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    HostId,
};

fn main() {
    let static_rb = StaticRb::<f32, 4096>::default();
    let (mut tx, mut rx) = static_rb.split();

    let should_exit = Arc::new(AtomicBool::new(false));

    #[cfg(target_os = "linux")]
    const HOST_ID: HostId = cpal::HostId::Jack;
    #[cfg(not(target_os = "linux"))]
    const HOST_ID: HostId = cpal::HostId::Asio;

    let device = cpal::host_from_id(HOST_ID)
        .unwrap_or(cpal::default_host())
        .default_input_device()
        .expect("No input devices found.");

    let config = device
        .supported_input_configs()
        .expect("No audio device configs found. Are there any connected?")
        .next()
        .expect("No audio configurations avaiable for default device...")
        .with_max_sample_rate()
        .config();

    let mins = match std::env::args().skip(1).next() {
        Some(min) => min,
        None => {
            eprintln!("Did not receive argument for number of minutes to buffer...");
            std::process::exit(1);
        }
    };

    let buff_len = config.sample_rate.0 as usize
        * mins
            .parse::<usize>()
            .expect("Did not receive a valid number of minutes.")
        * 60; // take in number of minutes to record from arguments
              // It's probably faster/more efficient to use ringbuf crate without Arc<Mutex>, but push_slice_overwrite isn't working
    let buff = Arc::new(Mutex::new(RingBuff::<f32>::with_capacity(buff_len)));

    let buff_clone = buff.clone();
    let config_clone = config.clone();
    let should_exit_clone = should_exit.clone();

    let write_handle = thread::spawn(move || {
        let mut data = Vec::<f32>::new();
        data.resize(2048, 0.0);
        loop {
            if rx.is_empty() {
                thread::park();
            }

            break_if!(should_exit_clone.load(std::sync::atomic::Ordering::Relaxed));

            let num_bytes = rx.pop_slice(&mut data);
            let mut lock = buff_clone.lock().unwrap();
            lock.push_slice(&data[..num_bytes]);
        }

        debug_print!("Write thread exiting...");
    });

    let write_thread = write_handle.thread().clone();

    // cpal has its own thread, but looking to transition away from libraries, so leaving it in its own thread for easier refactoring later
    let read_handle = thread::spawn(move || {
        let stream = device
            .build_input_stream(
                &config_clone,
                move |data: &[f32], _| {
                    write_thread.unpark();
                    tx.push_slice(data);
                },
                |err| eprintln!("An error has been detected during recording: {err:?}"),
                None,
            )
            .expect("Unable to build stream");

        stream.play().expect("Unable to record...");
        println!("Recording started with buffer length of {mins} minutes...");
        thread::park();
        debug_print!("Read-thread exiting...");
    });

    // let ui = MainWindow::new().unwrap();
    // ui.on_save(move || {
    //     println!("Saving data...");
    //     let lock = buff.lock().unwrap();
    //     write_input_data::<f32, f32>(&lock.vectorize(), &config);
    // });

    // ui.on_exit(|| {
    //     println!("Exiting...");
    //     quit_event_loop().unwrap();
    // });

    // ui.window().on_close_requested(move || {
    //     quit_event_loop().unwrap();
    //     CloseRequestResponse::HideWindow
    // });

    // ui.run().unwrap();

    let mut line = String::new();
    loop {
        let _ = std::io::stdin().read_line(&mut line);

        break_if!(line.trim().starts_with("q"));

        if line.trim().starts_with("h") {
            println!("Press enter to save, or type q to quit without saving.");
            continue;
        }

        println!("Saving data...");
        let lock = buff.lock().unwrap();
        write_input_data::<f32, f32>(&lock.vectorize(), &config);
    }

    // Clean up
    should_exit.store(true, std::sync::atomic::Ordering::SeqCst);

    write_handle.thread().unpark();
    read_handle.thread().unpark();

    write_handle.join().unwrap();
    read_handle.join().unwrap();

    println!("Goodbye!");
}
