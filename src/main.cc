/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
#include <gtkmm/application.h>

#include "window.h"

int
main(int argc, char* argv[])
{
    Glib::RefPtr< Gtk::Application > app = Gtk::Application::create(
            argc, argv, "com.taylorcopeland.spectre");
    
    Spectre::Window app_window ("");
    
    return app->run(app_window);
}

