.PHONY: v0

v0:="definitions/v0"

v0:
	widl-template ${v0}/manifest.widl ${v0}/templates/rust/root.hbs -p ${v0}/templates/rust/partials |rustfmt > src/v0.rs

