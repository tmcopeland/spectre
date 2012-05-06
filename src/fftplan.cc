/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
#include "fftplan.h"

#include <math.h>

extern "C"
{
#include <libavutil/mem.h>
}

using namespace Spectre;

FftPlan::FftPlan(int n, int threshold)
{
    int bits;
    
    this->input = (float*) av_mallocz(sizeof(float) * n);
    this->output = (float*) av_mallocz(sizeof(float) * (n / 2 + 1));
    this->threshold = threshold;
    
    for (bits = 0; n; n >>= 1, bits++);
    
    this->n = 1 << --bits;
    this->ctx = av_rdft_init(bits, DFT_R2C);
}

void
FftPlan::execute(void)
{
    int i;
    int n = this->n;
    
    av_rdft_calc(this->ctx, this->input);
    
    /* Calculate Magnitudes */
    this->output[0] = pow((this->input[0]), 2) / pow(n, 2);
    this->output[n / 2] = pow((this->input[1]), 2) / pow(n, 2);
    
    for (i = 1; i < n; i++)
    {
        float val;
        val = 10.0 * log10f((pow((this->input[i * 2]), 2)
                            + pow((this->input[i * 2 + 1]), 2)) / pow(n, 2));
        
        this->output[i] = (val < this->threshold) ? this->threshold : val;
    }
}

FftPlan::~FftPlan(void)
{
    av_rdft_end(this->ctx);
    av_free(this->input);
    av_free(this->output);
    
    delete this->input;
    delete this->output;
    delete this;
}
