.PHONY: css css-watch css-prod build dev clean

# Tailwind CSS compilation
css:
	./tailwindcss -i static/input.css -o static/style.css

css-watch:
	./tailwindcss -i static/input.css -o static/style.css --watch

css-prod:
	./tailwindcss -i static/input.css -o static/style.css --minify

# Rust build
build: css
	cargo build --release

dev: css
	cargo run

# Clean
clean:
	cargo clean
	rm -f static/style.css

