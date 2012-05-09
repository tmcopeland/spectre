/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
/*
 * spectrogram.h
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

