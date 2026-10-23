// SPDX-License-Identifier: EUPL-1.2

use std::io::{self, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tiny4linux::CameraModel;

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 360;
const FRAME_BYTES: usize = (WIDTH * HEIGHT * 4) as usize;
const STALL_TIMEOUT: Duration = Duration::from_secs(5);
const ERROR_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureError {
    MissingFfmpeg,
    Failed(String),
    TimedOut,
}

struct Mailbox {
    // A single slot prevents a slow renderer from accumulating old video frames.
    frame: Option<Vec<u8>>,
    stderr: Vec<u8>,
    finished: bool,
    last_frame: Instant,
}

pub struct Capture {
    child: Option<Child>,
    readers: Vec<JoinHandle<()>>,
    mailbox: Arc<Mutex<Mailbox>>,
}

impl Capture {
    pub fn start(device: &Path, model: CameraModel) -> Result<Self, CaptureError> {
        let mut command = Command::new("ffmpeg");
        command.args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "v4l2",
            "-input_format",
            "yuyv422",
            "-video_size",
            if model == CameraModel::Tiny4K {
                "640x480"
            } else {
                "640x360"
            },
            "-framerate",
            "15",
            "-i",
        ]);
        command.arg(device).args([
            "-an",
            "-vf",
            "scale=640:360:force_original_aspect_ratio=decrease,pad=640:360:(ow-iw)/2:(oh-ih)/2",
            "-threads",
            "1",
            "-pix_fmt",
            "rgba",
            "-f",
            "rawvideo",
            "pipe:1",
        ]);
        Self::spawn(command, FRAME_BYTES)
    }

    fn spawn(mut command: Command, frame_bytes: usize) -> Result<Self, CaptureError> {
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| match error.kind() {
                io::ErrorKind::NotFound => CaptureError::MissingFfmpeg,
                _ => CaptureError::Failed(error.to_string()),
            })?;
        let stdout = child.stdout.take().expect("piped stdout");
        let mut stderr = child.stderr.take().expect("piped stderr");
        let mailbox = Arc::new(Mutex::new(Mailbox {
            frame: None,
            stderr: Vec::new(),
            finished: false,
            last_frame: Instant::now(),
        }));
        let frames = Arc::clone(&mailbox);
        let frame_reader = thread::spawn(move || read_frames(stdout, frame_bytes, &frames));
        let errors = Arc::clone(&mailbox);
        let error_reader = thread::spawn(move || {
            let mut buffer = [0; 1024];
            while let Ok(count) = stderr.read(&mut buffer) {
                if count == 0 {
                    break;
                }
                let mut state = errors.lock().expect("preview mailbox");
                state.stderr.extend_from_slice(&buffer[..count]);
                let excess = state.stderr.len().saturating_sub(ERROR_BYTES);
                state.stderr.drain(..excess);
            }
        });
        Ok(Self {
            child: Some(child),
            readers: vec![frame_reader, error_reader],
            mailbox,
        })
    }

    pub fn poll(&mut self) -> (Option<Vec<u8>>, Option<CaptureError>) {
        let exited = match self
            .child
            .as_mut()
            .expect("live capture process")
            .try_wait()
        {
            Ok(status) => status.is_some(),
            Err(error) => return (None, Some(CaptureError::Failed(error.to_string()))),
        };
        let mut state = self.mailbox.lock().expect("preview mailbox");
        let frame = state.frame.take();
        let error = if exited || state.finished {
            let details = String::from_utf8_lossy(&state.stderr).trim().to_owned();
            Some(CaptureError::Failed(if details.is_empty() {
                "The video stream ended. The camera may be disconnected or in use by another app."
                    .into()
            } else {
                details
            }))
        } else if state.last_frame.elapsed() >= STALL_TIMEOUT {
            Some(CaptureError::TimedOut)
        } else {
            None
        };
        (frame, error)
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            // Killing the child unblocks pipe reads. Reaping it and joining readers
            // happens away from the UI, including when a device has stopped replying.
            let _ = child.kill();
            let readers = std::mem::take(&mut self.readers);
            thread::spawn(move || {
                let _ = child.wait();
                for reader in readers {
                    let _ = reader.join();
                }
            });
        }
    }
}

