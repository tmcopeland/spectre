/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
#include "fftplan.h"

#include <float.h>
#include <math.h>

//#include <glibmm/value.h>
#include <libavutil/mem.h>

using namespace Spectre;

FftPlan::FftPlan(int n, int threshold)
{
    int bits;
    
    input = av_mallocz((float) FLT_MAX * (float) n);
    output = av_mallocz(FLT_MAX * (n / 2 + 1));
    this->threshold = threshold;
    
    for (bits = 0; n; n >>=1, bits++);
    
    this->n = 1 << --bits;
    ctx = av_rdft_init(bits, DFT_R2C);
}

void
FftPlan::execute(void)
{
    int i;
    int n = this->n;
    
    av_rdft_calc(this->ctx, this->input);
    
    this->output[0] = this->input[0] * this->input[0] / (n * n);
    this->output[n / 2] = this->input[1] * this->input[1] / (n * n);
    
    for (i = 1; i < n / 2; i++)
    {
        float val;
        val = this->input[i * 2] * this->input[ i * 2] + this->input[i * 2 + 1] * this->input[i * 2 + 1];
        val /= n * n;
        val = 10.0F * log10f(val);
        
        this->output[i] = val < this->threshold ? this->threshold : val;
    }
}

FftPlan::~FftPlan(void)
{
    av_rdft_end(this->ctx);
    av_free(this->input);
    av_free(this->output);
    
    delete ctx;
    delete input;
    delete output;
    delete this;
}

