// FIXME: Once remove macros, can delete
#![allow(trivial_casts, trivial_numeric_casts)]

use core::f64::consts::PI as PI_64;
use core::mem;

#[derive(Clone)]
pub(crate) struct ResamplerState {
    pub(crate) filt_len: u32,
    pub(crate) mem_alloc_size: u32,
    pub(crate) buffer_size: u32,
    pub(crate) int_advance: u32,
    pub(crate) frac_advance: u32,
    pub(crate) cutoff: f32,
    pub(crate) oversample: u32,
    pub(crate) started: u32,
    pub(crate) mem: Vec<f32>,
    pub(crate) sinc_table: Vec<f32>,
    pub(crate) sinc_table_length: u32,
    pub(crate) resampler_ptr: ResamplerBasicFunc,

    // ex-vecs
    pub(crate) last_sample: u32,
    pub(crate) samp_frac_num: u32,
    pub(crate) magic_samples: u32,
}

impl Default for ResamplerState {
    fn default() -> Self {
        Self {
            started: 0,
            sinc_table: Vec::new(),
            sinc_table_length: 0,
            mem: Vec::new(),
            frac_advance: 0,
            int_advance: 0,
            mem_alloc_size: 0,
            filt_len: 0,
            resampler_ptr: None,
            cutoff: 1.0,
            buffer_size: 160,
            oversample: 0,
            last_sample: 0,
            magic_samples: 0,
            samp_frac_num: 0,
        }
    }
}

impl core::fmt::Debug for ResamplerState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ResamplerState")
    }
}

#[derive(Copy, Clone)]
pub(crate) struct QualityMapping {
    base_length: usize,
    oversample: usize,
    downsample_bandwidth: f32,
    upsample_bandwidth: f32,
}

impl QualityMapping {
    pub(crate) const fn new(
        base_length: usize,
        oversample: usize,
        downsample_bandwidth: f32,
        upsample_bandwidth: f32,
    ) -> Self {
        Self {
            base_length,
            oversample,
            downsample_bandwidth,
            upsample_bandwidth,
        }
    }
}

pub(crate) type ResamplerBasicFunc = Option<
    fn(
        _: &mut ResamplerState,
        _: &[f32],
        _: &mut u32,
        _: &mut [f32],
        _: &mut u32,
        _: u32,
    ) -> i32,
>;

// FIXME: Evaluate macro.
macro_rules! chunk_assign {
    ($ch_mut:ident, $lbound_mut:expr, $ubound_mut:expr, $val:expr) => {
        $ch_mut[$lbound_mut as usize..$ubound_mut as usize]
            .iter_mut()
            .for_each(|x| *x = $val);
    };
}

// FIXME: macro => function.
macro_rules! chunk_copy {
    ($ch_mut:ident, $lbound_mut:expr, $ubound_mut:expr,
     $ch:ident, $lbound:expr, $ubound:expr) => {{
        $ch_mut[$lbound_mut as usize..$ubound_mut as usize]
            .iter_mut()
            .zip($ch[$lbound as usize..$ubound as usize].iter())
            .for_each(|(x, y)| *x = *y);
    }};
}

// FIXME: Evaluate macro.
macro_rules! algo {
    ($self:ident, $ch_mut:ident, $ch:ident,
     $old_length:ident, $magic:expr) => {
        let olen = $old_length + 2 * $magic;
        let filt_len = $self.filt_len - 1;
        if $self.filt_len > olen {
            let new_filt_len = $self.filt_len - olen;
            let new_last_sample = &mut $self.last_sample;
            {
                chunk_copy!($ch_mut, new_filt_len, filt_len, $ch, 0, olen - 1);
                chunk_assign!($ch_mut, 0, new_filt_len, 0.0);
                $magic = 0;
                *new_last_sample += new_filt_len / 2;
            }
        } else {
            $magic = (olen - $self.filt_len) / 2;
            let ubound_mut = filt_len + $magic;
            let ubound = ubound_mut + $magic;
            chunk_copy!($ch_mut, 0, ubound_mut, $ch, $magic, ubound);
        }
    };
}

