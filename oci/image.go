package oci

import (
	"regexp"
)

const AppMediaType = "application/vnd.nanobus.app"

var reImageReference = regexp.MustCompile(`(?m)^([a-zA-Z0-9\-\.]+)\/([a-zA-Z0-9\-\.]+)\/([a-zA-Z0-9\-\.]+):([a-zA-Z0-9\-\.]+)$`)

// IsImageReference tests if a string is an OCI image reference.
func IsImageReference(location string) bool {
	return reImageReference.MatchString(location)
}
