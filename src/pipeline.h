/* -*- Mode: C++; coding: utf-8; indent-tabs-mode: nil; c-basic-offset: 4; tab-width: 4 -*- */
#ifndef __SPECTRE_PIPELINE_H__
#define __SPECTRE_PIPELINE_H__

#include <sigc++/sigc++.h>

namespace Spectre
{

class Pipeline : public sigc::trackable
{
public:
    Pipeline(Glib::ustring);
    virtual ~Pipeline(void);
protected:
private:
};

}

#endif /* __SPECTRE_PIPELINE_H__ */

