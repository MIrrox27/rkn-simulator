fn main() {
    cc::Build::new()
        .file("c_src/filter.c")
        .compile("filter");
}