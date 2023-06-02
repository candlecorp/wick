---
title: "Markdown Syntax"
date: 2020-11-17T15:26:15Z
draft: false
weight: 10
description: "calling custom Shortcodes into your content files."
---

## Headings

```markdown
# Heading level 1
## Heading level 2
### Heading level 3
#### Heading level 4
##### Heading level 5
###### Heading level 6
```

## Emphasis

```markdown
*Italic*  
**Bold**  
~~Strikethrough~~
```

## Horizontal Rule

```markdown
---
```

## Lists

### Unordered Lists

```markdown
- First item
- Second item
- Third item
- Fourth item
```

or

```markdown
* First item
* Second item
* Third item
* Fourth item
```

### Ordered Lists

```markdown
1. First item
2. Second item
3. Third item
4. Fourth item
```

## Code

````markdown
```ruby
puts 'The best way to log and share programmers knowledge.'
puts 'The best way to log and share programmers knowledge.'
```
````

## Inline code

```markdown
`#ffce44`
```

## Blockquote

```markdown
> this is a blockquote. this is a blockquote. this is a blockquote. this is a blockquote. this is a blockquote. this is a blockquote.
>
> this is a blockquote.
>
> this is a blockquote.
>
> this is a blockquote
```

## Links

```markdown
[Hugo Techdoc Theme demo](https://themes.gohugo.io/theme/hugo-theme-techdoc/)
```

## Table

```markdown
| header | header | header |
|------------|-------------|--------------|
| Lorem      | Lorem       | Lorem        |
| ipsum      | ipsum       | ipsum        |
| dolor      | dolor       | dolor        |
| sit        | sit         | sit          |
| amet       | amet        | amet         |
```

## Images

```markdown
![2 People Sitting With View of Yellow Flowers during Daytime](../images/pexels-photo-196666.jpeg "sample")
```


## Image with link

```markdown
[![2 People Sitting With View of Yellow Flowers during Daytime](../images/pexels-photo-196666.jpeg)](https://www.pexels.com/photo/2-people-sitting-with-view-of-yellow-flowers-during-daytime-196666/)
```

## Definition Lists

```markdown
First Term
: This is the definition.

Second Term
: This is the definition.
: This is the definition.
```

## Task Lists

```markdown
- [x] to do task 1
- [ ] to do task 2
- [ ] to do task 3
```

## Footnotes

```markdown
this is a footnote,[^1] and this is the second footnote.[^2]

[^1]: This is the first footnote.
[^2]: This is the second footnote.
```
