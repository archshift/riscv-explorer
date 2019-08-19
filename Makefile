PROFILE := release
TARGET := wasm32-unknown-unknown

.PHONY: all www target/$(TARGET)/$(PROFILE)/moesi.wasm

all: www

www: target/$(TARGET)/$(PROFILE)/moesi.wasm www/style.css
	wasm-bindgen ./target/$(TARGET)/$(PROFILE)/moesi.wasm --out-dir ./www/moesi --no-typescript --target web

target/$(TARGET)/$(PROFILE)/moesi.wasm:
	cargo build --target $(TARGET) --$(PROFILE) 

www/style.css: www/style.scss
	sass $< > $@
