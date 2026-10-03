use forma_lib::{conversion, engines::EngineManager, models::ConversionOptions, probing, process::{self, Cancel}};
use std::{fs, path::{Path, PathBuf}, time::{Duration, Instant}};

fn engines() -> EngineManager { EngineManager::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries")) }
fn fixture(name: &str) -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../test-fixtures").join(name) }
fn execute(source: &Path, target: &str, root: &Path, engine: &EngineManager) -> Result<PathBuf, String> {
    let input=probing::inspect(source,engine)?;
    let options=ConversionOptions { format:target.into(),output_dir:root.to_string_lossy().into(),..Default::default() };
    conversion::execute(&input,&options,engine,root,&Cancel::default(),|_,_|{}).map(|v|v.0)
}

#[test]
fn office_supported_documents_produce_readable_pdf_without_touching_input() {
    let engine=engines();
    for name in ["document.docx","document.odt","document.rtf","document.pptx","document.xlsx"] {
        let source=fixture(name);let before=fs::read(&source).unwrap();let root=tempfile::tempdir().unwrap();
        let pdf=execute(&source,"pdf",root.path(),&engine).unwrap();
        assert!(fs::metadata(&pdf).unwrap().len()>100);
        let inspected=probing::inspect(&pdf,&engine).unwrap();assert_eq!(inspected.format,"pdf");
        let info=process::capture(&engine.executable("pdfinfo").unwrap(),&[pdf.to_str().unwrap()],root.path()).unwrap();
        let pages=info.lines().find_map(|l|l.strip_prefix("Pages:")).unwrap().trim().parse::<u32>().unwrap();
        assert!((1..=5).contains(&pages),"implausible page count: {name}");
        assert_eq!(fs::read(&source).unwrap(),before);assert_eq!(fs::read_dir(root.path()).unwrap().count(),1,"temporary files leaked");
    }
}

#[test]
fn malformed_documents_fail_without_leaving_jobs_or_temporary_files() {
    let engine=engines();let source=tempfile::tempdir().unwrap();
    let cases=[("invalid.docx",b"not a document".to_vec()),("truncated.docx",fs::read(fixture("document.docx")).unwrap()[..80].to_vec()),("false.xlsx",b"a spreadsheet only by name".to_vec()),("corrupt.odt",b"PK\x03\x04\x00\x00garbage".to_vec())];
    for (name,bytes) in cases {
        let path=source.path().join(name);fs::write(&path,&bytes).unwrap();let root=tempfile::tempdir().unwrap();let start=Instant::now();
        assert!(execute(&path,"pdf",root.path(),&engine).is_err());assert!(start.elapsed()<Duration::from_secs(10));
        assert_eq!(fs::read(&path).unwrap(),bytes);assert_eq!(fs::read_dir(root.path()).unwrap().count(),0);
    }
}
