
pub(crate) fn parse_options() -> Options {
    let mut merge_all = false;
    let mut remove_old = false;
    let mut out_dir: Option<String> = None;
    let args = std::env::args();

    for x in args {
        if x.starts_with("-") {
            match x.as_str() {
                "-a" => merge_all = true,
                "-d" => remove_old = true,
                _ => panic!(format!("unknown option: {}", x)),
            }
        } else {
            out_dir = Some(x)
        }
    }

    Options{
        merge_all,
        remove_old,
        out_dir: out_dir.unwrap_or_else(|| panic!("no out dir found"))
    }
}

pub(crate) struct Options {
    pub(crate) merge_all: bool,
    pub(crate) remove_old: bool,
    pub(crate) out_dir: String,
}
