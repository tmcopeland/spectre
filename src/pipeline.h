/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
/*
 * pipeline.h
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
#ifndef __SPECTRE_PIPELINE_H__
#define __SPECTRE_PIPELINE_H__

#include "audio.h"

#include <glibmm/threads.h>
#include <glibmm/ustring.h>

namespace Spectre
{

class Pipeline
{
public:
    Pipeline(const Glib::ustring& file_name, int bands, int samples, int threshold);
    virtual ~Pipeline(void);
public:
    void start(void);
    void stop(void);
    const Glib::ustring& get_description(void);
    double get_duration(void);
    int get_sample_rate(void);
protected:
    Audio::Context ctx;
    FftPlan fft;
private:
    Glib::ustring description;    
    
    int bands;
    int input_pos;
    int input_size;
    int nfft;//Size of FFT transform.
    const int NFFT;//Number of FFTs to pre-fetch.
    int sample_rate;
    int samples;
    int threshold;
    
    std::vector< float > coss;
    std::vector< float > input;
    std::vector< float > output;
    
    Glib::Threads::Thread reader_thread;
    Glib::Threads::Thread worker_thread;
    
    Glib::Threads::Cond reader_cond;
    Glib::Threads::Cond worker_cond;
    
    Glib::Threads::Mutex reader_mutex;
    Glib::Threads::Mutex worder_mutex;
    
    bool worder_done;
    bool quit;
};

}

#endif /* __SPECTRE_PIPELINE_H__ */

