use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::process::{Command, Stdio};
use std::env;
use std::sync::{Arc, Mutex};
use ringbuf::{traits::*, HeapRb};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

fn get_url(url :&str)->String{
    let stream_url = Command::new("yt-dlp")
        .args(["--get-url"])
        .args(["--format"])
        .args(["bestaudio"])
        .args([url])
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to start yt-dlp");
    let out_stream = String::from_utf8(stream_url.stdout).expect("Output invalid for utf-8").trim().to_string();
    print!("The stream url is: {}", out_stream);
    return out_stream
}

fn get_audio( stream_url: String)-> Result<(), Box<dyn std::error::Error>>{
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Couldnt find any sound devices!");
    let config = device.default_output_config()?;
    
    let buffer = HeapRb::<u8>::new(32768);
    let (mut prod, mut cons) = buffer.split();
    let file = File::create("output.raw").expect("Failed to create file");
    let mut writer = BufWriter::new(file);
    let mut ffmpeg_cmd = Command::new("ffmpeg")
        .args([
    "-user_agent", 
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
    "-i", 
    &stream_url, // The URL you got from yt-dlp
    "-f", 
    "s16le", 
    "-ac", "2", 
    "-ar", "44100",
    "-"
])                      // Output to stdout
        .stdout(Stdio::piped())
        .spawn()
        .expect("ffmpeg failed");
    
    std::thread::spawn(move || {
        if let Some(mut stdout) =  ffmpeg_cmd.stdout.take(){
            //loop{
                //match prod.read_from(&mut stdout, None){
                    //Some(Ok(0))=>break,
                    //Some(Ok(_))=>continue,
                    //Some(Err(e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    //// Buffer is full; wait a bit for the consumer to catch up
                    //std::thread::sleep(std::time::Duration::from_millis(10));
                //}
                //Some(Err(_)) => break,
                //None=>std::thread::sleep(std::time::Duration::from_millis(10)),
                //}
            //}
        loop{
            if let Some(Ok(bytes)) = prod.read_from(&mut stdout, None){
                if bytes == 0 { break; }
            }
        }
        }
    });
    
    let stream = device.build_output_stream(
        &config.into(),
        move | data: &mut [f32], _:&cpal::OutputCallbackInfo | {
            let mut byte_buffer = [0u8; 2];
            for i in data.iter_mut(){
                
                if cons.pop_slice(&mut byte_buffer) == 2{
                    let sample_i16 = i16::from_le_bytes(byte_buffer);
                    *i = sample_i16 as f32 / 32768.0;
                }else {
                    *i = 0.0;
                }
            }
        },
        |err| eprintln!("An Error occured on stream {}", err), None
    )?;
    //if let Some(mut stdout) = ffmpeg_cmd.stdout.take(){
        //let mut total_bytes = 0;
        ////while let Ok(n) = stdout.read(&mut buffer) {
            ////if n == 0 { break; } 

            ////prod.push()expect("Failed to write to file");
            
            ////total_bytes += n;
            ////// Update the terminal with the current file size
            ////print!("\rSaved: {:.2} MB", total_bytes as f64 / 1_000_000.0);
            ////std::io::stdout().flush().unwrap();
        ////}
        ////
        //std::thread::spawn(move || {
			//if let Ok(bytes) = prod.read_from(&mut stdout, None){
				//if bytes == 0 {break;}
			//}
		//})
        //println!("\nDone! Saved {} bytes to {}", total_bytes, "outputffm.raw");
        //}
        print!("getting data...\n");
        stream.play()?;
        loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
        Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 {
        let stream_url = get_url(&args[1]);
        get_audio(stream_url);    
    }
    else{
        print!("Please put a proper argument!");
    }
}

