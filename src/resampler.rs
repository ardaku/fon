// Fon
// Copyright © 2020-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
// - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
// - MIT License (https://mit-license.org/)
// - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use crate::{Stream, Sink, Frame};

// Precomputed co-efficients for the algorithm.
const COEFFICIENTS: &[f64] = &include!("coefficients.txt")[..];
const COEFFICIENTS_HALFLEN: usize = COEFFICIENTS.len() - 2;
const COEFFICIENTS_INDEX_INCREMENT: usize = 128;

// Resampler state.
pub struct Resampler {
    // Last 
    last_ratio: f64,
    //
    src_ratio: f64,
}

impl Resampler {
    /// Create a new resampler context.
    pub fn new() -> Self {
        Self {
            last_ratio: 0.0,
            src_ratio: 0.0,
        }
    }

    /// Stream audio data into sink, resampling as it goes.
    ///
    /// Do not call if stream's sample rate is None.
    pub(crate) fn stream<M, S, F, G>(&mut self, mut stream: M, mut sink: S)
        where M: Stream<F>, S: Sink<G>, F: Frame, G: Frame
    {
        // Calculate the sample rate ratio.
        let ratio = sink.sample_rate() / stream.sample_rate().unwrap();
        // Calculate the index step within the stream for each frame of sink.
        let step = stream.sample_rate().unwrap() / sink.sample_rate();
    
        // Set the input and output counts to zero.
        let mut input_frames_used = 0;

        // Special case for when last_ratio has not been set.
        if self.last_ratio == 0.0 {
            self.last_ratio = self.src_ratio;
        }

        // Now process.
        filter->in_count = data->input_frames * state->channels;
        let out_count = sink.buffer().len();
        filter->in_used = 0;

        let mut src_ratio = self.last_ratio;

        // Check the sample rate ratio wrt the buffer len.
        let count = {
            let mut v = COEFFICIENTS.len() as f64 / COEFFICIENTS_INDEX_INCREMENT as f64;
            if self.last_ratio.min(self.src_ratio) < 1.0 {
                v /= self.last_ratio.min(self.src_ratio);
            }
            v
        };

        // Maximum coefficients on either side of center point.
        let half_filter_chan_len = state->channels * (int) (count.round() + 1);

        let input_index = state->last_position;
        let rem = input_index % 1.0;

        filter->b_current = (filter->b_current + state->channels * (input_index - rem).round()) % filter->b_len;

        let mut input_index = rem;

        let terminate = step + 1e-20;

        // Main processing loop.
        for (i, sample) in sink.buffer().iter_mut().enumerate() {
            // Need to reload buffer?
            let mut samples_in_hand = (filter->b_end - filter->b_current + filter->b_len) % filter->b_len ;
            if samples_in_hand <= half_filter_chan_len {
                if (state->error = prepare_data (filter, state->channels, data, half_filter_chan_len)) != 0 {
                    return state->error;
                }
                samples_in_hand = (filter->b_end - filter->b_current + filter->b_len) % filter->b_len;
                if (samples_in_hand <= half_filter_chan_len) {
                    break;
                }
            }

            // This is the termination condition.
            if filter->b_real_end >= 0 {
                if filter->b_current + input_index + terminate > filter->b_real_end {
                    break;
                }
            }

            if out_count > 0 && fabs (state->last_ratio - ratio) > 1e-10 {
                src_ratio = state->last_ratio + i * (ratio - state->last_ratio) / out_count;
            }

            let float_increment: f64 = filter->index_inc * 
                if src_ratio < 1.0 { src_ratio } else { 1.0 }
            ;
            let increment = double_to_fp(float_increment);

            *sample = ((float_increment / filter->index_inc) *
                self.calc_output_single(
                    increment,
                    double_to_fp(input_index * float_increment)
                )
            ) as f64;

            // Figure out the next index.
            input_index += step;
            let rem = input_index % 1.0;

            filter->b_current = (filter->b_current + state->channels * (input_index - rem).round()) % filter->b_len ;
            input_index = rem;
        }

        state->last_position = input_index;

        /* Save current ratio rather then target ratio. */
        self.last_ratio = src_ratio;

        input_frames_used = filter->in_used / state->channels;
    }
    
