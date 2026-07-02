.PHONY: all clean appimage

all: appimage

appimage:
	./build-appimage.sh

appimage-tool:
	ARCH=x86_64 appimagetool AppDir waylyrics-$(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')-x86_64.AppImage

clean:
	rm -rf AppDir *.AppImage target
