use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::process::{Command, Stdio};

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

fn get_audio( stream_url: String){
    let mut buffer = [0u8; 1024];
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
    if let Some(mut stdout) = ffmpeg_cmd.stdout.take(){
        let mut total_bytes = 0;
        while let Ok(n) = stdout.read(&mut buffer) {
            if n == 0 { break; } 

            writer.write_all(&buffer[..n]).expect("Failed to write to file");
            
            total_bytes += n;
            // Update the terminal with the current file size
            print!("\rSaved: {:.2} MB", total_bytes as f64 / 1_000_000.0);
            std::io::stdout().flush().unwrap();
        }
        writer.flush().expect("Final flush failed");
        println!("\nDone! Saved {} bytes to {}", total_bytes, "outputffm.raw");
        }
           print!("getting data...\n"); 
    
}

fn main() {
    let stream_url = get_url("https://youtu.be/uDl_oBmBIkY?si=eEuMGcL3ZBl83WGl");
    get_audio(stream_url);
}