fn read_frames(mut input: impl Read, frame_bytes: usize, mailbox: &Mutex<Mailbox>) {
    loop {
        let mut frame = vec![0; frame_bytes];
        if input.read_exact(&mut frame).is_err() {
            mailbox.lock().expect("preview mailbox").finished = true;
            break;
        }
        let mut state = mailbox.lock().expect("preview mailbox");
        state.frame = Some(frame);
        state.last_frame = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mailbox() -> Mutex<Mailbox> {
        Mutex::new(Mailbox {
            frame: None,
            stderr: Vec::new(),
            finished: false,
            last_frame: Instant::now(),
        })
    }

    #[test]
    fn slow_renderer_keeps_only_the_newest_complete_frame() {
        let mailbox = mailbox();
        read_frames(io::Cursor::new(vec![1, 2, 3, 4, 5, 6, 7, 8]), 4, &mailbox);
        let state = mailbox.lock().unwrap();
        assert_eq!(state.frame, Some(vec![5, 6, 7, 8]));
        assert!(state.finished);
    }

    #[test]
    fn truncated_frame_is_never_displayed() {
        let mailbox = mailbox();
        read_frames(io::Cursor::new(vec![1, 2, 3]), 4, &mailbox);
        let state = mailbox.lock().unwrap();
        assert!(state.frame.is_none());
        assert!(state.finished);
    }

    #[test]
    fn stopping_preview_terminates_the_capture_process() {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "exec sleep 60"]);
        let capture = Capture::spawn(command, 4).unwrap();
        let process = capture.child.as_ref().unwrap().id();
        drop(capture);
        let deadline = Instant::now() + Duration::from_secs(2);
        while Path::new(&format!("/proc/{process}")).exists() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(!Path::new(&format!("/proc/{process}")).exists());
    }

    #[test]
    fn failed_capture_reports_stderr_instead_of_hanging() {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "printf 'Device or resource busy' >&2; exit 1"]);
        let mut capture = Capture::spawn(command, 4).unwrap();
        for reader in capture.readers.drain(..) {
            reader.join().unwrap();
        }
        let (_, error) = capture.poll();
        assert_eq!(
            error,
            Some(CaptureError::Failed("Device or resource busy".into()))
        );
    }

    #[test]
    fn missing_ffmpeg_is_reported_without_starting_workers() {
        assert!(matches!(
            Capture::spawn(Command::new("/nonexistent/tiny4linux/ffmpeg"), 4),
            Err(CaptureError::MissingFfmpeg)
        ));
    }

    #[test]
    #[ignore = "requires FFmpeg and a connected OBSBOT camera"]
    fn live_camera_delivers_frames_and_releases_device() {
        let camera = tiny4linux::Camera::detect().expect("connected OBSBOT camera");
        let path = camera.video_device_path().unwrap();
        for _ in 0..2 {
            let mut capture = Capture::start(&path, camera.model()).unwrap();
            let process = capture.child.as_ref().unwrap().id();
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut frames = 0;
            while frames < 5 && Instant::now() < deadline {
                let (frame, error) = capture.poll();
                assert!(error.is_none(), "capture failed: {error:?}");
                if let Some(frame) = frame {
                    assert_eq!(frame.len(), FRAME_BYTES);
                    assert!(frame.chunks_exact(4).all(|pixel| pixel[3] == 255));
                    frames += 1;
                }
                thread::sleep(Duration::from_millis(40));
            }
            assert_eq!(frames, 5, "camera did not deliver five frames");
            drop(capture);
            let deadline = Instant::now() + Duration::from_secs(2);
            while Path::new(&format!("/proc/{process}")).exists() && Instant::now() < deadline {
                thread::sleep(Duration::from_millis(10));
            }
            assert!(!Path::new(&format!("/proc/{process}")).exists());
        }
        println!(
            "Captured two five-frame streams from {}, then released the camera. No images were saved.",
            path.display()
        );
    }

    #[test]
    fn stalled_stream_times_out() {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "exec sleep 60"]);
        let mut capture = Capture::spawn(command, 4).unwrap();
        capture.mailbox.lock().unwrap().last_frame = Instant::now() - STALL_TIMEOUT;
        assert_eq!(capture.poll().1, Some(CaptureError::TimedOut));
    }
}
