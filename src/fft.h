/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
#ifndef __SPECTRE_FFT_PLAN_H__
#define __SPECTRE_FFT_PLAN_H__

namespace Spectre
{

class FftPlan
{
public:
    Fft(int n, int threshold);
    virtual ~Fft(void);
    
    void execute(void);
public:
    int n;
    int threshold;
    float* input;
    float* output;
    RDFTContext* ctx;
};

}

#endif /* __SPECTRE_FFT_PLAN_H__ */

