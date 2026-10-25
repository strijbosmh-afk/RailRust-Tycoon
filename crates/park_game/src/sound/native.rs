//! Windows PCM streaming. Headers/data stay boxed until reset/unprepare succeeds.
use super::{
    Message,
    synth::{Mixer, RATE},
};
use std::{
    sync::{Arc, Mutex, mpsc::Receiver},
    time::Duration,
};
use windows::Win32::Media::Audio::*;
use windows::core::PSTR;
const FRAMES: usize = 2205;
struct Block {
    data: Box<[i16]>,
    header: Box<WAVEHDR>,
    prepared: bool,
}
struct Device {
    handle: HWAVEOUT,
    blocks: Vec<Block>,
}
impl Device {
    fn open() -> Result<Self, String> {
        let format = WAVEFORMATEX {
            wFormatTag: WAVE_FORMAT_PCM as u16,
            nChannels: 2,
            nSamplesPerSec: RATE,
            nAvgBytesPerSec: RATE * 4,
            nBlockAlign: 4,
            wBitsPerSample: 16,
            cbSize: 0,
        };
        let mut handle = HWAVEOUT::default();
        // The format lives through open. CALLBACK_NULL uses completion flags;
        // no Rust callback or application-world pointer crosses the driver.
        let code = unsafe {
            waveOutOpen(
                Some(&mut handle),
                WAVE_MAPPER,
                &format,
                None,
                None,
                CALLBACK_NULL,
            )
        };
        if code != 0 {
            return Err(format!(
                "No audio device (Windows code {code}); game remains playable."
            ));
        }
        let mut device = Self {
            handle,
            blocks: Vec::new(),
        };
        for _ in 0..3 {
            let mut data = vec![0i16; FRAMES * 2].into_boxed_slice();
            let mut header = Box::new(WAVEHDR {
                lpData: PSTR(data.as_mut_ptr().cast()),
                dwBufferLength: (FRAMES * 4) as u32,
                ..Default::default()
            });
            let code = unsafe {
                waveOutPrepareHeader(handle, &mut *header, std::mem::size_of::<WAVEHDR>() as u32)
            };
            if code != 0 {
                return Err(format!("Audio buffer unavailable (Windows code {code})."));
            }
            device.blocks.push(Block {
                data,
                header,
                prepared: true,
            });
        }
        Ok(device)
    }
    fn fill(&mut self, index: usize, mixer: &mut Mixer) -> Result<(), String> {
        let b = &mut self.blocks[index];
        b.data.copy_from_slice(&mixer.render(FRAMES));
        let code = unsafe {
            waveOutWrite(
                self.handle,
                &mut *b.header,
                std::mem::size_of::<WAVEHDR>() as u32,
            )
        };
        if code != 0 {
            return Err(format!("Audio output stopped (Windows code {code})."));
        }
        Ok(())
    }
    fn done(&self, index: usize) -> bool {
        // The packed driver-owned flag may be unaligned. Read its bytes with
        // volatile accesses, without constructing an aligned Rust reference.
        let ptr = std::ptr::addr_of!(self.blocks[index].header.dwFlags).cast::<u8>();
        let mut bytes = [0; 4];
        for (i, b) in bytes.iter_mut().enumerate() {
            *b = unsafe { std::ptr::read_volatile(ptr.add(i)) };
        }
        u32::from_ne_bytes(bytes) & WHDR_DONE != 0
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        // Reset returns every queued block before releasing its backing memory.
        let reset = unsafe { waveOutReset(self.handle) };
        for b in self.blocks.drain(..) {
            let mut b = b;
            let code = if b.prepared && reset == 0 {
                unsafe {
                    waveOutUnprepareHeader(
                        self.handle,
                        &mut *b.header,
                        std::mem::size_of::<WAVEHDR>() as u32,
                    )
                }
            } else {
                1
            };
            if code != 0 {
                // A failed driver must never retain pointers into freed memory.
                // Leak at most these three fixed buffers on this fatal path.
                std::mem::forget(b);
            }
        }
        unsafe {
            waveOutClose(self.handle);
        }
    }
}
pub(super) fn stream(receiver: Receiver<Message>, status: Arc<Mutex<String>>) {
    let result = (|| -> Result<(), String> {
        let mut device = Device::open()?;
        let mut mixer = Mixer::default();
        mixer.state.focused = false;
        for index in 0..3 {
            device.fill(index, &mut mixer)?;
        }
        if let Ok(mut s) = status.lock() {
            *s = "Playing original PCM sound / 44.1 kHz stereo".into();
        }
        loop {
            while let Ok(message) = receiver.try_recv() {
                match message {
                    Message::State(state) => mixer.state = state,
                    Message::Cue(c) => mixer.cue(c),
                    Message::Quit => return Ok(()),
                }
            }
            for i in 0..3 {
                if device.done(i) {
                    device.fill(i, &mut mixer)?;
                }
            }
            match receiver.recv_timeout(Duration::from_millis(5)) {
                Ok(Message::State(s)) => mixer.state = s,
                Ok(Message::Cue(c)) => mixer.cue(c),
                Ok(Message::Quit) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    return Ok(());
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            }
        }
    })();
    if let Err(error) = result
        && let Ok(mut s) = status.lock()
    {
        *s = error;
    }
}
pub(super) fn device_check() -> Result<(), String> {
    let mut d = Device::open()?;
    let mut m = Mixer::default();
    m.state.settings.levels[0] = 20;
    m.cue(super::Cue::Build);
    d.fill(0, &mut m)?;
    let start = std::time::Instant::now();
    while !d.done(0) {
        if start.elapsed() > Duration::from_secs(3) {
            return Err("Audio device did not return the PCM block".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Ok(())
}