impl ResamplerState {
    /* * Resample a float array. The input and output buffers must *not* overlap.
     * @param st Resampler state
     * @param in Input buffer
     * @param in_len number of input samples in the input buffer. Returns the
     * number of samples processed
     * @param out Output buffer
     * @param out_len Size of the output buffer. Returns the number of samples written
     */
    pub(crate) fn process_float(
        &mut self,
        mut in_0: &[f32],
        in_len: &mut u32,
        mut out: &mut [f32],
        out_len: &mut u32,
        den: u32,
    ) {
        if in_0.is_empty() {
            panic!("Empty slice is not allowed");
        }
        let mut ilen = *in_len;
        let mut olen = *out_len;
        let filt_offs = (self.filt_len - 1) as usize;
        let mem_idx = filt_offs;
        let xlen = self.mem_alloc_size - self.filt_len - 1;
        if self.magic_samples != 0 {
            olen -= speex_resampler_magic(self, &mut out, olen, den);
        }
        if self.magic_samples == 0 {
            while 0 != ilen && 0 != olen {
                let mut ichunk: u32 = if ilen > xlen { xlen } else { ilen };
                let mut ochunk: u32 = olen;
                let mem_slice = &mut self.mem[mem_idx..];
                let mem_iter = mem_slice.iter_mut();
                let in_iter = in_0.iter();
                mem_iter.zip(in_iter).take(ichunk as usize).for_each(
                    |(x, &y)| {
                        *x = y;
                    },
                );
                speex_resampler_process_native(
                    self,
                    &mut ichunk,
                    out,
                    &mut ochunk,
                    den,
                );
                ilen -= ichunk;
                olen -= ochunk;
                out = &mut out[ochunk as usize..][..];
                in_0 = &in_0[ichunk as usize..][..];
            }
        }
        *in_len -= ilen;
        *out_len -= olen;
        let resampler = self.resampler_ptr.unwrap();
        if resampler as usize == resampler_basic_zero as usize {
            panic!("alloc failed");
        }
    }

    /* * Make sure that the first samples to go out of the resamplers don't have
     * leading zeros. This is only useful before starting to use a newly created
     * resampler. It is recommended to use that when resampling an audio file, as
     * it will generate a file with the same length. For real-time processing,
     * it is probably easier not to use this call (so that the output duration
     * is the same for the first frame).
     * @param st Resampler state
     */
    pub(crate) fn skip_zeros(&mut self) {
        let filt_len = self.filt_len / 2;
        self.last_sample = filt_len;
    }

    /* * Reset a resampler so a new (unrelated) stream can be processed.
     * @param st Resampler state
     */
    #[allow(unused)] // For now.
    pub(crate) fn reset_mem(&mut self) {
        self.last_sample = 0;
        self.magic_samples = 0;
        self.samp_frac_num = 0;

        self.mem.iter_mut().for_each(|elem| *elem = 0.);
    }

    #[inline]
    fn num_den(&mut self, num: u32, den: u32) {
        self.cutoff =
            QUALITY_MAPPING.downsample_bandwidth * den as f32 / num as f32;
        let pass = self.filt_len;
        _muldiv(&mut self.filt_len, pass, num, den);
        self.filt_len = ((self.filt_len - 1) & (!7)) + 8;
        self.oversample = (1..5)
            .filter(|x| 2u32.pow(*x) * den < num)
            .fold(self.oversample, |acc, _| acc >> 1);

        if self.oversample < 1 {
            self.oversample = 1;
        }
    }

    #[inline]
    fn use_direct(&mut self, den: u32) {
        let iter_chunk = self.sinc_table.chunks_mut(self.filt_len as usize);
        for (i, chunk) in iter_chunk.enumerate() {
            for (j, elem) in chunk.iter_mut().enumerate() {
                *elem = sinc(
                    self.cutoff,
                    (j as f32 - self.filt_len as f32 / 2.0 + 1.0)
                        - (i as f32) / den as f32,
                    self.filt_len as i32,
                );
            }
        }
        self.resampler_ptr = Some(resampler_basic_direct);
    }

