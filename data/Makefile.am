SUBDIRS = config ui

@INTLTOOL_DESKTOP_RULE@

desktopdir = $(datadir)/applications

desktop_DATA = spectre.desktop.in.in
nodist_desktop_DATA =\
	spectre.desktop.in\
	spectre.desktop

EXTRA_DIST =\
	$(desktop_DATA)\
	$(INTLTOOL)

DISTCLEANFILES =\
	$(nodist_desktop_DATA)

# vim: set noexpandtab:

