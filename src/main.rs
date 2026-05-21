use std::fs::File;
use std::io::{BufWriter,Seek, SeekFrom,BufReader, Read, Write};
use std::process::{Command, Stdio};
use std::env;
use std::sync::{Arc, Mutex};
use ringbuf::{traits::*, HeapRb};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustypipe::{client::RustyPipe, client::RustyPipeBuilder, param::StreamFilter};
use ringbuf::storage::Heap;
use std::time::Duration;

fn play_audio( stream_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut yt_cmd =  Command::new("yt-dlp").args([
        "-f", "bestaudio", "-o", "-", stream_url
    ])
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()?;
    
    let mut ff = Command::new("ffmpeg").args([
        "-i", "pipe:0",
        "-f", "f32le", 
        "-ar", "48000",
        "-ac", "2",
        "pipe:1",
    ])
    .stdin(yt_cmd.stdout.unwrap())
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()?;
    
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");
    let config = device.default_output_config()?;
    println!("Starting stream...");
    
    let mut stdout = ff.stdout.take().expect("Error opening stdout");
    
    let rb = HeapRb::<f32>::new(32768);
    let (mut prod, mut cons) = rb.split();
    
    let reader = std::thread::spawn( move || {
        let mut byte_buf = [0u8; 4];
        
        while let Ok(()) = stdout.read_exact(&mut byte_buf) {
            let sample = f32::from_le_bytes(byte_buf);
            
            while prod.is_full() {
               std::thread::sleep(std::time::Duration::from_millis(1));
            }
            let _ = prod.try_push(sample);
        }
    });
    
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _| {
            for sample in data.iter_mut() {
                *sample = cons.try_pop().unwrap_or(0.0);
            }
        },

        |err| eprintln!("cpal error: {}", err),
        None,
    )?;

    stream.play()?;
    reader.join().ok();
    ff.wait()?;
       
    Ok(())
    
}

fn main()-> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let rp = RustyPipe::new();
    if args.len() > 1 {
        println!("Playing......");
        play_audio(&args[1]);
    }
    else{
        print!("Please put a proper argument!");
    }
    
    Ok(())
}
