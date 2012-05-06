AC_INIT([spectre],[0.1])
AC_CONFIG_MACRO_DIR([m4])
AC_CONFIG_HEADERS([config.h])
AM_INIT_AUTOMAKE([1.11])
AM_SILENT_RULES([yes])
AC_PROG_CXX

AC_HEADER_ASSERT

dnl *************************************************************************
dnl Internationalization
dnl *************************************************************************
IT_PROG_INTLTOOL([0.35.0])

GETTEXT_PACKAGE=spectre
AC_SUBST(GETTEXT_PACKAGE)
AC_DEFINE_UNQUOTED(GETTEXT_PACKAGE, "$GETTEXT_PACKAGE", [GETTEXT package name])
AM_GLIB_GNU_GETTEXT

LT_INIT

PKG_CHECK_MODULES(GTK, [gtkmm-3.0])
PKG_CHECK_MODULES(LIBAV, [libavcodec libavformat libavutil])
PKG_CHECK_MODULES(YAML, [yaml-cpp])

AC_CONFIG_FILES([
Makefile
data/Makefile
data/spectre.desktop.in
data/config/Makefile
data/ui/Makefile
po/Makefile.in
src/Makefile
])
AC_OUTPUT
