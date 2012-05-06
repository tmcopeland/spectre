/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
#ifndef __SPECTRE_DATA_H__
#define __SPECTRE_DATA_H__

#ifdef SPECTRE_WINDOW_UI_FILE
#undef SPECTRE_WINDOW_UI_FILE
#define SPECTRE_WINDOW_UI_FILE PACKAGE_DATA_DIR"/spectre/ui/spectre-window.ui"
#else
#define SPECTRE_WINDOW_UI_FILE PACKAGE_DATA_DIR"/spectre/ui/spectre-window.ui"
#endif

#ifdef SPECTRE_LANGUAGES_YAML_FILE
#undef SPECTRE_LANGAUGES_YAML_FILE
#define SPECTRE_LANGUAGES_YAML_FILE PACKAGE_DATA_DIR"/spectre/config/languages.yaml"
#else
#define SPECTRE_LANGUAGES_YAML_FILE PACKAGE_DATA_DIR"/spectre/config/languages.yaml"
#endif

#include <vector>

#include <glibmm/ustring.h>

namespace Spectre
{

namespace Data
{

typedef struct _FileFormat  FileFormat;
typedef struct _Language    Language;

struct _FileFormat
{
    std::vector< Glib::ustring >    extension;
    std::vector< Glib::ustring >    mime;
    Glib::ustring                   name;
};

struct _Language
{
    Glib::ustring code;
    Glib::ustring name;
};

FileFormat  file_format_new             (const Glib::ustring& name,
                                         std::vector< Glib::ustring > extension,
                                         std::vector< Glib::ustring > mime);
FileFormat  file_format_new_from_arrays (const Glib::ustring& name, Glib::ustring extension[],
                                         Glib::ustring mime[]);
Language    language_new                (Glib::ustring& name, Glib::ustring& code);

static std::vector< FileFormat >    get_all_file_formats    (void);
static std::vector< Language >      get_all_languages       (void);
static std::vector< Language >      get_all_languages_yaml  (void);

}

}

#endif /* __SPECTRE_DATA_H__ */

