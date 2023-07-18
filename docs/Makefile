PREFIX?=/usr/local
MANDIR?=$(PREFIX)/share/man

all: sw.1

sw.1: sw.1.scd
	scdoc < $< > $@

install: all
	mkdir -p $(DESTDIR)/$(MANDIR)/man1
	install -m644 sw.1 $(DESTDIR)/$(MANDIR)/man1/sw.1

uninstall:
	rm -f $(DESTDIR)/$(MANDIR)/man1/sw.1

clean:
	rm -vf sw.1

.PHONY: all clean install
