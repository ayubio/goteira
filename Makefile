# Makefile para goteira monorepo

clean:
	snapcraft clean

build-rust:
	@echo "Preparando build da versão RUST..."
	@cp snap/local/goteira-rust/snapcraft.yaml snap/snapcraft.yaml
	snapcraft pack
	@mv *.snap goteira-rust.snap 2>/dev/null || true
	@echo "Build Rust concluído."

build-shell:
	@echo "Preparando build da versão SHELL..."
	@cp snap/local/goteira-shell/snapcraft.yaml snap/snapcraft.yaml
	snapcraft pack
	@mv *.snap goteira-shell.snap 2>/dev/null || true
	@echo "Build Shell concluído."
