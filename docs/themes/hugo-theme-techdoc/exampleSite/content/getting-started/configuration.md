---
title: "Configuration"
date: 2017-10-17T15:26:15Z
lastmod: 2019-10-26T15:26:15Z
draft: false
weight: 20
---

You may specify options in config.toml (or config.yaml/config.json) of your site to make use of this theme’s features.

For an example of `config.toml`, see [config.toml](https://github.com/thingsym/hugo-theme-techdoc/blob/master/exampleSite/config.toml) in exampleSite.

## Params

    # Source Code repository section
    description = "put your description"
    github_repository = "https://github.com/thingsym/hugo-theme-techdoc"
    version = "0.9.9"

    # Documentation repository section
    # documentation repository (set edit link to documentation repository)
    github_doc_repository = "https://github.com/thingsym/hugo-theme-techdoc"
    github_doc_repository_path = ""

    # Analytic section
    google_analytics_id = "" # Your Google Analytics tracking id
    tag_manager_container_id = "" # Your Google Tag Manager container id
    google_site_verification = "" # Your Google Site Verification for Search Console

    # Open Graph and Twitter Cards settings section
    # Open Graph settings for each page are set on the front matter.
    # See https://gohugo.io/templates/internal/#open-graph
    # See https://gohugo.io/templates/internal/#twitter-cards
    title = "Hugo Techdoc Theme"
    images = ["images/og-image.png"] # Open graph images are placed in `static/images`

    # Theme settings section
    # Theme color
    # See color value reference https://developer.mozilla.org/en-US/docs/Web/CSS/color
    custom_font_color = ""
    custom_background_color = ""

    # Documentation Menu section
    # Menu style settings
    menu_style = "open-menu" # "open-menu" or "slide-menu" or "" blank is as no sidebar

    # Date format
    dateformat = "" # default "2 Jan 2006"
    # See the format reference https://gohugo.io/functions/format/#hugo-date-and-time-templating-reference

    # path name excluded from documentation menu
    menu_exclusion = [
        "archives",
        "archive",
        "blog",
        "entry",
        "post",
        "posts",
    ]

    # Algolia site search section
    # See https://www.algolia.com/doc/
    algolia_search_enable = true
    algolia_indexName = "hugo-demo-techdoc"
    algolia_appId = "7W4SAN4PLK"
    algolia_apiKey = "cbf12a63ff72d9c5dc0c10c195cf9128" # Search-Only API Key

#### `description`

The document summary

default: `put your description`

#### `github_repository`

URL of souce code repository

default: `https://github.com/thingsym/hugo-theme-techdoc`

#### `version`

The version of souce code

default: `0.9.9`

#### `github_doc_repository`

URL of documentation repository for editting

default: `https://github.com/thingsym/hugo-theme-techdoc`

#### `github_doc_repository_path`

content directory path (when including the content directory in the repository)

default: `""`

#### `google_analytics_id`

ID of Google Analytics

default: `""`

#### `tag_manager_container_id`

Container ID of Google Tag Manager

default: `""`

#### `google_site_verification`

Content value in meta tag `google-site-verification` for Google Search Console

```
<meta name="google-site-verification" content="e7-viorjjfiihHIoowh8KLiowhbs" />
```

default: `""`

#### `title`

default open graph title for open graph

default: `"Hugo Techdoc Theme"`

#### `images`

default open graph image for open graph

Open graph images are placed in `static/images`.

default: `["images/og-image.png"]`

#### `custom_font_color`

Header font color

See color value reference https://developer.mozilla.org/en-US/docs/Web/CSS/color


default: `""`

#### `custom_background_color`

Header background color

See color value reference https://developer.mozilla.org/en-US/docs/Web/CSS/color

default: `""`

#### `menu_style`

Documentation Menu style, Open Menu or Slide Menu

default: `open-menu`  
value: `open-menu` | `slide-menu`

#### `dateformat`

default: `""` as `2 Jan 2006`

#### `menu_exclusion`

Path name excluded from documentation menu

By default, we exclude commonly used folder names in blogs.

default: `[
        "archives",
        "archive",
        "blog",
        "entry",
        "post",
        "posts"
    ]`


#### `algolia_search_enable`

Enable Algolia search

default: `true`

value: `true` | `false`

#### `algolia_indexName`

Algolia index name

default: `hugo-demo-techdoc`

#### `algolia_appId`

Application id

default: `7W4SAN4PLK`

#### `algolia_apiKey`

Search-Only API Key

default: `cbf12a63ff72d9c5dc0c10c195cf9128`
