/// Attempts to remove the contents of the provided directory path
pub fn remove_dir_contents<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<()> {
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;

        let path = entry.path();

        if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
    }

    Ok(())
}
