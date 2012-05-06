/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
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

