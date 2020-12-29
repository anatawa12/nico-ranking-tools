use std::fs::File;
use std::io::{BufWriter, Write};

const ROOT_PAGE_COUNT_LIMIT: u64 = 1_0000;

pub struct RankingPage {
    pub index: u64,
    pub first_rank: u64,
    pub last_rank: u64,
}

pub fn index_file(
    output_dir: &String,
    page_infos: &[RankingPage],
) -> std::io::Result<()> {
    let index_pages = page_infos.chunks(ROOT_PAGE_COUNT_LIMIT as usize)
        .collect::<Vec<_>>();

    if index_pages.len() == 1 {
        let output_html = format!("{}/index.html", output_dir);
        process_index(
            &output_html,
            index_pages[0],
            true,
            "ranking-"
        )?
    } else {
        let index_page_infos = index_pages.iter()
            .enumerate()
            .map(|(index, ranking_pages)| {
                let first = ranking_pages.first().unwrap();
                let last = ranking_pages.last().unwrap();
                RankingPage {
                    index: index as u64,
                    first_rank: first.first_rank,
                    last_rank: last.last_rank,
                }
            })
            .collect::<Vec<_>>();

        for (i, x) in index_pages.iter().enumerate() {
            let output_html = format!("{}/index-{}.html", output_dir, i);
            process_index(
                &output_html,
                x,
                false,
                "ranking-",
            )?
        }

        let output_html = format!("{}/index.html", output_dir);
        process_index(
            &output_html,
            &index_page_infos,
            true,
            "index-",
        )?
    }

    Ok(())
}

fn process_index(
    output_html: &String,
    page_infos: &[RankingPage],
    is_root: bool,
    link_page_prefix: &str,
) -> std::io::Result<()> {
    let output_html = File::create(output_html)?;
    let mut output_html = BufWriter::new(output_html);

    let range = if is_root {
        None
    } else {
        Some(format!("{}位~{}位",
                page_infos.first().unwrap().first_rank,
                page_infos.last().unwrap().last_rank))
    };

    let title = if let Some(range) = &range {
        format!("人類が動画にかけた時間のランキング({})", range)
    } else {
        "人類が動画にかけた時間のランキング".to_string()
    };

    writeln!(output_html, r#"<!DOCTYPE html>"#)?;
    writeln!(output_html, r#"<html lang="en">"#)?;
    writeln!(output_html, r#"<head>"#)?;
    writeln!(output_html, "{}", include_str!("template.head.html").replace("{title}", &title))?;
    writeln!(output_html, r#"</head>"#)?;
    writeln!(output_html, r#"<body>"#)?;
    writeln!(output_html, "{}", include_str!("template.body.head.html"))?;
    if let Some(range) = &range {
        writeln!(output_html, r#"<header class="header">"#)?;
        writeln!(output_html, r#"    <div class="center">{}</div>"#, range)?;
        writeln!(output_html, r#"</header>"#)?;
    }
    writeln!(&mut output_html, r#"<ul class="container">"#)?;

    for x in page_infos {
        writeln!(&mut output_html, r#"    <li class="grid-item"><a href="{}{}.html">{}位~{}位</a></li>"#,
                 link_page_prefix, x.index, x.first_rank, x.last_rank)?;
    }

    writeln!(&mut output_html, r#"</ul>"#)?;
    writeln!(output_html, "{}", include_str!("template.body.foot.html"))?;
    writeln!(output_html, r#"</body>"#)?;
    writeln!(output_html, r#"</html>"#)?;

    Ok(())
}
