PREFIX ?= /usr/local
MANDIR ?= $(PREFIX)/share/man

.SUFFIXES: .1 .1.scd

all: sw.1

install: all
	mkdir -p $(DESTDIR)$(MANDIR)/man1
	install -m644 sw.1 $(DESTDIR)$(MANDIR)/man1/sw.1

uninstall:
	rm -f $(DESTDIR)$(MANDIR)/man1/sw.1

clean:
	rm -vf sw.1

.PHONY: all clean install uninstall

.1.scd.1:
	scdoc < $< > $@
