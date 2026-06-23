use msgbox::IconType;

pub fn message(content: &str) -> Result<(), Box<dyn std::error::Error>> {
    msgbox::create("TODO", content, IconType::Info)?;
    Ok(())
}
