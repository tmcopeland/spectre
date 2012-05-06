AM_CPPFLAGS =\
	-DPACKAGE_LOCALE_DIR=\""$(localdir)"\"\
	-DPACKAGE_SRC_DIR=\""$(srcdir)"\"\
	-DPACKAGE_DATA_DIR=\""$(datadir)"\"\
	-D__STDC_CONSTANT_MACROS\
	-I$(top_srcdir)\
	-I$(srcdir)\
	-I$(uidir)

spectredir = $(datadir)/spectre

bin_PROGRAMS = spectre

spectre_CXXFLAGS =\
	$(GTK_CFLAGS)\
	$(YAML_CFLAGS)

spectre_CFLAGS =\
	$(LIBAV_CFLAGS)

spectre_LDADD =\
	$(GTK_LIBS)\
	$(YAML_LIBS)\
	$(LIBAV_LIBS)

spectre_LDFLAGS =\
	-O3\
	-g\
	-Wall

spectre_SOURCES =\
	audio.h\
	audio.cc\
	data.h\
	data.cc\
	aboutdialog.h\
	aboutdialog.cc\
	fftplan.h\
	fftplan.cc\
	filechooserdialog.h\
	filechooserdialog.cc\
	window.h\
	window.cc\
	main.cc

clean-local:
	-rm -rf *.o

uninstall-local:
	-rm -r $(pkgdatadir)

# vim: set noexpandtab:

