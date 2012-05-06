/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
#ifndef __SPECTRE_ABOUT_DIALOG_H__
#define __SPECTRE_ABOUT_DIALOG_H__

#include <gtkmm/aboutdialog.h>
#include <gtkmm/window.h>

namespace Spectre
{

class AboutDialog : public Gtk::AboutDialog
{
public:
    //Constructor(s)
    AboutDialog(void);
    AboutDialog(Gtk::Window parent);
    
    //Destructor
    virtual ~AboutDialog(void);
};

}

#endif /* __SPECTRE_ABOUT_DIALOG_H__ */

