use core::mem;
use core::f64::consts::PI as PI_64;
use alloc::collections::VecDeque;

pub(crate) const RESAMPLER_ERR_SUCCESS: usize = 0;
pub(crate) const RESAMPLER_ERR_ALLOC_FAILED: usize = 1;
pub(crate) const RESAMPLER_ERR_INVALID_ARG: usize = 3;
pub(crate) const RESAMPLER_ERR_OVERFLOW: usize = 5;

#[derive(Clone)]
pub(crate) struct ResamplerState {
    pub(crate) in_rate: u32,
    pub(crate) out_rate: u32,
    pub(crate) num_rate: u32,
    pub(crate) den_rate: u32,
    pub(crate) nb_channels: u32,
    pub(crate) filt_len: u32,
    pub(crate) mem_alloc_size: u32,
    pub(crate) buffer_size: u32,
    pub(crate) int_advance: u32,
    pub(crate) frac_advance: u32,
    pub(crate) cutoff: f32,
    pub(crate) oversample: u32,
    pub(crate) initialised: u32,
    pub(crate) started: u32,
    pub(crate) last_sample: Vec<u32>,
    pub(crate) samp_frac_num: Vec<u32>,
    pub(crate) magic_samples: Vec<u32>,
    pub(crate) mem: Vec<f32>,
    pub(crate) sinc_table: Vec<f32>,
    pub(crate) sinc_table_length: u32,
    pub(crate) resampler_ptr: ResamplerBasicFunc,
    pub(crate) in_stride: u32,
    pub(crate) out_stride: u32,
}

impl core::fmt::Debug for ResamplerState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ResamplerState")
    }
}

#[derive(Copy, Clone)]
pub(crate) struct FuncDef {
    table: &'static [f64],
    oversample: usize,
}

