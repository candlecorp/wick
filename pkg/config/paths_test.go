package config

import (
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/assert"
)

var cwd, _ = os.Getwd()

func TestRelativeUrl(t *testing.T) {
	url, err := NormalizeUrl("./here.txt")
	assert.Nil(t, err)
	assert.Equal(t, fmt.Sprintf("file://%s/here.txt", cwd), url)
}

func TestUglyRelativeUrl(t *testing.T) {
	url, err := NormalizeUrl("./././foo/..////////here.txt")
	assert.Nil(t, err)
	assert.Equal(t, fmt.Sprintf("file://%s/here.txt", cwd), url)
}

func TestRelativeUrlWithRelativeBase(t *testing.T) {
	url, err := NormalizeUrl("./here.txt", "./dev/src")
	assert.Nil(t, err)
	assert.Equal(t, "file:///dev/src/here.txt", url)
}

func TestRelativeUrlWithAbsoluteBase(t *testing.T) {
	url, err := NormalizeUrl("./here.txt", "/dev/src")
	assert.Nil(t, err)
	assert.Equal(t, "file:///dev/src/here.txt", url)
}

func TestAbsoluteUrl(t *testing.T) {
	url, err := NormalizeUrl("/here.txt", "/dev/src")
	assert.Nil(t, err)
	assert.Equal(t, "file:///here.txt", url)
}

func TestHttpUrl(t *testing.T) {
	url, err := NormalizeUrl("http://this.com/that.txt", "/dev/src")
	assert.Nil(t, err)
	assert.Equal(t, "http://this.com/that.txt", url)
}

func TestUglyHttpUrl(t *testing.T) {
	url, err := NormalizeUrl("http://this.com/foo/../foo/../foo////////that.txt", "/dev/src")
	assert.Nil(t, err)
	assert.Equal(t, "http://this.com/foo/that.txt", url)
}
