const VAR_BASE: &str = "{{base}}";
const VAR_BREADCRUMB: &str = "{{breadcrumb}}";
const VAR_EXT: &str = "{{ext}}";
const VAR_FILES: &str = "{{files}}";
const VAR_HREF: &str = "{{href}}";
const VAR_TITLE: &str = "{{title}}";
const VAR_KIND: &str = "{{kind}}";

const TPL_DIRECTORY: &str = include_str!("directory.html");
const TPL_BREADCRUMB: &str = r#"<a href="{{href}}">{{base}}/</a>"#;
const TPL_FILE: &str =
    r#"<li><a href="{{href}}" title="{{title}}" class="{{kind}} {{ext}}">{{base}}</a></li>"#;

pub const FILE: &str = "file";
pub const FOLDER: &str = "folder";
pub const DIRECTORY: &str = "directory";

pub fn render_file<S: AsRef<str>>(href: S, title: S, kind: S, ext: S, base: S) -> String {
    TPL_FILE
        .replace(VAR_HREF, href.as_ref())
        .replace(VAR_TITLE, title.as_ref())
        .replace(VAR_KIND, kind.as_ref())
        .replace(VAR_EXT, ext.as_ref())
        .replace(VAR_BASE, base.as_ref())
}

pub fn render_directory<S: AsRef<str>>(title: S, breadcrumb: S, files: S) -> String {
    TPL_DIRECTORY
        .replace(VAR_TITLE, title.as_ref())
        .replace(VAR_BREADCRUMB, breadcrumb.as_ref())
        .replace(VAR_FILES, files.as_ref())
}

pub fn render_breadcrumb<S: AsRef<str>>(href: S, base: S) -> String {
    TPL_BREADCRUMB
        .replace(VAR_HREF, href.as_ref())
        .replace(VAR_BASE, base.as_ref())
}
