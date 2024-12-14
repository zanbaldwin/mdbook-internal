# mdbook-internal

An [mdbook](https://github.com/rust-lang-nursery/mdBook) preprocessor for
defining and (optionally) hiding internal sections and chapters, to be excluded
from a public build of your book.

> _Heavily inspired by [`mdbook-private`](). Think of this project as
> `mdbook-private` but with extra configuration options._ As such, the license
> remains [MDL-2.0](https://www.mozilla.org/en-US/MPL/2.0/).

## Installation
```sh
cargo install --git "https://github.com/zanbaldwin/mdbook-internal"
```

## Example
Using the default configuration, the following Markdown file:

```markdown
# Chapter 1
This sentence will be _public_.
<!--[internal] This sentence will be _internal_. [/internal]-->
```

Will result in the following:

```markdown
# Chapter 1
This sentence will be _public_.

<blockquote class='mdbook-internal' style='position: relative; padding: 20px 20px;'>
<span class='mdbook-internal' style='position: absolute; top: 0; right: 5px; font-size: 80%; opacity: 0.4;'>Internal</span>

This sentence will be _internal_.

</blockquote>
```

> Double newlines will **always** be added between the wrapping HTML element
and other content so that it will still be recognised as Markdown when it gets
to the processing stage. This plugin does _not_ support inline sections.

## Configuration

### Default Configuration
```toml
[preprocessor.internal]
remove = false
[preprocessor.internal.sections]
comment = "internal"
wrap = "blockquote"
class = "mdbook-internal"
label = "Internal"
[preprocessor.internal.sections.style]
wrap = ""
label = ""
[preprocessor.internal.chapters]
prefix = "_"
strip = false
recalculate = true
children = "keep"
```

#### Option: `remove` (_boolean_)
Whether or not to remove internal sections and/or chapters when building your book.

> This option is best left as the default (`false`), and configured using the
> `MDBOOK__INTERNAL__REMOVE` environment variable when building via your CI
> pipeline.

##### Example
```toml
[preprocessor.internal]
remove = true
```

```markdown
# Chapter 1

<!--[internal]
This paragraph will be removed from the built book.
[/internal]-->
```

### Sections (_table_, or _false_)
**Sections** are blocks of Markdown _inside_ your chapters that are marked as
_internal_. They are everything between an opening `<!--[internal]` tag and a
closing `[/internal]-->` tag (the word can be customised).

HTML comments are used since the `mdbook`'s Markdown parser will remove them
if the plugin is not enabled.

> Setting `sections` to `false` (equivalent to setting `sections.comment` to
> `false`) will disable **sections** functionality.

#### Option: `sections.comment` (_string_, or _false_)
Customise the word used by the HTML comment when surrounding internal sections.
The default value is `internal`. Can be any string, or a boolean. Setting
`sections.comment` to `false` will disable the processing of HTML comments,
even when `remove` is enabled, sending them to the Markdown processor as-is.

##### Example
```toml
[preprocessor.internal]
remove = false
tag = "hidden"
```

```markdown
<!--[hidden]
This is hidden content (but will show up in the built book).
[/hidden]-->
```

#### Option: `sections.wrap` (_string_, or _false_)
Defines the _wrapping element_, used to surround your internal sections (when `remove` is kept as `false`). Defaults to `blockquote` but can be any string.

Alternatively, setting this option to `false` will prevent your internal
sections from wrapped at all, and instead only strip the HTML comment before sending to the Markdown processor.


#### Option: `sections.class` (_string_, or _false_)
The CSS class to add to both the _wrapping element_, and the _label element_.

Default value is `mdbook-internal`, but this can be set to either a string, or
`false` to disable CSS classes on both HTML elements.

#### Option: `sections.label` (_string_, or _false_)
The text to use for the _label element_. The _label element_ is a `<span>`
inserted directly inside the _wrapping element_ before the internal content.

This can be either a string, or set to `false` to disable adding the _label
element_ to the _wrapping element_.

#### Option: `sections.styles` (_table_, or _false_)
Define inline styles to be added to the _wrapping_ and _label element_. This can
be set to `false` to disable inline styling altogether, or a table consisting of
`wrap` and `label` elements (both _strings_ or `false`) to set them individually.

##### Example

```toml
[preprocessor.internal.sections.styles]
wrap = false
label = "text-transform: uppercase"
```

### Chapters (_table_, or _false_)
Entire chapters (Markdown files) can be marked as internal by prefixing the
filename with a pre-determined string.

> Setting `chapters` to `false` (equivalent to setting `chapters.prefix` to
> `false`) will disable **sections** functionality.

#### Option: `chapters.prefix` (_string_, or _false_)
A string prefix to check Markdown filenames for. If any Markdown file starts
with the defined prefix, they are removed as a chapter from the book when
building with `remove` enabled. Defaults to an underscore (`_`).

> Setting `prefix` to `false` will disable removing Markdown files/chapters,
> even when `remove` is enabled.

#### Option: `chapters.strip` (_boolean_)
Whether or not to remove the prefix from filenames when building your book (when
`remove` is disabled).

> Take care with this option. If you have two files with the same name (one of
> them with the prefix and one without) then this is undefined behaviour. One
> will override the other.

#### Option: `chapters.recalculate` (_boolean_)
When internal chapters are removed the numbering in the sidebar doesn't change,
meaning there will be missing chapter numbers. Enabling this option recalculates
the chapter numbers so they remain sequential.

##### Example

```markdown
# Summary
- [Introduction](one.md)
  - [Preface](one_one.md)
- [Secret Recipe](_two.md)
- [Summary](three.md)
```

```
(with recalculate = false)

1. Introduction
   1.1 Preface
3. Summary
```

```
(with recalculate = true)

1. Introduction
   1.1 Preface
2. Summary
```

#### Option: `chapters.children` (_string_)
Can be either `keep` or `remove`, any other value will error.

Internal chapters may have, however deeply nested, children that are public.
This option allows you to choose whether:

- internal chapters and _all_ their descendants should be indiscriminately removed, or
- internal chapters should have their turned into a
  [draft chapter](https://rust-lang.github.io/mdBook/format/summary.html#structure),
  but kept in the tree so that its public descendants are still accessible.
