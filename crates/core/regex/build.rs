use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn generate_rules(manifest_dir: &Path) -> Vec<(String, String)> {
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
            (message.to_owned(), pattern.to_owned())
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

    rules
}

fn precompile_database(
    out_dir: &Path,
    include_dir: &Path,
    lib_dir: &Path,
    rules: &[(String, String)],
) {
    let mut patterns = String::new();
    for (_, pattern) in rules {
        patterns.push_str("    \"");
        for byte in pattern.as_bytes() {
            match *byte {
                b'\\' => patterns.push_str("\\\\"),
                b'"' => patterns.push_str("\\\""),
                0x20..=0x7e => patterns.push(*byte as char),
                other => patterns.push_str(&format!("\\x{other:02x}\"\"")),
            }
        }
        patterns.push_str("\",\n");
    }
    let count = rules.len();

    let source = format!(
        r#"#include <hs/hs.h>
#include <stdio.h>
#include <stdlib.h>
static const char *const P[{count}] = {{
{patterns}}};
int main(int argc, char **argv) {{
    if (argc != 2) return 2;
    unsigned int f[{count}], i[{count}];
    for (int k = 0; k < {count}; ++k) {{ f[k] = HS_FLAG_DOTALL; i[k] = (unsigned int)k; }}
    hs_database_t *db = NULL; hs_compile_error_t *e = NULL;
    if (hs_compile_multi(P, f, i, {count}, HS_MODE_BLOCK, NULL, &db, &e) != HS_SUCCESS) {{
        fprintf(stderr, "hs_compile_multi: %s\n", e ? e->message : "?"); return 1;
    }}
    char *bytes = NULL; size_t len = 0;
    if (hs_serialize_database(db, &bytes, &len) != HS_SUCCESS) return 1;
    FILE *o = fopen(argv[1], "wb");
    if (!o || fwrite(bytes, 1, len, o) != len) return 1;
    fclose(o); free(bytes); hs_free_database(db); return 0;
}}
"#
    );

    let src = out_dir.join("precompile_db.c");
    let bin = out_dir.join("precompile_db");
    let dst = out_dir.join("rules_database.bin");
    fs::write(&src, source).expect("write precompile_db.c");

    let cc = std::env::var("HOST_CC")
        .or_else(|_| std::env::var("CC"))
        .unwrap_or_else(|_| "cc".into());
    let mut cmd = Command::new(&cc);
    cmd.arg(&src)
        .arg("-O2")
        .arg("-o")
        .arg(&bin)
        .arg(format!("-I{}", include_dir.display()))
        .arg(format!("-L{}", lib_dir.display()))
        .arg("-lhs");
    if env("TARGET").contains("apple") {
        cmd.arg("-lc++");
    } else if !env("TARGET").contains("msvc") {
        cmd.arg("-lstdc++");
    }
    cmd.arg("-lm").arg("-lpthread");
    assert!(
        cmd.status().expect("invoke cc").success(),
        "failed to compile Hyperscan precompiler"
    );
    assert!(
        Command::new(&bin)
            .arg(&dst)
            .status()
            .expect("run precompile_db")
            .success(),
        "Hyperscan precompiler failed"
    );
    assert!(
        fs::metadata(&dst).map(|m| m.len() > 0).unwrap_or(false),
        "rules_database.bin is empty"
    );
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=src/bindings.rs");

    let manifest_dir = PathBuf::from(env("CARGO_MANIFEST_DIR"));
    let rules = generate_rules(&manifest_dir);

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

    let include_dir = env_opt("VECTORSCAN_INCLUDE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| build_dir.join("include"));
    let out_dir = PathBuf::from(env("OUT_DIR"));

    precompile_database(&out_dir, &include_dir, &lib_search_dir, &rules);

    #[cfg(feature = "gen")]
    {
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
