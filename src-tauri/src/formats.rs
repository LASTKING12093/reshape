use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatCapability {
    pub archive_outputs: Vec<String>,
    pub input: String,
    pub outputs: Vec<String>,
    pub engine: String,
    pub category: String,
    pub lossless_outputs: Vec<String>,
    pub target_size: bool,
    pub metadata: bool,
    pub layout: String,
    pub advanced: Vec<String>,
}
pub fn registry() -> Vec<FormatCapability> {
    serde_json::from_str(include_str!("../../capabilities.json")).expect("valid registry")
}
pub fn capability(input: &str) -> Result<FormatCapability, String> {
    registry()
        .into_iter()
        .find(|r| r.input == input)
        .ok_or_else(|| "Unsupported file format.".into())
}
pub fn validate(input: &str, output: &str, lossless: bool) -> Result<FormatCapability, String> {
    let mut r = capability(input)?;
    if r.archive_outputs.iter().any(|s| s == output) {
        r.engine = "7zip".into();
        r.category = "archive".into();
        r.outputs = r.archive_outputs.clone();
        r.lossless_outputs = r.archive_outputs.clone();
        return Ok(r);
    }
    if !r.outputs.iter().any(|s| s == output) {
        return Err("Unsupported conversion.".into());
    }
    if lossless && !r.lossless_outputs.iter().any(|s| s == output) {
        return Err("This output does not support the selected lossless mode.".into());
    }
    Ok(r)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reject_vectorization() {
        assert!(validate("jpg", "svg", false).is_err())
    }
    #[test]
    fn reject_lossy_lossless() {
        assert!(validate("wav", "mp3", true).is_err())
    }
    #[test]
    fn lossless_flac() {
        assert!(validate("wav", "flac", true).is_ok())
    }
}