    #[inline]
    fn not_use_direct(&mut self) {
        let cutoff = self.cutoff;
        let oversample = self.oversample;
        let filt_len = self.filt_len;
        self.sinc_table
            .iter_mut()
            .enumerate()
            .take((oversample * filt_len + 8) as usize)
            .for_each(|(i, x)| {
                *x = sinc(
                    cutoff,
                    (i as i32 - 4) as f32 / oversample as f32
                        - filt_len as f32 / 2.0,
                    filt_len as i32,
                )
            });
        self.resampler_ptr = Some(resampler_basic_interpolate);
    }

    #[inline(always)]
    fn chunks_iterator(
        &mut self,
        old_length: u32,
        alloc_size: usize,
        algo: usize,
    ) {
        let mem_copy = self.mem.clone();

        let mut_mem = self.mem.chunks_mut(self.mem_alloc_size as usize);
        let mem = mem_copy.chunks(alloc_size);

        for (ch_mut, ch) in mut_mem.zip(mem) {
            let magic = &mut self.magic_samples;
            {
                if algo == 0 {
                    let range = old_length - 1 + *magic;
                    chunk_copy!(ch_mut, *magic, range, ch, 0, range);
                    chunk_assign!(ch_mut, 0, *magic, 0.0);
                } else if algo == 1 {
                    algo!(self, ch_mut, ch, old_length, *magic);
                } else {
                    let skip = (old_length - self.filt_len) / 2;
                    let ubound = self.filt_len - 1 + skip + *magic;
                    chunk_copy!(ch_mut, 0, ubound, ch, skip, ubound + skip);
                    *magic += skip;
                }
            }
        }
    }

    pub(super) fn update_filter(&mut self, num: u32, den: u32) {
        let old_length = self.filt_len;
        let old_alloc_size = self.mem_alloc_size as usize;
        self.int_advance = num / den;
        self.frac_advance = num % den;
        self.oversample = QUALITY_MAPPING.oversample as u32;
        self.filt_len = QUALITY_MAPPING.base_length as u32;
        if num > den {
            self.num_den(num, den);
        } else {
            self.cutoff = QUALITY_MAPPING.upsample_bandwidth;
        }

        let use_direct = self.filt_len * den
            <= self.filt_len * self.oversample + 8
            && 2147483647_u64
                / ::std::mem::size_of::<f32>() as u64
                / den as u64
                >= self.filt_len as u64;

        let min_sinc_table_length = if !use_direct {
            self.filt_len * self.oversample + 8
        } else {
            self.filt_len * den
        };

        if self.sinc_table_length < min_sinc_table_length {
            self.sinc_table = vec![0.0; min_sinc_table_length as usize];
            self.sinc_table_length = min_sinc_table_length;
        }

        if use_direct {
            self.use_direct(den);
        } else {
            self.not_use_direct();
        }

        let min_alloc_size = self.filt_len - 1 + self.buffer_size;
        if min_alloc_size > self.mem_alloc_size {
            let mem = self.mem.clone();
            self.mem = vec![0.0; (min_alloc_size) as usize];
            self.mem[0..mem.len()].copy_from_slice(&mem);
            self.mem_alloc_size = min_alloc_size;
        }

        if self.started == 0 {
            let dim = self.mem_alloc_size as usize;
            self.mem = vec![0.0; dim];
        } else if self.filt_len > old_length {
            self.chunks_iterator(old_length, old_alloc_size, 0);
            self.chunks_iterator(old_length, self.mem_alloc_size as usize, 1);
        } else if self.filt_len < old_length {
            self.chunks_iterator(old_length, self.mem_alloc_size as usize, 2);
        }

        // skip zeros.
        self.skip_zeros();
    }
}

fn resampler_basic_zero(
    st: &mut ResamplerState,
    _in_0: &[f32],
    in_len: &mut u32,
    out: &mut [f32],
    out_len: &mut u32,
    den_rate: u32,
) -> i32 {
    let mut out_sample: u32 = 0;
    let mut last_sample = st.last_sample;
    let mut samp_frac_num = st.samp_frac_num;
    let int_advance = st.int_advance;
    let frac_advance = st.frac_advance;
    while !(last_sample >= *in_len || out_sample >= *out_len) {
        out[out_sample as usize] = 0.0;
        out_sample += 1;
        last_sample += int_advance;
        samp_frac_num += frac_advance as u32;
        if samp_frac_num >= den_rate {
            samp_frac_num -= den_rate as u32;
            last_sample += 1
        }
    }
    st.last_sample = last_sample;
    st.samp_frac_num = samp_frac_num;
    out_sample as i32
}

