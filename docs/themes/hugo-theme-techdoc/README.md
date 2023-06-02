# Hugo Theme Techdoc

The Techdoc is a Hugo Theme for technical documentation.

![The Techdoc screenshot](https://raw.githubusercontent.com/thingsym/hugo-theme-techdoc/master/images/screenshot.png)

- Hugo Themes page: [https://themes.gohugo.io/themes/hugo-theme-techdoc/](https://themes.gohugo.io/themes/hugo-theme-techdoc/)
- Demo site: [https://thingsym.github.io/hugo-theme-techdoc/](https://thingsym.github.io/hugo-theme-techdoc/)
- Download: [https://github.com/thingsym/hugo-theme-techdoc](https://github.com/thingsym/hugo-theme-techdoc)

## Features

* Modern, Simple layout
* Responsive web design
* Documentation menu (Select Menu style)
* Table Of Contents for the page (selective)
* Theme color
* Edit link to documentation repository
* Header link to HTML headings
* Custom Shortcodes
	* Code highlight with clipboard
	* Alert panel
	* Button
* Search Shortcode powered by [Algolia](https://www.algolia.com/)
* Open Graph
* Analytics with Google Analytics, Google Tag Manager

## Screenshot

### Theme color

![Theme color](https://raw.githubusercontent.com/thingsym/hugo-theme-techdoc/master/images/screenshot-theme-color.png)

### Menu style

#### Open Menu

![Open Menu](https://raw.githubusercontent.com/thingsym/hugo-theme-techdoc/master/images/screenshot-open-menu.png)

#### Slide Menu

![Slide Menu](https://raw.githubusercontent.com/thingsym/hugo-theme-techdoc/master/images/screenshot-slide-menu.gif)

### Edit link

![Edit link](https://raw.githubusercontent.com/thingsym/hugo-theme-techdoc/master/images/screenshot-edit-link.png)

## Getting Started

### Requirement

Hugo minimum version: 0.60.0

Default Markdown parser library `Goldmark` compatible

### Install Hugo theme on your project

If you have git installed, you can include hugo-theme-techdoc repository into your core repository as submodule using `git submodule` within your project directory.

```
cd your_project
git submodule add https://github.com/thingsym/hugo-theme-techdoc.git themes/hugo-theme-techdoc
```

For more information read [the Hugo documentation](https://gohugo.io/getting-started/quick-start/).

### Or download Hugo theme on your project

If you have git installed, you can do the following at the command-line-interface within your project directory.

```
cd your_project/themes
git clone https://github.com/thingsym/hugo-theme-techdoc.git
```

### Configure

You may specify options in config.toml (or config.yaml/config.json) of your site to make use of this theme's features.

For an example of `config.toml`, [config.toml](https://github.com/thingsym/hugo-theme-techdoc/blob/master/exampleSite/config.toml) in exampleSite.

### Update Hugo theme for git submodule

```
git submodule update --remote
git add themes/hugo-theme-techdoc
git commit
```

### Directory layout

```
tree . -I node_modules

.
├── LICENSE.md
├── README.md
├── archetypes
│   └── default.md
├── docker-compose.yml
├── exampleSite
│   └── ..
├── gulpfile.js
├── images
│   └── ..
├── layouts
│   ├── 404.html
│   ├── blog
│   │   ├── li.html
│   │   ├── list.html
│   │   ├── single.html
│   │   └── summary.html
│   ├── _default
│   │   ├── baseof.html
│   │   ├── list.algolia.json
│   │   ├── list.html
│   │   └── single.html
│   ├── index.html
│   ├── partials
│   │   ├── content-footer.html
│   │   ├── custom-css.html
│   │   ├── custom-head.html
│   │   ├── edit-meta.html
│   │   ├── edit-page.html
│   │   ├── footer.html
│   │   ├── global-menu.html
│   │   ├── head.html
│   │   ├── last-updated.html
│   │   ├── menu
│   │   │   ├── open-menu.html
│   │   │   └── slide-menu.html
│   │   ├── meta
│   │   │   ├── chroma.html
│   │   │   ├── google-analytics-async.html
│   │   │   ├── google-site-verification.html
│   │   │   └── tag-manager.html
│   │   ├── notification.html
│   │   ├── pagination.html
│   │   ├── powered.html
│   │   ├── prepend-body.html
│   │   ├── search.html
│   │   ├── sidebar-footer.html
│   │   ├── sidebar.html
│   │   ├── site-header.html
│   │   └── table-of-contents.html
│   ├── posts
│   │   ├── list.html
│   │   └── single.html
│   ├── shortcodes
│       ├── button.html
│       ├── code.html
│       ├── panel.html
│       └── search.html
├── package-lock.json
├── package.json
├── resources
├── src
│   ├── js
│   │   ├── code.js
│   │   ├── headerlink.js
│   │   ├── jquery.backtothetop
│   │   │   ├── jquery.backtothetop.js
│   │   │   └── jquery.backtothetop.min.js
│   │   ├── keydown-nav.js
│   │   ├── main.js
│   │   └── sidebar-menu.js
│   └── scss
│       ├── _component.scss
│       ├── _project.scss
│       ├── _structure.scss
│       ├── _variable.scss
│       ├── chroma.scss
│       ├── foundation
│       │   ├── _element.scss
│       │   ├── _index.scss
│       │   ├── _normalize.scss
│       │   ├── _reset.scss
│       │   └── _stack.scss
│       ├── function
│       │   ├── _calc-font-size.scss
│       │   ├── _calc-stack.scss
│       │   ├── _contrast-color.scss
│       │   ├── _index.scss
│       │   └── _strip-unit.scss
│       └── theme.scss
├── static
│   ├── css
│   │   ├── chroma.css
│   │   ├── chroma.min.css
│   │   ├── theme.css
│   │   └── theme.min.css
│   ├── images
│   └── js
│       └── bundle.js
├── theme.toml
└── webpack.config.js
```

### Preview site

To preview your site, run Hugo's built-in local server.

```
hugo server -t hugo-theme-techdoc
```

Browse site on http://localhost:1313

## Deploy Site to public_html directory

```
hugo -t hugo-theme-techdoc -d public_html
```

## Local development environment

### Preview exampleSite

```
cd /path/to/dir/themes/hugo-theme-techdoc/exampleSite

hugo server --themesDir ../..
```

Browse site on http://localhost:1313

### Build development

```
cd /path/to/hugo-theme-techdoc
npm install
npm run gulp watch
```

## Docker development environment

### Run Docker and Preview exampleSite

```
cd /path/to/hugo-theme-techdoc
docker-compose up -d
```

Browse site on http://localhost:1313

### Build development

```
cd /path/to/hugo-theme-techdoc
docker-compose run --rm node npm install
docker-compose run --rm node npm run watch
```

## Contribution

### Patches and Bug Fixes

Small patches and bug reports can be submitted a issue tracker in Github. Forking on Github is another good way. You can send a pull request.

1. Fork [Hugo Theme Techdoc](https://github.com/thingsym/hugo-theme-techdoc) from GitHub repository
2. Create a feature branch: git checkout -b my-new-feature
3. Commit your changes: git commit -am 'Add some feature'
4. Push to the branch: git push origin my-new-feature
5. Create new Pull Request

## Changelog

* Version 0.9.9 - 2022.07.08
	* fix scss
	* bump up version on jquery, jquery.easing and clipboard
	* update node package dependencies
	* change service name
	* change fontawesome delivery from cdn to self-host
	* fix config.toml
	* id and class elements added to menu items
	* pre and post elements added to menu items [#48]
	* use SRI for CDN js sources [#45]
	* fix heading styles
	* change to using math.div for division

* Version 0.9.8 - 2021.10.18
	* fix sample document
	* edit README
	* update package.json
	* add github_doc_repository_path
	* change to relative link
	* add workflows for publishing demo site to gh-pages

* Version 0.9.7 - 2021.03.08
	* add docker-compose.yml for development environment
	* change keyboard event from event.keyCode to event.key because it is deprecated
	* update package.json
	* add FUNDING.yml
	* fix space for minify publish
	* change flexbox-grid-mixins from libsass to dart-sass
* Version 0.9.6 - 2020.11.22
	* add sample Markdown Syntax
	* update sample document
	* replace scss from node-sass to dart-sass
* Version 0.9.5 - 2020.11.05
	* fix link style with Alert panel
* Version 0.9.4 - 2020.10.08
	* improve scss for custom shortcodes using css custom properties
	* change stack to css custom properties
	* fix scss
	* fix button shortcode, adding notice color
	* add Code highlight with clipboard custom shortcode
	* fix webpack.config.js
	* fix npm scripts
	* update package.json
	* adjust no sidebar layout
* Version 0.9.3 - 2020.08.02
	* remove jQuery dependency with keydown nav
	* add header link
* Version 0.9.2 - 2020.06.14
	* add note and sample to document
	* fix tableOfContents endLevel
	* add chapter 'unlimited levels' to document
	* add menu indentation up to 5 levels
* Version 0.9.1 - 2020.05.24
	* fix config.toml
	* fix url in rss meta link
	* remove line break	in algolia.json
* Version 0.9.0 - 2020.04.01
	* fix lint config
	* update Sample Document
	* update jQuery to v3.4.1
	* add search function and shortcode powered by Algolia
	* fix config.toml
	* fix sass
	* fix hugo deprecated warning
* Version 0.8.3 - 2020.03.19
	* fix edit page link
* Version 0.8.2 - 2020.03.07
	* fix open graph image path
* Version 0.8.1 - 2020.03.07
	* fix open graph image path
* Version 0.8.0 - 2020.02.27
	* update Sample Document
	* config.toml
	* add open graph image to exampleSite
	* add safeCSS for ZgotmplZ with generated by Hugo Template
* Version 0.7.0 - 2020.02.07
	* bump up Hugo minimum version to 0.60.0
	* change shortcode delimiter from % to <
	* improve tableOfContents for Goldmark
* Version 0.6.0 - 2020.01.13
	* fix scss
	* gulp bump up version to 4.0
	* fix hugo deprecated warning
* Version 0.5.0 - 2019.12.08
	* update Sample Document
	* add table of contents
	* add open graph
	* add function and stack scss
* Version 0.4.0 - 2019.11.02
	* update Sample Document
	* add Theme color
	* add Menu style
	* improve scss
* Version 0.3.0 - 2019.10.06
	* fix archetypes
	* add prepend-body.html for Tag Manager noscript version
	* change class name from menu to global-menu
	* rename partials files
	* fix javascript path for webpack
	* improve development environment
	* move javascript files to src directory
* Version 0.2.2 - 2019.04.27
	* fix Lastmod's and PublishDate's initial value of 0001-01-01
* Version 0.2.1 - 2018.12.07
	* fix scss lint errors
	* change lint from scss-lint to stylelint
	* add published date
	* change the font color of powered by
	* fix link on powered by
* Version 0.2.0 - 2018.11.21
	* add screenshot images
	* add exampleSite
	* fix sub-menu for responsive
	* improve menu and pagination
* Version 0.1.0 - 2018.03.04
	* initial release

## License

Techdoc is licensed under the MIT License.

Techdoc bundles the following third-party resources:

* CSS reset by [normalize.css](https://necolas.github.io/normalize.css/), [MIT](https://opensource.org/licenses/MIT)
* jQuery Plugin [Back to the Top](https://github.com/thingsym/jquery.backtothetop), [MIT](https://opensource.org/licenses/MIT)
* Sass Mixin [Flexbox Grid Mixins](https://thingsym.github.io/flexbox-grid-mixins/), [MIT](https://opensource.org/licenses/MIT)
* [jQuery](https://jquery.com/)
* [jQuery Easing](https://github.com/gdsmith/jquery.easing)
* [Font Awesome](https://fontawesome.com/)
* [clipboard.js](https://clipboardjs.com/)
* [algoliasearch](https://github.com/algolia/algoliasearch-client-javascript)
* [Day.js](https://github.com/iamkun/dayjs)

## Author

[thingsym](https://github.com/thingsym)

Copyright (c) 2017-2020 by [thingsym](https://management.thingslabo.com/)
