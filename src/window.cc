/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
#include "window.h"

#include <iostream>

#include <glibmm/error.h>
#include <gtkmm/action.h>
#include <gtkmm/menubar.h>
#include <gtkmm/stock.h>
#include <gtkmm/toolbar.h>

#include <glibmm/i18n.h>

using namespace Spectre;

Window::Window(Glib::ustring file_name) :
  Gtk::Window(Gtk::WINDOW_TOPLEVEL),
  m_box(Gtk::ORIENTATION_VERTICAL)
{
    set_title(_("Spectre"));
    set_default_icon_name("Spectre");
    set_default_size(640, 480);
    
    actions = Gtk::ActionGroup::create("Spectre");
    
#ifdef GETTEXT_PACKAGE
    actions->set_translation_domain(GETTEXT_PACKAGE);
#endif /* GETTEXT_PACKAGE */
    
    actions->add(Gtk::Action::create("File", Gtk::Stock::FILE));
    
    actions->add(Gtk::Action::create("FileOpen", Gtk::Stock::OPEN),
                 sigc::mem_fun(*this, &Window::on_file_open));
    
    actions->add(Gtk::Action::create("FileSave", Gtk::Stock::SAVE),
                 sigc::mem_fun(*this, &Window::on_file_save));
    
    actions->add(Gtk::Action::create("FileQuit", Gtk::Stock::QUIT),
                 sigc::mem_fun(*this, &Window::on_file_quit));
    
    actions->add(Gtk::Action::create("Edit", Gtk::Stock::EDIT));
    
    actions->add(Gtk::Action::create("EditPreferences", Gtk::Stock::PREFERENCES),
                 sigc::mem_fun(*this, &Window::on_edit_preferences));
    
    actions->add(Gtk::Action::create("Help", Gtk::Stock::HELP));
    
    actions->add(Gtk::Action::create("HelpAbout", Gtk::Stock::ABOUT),
                 sigc::mem_fun(*this, &Window::on_help_about));
    
    ui = Gtk::UIManager::create();
    ui->insert_action_group(actions, 0);
    
    add_accel_group(ui->get_accel_group());
    
    try
    {
        ui->add_ui_from_file(PACKAGE_DATA_DIR "/spectre/ui/spectre-window.ui");
    }
    catch (Glib::Error e)
    {
        std::cerr << "Could not load the UI." << std::endl;
    }
    
    Gtk::Widget* menubar = ui->get_widget("/MenuBar");
    Gtk::Widget* toolbar = (Gtk::Toolbar*) ui->get_widget("/ToolBar");
    
    ((Gtk::Toolbar*) toolbar)->set_toolbar_style(Gtk::TOOLBAR_BOTH_HORIZ);
    
    ((Gtk::ToolItem*) ui->get_widget("/ToolBar/FileOpen"))->set_is_important(TRUE);
    ((Gtk::ToolItem*) ui->get_widget("/ToolBar/FileSave"))->set_is_important(TRUE);
    ((Gtk::ToolItem*) ui->get_widget("/ToolBar/HelpAbout"))->set_is_important(TRUE);
    
    m_box.pack_start(*menubar, FALSE, TRUE, 0);
    m_box.pack_start(*toolbar, FALSE, TRUE, 0);
    
    menubar->show_all();
    toolbar->show_all();
    
    m_box.show();
    
    add(m_box);
    
    //targets.push_back(*(new Gtk::TargetEntry("text/uri-list")));
    
    //drag_dest_set(targets);
    //signal_drag_data_received().connect(sigc::mem_fun(*this, &Window::on_dropped));
}

void
Window::on_dropped(Gdk::DragContext ctx, int x, int y,
                   Gtk::SelectionData sel, int info, int time)
{
}

void
Window::on_edit_preferences(void)
{
}

void
Window::on_file_open(void)
{
}

void
Window::on_file_quit(void)
{
    hide();
}

void
Window::on_file_save(void)
{
}

void
Window::on_help_about(void)
{
}

Window::~Window(void)
{
    actions.reset();
    ui.reset();
}

