ACLOCAL_AMFLAGS = -I m4

SUBDIRS = data po src

@INTLTOOL_DESKTOP_RULE@

spectredocdir= ${prefix}/doc/spectre
spectredoc_DATA =\
	AUTHORS\
	ChangeLog\
	COPYING\
	INSTALL\
	NEWS\
	README

INTLTOOL_FILES =\
	intltool-extract.in\
	intltool-merge.in\
	intltool-update.in

EXTRA_DIST =\
	$(spectredoc_DATA)\
	$(INTLTOOL_FILES)

DISCLEANFILES =\
	intltool-extract\
	intltool-merge\
	intltool-update\
	po/.intltool-merge-cache

uninstall-local:
	-rm -r $(spectredocdir)

# vim: set noexpandtab:

