use std::process::{Command, Stdio, Child};
use std::fs;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{channel, Sender};

pub struct SessionRecorder {
    video_sender: Sender<Vec<u8>>,
    audio_sender: Sender<Vec<u8>>,
    output_path: String,
    ffmpeg_child: Option<Child>,
    video_thread: Option<JoinHandle<()>>,
    audio_thread: Option<JoinHandle<()>>,
    width: u32,
    height: u32,
    fps: u32,
    sample_rate: u32,
    frame_count: u64,
    fifo_dir: String,
}

pub struct RecorderConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub sample_rate: u32,
    pub output_path: String,
}

impl SessionRecorder {
    pub fn new(config: RecorderConfig) -> Result<Self, String> {
        // Create temp dir for fifos
        let fifo_dir = format!("/tmp/aurex_rec_{}", std::process::id());
        fs::create_dir_all(&fifo_dir).map_err(|e| format!("create dir: {}", e))?;

        let video_fifo = format!("{}/video.fifo", fifo_dir);
        let audio_fifo = format!("{}/audio.fifo", fifo_dir);

        // Create named pipes
        std::process::Command::new("mkfifo")
            .arg(&video_fifo)
            .status()
            .map_err(|e| format!("mkfifo video: {}", e))?;

        std::process::Command::new("mkfifo")
            .arg(&audio_fifo)
            .status()
            .map_err(|e| format!("mkfifo audio: {}", e))?;

        // Spawn ffmpeg
        let child = Command::new("ffmpeg")
            .arg("-y")
            .arg("-f").arg("rawvideo")
            .arg("-pix_fmt").arg("rgb24")
            .arg("-s").arg(format!("{}x{}", config.width, config.height))
            .arg("-r").arg(config.fps.to_string())
            .arg("-thread_queue_size").arg("512")
            .arg("-i").arg(&video_fifo)
            .arg("-f").arg("s16le")
            .arg("-ar").arg(config.sample_rate.to_string())
            .arg("-ac").arg("2")
            .arg("-thread_queue_size").arg("512")
            .arg("-i").arg(&audio_fifo)
            .arg("-c:v").arg("libx264")
            .arg("-pix_fmt").arg("yuv420p")
            .arg("-preset").arg("ultrafast")
            .arg("-crf").arg("23")
            .arg("-c:a").arg("aac")
            .arg("-b:a").arg("128k")
            .arg("-movflags").arg("+faststart")
            .arg("-shortest")
            .arg(&config.output_path)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("spawn ffmpeg: {}", e))?;

        // Open FIFOs and spawn writer threads
        let (video_tx, video_rx) = channel::<Vec<u8>>();
        let (audio_tx, audio_rx) = channel::<Vec<u8>>();

        let video_fifo_clone = video_fifo.clone();
        let video_thread = thread::spawn(move || {
            use std::io::Write;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .open(&video_fifo_clone)
                .expect("open video fifo");
            while let Ok(data) = video_rx.recv() {
                let _ = file.write_all(&data);
            }
        });

        let audio_fifo_clone = audio_fifo.clone();
        let audio_thread = thread::spawn(move || {
            use std::io::Write;
            let mut file = fs::OpenOptions::new()
                .write(true)
                .open(&audio_fifo_clone)
                .expect("open audio fifo");
            while let Ok(data) = audio_rx.recv() {
                let _ = file.write_all(&data);
            }
        });

        Ok(Self {
            video_sender: video_tx,
            audio_sender: audio_tx,
            output_path: config.output_path,
            ffmpeg_child: Some(child),
            video_thread: Some(video_thread),
            audio_thread: Some(audio_thread),
            width: config.width,
            height: config.height,
            fps: config.fps,
            sample_rate: config.sample_rate,
            frame_count: 0,
            fifo_dir,
        })
    }

    /// Convert RGB555 framebuffer to RGB24 and send to video writer thread
    pub fn write_frame(&mut self, framebuffer: &[u16], audio_samples: &[i16]) -> Result<(), String> {
        // Convert RGB555 -> RGB24
        let mut rgb24 = Vec::with_capacity((self.width * self.height * 3) as usize);
        for &pixel in framebuffer {
            let r5 = ((pixel >> 10) & 0x1F) as u8;
            let g5 = ((pixel >> 5) & 0x1F) as u8;
            let b5 = (pixel & 0x1F) as u8;
            // Expand 5->8 bits
            rgb24.push((r5 << 3) | (r5 >> 2));
            rgb24.push((g5 << 3) | (g5 >> 2));
            rgb24.push((b5 << 3) | (b5 >> 2));
        }

        // Convert audio to bytes (little-endian i16)
        let mut audio_bytes = Vec::with_capacity(audio_samples.len() * 2);
        for &sample in audio_samples {
            audio_bytes.extend_from_slice(&sample.to_le_bytes());
        }

        // Send data through channels to writer threads
        self.video_sender.send(rgb24)
            .map_err(|e| format!("send video: {}", e))?;
        self.audio_sender.send(audio_bytes)
            .map_err(|e| format!("send audio: {}", e))?;

        self.frame_count += 1;
        Ok(())
    }

    pub fn finish(mut self) -> Result<String, String> {
        // Drop senders so threads exit
        drop(self.video_sender);
        drop(self.audio_sender);

        // Join writer threads
        if let Some(t) = self.video_thread.take() {
            let _ = t.join();
        }
        if let Some(t) = self.audio_thread.take() {
            let _ = t.join();
        }

        // Close our references to FIFOs - ffmpeg will see EOF
        // Then wait for ffmpeg to finish encoding
        if let Some(mut child) = self.ffmpeg_child.take() {
            // Give ffmpeg time to flush
            std::thread::sleep(std::time::Duration::from_millis(500));

            match child.try_wait() {
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                _ => {}
            }
        }

        // Clean up FIFOs
        let video_fifo = format!("{}/video.fifo", self.fifo_dir);
        let audio_fifo = format!("{}/audio.fifo", self.fifo_dir);
        let _ = fs::remove_file(&video_fifo);
        let _ = fs::remove_file(&audio_fifo);
        let _ = fs::remove_dir(&self.fifo_dir);

        Ok(self.output_path)
    }
}
