/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
#ifndef __SPECTRE_SPECTROGRAM_H__
#define __SPECTRE_SPECTROGRAM_H__

#include <gtkmm/

namespace Spectre
{

class Spectrogram : public Gdk::DrawingArea
{
public:
    Spectrogram(void);
    virtual ~Spectrogram(void);
    
    Glib::ustring   get_file_name(void);
protected:
    Glib::ustring   file_name;
private:
};

}

#endif /* __SPECTRE_SPECTROGRAM_H__ */

