/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
#ifndef __SPECTRE_FFT_PLAN_H__
#define __SPECTRE_FFT_PLAN_H__

extern "C"
{
#include <glib.h>
#include <libavcodec/avfft.h>
}

namespace Spectre
{

class FftPlan
{
public:
    FftPlan(int n, int threshold);
    
    virtual ~FftPlan(void);
    void execute(void);
protected:
    int n;
    int threshold;
    
    float* input;
    float* output;
    
    RDFTContext* ctx;
};

}

#endif /* __SPECTRE_FFT_PLAN_H__ */

