quickstudy: $(shell find src -type f)
	cargo build --release && cp target/release/quickstudy .

quickstudy.AppImage: quickstudy icon.png quickstudy.desktop
	mkdir AppDir && cp icon.png quickstudy quickstudy.desktop AppDir
	echo -e "#!/usr/bin/env sh \n./quickstudy" > AppDir/AppRun && chmod +x AppDir/AppRun
	linuxdeploy -e quickstudy --appdir AppDir --plugin=ncurses --output=appimage
	mv quickstudy*.AppImage quickstudy.AppImage
	rm -rf AppDir

.PHONY: appimage
appimage: quickstudy.AppImage
