use askama::Template;

#[derive(Template)]
#[template(path = "upload.html")]
pub struct UploadTemplate {}