    fn calc_output_single(&mut self, increment: i32, start_filter_index: i32) -> f64 {
        double        fraction, right, icoeff ;
        increment_t    filter_index, max_filter_index ;
        int            data_index, coeff_count, indx ;

        /* Convert input parameters into fixed point. */
        max_filter_index = int_to_fp (filter->coeff_half_len) ;

        /* First apply the left half of the filter. */
        filter_index = start_filter_index ;
        coeff_count = (max_filter_index - filter_index) / increment ;
        filter_index = filter_index + coeff_count * increment ;
        data_index = filter->b_current - coeff_count ;

        // Avoid underflow access to filter->buffer.
        if data_index < 0 {
            let steps = -data_index;
            filter_index -= increment * steps;
            data_index += steps;
        }
        let left: f64 = 0.0;
        while filter_index >= 0 {
            fraction = fixed_to_double (filter_index) ;
            indx = fp_to_int (filter_index) ;
            debug_assert!(indx >= 0 && indx + 1 < filter->coeff_half_len + 2) ;
            icoeff = filter->coeffs [indx] + fraction * (filter->coeffs [indx + 1] - filter->coeffs [indx]) ;
            debug_assert!(data_index >= 0 && data_index < filter->b_len) ;
            debug_assert!(data_index < filter->b_end) ;
            left += icoeff * filter->buffer [data_index] ;

            filter_index -= increment ;
            data_index = data_index + 1 ;
        }

        /* Now apply the right half of the filter. */
        filter_index = increment - start_filter_index ;
        coeff_count = (max_filter_index - filter_index) / increment ;
        filter_index = filter_index + coeff_count * increment ;
        data_index = filter->b_current + 1 + coeff_count ;

        right = 0.0 ;
        do
        {
            fraction = fixed_to_double (filter_index) ;
            indx = fp_to_int (filter_index) ;
            debug_assert!(indx < filter->coeff_half_len + 2);
            icoeff = filter->coeffs [indx] + fraction * (filter->coeffs [indx + 1] - filter->coeffs [indx]) ;
            debug_assert!(data_index >= 0 && data_index < filter->b_len);
            debug_assert!(data_index < filter->b_end);
            right += icoeff * filter->buffer[data_index];

            filter_index -= increment ;
            data_index = data_index - 1 ;
            }
        while (filter_index > 0);

        left + right
    }
    
    fn prepare_data(&mut self, int channels, SRC_DATA *data, int half_filter_chan_len) {
        let len = 0i32;

	    if filter->b_real_end >= 0 {
		    return;	/* Should be terminating. Just return. */
	    }

	    if (data->data_in == NULL) {
		    return;
	    }

	    let len = if filter->b_current == 0 {
		    /* Initial state. Set up zeros at the start of the buffer and
		    ** then load new data after that.
		    */
		    filter->b_current = filter->b_end = half_filter_chan_len;
		    filter->b_len - 2 * half_filter_chan_len
	    } else if filter->b_end + half_filter_chan_len + channels < filter->b_len {
	        /*  Load data at current end position. */
		    (filter->b_len - filter->b_current - half_filter_chan_len).max(0)
	    } else {
		    /* Move data at end of buffer back to the start of the buffer. */
		    len = filter->b_end - filter->b_current ;
		    memmove (filter->buffer, filter->buffer + filter->b_current - half_filter_chan_len,
						    (half_filter_chan_len + len) * sizeof (filter->buffer [0])) ;

		    filter->b_current = half_filter_chan_len ;
		    filter->b_end = filter->b_current + len ;

		    /* Now load data at current end of buffer. */
		    (filter->b_len - filter->b_current - half_filter_chan_len).max(0)
	    };

	    let len = (filter->in_count - filter->in_used).min(len) as i32;
	    let len = len - (len % channels);

	    if len < 0 || filter->b_end + len > filter->b_len {
		    return SRC_ERR_SINC_PREPARE_DATA_BAD_LEN;
	    }

	    memcpy (filter->buffer + filter->b_end, data->data_in + filter->in_used,
						    len * sizeof (filter->buffer [0]));

	    filter->b_end += len;
	    filter->in_used += len;

	    if (filter->in_used == filter->in_count
	        && filter->b_end - filter->b_current < 2 * half_filter_chan_len
            && data->end_of_input
        )
	    {
	        /* Handle the case where all data in the current buffer has been
		    ** consumed and this is the last buffer.
		    */

		    if (filter->b_len - filter->b_end < half_filter_chan_len + 5)
		    {	/* If necessary, move data down to the start of the buffer. */
			    len = filter->b_end - filter->b_current ;
			    memmove (filter->buffer, filter->buffer + filter->b_current - half_filter_chan_len,
							    (half_filter_chan_len + len) * sizeof (filter->buffer [0])) ;

			    filter->b_current = half_filter_chan_len ;
			    filter->b_end = filter->b_current + len ;
		    }

		    filter->b_real_end = filter->b_end ;
		    len = half_filter_chan_len + 5 ;

		    if (len < 0 || filter->b_end + len > filter->b_len)
			    len = filter->b_len - filter->b_end ;

		    memset (filter->buffer + filter->b_end, 0, len * sizeof (filter->buffer [0])) ;
		    filter->b_end += len;
	    }
    }
}

#[inline(always)]
fn fixed_to_double (x: i32) -> f64 {
    (x & ((1i32 << 12) - 1)) * (1.0 / (1i32 << 12) as f64)
}

#[inline(always)]
fn double_to_fp(x: f64) -> i32{
    (x * (1i32 << 12) as f64).round() as i32
}
