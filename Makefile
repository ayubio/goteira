# Makefile para goteira

.PHONY: clean build build-rust-final

clean:
	snapcraft clean

# Snap oficial (Rust), definido em snap/snapcraft.yaml
build:
	snapcraft pack
	@mv *.snap goteira.snap 2>/dev/null || true

# Ultima revisao do snap legado "goteira-rust" (deprecado), montada em
# diretorio temporario a partir do HEAD para nao tocar em snap/snapcraft.yaml
build-rust-final:
	@tmp=$$(mktemp -d) && \
	git archive HEAD | tar -x -C $$tmp && \
	cp packaging/goteira-rust-final/snapcraft.yaml $$tmp/snap/snapcraft.yaml && \
	(cd $$tmp && snapcraft pack --use-lxd) && \
	mv $$tmp/*.snap ./goteira-rust_0.4.0_amd64.snap && rm -rf $$tmp
