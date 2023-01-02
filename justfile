VERSION := `git describe --tags $(git rev-list --tags --max-count=1)`
BINARY := "nanobus"
MAIN := "cmd/nanobus/main.go"

BUILDDIR := "build"
GITREV := `git rev-parse --short HEAD`
BUILDTIME := `date +'%FT%TZ%z'`
GO_BUILDER_VERSION := "latest"
GOPATH := `go env GOPATH`

all: build

deps:
	go get -u github.com/golangci/golangci-lint/cmd/golangci-lint
	go get -u github.com/git-chglog/git-chglog/cmd/git-chglog
	go get -u golang.org/x/tools/cmd/goimports

test:
  #!/usr/bin/env bash
  FILES=$(gofmt -l .); if [[ "$FILES" == "" ]]; then echo 'Formatting is OK'; else echo "The following files need to be formatting: \n$FILES"; exit 1; fi
  go test ./pkg/...

format:
  go fmt ./pkg/... ./cmd/...

build:
	CGO_ENABLED=0 go build -o {{BUILDDIR}}/{{BINARY}} {{MAIN}}
	@echo "Build {{BINARY}} complete."
	@echo "Run \"{{BUILDDIR}}/{{BINARY}}\" to start {{BINARY}}."

install:
	CGO_ENABLED=0 go install -buildvcs=false -ldflags="-X 'main.Version={{VERSION}}'" ./cmd/...
	@echo "Install {{BINARY}} complete. Make sure {{GOPATH}}/bin is in your path."

codegen:
  for file in `find . -name 'apex.yaml'`; do echo $(cd $(dirname $file); apex generate); done

clean:
	rm -rf {{BUILDDIR}}

changelog:
	git-chglog {{VERSION}} > CHANGELOG.md

release-dry-run:
	goreleaser --rm-dist --timeout=60m --skip-validate --skip-publish --snapshot

release: changelog
	goreleaser --rm-dist --timeout=60m --release-notes=CHANGELOG.md

lint:
	golangci-lint run --fix

build-linux-amd64:
	docker run \
		--rm \
		-v $(pwd):/go/src/github.com/nanobus/nanobus \
		-v /var/run/docker.sock:/var/run/docker.sock \
		-v {{GOPATH}}/src:/go/src \
		-v {{GOPATH}}/pkg:/go/pkg \
		-w /go/src/github.com/nanobus/nanobus \
		-e CGO_ENABLED=0 \
		golang:1.19.4 \
		go build -o dist/nanobus-linux_linux_amd64/nanobus {{MAIN}}

docker: release-dry-run
	docker build --platform linux/amd64 -f docker/Dockerfile-base -t nanobus/base .
	docker build --platform linux/amd64 -f docker/Dockerfile-java11 -t nanobus/java11 .
	docker build --platform linux/amd64 -f docker/Dockerfile-java17 -t nanobus/java17 .
	docker build --platform linux/amd64 -f docker/Dockerfile-nodejs:16 -t nanobus/nodejs:16 .
	docker build --platform linux/amd64 -f docker/Dockerfile-python3 -t nanobus/python3 .
