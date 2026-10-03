# Makefile para goteira

.PHONY: clean build build-shell

clean:
	snapcraft clean

# Snap oficial (Rust), definido em snap/snapcraft.yaml
build:
	snapcraft pack
	@mv *.snap goteira.snap 2>/dev/null || true

# Snap legado (shell, deprecated): montado em diretório temporário para não
# sobrescrever snap/snapcraft.yaml
build-shell:
	@tmp=$$(mktemp -d) && \
	mkdir -p $$tmp/snap && \
	cp goteira.sh $$tmp/ && \
	cp snap/local/goteira-shell/snapcraft.yaml $$tmp/snap/snapcraft.yaml && \
	(cd $$tmp && snapcraft pack) && \
	mv $$tmp/*.snap ./goteira-shell.snap && rm -rf $$tmp
