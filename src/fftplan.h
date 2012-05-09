/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
/*
 * fftplan.h
 * This file is part of Spectre
 *
 * Copyright (C) 2012 - Taylor Copeland <taylor@taylorcopeland.com>
 *
 * Spectre is free software: you can redistribute is and/or modify
 * it under the terms of the GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Spectre is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Spectre; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin St, Fifth Floor,
 * Boston, MA  02110-1301  USA or see <http://www.gnu.org/licenses/>.
 */
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

