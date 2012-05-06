/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
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
