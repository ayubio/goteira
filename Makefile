# Makefile para goteira

.PHONY: clean build

clean:
	snapcraft clean

# Snap oficial (Rust), definido em snap/snapcraft.yaml
build:
	snapcraft pack
	@mv *.snap goteira.snap 2>/dev/null || true
