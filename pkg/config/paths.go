package config

import (
	"fmt"
	URL "net/url"
	"os"
	"path"
	"strings"

	"github.com/PuerkitoBio/purell"
)

func GetBase(base *string) string {
	if base == nil || *base == "" {
		baseUrl, err := os.Getwd()
		if err != nil {
			return ""
		}
		return baseUrl
	} else {
		return *base
	}
}

func NormalizeUrl(url string, base ...string) (string, error) {
	var normalUrl *URL.URL
	var err error
	var baseUrl string
	if len(base) > 0 {
		baseUrl = GetBase(&base[0])
	} else {
		baseUrl = GetBase(nil)
	}
	normalUrl, err = URL.Parse(url)

	if err != nil || normalUrl.Scheme == "" {
		if strings.HasPrefix(url, "/") {
			normalUrl, err = URL.Parse(fmt.Sprintf("file:///%s", url))
		} else {
			normalUrl, err = URL.Parse(fmt.Sprintf("file:///%s", path.Join(baseUrl, url)))
		}
	}
	if err != nil {
		return "", err
	}
	normal := purell.NormalizeURL(normalUrl, purell.FlagsUsuallySafeGreedy|purell.FlagRemoveDuplicateSlashes|purell.FlagRemoveFragment)
	return normal, err
}