impl FuncDef {
    pub(crate) const fn new(table: &'static [f64], oversample: usize) -> Self {
        Self { table, oversample }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct QualityMapping {
    base_length: usize,
    oversample: usize,
    downsample_bandwidth: f32,
    upsample_bandwidth: f32,
    window_func: &'static FuncDef,
}

impl QualityMapping {
    pub(crate) const fn new(
        base_length: usize,
        oversample: usize,
        downsample_bandwidth: f32,
        upsample_bandwidth: f32,
        window_func: &'static FuncDef,
    ) -> Self {
        Self {
            base_length,
            oversample,
            downsample_bandwidth,
            upsample_bandwidth,
            window_func,
        }
    }
}

pub(crate) type ResamplerBasicFunc = Option<
    fn(
        _: &mut ResamplerState,
        _: u32,
        _: &[f32],
        _: &mut u32,
        _: &mut [f32],
        _: &mut u32,
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

// FIXME: Evaluate macro.
macro_rules! chunk_copy {
    ($ch_mut:ident, $lbound_mut:expr, $ubound_mut:expr,
     $ch:ident, $lbound:expr, $ubound:expr) => {
        $ch_mut[$lbound_mut as usize..$ubound_mut as usize]
            .iter_mut()
            .zip($ch[$lbound as usize..$ubound as usize].iter())
            .for_each(|(x, y)| *x = *y);
    };
}

// FIXME: Evaluate macro.
macro_rules! algo {
    ($self:ident, $ch_mut:ident, $ch:ident,
     $old_length:ident, $magic:expr) => {
        let olen = $old_length + 2 * $magic;
        let filt_len = $self.filt_len - 1;
        if $self.filt_len > olen {
            let new_filt_len = $self.filt_len - olen;
            for new_last_sample in &mut $self.last_sample {
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
    /* * Create a new resampler with integer input and output rates.
     * @param nb_channels number of channels to be processed
     * @param in_rate Input sampling rate (integer number of Hz).
     * @param out_rate Output sampling rate (integer number of Hz).
     * @param quality Resampling quality between 0 and 10, where 0 has poor quality
     * and 10 has very high quality.
     * @return newly created resampler state
     */
    pub(crate) fn new(
        nb_channels: usize,
        in_rate: usize,
        out_rate: usize,
    ) -> Self {
        Self::init_frac(
            nb_channels,
            in_rate,
            out_rate,
            in_rate,
            out_rate,
        )
    }

    /* * Create a new resampler with fractional input/output rates. The sampling
     * rate ratio is an arbitrary rational number with both the numerator and
     * denominator being 32-bit integers.
     * @param nb_channels number of channels to be processed
     * @param ratio_num numerator of the sampling rate ratio
     * @param ratio_den Denominator of the sampling rate ratio
     * @param in_rate Input sampling rate rounded to the nearest integer (in Hz).
     * @param out_rate Output sampling rate rounded to the nearest integer (in Hz).
     * @param quality Resampling quality between 0 and 10, where 0 has poor quality
     * and 10 has very high quality.
     * @return newly created resampler state
     * @retval nULL Error: not enough memory
     */
    pub(crate) fn init_frac(
        nb_channels: usize,
        ratio_num: usize,
        ratio_den: usize,
        in_rate: usize,
        out_rate: usize,
    ) -> Self {
        let mut st = Self {
            initialised: 0,
            started: 0,
            in_rate: 0,
            out_rate: 0,
            num_rate: 0,
            den_rate: 0,
            sinc_table: Vec::new(),
            sinc_table_length: 0,
            mem: Vec::new(),
            frac_advance: 0,
            int_advance: 0,
            mem_alloc_size: 0,
            filt_len: 0,
            resampler_ptr: None,
            cutoff: 1.0,
            nb_channels: nb_channels as u32,
            in_stride: 1,
            out_stride: 1,
            buffer_size: 160,
            oversample: 0,
            last_sample: vec![0; nb_channels as usize],
            magic_samples: vec![0; nb_channels as usize],
            samp_frac_num: vec![0; nb_channels as usize],
        };
        st.set_rate_frac(ratio_num, ratio_den, in_rate, out_rate);
        let filter_err = st.update_filter();
        if filter_err == RESAMPLER_ERR_SUCCESS {
            st.initialised = 1;
        } else {
            panic!("Error");
        }
        st
    }

    /* * Resample a float array. The input and output buffers must *not* overlap.
     * @param st Resampler state
     * @param channel_index Index of the channel to process for the multi-channel
     * base (0 otherwise)
     * @param in Input buffer
     * @param in_len number of input samples in the input buffer. Returns the
     * number of samples processed
     * @param out Output buffer
     * @param out_len Size of the output buffer. Returns the number of samples written
     */
    pub(crate) fn process_float(
        &mut self,
        channel_index: u32,
        mut in_0: &[f32],
        in_len: &mut u32,
        mut out: &mut [f32],
        out_len: &mut u32,
    ) -> usize {
        if in_0.is_empty() {
            panic!("Empty slice is not allowed");
        }
        let mut ilen = *in_len;
        let mut olen = *out_len;
        let channel_idx = channel_index as usize;
        let filt_offs = (self.filt_len - 1) as usize;
        let mem_idx = filt_offs + channel_idx * self.mem_alloc_size as usize;
        let xlen = self.mem_alloc_size - self.filt_len - 1;
        let istride = self.in_stride as usize;
        if self.magic_samples[channel_idx] != 0 {
            olen -= speex_resampler_magic(self, channel_index, &mut out, olen);
        }
        if self.magic_samples[channel_idx] == 0 {
            while 0 != ilen && 0 != olen {
                let mut ichunk: u32 = if ilen > xlen { xlen } else { ilen };
                let mut ochunk: u32 = olen;
                let mem_slice = &mut self.mem[mem_idx..];
                let mem_iter = mem_slice.iter_mut();
                let in_iter = in_0.iter().step_by(istride);
                mem_iter.zip(in_iter).take(ichunk as usize).for_each(
                    |(x, &y)| {
                        *x = y;
                    },
                );
                speex_resampler_process_native(
                    self,
                    channel_index,
                    &mut ichunk,
                    out,
                    &mut ochunk,
                );
                ilen -= ichunk;
                olen -= ochunk;
                out = &mut out[(ochunk * self.out_stride) as usize..][..];
                in_0 = &in_0[(ichunk * self.in_stride) as usize..][..];
            }
        }
        *in_len -= ilen;
        *out_len -= olen;
        let resampler = self.resampler_ptr.unwrap();
        if resampler as usize == resampler_basic_zero as usize {
            RESAMPLER_ERR_ALLOC_FAILED
        } else {
            RESAMPLER_ERR_SUCCESS
        }
    }

    /* * Set (change) the input/output sampling rates and resampling ratio
     * (fractional values in Hz supported).
     * @param st Resampler state
     * @param ratio_num numerator of the sampling rate ratio
     * @param ratio_den Denominator of the sampling rate ratio
     * @param in_rate Input sampling rate rounded to the nearest integer (in Hz).
     * @param out_rate Output sampling rate rounded to the nearest integer (in Hz).
     */
    pub(crate) fn set_rate_frac(
        &mut self,
        ratio_num: usize,
        ratio_den: usize,
        in_rate: usize,
        out_rate: usize,
    ) -> usize {
        if ratio_num == 0 || ratio_den == 0 {
            RESAMPLER_ERR_INVALID_ARG
        } else if self.in_rate == in_rate as u32
            && self.out_rate == out_rate as u32
            && self.num_rate == ratio_num as u32
            && self.den_rate == ratio_den as u32
        {
            RESAMPLER_ERR_SUCCESS
        } else {
            let old_den = self.den_rate;
            self.in_rate = in_rate as u32;
            self.out_rate = out_rate as u32;
            self.num_rate = ratio_num as u32;
            self.den_rate = ratio_den as u32;
            let fact = gcd(self.num_rate, self.den_rate);
            self.num_rate /= fact;
            self.den_rate /= fact;
            if old_den > 0 {
                for val in &mut self.samp_frac_num {
                    let res = _muldiv(val, *val, self.den_rate, old_den);
                    if res != RESAMPLER_ERR_SUCCESS {
                        return RESAMPLER_ERR_OVERFLOW;
                    } else if *val >= self.den_rate {
                        *val = self.den_rate - 1;
                    }
                }
            }
            if self.initialised != 0 {
                self.update_filter()
            } else {
                RESAMPLER_ERR_SUCCESS
            }
        }
    }

    /* * Get the current input/output sampling rates (integer value).
     * @param st Resampler state
     */
    pub(crate) fn get_rate(&self) -> (usize, usize) {
        (self.in_rate as usize, self.out_rate as usize)
    }

    /* * Get the current resampling ratio. This will be reduced to the least
     * common denominator.
     * @param st Resampler state
     */
    pub(crate) fn get_ratio(&self) -> (usize, usize) {
        (self.num_rate as usize, self.den_rate as usize)
    }

    /* * Get the latency introduced by the resampler measured in input samples.
     * @param st Resampler state
     */
    pub(crate) fn get_input_latency(&self) -> usize {
        (self.filt_len / 2) as usize
    }

    /* * Get the latency introduced by the resampler measured in output samples.
     * @param st Resampler state
     */
    pub(crate) fn get_output_latency(&self) -> usize {
        (((self.filt_len / 2) * self.den_rate + (self.num_rate >> 1))
            / self.num_rate) as usize
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
        self.last_sample.iter_mut().for_each(|v: &mut u32| {
            *v = filt_len;
        });
    }

    /* * Reset a resampler so a new (unrelated) stream can be processed.
     * @param st Resampler state
     */
    pub(crate) fn reset_mem(&mut self) {
        self.last_sample.iter_mut().for_each(|elem| *elem = 0);
        self.magic_samples.iter_mut().for_each(|elem| *elem = 0);
        self.samp_frac_num.iter_mut().for_each(|elem| *elem = 0);

        self.mem.iter_mut().for_each(|elem| *elem = 0.);
    }

    #[inline]
    fn num_den(&mut self) {
        self.cutoff = QUALITY_MAPPING.downsample_bandwidth
            * self.den_rate as f32
            / self.num_rate as f32;
        let pass = self.filt_len;
        _muldiv(&mut self.filt_len, pass, self.num_rate, self.den_rate);
        self.filt_len = ((self.filt_len - 1) & (!7)) + 8;
        self.oversample = (1..5)
            .filter(|x| 2u32.pow(*x) * self.den_rate < self.num_rate)
            .fold(self.oversample, |acc, _| acc >> 1);

        if self.oversample < 1 {
            self.oversample = 1;
        }
    }

    #[inline]
    fn use_direct(&mut self) {
        let iter_chunk = self.sinc_table.chunks_mut(self.filt_len as usize);
        for (i, chunk) in iter_chunk.enumerate() {
            for (j, elem) in chunk.iter_mut().enumerate() {
                *elem = sinc(
                    self.cutoff,
                    (j as f32 - self.filt_len as f32 / 2.0 + 1.0)
                        - (i as f32) / self.den_rate as f32,
                    self.filt_len as i32,
                    QUALITY_MAPPING.window_func,
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
                    QUALITY_MAPPING.window_func,
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
            for magic in &mut self.magic_samples {
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

    fn update_filter(&mut self) -> usize {
        let old_length = self.filt_len;
        let old_alloc_size = self.mem_alloc_size as usize;
        self.int_advance = self.num_rate / self.den_rate;
        self.frac_advance = self.num_rate % self.den_rate;
        self.oversample = QUALITY_MAPPING.oversample as u32;
        self.filt_len = QUALITY_MAPPING.base_length as u32;
        if self.num_rate > self.den_rate {
            self.num_den();
        } else {
            self.cutoff = QUALITY_MAPPING.upsample_bandwidth;
        }

        let use_direct = self.filt_len * self.den_rate
            <= self.filt_len * self.oversample + 8
            && 2147483647 as u64
                / ::std::mem::size_of::<f32>() as u64
                / self.den_rate as u64
                >= self.filt_len as u64;

        let min_sinc_table_length = if !use_direct {
            self.filt_len * self.oversample + 8
        } else {
            self.filt_len * self.den_rate
        };

        if self.sinc_table_length < min_sinc_table_length {
            self.sinc_table = vec![0.0; min_sinc_table_length as usize];
            self.sinc_table_length = min_sinc_table_length;
        }

        if use_direct {
            self.use_direct();
        } else {
            self.not_use_direct();
        }

        let min_alloc_size = self.filt_len - 1 + self.buffer_size;
        if min_alloc_size > self.mem_alloc_size {
            let mem = self.mem.clone();
            self.mem = vec![0.0; (self.nb_channels * min_alloc_size) as usize];
            self.mem[0..mem.len()].copy_from_slice(&mem);
            self.mem_alloc_size = min_alloc_size;
        }

        if self.started == 0 {
            let dim = (self.nb_channels * self.mem_alloc_size) as usize;
            self.mem = vec![0.0; dim];
        } else if self.filt_len > old_length {
            self.chunks_iterator(old_length, old_alloc_size, 0);
            self.chunks_iterator(old_length, self.mem_alloc_size as usize, 1);
        } else if self.filt_len < old_length {
            self.chunks_iterator(old_length, self.mem_alloc_size as usize, 2);
        }
        RESAMPLER_ERR_SUCCESS
    }
}

fn resampler_basic_zero(
    st: &mut ResamplerState,
    channel_index: u32,
    _in_0: &[f32],
    in_len: &mut u32,
    out: &mut [f32],
    out_len: &mut u32,
) -> i32 {
    let mut out_sample: u32 = 0;
    let mut last_sample = st.last_sample[channel_index as usize];
    let mut samp_frac_num = st.samp_frac_num[channel_index as usize];
    let out_stride = st.out_stride;
    let int_advance = st.int_advance;
    let frac_advance = st.frac_advance;
    let den_rate: u32 = st.den_rate;
    while !(last_sample >= *in_len || out_sample >= *out_len) {
        out[(out_stride * out_sample) as usize] = 0.0;
        out_sample += 1;
        last_sample += int_advance;
        samp_frac_num += frac_advance as u32;
        if samp_frac_num >= den_rate {
            samp_frac_num -= den_rate as u32;
            last_sample += 1
        }
    }
    st.last_sample[channel_index as usize] = last_sample;
    st.samp_frac_num[channel_index as usize] = samp_frac_num;
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
    out_stride: usize,
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
    out_slice[(out_stride * out_sample) as usize] = interp
        .iter()
        .zip(accum.iter())
        .map(|(&x, &y)| x * y)
        .fold(0., |acc, x| acc + x);
}

#[inline(always)]
fn direct_step(
    in_slice: &[f32],
    out_slice: &mut [f32],
    out_stride: usize,
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
    out_slice[(out_stride * out_sample) as usize] = sum;
}

fn resampler_basic_interpolate(
    st: &mut ResamplerState,
    channel_index: u32,
    in_0: &[f32],
    in_len: &mut u32,
    out: &mut [f32],
    out_len: &mut u32,
) -> i32 {
    let n = st.filt_len as usize;
    let channel_idx = channel_index as usize;
    let mut last_sample = st.last_sample[channel_idx];
    let mut samp_frac_num = st.samp_frac_num[channel_idx];
    let out_stride = st.out_stride;
    let int_advance = st.int_advance;
    let frac_advance = st.frac_advance;
    let den_rate = st.den_rate;
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
            out_stride as usize,
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
    st.last_sample[channel_idx] = last_sample;
    st.samp_frac_num[channel_idx] = samp_frac_num;
    out_sample as i32
}

static QUALITY_MAPPING: QualityMapping
    = QualityMapping::new(160, 16, 0.96, 0.96, &_KAISER10);

static _KAISER10: FuncDef = FuncDef::new(&KAISER10_TABLE, 32);

static KAISER10_TABLE: [f64; 36] = {
    [
        0.99537781, 1.0, 0.99537781, 0.98162644, 0.95908712, 0.92831446,
        0.89005583, 0.84522401, 0.79486424, 0.74011713, 0.68217934,
        0.62226347, 0.56155915, 0.5011968, 0.44221549, 0.38553619, 0.33194107,
        0.28205962, 0.23636152, 0.19515633, 0.15859932, 0.1267028, 0.09935205,
        0.07632451, 0.05731132, 0.0419398, 0.02979584, 0.0204451, 0.01345224,
        0.00839739, 0.00488951, 0.00257636, 0.00115101, 0.00035515, 0.0, 0.0,
    ]
};

fn sinc(cutoff: f32, x: f32, n: i32, window_func: &FuncDef) -> f32 {
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
        let second_factor = compute_func(
            (2.0 * f64::from(x) / n_64).abs() as f32,
            window_func,
        );
        (first_factor * second_factor) as f32
    }
}

fn compute_func(x: f32, func: &FuncDef) -> f64 {
    let mut interp: [f64; 4] = [0.0; 4];
    let y = x * func.oversample as f32;
    let ind = y.floor() as usize;
    let frac = f64::from(y - ind as f32);
    interp[3] = -0.1666666667 * frac + 0.1666666667 * frac.powi(3);
    interp[2] = frac + 0.5 * frac.powi(2) - 0.5 * frac.powi(3);
    interp[0] = -0.3333333333 * frac + 0.5 * frac.powi(2)
        - 0.1666666667 * frac.powi(3);

    interp[1] = 1.0 - interp[3] - interp[2] - interp[0];

    interp
        .iter()
        .zip(func.table.iter().skip(ind))
        .map(|(&x, &y)| x * y)
        .sum()
}

fn resampler_basic_direct(
    st: &mut ResamplerState,
    channel_index: u32,
    in_0: &[f32],
    in_len: &mut u32,
    out: &mut [f32],
    out_len: &mut u32,
) -> i32 {
    let n = st.filt_len as usize;
    let mut out_sample: u32 = 0;
    let mut last_sample = st.last_sample[channel_index as usize];
    let mut samp_frac_num = st.samp_frac_num[channel_index as usize];
    let out_stride = st.out_stride;
    let int_advance = st.int_advance;
    let frac_advance = st.frac_advance;
    let den_rate: u32 = st.den_rate;
    while !(last_sample >= *in_len || out_sample >= *out_len) {
        let sinct: &[f32] =
            &st.sinc_table[(samp_frac_num * n as u32) as usize..];
        let iptr: &[f32] = &in_0[last_sample as usize..];

        direct_step(
            iptr,
            out,
            out_stride as usize,
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
    st.last_sample[channel_index as usize] = last_sample;
    st.samp_frac_num[channel_index as usize] = samp_frac_num;
    out_sample as i32
}

fn _muldiv(result: &mut u32, value: u32, mul: u32, div: u32) -> usize {
    let major: u32 = value / div;
    let remainder: u32 = value % div;
    if remainder > 4294967295 / mul
        || major > 4294967295 / mul
        || major * mul > 4294967295 - remainder * mul / div
    {
        RESAMPLER_ERR_OVERFLOW
    } else {
        *result = remainder * mul / div + major * mul;
        RESAMPLER_ERR_SUCCESS
    }
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let temp = a;
        a = b;
        b = temp % b;
    }
    a
}

fn speex_resampler_process_native(
    st: &mut ResamplerState,
    channel_index: u32,
    in_len: &mut u32,
    out: &mut [f32],
    out_len: &mut u32,
) -> usize {
    let n: usize = st.filt_len as usize;
    let mem_idx = (channel_index * st.mem_alloc_size) as usize;
    st.started = 1;
    let mem = &st.mem.clone();
    let out_sample: i32 = st.resampler_ptr.expect("non-null function pointer")(
        st,
        channel_index,
        mem,
        in_len,
        out,
        out_len,
    );
    if st.last_sample[channel_index as usize] < *in_len {
        *in_len = st.last_sample[channel_index as usize] as u32;
    }
    *out_len = out_sample as u32;
    st.last_sample[channel_index as usize] -= *in_len;
    let ilen = *in_len as usize;

    st.mem[mem_idx..(mem_idx + n - 1)]
        .copy_from_slice(&mem[(mem_idx + ilen)..(mem_idx + ilen + n - 1)]);

    RESAMPLER_ERR_SUCCESS
}

fn speex_resampler_magic<'a, 'b>(
    st: &mut ResamplerState,
    channel_index: u32,
    out: &'a mut &'b mut [f32],
    mut out_len: u32,
) -> u32 {
    let channel_idx = channel_index as usize;
    let mut tmp_in_len = st.magic_samples[channel_idx];
    let mem_idx = (st.filt_len + channel_index * st.mem_alloc_size) as usize;
    speex_resampler_process_native(
        st,
        channel_index,
        &mut tmp_in_len,
        *out,
        &mut out_len,
    );
    st.magic_samples[channel_idx] -= tmp_in_len;
    if st.magic_samples[channel_idx] != 0 {
        let mem = &st.mem[mem_idx - 1 + tmp_in_len as usize..].to_vec();
        st.mem
            .iter_mut()
            .skip(mem_idx - 1)
            .zip(mem.iter())
            .take(st.magic_samples[channel_idx] as usize)
            .for_each(|(x, &y)| *x = y);
    }
    let value: &'b mut [f32] = mem::replace(out, &mut []);
    *out = &mut value[(out_len * st.out_stride as u32) as usize..];
    out_len
}
