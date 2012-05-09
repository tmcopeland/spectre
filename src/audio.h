/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
/*
 * audio.h
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
#ifndef __SPECTRE_AUDIO_H__
#define __SPECTRE_AUDIO_H__

#include <glibmm/ustring.h>

extern "C"
{
#include <libavcodec/avcodec.h>
#include <libavformat/avformat.h>
}

namespace Spectre
{

namespace Audio
{

void init(void);

class Context
{
public:
    Context             (const Glib::ustring& file_name);
    virtual ~Context    (void);
public:
    void    start          (int samples);
    int     read           (void);
public:
    Glib::ustring file_name;
    Glib::ustring codec_name;
    Glib::ustring error;
    
    int bit_rate;
    int sample_rate;
    int bits_per_sample;
    int width;
    int channels;
    
    bool flt_pt;
    
    double duration;
    
    uint8_t* buffer;
    
    int64_t frames_per_interval;
    int64_t error_per_interval;
    int64_t error_base;
protected:
    Glib::ustring short_name;
    
    AVCodec*            codec;
    AVCodecContext*     codec_context;
    AVFormatContext*    format_context;
    AVPacket*           packet;
    AVStream*           stream;
    
    int audio_stream;
    int buffer_size;
    int offset;
};

}

}

#endif /* __SPECTRE_AUDIO_H__ */

