use structs::NewVideoInfo;

pub type Filter = Box<dyn Fn(&NewVideoInfo) -> bool>;

pub fn parse<'a, I: 'a + Iterator<Item = &'a str>>(iter: &mut I) -> Option<Filter> {
    let filter = parse_expr(iter);
    if let Some(key) = iter.next() {
        panic!("unexpected keyword: {}", key);
    }
    return filter;
}

fn parse_expr<'a, I: 'a + Iterator<Item = &'a str>>(iter: &mut I) -> Option<Filter> {
    let filter: Filter = match iter.next() {
        Some("not") => {
            let inner = parse_expr(iter).expect("expr but nothing found");
            Box::new(move |x| !inner(x))
        }
        Some("in_tags") => {
            let tag = iter.next().expect("tag name" as &'static str).to_string();
            Box::new(move |x| x.tags.iter().any(|x| x.as_str() == tag.as_str()))
        }
        Some(key) => panic!("unknown keyword: {}", key),
        None => return None
    };
    let filter: Filter = match iter.next() {
        Some("and") => {
            let another_filter = parse_expr(iter).expect("expr but nothing found" as &'a str);
            Box::new(move |x| filter(x) && another_filter(x))
        }
        Some("or") => {
            let another_filter = parse_expr(iter).expect("expr but nothing found" as &'a str);
            Box::new(move |x| filter(x) || another_filter(x))
        }
        Some(key) => panic!("unknown keyword: {}", key),
        None => filter,
    };
    Some(filter)
}
