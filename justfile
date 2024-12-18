alias rp := release-pr
alias pt := push-tag

default:
	just --list --unsorted

show-toolchain:
	rustup -V
	rustc -V
	cargo -V
	cargo fmt --version
	cargo clippy -V

fmt:
	cargo fmt
	taplo fmt

lint: fmt
	cargo clippy --all-features

check:
	cargo fmt --check
	taplo fmt --check
	cargo clippy --all-features -- -D warnings

release-pr tag:
	git checkout -b "release-{{tag}}"
	cargo set-version {{tag}}
	git commit -am "chore(release): {{tag}}"
	git push --set-upstream origin release-{{tag}}

push-tag tag:
	git tag {{tag}}
	git push origin {{tag}}

run:
	cargo run -F _dev
