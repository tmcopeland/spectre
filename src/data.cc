/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
/*
 * data.cc
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
#include "data.h"

#include <string>

//TODO: Implement YAML
#include <yaml-cpp/yaml.h>

using namespace Spectre::Data;

namespace YAML
{
    template<>
    struct convert< Glib::ustring >
    {
        static Node
        encode(const Glib::ustring& rhs)
        {
            return Node(rhs);
        }
        
        static bool
        decode(const Node& node, Glib::ustring& rhs)
        {
            if (!node.IsScalar())
            {
                return FALSE;
            }
            
            rhs = node.Scalar();
            
            return TRUE;
        }
    };
    
    template<>
    struct convert< FileFormat >
    {
        static Node
        encode(const FileFormat& rhs)
        {
            Node node;
            node.push_back(rhs.extension);
            node.push_back(rhs.mime);
            node.push_back(rhs.name);
            
            return node;
        }
        
        static bool
        decode(const Node& node, FileFormat& rhs)
        {
            if (!node.IsSequence())
            {
                return FALSE;
            }
            
            if (!node.size() == 3)
            {
                return FALSE;
            }
            
            rhs.extension = node[0].as< std::vector< Glib::ustring > >();
            rhs.mime = node[1].as< std::vector< Glib::ustring > >();
            rhs.name = node[2].as< Glib::ustring >();
            
            return TRUE;
        }
    };
    
    template<>
    struct convert< Language >
    {
        static Node
        encode(const Language& rhs)
        {
            Node node;
            node.push_back(rhs.name);
            node.push_back(rhs.code);
            
            return node;
        }
        
        static bool
        decode(const Node& node, Language& rhs)
        {
            if (!node.IsSequence())
            {
                return FALSE;
            }
            
            if (!node.size() == 2)
            {
                return FALSE;
            }
            
            rhs.name = (Glib::ustring) node[0].as< std::string >();
            rhs.code = (Glib::ustring) node[1].as< std::string >();
            
            return TRUE;
        }
    };
}

FileFormat
Spectre::Data::file_format_new(const Glib::ustring& name,
                               std::vector< Glib::ustring > extension,
                               std::vector< Glib::ustring > mime)
{
    FileFormat self;
    
    self.extension  = extension;
    self.mime       = mime;
    self.name       = name;
    
    return self;
}

/** @deprecated */
FileFormat
Spectre::Data::file_format_new_from_arrays(const Glib::ustring& name,
                                           Glib::ustring extension[],
                                           Glib::ustring mime[])
{
    int i; // For-loop iterator.
    FileFormat self;
    
    self.name = name;
    
    for (i = 0; i < ((sizeof(extension)) / (sizeof(*extension))); i++)
    {
        self.extension.push_back(extension[i]);
    }
    
    for (i = 0; i < ((sizeof(mime)) / (sizeof(*mime))); i++)
    {
        self.mime.push_back(mime[i]);
    }
    
    return self;
}

Language
language_new(const Glib::ustring& name, const Glib::ustring& code)
{
    Language self;
    self.name = name;
    self.code = code;
    
    return self;
}

static std::vector< FileFormat >
get_all_file_formats(void)
{
    std::vector< FileFormat > all_formats;
    
    Glib::ustring                   name;
    std::vector< Glib::ustring >    extension;
    std::vector< Glib::ustring >    mime;
    
    name = "Advanced Audio Coding File";
    
    extension.push_back("*.aac");
    extension.push_back("*.m4b");
    extension.push_back("*.m4p");
    extension.push_back("*.m4r");
    extension.push_back("*.mp4");
    extension.push_back("*.3gp");
    
    mime.push_back("audio/aac");
    mime.push_back("audio/aacp");
    mime.push_back("audio/3gpp");
    mime.push_back("audio/3gpp2");
    mime.push_back("audio/mp4");
    mime.push_back("audio/MP4A-LATM");
    mime.push_back("audio/mpeg4-generic");
    
    all_formats.push_back(file_format_new(name, extension, mime));
    
    return all_formats;
}

static std::vector< Language >
get_all_languages(void)
{
    std::vector< Language > all_languages;
    
    // The use of UTF-encoded characters is not an issue; the Glib::ustring
    // type handles all of the necessary encoding operations.
    all_languages.push_back(language_new(NULL, ""));
    all_languages.push_back(language_new("Čeština", "cs"));
    all_languages.push_back(language_new("Deutsch", "de"));
    all_languages.push_back(language_new("English", "en"));
    all_languages.push_back(language_new("Français", "fr"));
    all_languages.push_back(language_new("Italiano", "it"));
    all_languages.push_back(language_new("日本語", "ja"));
    all_languages.push_back(language_new("Nederlands", "nl"));
    all_languages.push_back(language_new("Polski", "pl"));
    all_languages.push_back(language_new("Português brasileiro", "pt_BR"));
    all_languages.push_back(language_new("Русский", "ru"));
    all_languages.push_back(language_new("Svenska", "sv"));
    all_languages.push_back(language_new("Українська", "uk"));
    
    return all_languages;
}

static std::vector< Language >
get_all_languages_yaml(void)
{
    std::vector< Language > all_languages;
    
    Language language;
    language.name = "";
    language.code = "";
    
    YAML::Node lang_file = YAML::LoadFile(PACKAGE_DATA_DIR "/spectre/config/languages.yaml");
    
    for (YAML::const_iterator it = lang_file.begin(); it != lang_file.end(); ++it)
    {
        language.name = (Glib::ustring) it->second["name"].as< std::string >();
        language.code = (Glib::ustring) it->second["code"].as< std::string >();
        
        all_languages.push_back(language);
    }  
    
    return all_languages;
}

