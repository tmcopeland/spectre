/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
/*
 * filechooserdialog.cc
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
#include "filechooserdialog.h"

#include <algorithm>
#include <glibmm/i18n.h>
#include <yaml-cpp/yaml.h>

#ifdef ENABLE_NLS
    #include <libintl.h>
#endif /* ENABLE_NLS */

#include <config.h>

using namespace Spectre;

FileChooserDialog::FileChooserDialog(Gtk::Window& parent,
                                     Gtk::FileChooserAction action,
                                     Glib::ustring directory=NULL) :
  Gtk::FileChooserDialog(parent, "", action)
{
    /* Creation of File Filters */
    filter_all = Gtk::FileFilter::create();
    filter_audio = Gtk::FileFilter::create();
    filter_png = Gtk::FileFilter::create();
    
    //TODO Explain.
    switch (action)
    {
    case Gtk::FILE_CHOOSER_ACTION_SAVE:
        set_title(_("Save Spectrogram"));
        
        break;
    case Gtk::FILE_CHOOSER_ACTION_OPEN:
    default:
        set_title(_("Open File"));
    }
}


FileChooserDialog::~FileChooserDialog(void)
{
    filter_all.reset();
    filter_audio.reset();
    filter_png.reset();
    
    delete this;
}
