/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
/*
 * filechooserdialog.h
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
#ifndef __SPECTRE_FILE_CHOOSER_DIALOG_H__
#define __SPECTRE_FILE_CHOOSER_DIALOG_H__

#include <gtkmm/filechooserdialog.h>
#include <gtkmm/filefilter.h>
#include <gtkmm/window.h>

namespace Spectre
{

class FileChooserDialog : public Gtk::FileChooserDialog
{
public:
    //Constructor(s)
    FileChooserDialog(Gtk::Window& parent, Gtk::FileChooserAction action,
                      Glib::ustring directory);
    
    //Destructor
    virtual ~FileChooserDialog(void);
private:
    //File Filters for Opening of Files
    Glib::RefPtr< Gtk::FileFilter > filter_all;
    Glib::RefPtr< Gtk::FileFilter > filter_audio;
    Glib::RefPtr< Gtk::FileFilter > filter_png;
};

}

#endif /* __SPECTRE_FILE_CHOOSER_DIALOG_H__ */

