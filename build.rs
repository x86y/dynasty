use std::{
    env,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};
fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("svg_logos.rs");
    let mut file = BufWriter::new(File::create(path).unwrap());

    let mut map = phf_codegen::Map::new();

    for entry in fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logos/")).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        let Some(name) = file_name.strip_suffix(".svg") else {
            panic!("expected .svg file, got {file_name}");
        };

        map.entry(
            name.to_uppercase(),
            &format!(r#"include_bytes!("{}")"#, entry.path().display()),
        );
    }

    write!(
        &mut file,
        "pub(crate) static LOGOS: phf::Map<&'static str, &'static [u8]> = {}",
        map.build()
    )
    .unwrap();
    writeln!(&mut file, ";").unwrap();
}