#[inline(always)]
fn cubic_coef(frac: f32, interp: &mut [f32]) {
    interp[0] = -0.166_67 * frac + 0.166_67 * frac * frac * frac;
    interp[1] = frac + 0.5 * frac * frac - 0.5f32 * frac * frac * frac;
    interp[3] =
        -0.333_33 * frac + 0.5 * frac * frac - 0.166_67 * frac * frac * frac;
    interp[2] =
        (1.0f64 - interp[0] as f64 - interp[1] as f64 - interp[3] as f64)
            as f32;
}

#[allow(clippy::too_many_arguments)]
#[inline(always)]
fn interpolate_step(
    in_slice: &[f32],
    out_slice: &mut [f32],
    out_sample: usize,
    oversample: usize,
    offset: usize,
    n: usize,
    sinc_table: &[f32],
    frac: f32,
) {
    let mut accum: [f32; 4] = [0.; 4];
    in_slice.iter().zip(0..n).for_each(|(&curr_in, j)| {
        let idx = (2 + (j + 1) * oversample as usize) - offset as usize;
        accum.iter_mut().zip(sinc_table.iter().skip(idx)).for_each(
            |(v, &s)| {
                *v += curr_in * s;
            },
        );
    });
    let mut interp: [f32; 4] = [0.; 4];
    cubic_coef(frac, &mut interp);
    out_slice[out_sample as usize] = interp
        .iter()
        .zip(accum.iter())
        .map(|(&x, &y)| x * y)
        .fold(0., |acc, x| acc + x);
}

#[inline(always)]
fn direct_step(
    in_slice: &[f32],
    out_slice: &mut [f32],
    out_sample: usize,
    n: usize,
    sinc_table: &[f32],
) {
    let mut sum: f32 = 0.0;
    let mut j = 0;
    while j < n {
        sum += sinc_table[j as usize] * in_slice[j as usize];
        j += 1
    }
    out_slice[out_sample as usize] = sum;
}

fn resampler_basic_interpolate(
    st: &mut ResamplerState,
    in_0: &[f32],
    in_len: &mut u32,
    out: &mut [f32],
    out_len: &mut u32,
    den_rate: u32,
) -> i32 {
    let n = st.filt_len as usize;
    let mut last_sample = st.last_sample;
    let mut samp_frac_num = st.samp_frac_num;
    let int_advance = st.int_advance;
    let frac_advance = st.frac_advance;
    let oversample = st.oversample;
    let sinc_table = &st.sinc_table;

    let mut out_sample: u32 = 0;
    while !(last_sample >= *in_len || out_sample >= *out_len) {
        let iptr = &in_0[last_sample as usize..];
        let offset = samp_frac_num * oversample / den_rate;
        let frac =
            ((samp_frac_num * oversample) % den_rate) as f32 / den_rate as f32;

        interpolate_step(
            iptr,
            out,
            out_sample as usize,
            oversample as usize,
            offset as usize,
            n,
            sinc_table,
            frac,
        );

        out_sample += 1;
        last_sample += int_advance;
        samp_frac_num += frac_advance as u32;
        if samp_frac_num >= den_rate {
            samp_frac_num -= den_rate;
            last_sample += 1;
        }
    }
    st.last_sample = last_sample;
    st.samp_frac_num = samp_frac_num;
    out_sample as i32
}

static QUALITY_MAPPING: QualityMapping =
    QualityMapping::new(160, 16, 0.96, 0.96);

fn sinc(cutoff: f32, x: f32, n: i32) -> f32 {
    let xx = f64::from(x * cutoff);
    let x_abs = f64::from(x).abs();
    let n_64 = f64::from(n);
    let cutoff_64 = f64::from(cutoff);
    if x_abs < 0.000001 {
        cutoff
    } else if x_abs > 0.5 * n_64 {
        0.0
    } else {
        let first_factor = cutoff_64 * (PI_64 * xx).sin() / (PI_64 * xx);
        let second_factor =
            compute_func((2.0 * f64::from(x) / n_64).abs() as f32);
        (first_factor * second_factor) as f32
    }
}

