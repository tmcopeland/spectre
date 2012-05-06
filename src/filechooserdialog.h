/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
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

