"""Regenerate the embedded CC0 cheer from the retained Ogg recording."""
import argparse
from pathlib import Path
import wave

import numpy as np
import soundfile as sf


def main():
    root = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=root / "assets/audio/cheers-original.ogg")
    parser.add_argument("--output", type=Path, default=root / "assets/audio/guest-cheer.wav")
    args = parser.parse_args()

    recording, source_rate = sf.read(args.source, always_2d=True)
    mono = recording.mean(axis=1)
    target_rate = 44100
    frames = round(len(mono) * target_rate / source_rate)
    positions = np.arange(frames) * source_rate / target_rate
    converted = np.interp(positions, np.arange(len(mono)), mono)
    peak = float(np.max(np.abs(converted)))
    if peak == 0:
        raise ValueError("The source recording is silent")
    converted *= 0.75 / peak
    fade_frames = target_rate // 100
    fade = np.minimum(1, np.minimum(np.arange(frames) / fade_frames,
                                   (frames - 1 - np.arange(frames)) / fade_frames))
    pcm = (converted * fade * 32767).astype("<i2")
    with wave.open(str(args.output), "wb") as output:
        output.setparams((1, 2, target_rate, 0, "NONE", "not compressed"))
        output.writeframes(pcm.tobytes())
    print(f"Wrote {frames} mono PCM frames at {target_rate} Hz to {args.output}")


if __name__ == "__main__":
    main()
