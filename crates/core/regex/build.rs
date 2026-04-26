use std::fs;
use std::path::{Path, PathBuf};

fn env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("`{name}` should be set in the environment"))
}

fn env_opt(name: &str) -> Option<String> {
    println!("cargo:rerun-if-env-changed={name}");
    std::env::var(name).ok()
}

fn default_vectorscan_build_dir(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join("build").join("vectorscan")
}

fn lib_dir(build_dir: &Path) -> PathBuf {
    let lib64 = build_dir.join("lib64");
    if lib64.join("libhs.a").exists() {
        return lib64;
    }
    build_dir.join("lib")
}

fn emit_cxx_runtime_link() {
    let target = env("TARGET");
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else if !target.contains("msvc") {
        println!("cargo:rustc-link-lib=stdc++");
    }
}

fn parse_rule_line(line_number: usize, line: &str) -> (&str, &str) {
    let Some((message, pattern)) = line.split_once('\t') else {
        panic!("invalid TSV row at src/rules/rules.tsv:{line_number}");
    };
    assert!(
        !pattern.contains('\t')
            && [message, pattern]
                .iter()
                .all(|field| !field.is_empty() && !field.as_bytes().contains(&0)),
        "invalid TSV row at src/rules/rules.tsv:{line_number}"
    );

    (message, pattern)
}

fn generate_rules(manifest_dir: &Path) {
    let rules_path = manifest_dir.join("src").join("rules").join("rules.tsv");
    println!("cargo:rerun-if-changed={}", rules_path.display());

    let input = fs::read_to_string(&rules_path)
        .unwrap_or_else(|error| panic!("failed to read `{}`: {error}", rules_path.display()));
    let mut lines = input.lines();
    assert_eq!(
        lines.next(),
        Some("message\tpattern"),
        "`{}` must start with the message/pattern TSV header",
        rules_path.display()
    );

    let rules = lines
        .enumerate()
        .map(|(index, line)| {
            let (message, pattern) = parse_rule_line(index + 2, line);
            (message, pattern)
        })
        .collect::<Vec<_>>();
    assert!(
        !rules.is_empty(),
        "`{}` must define at least one rule",
        rules_path.display()
    );
    assert!(
        u32::try_from(rules.len()).is_ok(),
        "`{}` defines more rules than Vectorscan ids support",
        rules_path.display()
    );

    let ids = (0..rules.len())
        .map(|id| format!("{id},"))
        .collect::<String>();
    let patterns = rules
        .iter()
        .map(|(_, pattern)| format!("    c{pattern:?},\n"))
        .collect::<String>();
    let metadata = rules
        .iter()
        .enumerate()
        .map(|(id, (message, _))| format!("    Rule {{ id: {id}, message: {message:?} }},\n"))
        .collect::<String>();
    let output = format!(
        "pub const RULE_COUNT: usize = {};\n\
         pub const VECTORSCAN_DOTALL_FLAG: u32 = 2;\n\
         pub const VECTORSCAN_PATTERN_FLAGS: [u32; RULE_COUNT] = [VECTORSCAN_DOTALL_FLAG; RULE_COUNT];\n\
         pub const VECTORSCAN_PATTERN_IDS: [u32; RULE_COUNT] = [{ids}];\n\n\
         pub const VECTORSCAN_PATTERNS: [&CStr; RULE_COUNT] = [\n{patterns}];\n\n\
         pub static RULES: [Rule; RULE_COUNT] = [\n{metadata}];\n",
        rules.len(),
    );

    let out_dir = PathBuf::from(env("OUT_DIR"));
    fs::write(out_dir.join("rules.rs"), output).expect("failed to write generated rules");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=src/bindings.rs");

    let manifest_dir = PathBuf::from(env("CARGO_MANIFEST_DIR"));
    generate_rules(&manifest_dir);

    let build_dir = env_opt("VECTORSCAN_BUILD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| default_vectorscan_build_dir(&manifest_dir));

    let lib_search_dir = lib_dir(&build_dir);

    assert!(
        lib_search_dir.join("libhs.a").exists(),
        "libhs.a not found in `{}`. Build Vectorscan first with `make build` in `{}`.",
        lib_search_dir.display(),
        manifest_dir.display()
    );

    println!(
        "cargo:rustc-link-search=native={}",
        lib_search_dir.display()
    );
    println!("cargo:rustc-link-lib=static=hs");
    emit_cxx_runtime_link();

    #[cfg(feature = "gen")]
    {
        let include_dir = env_opt("VECTORSCAN_INCLUDE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| build_dir.join("include"));
        let out_dir = PathBuf::from(env("OUT_DIR"));
        let bindings = bindgen::Builder::default()
            .header(manifest_dir.join("wrapper.h").to_string_lossy())
            .clang_arg(format!("-I{}", include_dir.display()))
            .rust_edition(bindgen::RustEdition::Edition2024)
            .allowlist_function("hs_.*")
            .allowlist_type("hs_.*")
            .allowlist_var("HS_.*")
            .generate()
            .expect("failed to generate Vectorscan bindings");
        bindings
            .write_to_file(out_dir.join("bindings.rs"))
            .expect("failed to write Vectorscan bindings");
    }
}
