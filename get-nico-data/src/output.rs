use crate::Packet;
use std::io::{Write, BufWriter, stdout, Stdout};
use std::sync::mpsc::Receiver;
use structs::NewVideoInfo;
use crate::options::Options;
use std::fs::{create_dir_all, File};
use std::path::Path;
use either::{Either, Left, Right};
use chrono::Utc;

pub(crate) fn run(receiver: Receiver<Packet>, options: &Options) {
    let mut writer: Either<Stdout, File> = match &options.out {
        None => Left(stdout()),
        Some(name) => {
            create_dir_all(Path::new(&name).parent().unwrap()).unwrap();
            Right(File::create(name).unwrap())
        }
    };
    let writer: &mut dyn Write = match &mut writer {
        Left(left) => left,
        Right(right) => right,
    };
    let mut writer = BufWriter::new(writer);

    let mut contents_id_out = options.contents_id_out.as_ref().map(|name| {
        create_dir_all(Path::new(&name).parent().unwrap()).unwrap();
        BufWriter::new(File::create(name).unwrap())
    });

    let mut list = Vec::<NewVideoInfo>::new();
    for packet in receiver.iter() {
        if packet.last_modified.offset().utc_minus_local() == 0 && packet.last_modified.timestamp() == 0 {
            break
        }
        for video in packet.videos {
            if let Some(out) = &mut contents_id_out {
                writeln!(out, "{}", video.content_id.as_ref().unwrap()).unwrap();
                out.flush().unwrap();
            }
            list.push (NewVideoInfo {
                last_modified: packet.last_modified.with_timezone(&Utc),
                content_id: video.content_id.unwrap(),
                title: video.title.unwrap(),
                description: video.description,
                view_counter: video.view_counter.unwrap(),
                mylist_counter: video.mylist_counter.unwrap(),
                length_seconds: video.length_seconds.unwrap(),
                thumbnail_url: video.thumbnail_url,
                start_time: video.start_time.unwrap().with_timezone(&Utc),
                last_res_body: video.last_res_body,
                comment_counter: video.comment_counter.unwrap(),
                last_comment_time: video.last_comment_time.map(|x| x.with_timezone(&Utc)),
                category_tags: video.category_tags,
                tags: video.tags.unwrap(),
                genre: video.genre,
            });
        }
    }
    eprintln!("writeing.....");
    bincode::serialize_into(&mut writer, &list).unwrap();
}