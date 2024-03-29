/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
/*
 * window.h
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
#ifndef __SPECTRE_WINDOW_H__
#define __SPECTRE_WINDOW_H__

#include <vector>

#include <gdkmm/dragcontext.h>
#include <gtkmm/actiongroup.h>
#include <gtkmm/box.h>
#include <gtkmm/filefilter.h>
#include <gtkmm/selectiondata.h>
#include <gtkmm/targetentry.h>
#include <gtkmm/uimanager.h>
#include <gtkmm/window.h>

namespace Spectre
{

class Window : public Gtk::Window
{
public:
    //Constructor(s)
    Window(Glib::ustring file_name=Glib::ustring());
    
    // Destructor
    virtual ~Window (void);
protected:
    //Signal Handlers
    void on_dropped             (Gdk::DragContext ctx, int x, int y,
                                 Gtk::SelectionData sel, int info, int time);
    
    void on_edit_preferences    (void);
    void on_file_open           (void);
    void on_file_quit           (void);
    void on_file_save           (void);
    void on_help_about          (void);
private:
    //File Operations
    void open_file(Glib::ustring file_name);
protected:
    //Child Widgets
    Gtk::Box                            m_box;
protected:
    //Action Group
    Glib::RefPtr< Gtk::ActionGroup >    actions;
    //UI Manager
    Glib::RefPtr< Gtk::UIManager >      ui;
private:
    Glib::ustring                       current_directory;
    //Target Entry
    std::vector< Gtk::TargetEntry >     targets;
};

}

#endif /* __SPECTRE_WINDOW_H__ */

