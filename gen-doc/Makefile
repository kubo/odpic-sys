SOURCEDIR = ../odpic-sys/odpi/doc/src
DESTDIR = doc
DESTFILE = ../odpic-sys/doc.yaml
PYTHON = python

all: $(DESTFILE)

$(DESTFILE): doc make-odpic-doc.py
	$(PYTHON) make-odpic-doc.py > $(DESTFILE)

doc: $(SOURCEDIR)/*/*.rst
	@sphinx-build -M xml $(SOURCEDIR) doc
	touch doc

clean:
	rm -rf $(DESTFILE) doc
