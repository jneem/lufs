pub struct BiQuad {
    pub a: [f64; 2],
    pub b: [f64; 3],
}

impl BiQuad {
    pub fn ebu_prefilter_stage_1() -> BiQuad {
        BiQuad {
            a: [-1.69065929318241, 0.73248077421585],
            b: [1.53512485958697, -2.69169618940638, 1.19839281085285],
        }
    }

    pub fn ebu_prefilter_stage_2() -> BiQuad {
        BiQuad {
            a: [-1.99004745483398, 0.99007225036621],
            b: [1.0, -2.0, 1.0],
        }
    }
}

pub struct BiQuadIter<I: Iterator<Item = f32>> {
    f: BiQuad,
    prev_in: [f64; 2],
    prev_out: [f64; 2],
    input: I,
}

impl<I: Iterator<Item = f32>> BiQuadIter<I> {
    pub fn new(filter: BiQuad, iter: I) -> Self {
        BiQuadIter {
            f: filter,
            prev_in: [0.0, 0.0],
            prev_out: [0.0, 0.0],
            input: iter,
        }
    }
}

impl<I: Iterator<Item = f32>> Iterator for BiQuadIter<I> {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if let Some(input) = self.input.next() {
            let out = (input as f64) * self.f.b[0]
                + self.prev_in[0] * self.f.b[1]
                + self.prev_in[1] * self.f.b[2]
                - self.prev_out[0] * self.f.a[0]
                - self.prev_out[1] * self.f.a[1];
            self.prev_in[1] = self.prev_in[0];
            self.prev_in[0] = input as f64;
            self.prev_out[1] = self.prev_out[0];
            self.prev_out[0] = out;
            Some(out as f32)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_len() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let iter = BiQuadIter::new(BiQuad::ebu_prefilter_stage_1(), data.iter().copied());
        assert_eq!(iter.count(), data.len());
    }
}
