package errorz

import (
	"bufio"
	"fmt"
	"sort"
	"strings"
)

type TemplateError struct {
	Template string
	Metadata Metadata
}

func Return(template string, metadata ...Metadata) *TemplateError {
	var md Metadata
	l := len(metadata)

	if l == 1 {
		md = metadata[0]
	} else if l > 1 {
		md = make(Metadata)
		for _, m := range metadata {
			for k, v := range m {
				md[k] = v
			}
		}
	}

	return &TemplateError{
		Template: template,
		Metadata: md,
	}
}

func ParseTemplateError(message string) TemplateError {
	scanner := bufio.NewScanner(strings.NewReader(message))
	template := "unknown"
	var md Metadata

	if scanner.Scan() {
		template = scanner.Text()
	}
	for scanner.Scan() {
		line := scanner.Text()
		if !strings.HasPrefix(line, "[") {
			continue
		}
		line = line[1:]
		parts := strings.SplitN(line, "] ", 2)
		if len(parts) == 2 {
			if md == nil {
				md = make(Metadata)
			}
			md[parts[0]] = parts[1]
		}
	}

	return TemplateError{
		Template: template,
		Metadata: md,
	}
}

func (e *TemplateError) Error() string {
	var sb strings.Builder

	keys := make([]string, len(e.Metadata))
	i := 0
	for k := range e.Metadata {
		keys[i] = k
		i++
	}
	sort.Strings(keys)

	sb.WriteString(e.Template)
	for _, k := range keys {
		v := e.Metadata[k]
		sb.WriteString("\n[")
		sb.WriteString(k)
		sb.WriteString("] ")
		sb.WriteString(fmt.Sprintf("%v", v))
	}

	return sb.String()
}
