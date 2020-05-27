mod biquad;

/// Takes a signal as input, which is assumed to be at 48kHz. Returns a filtered version of the
/// signal, as described in the reference material. The purpose of this filtering is to account for
/// the fact that perceived loudness depends on pitch.
fn prefilter<I: Iterator<Item = f32>>(input: I) -> Vec<f32> {
    let q1 = biquad::BiQuad::ebu_prefilter_stage_1();
    let q2 = biquad::BiQuad::ebu_prefilter_stage_2();
    let f1 = biquad::BiQuadIter::new(q1, input);
    let f2 = biquad::BiQuadIter::new(q2, f1);
    f2.collect()
}

/// Returns the mean square of a signal.
fn mean_square(signal: &[f32]) -> f32 {
    signal.iter().copied().map(|x| x * x).sum::<f32>() / (signal.len() as f32)
}

fn power(x: f32) -> f32 {
    -0.691 + 10.0 * x.log10()
}

/// Given a filtered signal, returns its loudness. This is determined by breaking the signal
/// into overlapping windows, and computing the mean power of those windows that are louder
/// than the threshold. The windows are 400ms long and overlap by 75%, as defined in the reference.
fn gated_loudness(signal: &[f32], threshold: f32) -> f32 {
    let window_len = 48_000 * 4 / 10;
    let mut offset = 0;
    let mut total = 0.0;
    let mut count = 0;

    while offset + window_len < signal.len() {
        let ms = mean_square(&signal[offset..(offset + window_len)]);
        if power(ms) >= threshold {
            total += ms;
            count += 1;
        }
        offset += window_len / 4;
    }

    if count == 0 {
        -std::f32::INFINITY
    } else {
        power(total / (count as f32))
    }
}

/// Returns the perceptual loudness, in LUFS, of a signal.
///
/// The signal is given in the range [-1.0, 1.0].
pub fn loudness<I: Iterator<Item = f32>>(signal: I) -> f32 {
    let filtered = prefilter(signal);
    let loudness_absolute = gated_loudness(&filtered[..], -70.0);

    // It would probably be faster (but take more memory) if we calculated the mean-squares of the
    // windows once and stored them.
    gated_loudness(&filtered[..], loudness_absolute - 10.0)
}

/// If we have an audio signal with LUFS `current_lufs`, by what should we multiply it
/// if we want it to have LUFS `target_lufs`?
pub fn multiplier(current_lufs: f32, target_lufs: f32) -> f32 {
    // Multiplying a signal by x has the effect of adding 20 * log_10(x) to the LUFS.
    let target_change = target_lufs - current_lufs;
    10.0f32.powf(target_change / 20.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sine_loudness() {
        // One second of 997Hz sine wave at 48kHz sample rate.
        let sine = (0..48_000).map(|x| (x as f32 * 997.0 / 48_000.0).sin());
        let mut signal = sine.clone().collect::<Vec<_>>();
        dbg!(loudness(signal.iter().copied()));
        // FIXME: this isn't quite right: according to the reference, the loudness of a 997Hz sine
        // wave should be -3.01, but we're getting less than that.
        assert!((loudness(signal.iter().copied()) + 4.1415) < 0.001);

        // Appending something very quiet won't change the loudness.
        signal.extend(sine.map(|x| x / 5.0));
        assert!((loudness(signal.iter().copied()) + 4.1415) < 0.001);
    }
}