fn compute_func(x: f32) -> f64 {
    let mut interp: [f64; 4] = [0.0; 4];
    let y = x * super::WINDOW_FN_OVERSAMPLE as f32;
    let ind = y.floor() as usize;
    let frac = f64::from(y - ind as f32);
    interp[3] = -0.1666666667 * frac + 0.1666666667 * frac.powi(3);
    interp[2] = frac + 0.5 * frac.powi(2) - 0.5 * frac.powi(3);
    interp[0] =
        -0.3333333333 * frac + 0.5 * frac.powi(2) - 0.1666666667 * frac.powi(3);

    interp[1] = 1.0 - interp[3] - interp[2] - interp[0];

    interp
        .iter()
        .zip(super::WINDOW_FN_KAISER_TABLE.iter().skip(ind))
        .map(|(&x, &y)| x * y)
        .sum()
}

fn resampler_basic_direct(
    st: &mut ResamplerState,
    in_0: &[f32],
    in_len: &mut u32,
    out: &mut [f32],
    out_len: &mut u32,
    den_rate: u32,
) -> i32 {
    let n = st.filt_len as usize;
    let mut out_sample: u32 = 0;
    let mut last_sample = st.last_sample;
    let mut samp_frac_num = st.samp_frac_num;
    let int_advance = st.int_advance;
    let frac_advance = st.frac_advance;
    while !(last_sample >= *in_len || out_sample >= *out_len) {
        let sinct: &[f32] =
            &st.sinc_table[(samp_frac_num * n as u32) as usize..];
        let iptr: &[f32] = &in_0[last_sample as usize..];

        direct_step(
            iptr,
            out,
            out_sample as usize,
            n,
            sinct,
        );

        out_sample += 1;
        last_sample += int_advance;
        samp_frac_num += frac_advance as u32;
        if samp_frac_num >= den_rate {
            samp_frac_num -= den_rate as u32;
            last_sample += 1
        }
    }
    st.last_sample = last_sample;
    st.samp_frac_num = samp_frac_num;
    out_sample as i32
}

fn _muldiv(result: &mut u32, value: u32, mul: u32, div: u32) {
    let major: u32 = value / div;
    let remainder: u32 = value % div;
    if remainder > 4294967295 / mul
        || major > 4294967295 / mul
        || major * mul > 4294967295 - remainder * mul / div
    {
        panic!("overflow");
    } else {
        *result = remainder * mul / div + major * mul;
    }
}

fn speex_resampler_process_native(
    st: &mut ResamplerState,
    in_len: &mut u32,
    out: &mut [f32],
    out_len: &mut u32,
    den: u32,
) {
    let n: usize = st.filt_len as usize;
    st.started = 1;
    let mem = &st.mem.clone();
    let out_sample: i32 = st.resampler_ptr.expect("non-null function pointer")(
        st, mem, in_len, out, out_len, den,
    );
    if st.last_sample < *in_len {
        *in_len = st.last_sample as u32;
    }
    *out_len = out_sample as u32;
    st.last_sample -= *in_len;
    let ilen = *in_len as usize;

    st.mem[0..(n - 1)].copy_from_slice(&mem[ilen..(ilen + n - 1)]);
}

fn speex_resampler_magic(
    st: &mut ResamplerState,
    out: &mut &mut [f32],
    mut out_len: u32,
    den: u32,
) -> u32 {
    let mut tmp_in_len = st.magic_samples;
    let mem_idx = st.filt_len as usize;
    speex_resampler_process_native(
        st,
        &mut tmp_in_len,
        *out,
        &mut out_len,
        den,
    );
    st.magic_samples -= tmp_in_len;
    if st.magic_samples != 0 {
        let mem = &st.mem[mem_idx - 1 + tmp_in_len as usize..].to_vec();
        st.mem
            .iter_mut()
            .skip(mem_idx - 1)
            .zip(mem.iter())
            .take(st.magic_samples as usize)
            .for_each(|(x, &y)| *x = y);
    }
    let value: &mut [f32] = mem::replace(out, &mut []);
    *out = &mut value[(out_len as u32) as usize..];
    out_len
}
