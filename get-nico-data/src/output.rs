use crate::Packet;
use std::io::{Write, BufWriter, stdout, Stdout};
use std::sync::mpsc::Receiver;
use structs::NewVideoInfo;
use chrono::Timelike;
use crate::options::Options;
use std::fs::{create_dir_all, File};
use std::path::Path;
use either::{Either, Left, Right};

pub(crate) fn run(receiver: Receiver<Packet>, options: &Options) {
    let mut writer: Either<Stdout, File> = match &options.out {
        None => Left(stdout()),
        Some(name) => {
            create_dir_all(Path::new(&name).parent().unwrap()).unwrap();
            Right(File::create(name).unwrap())
        }
    };
    let mut writer: &mut dyn Write = match &mut writer {
        Left(left) => left,
        Right(right) => right,
    };
    let mut writer = BufWriter::new(writer);
    let mut list = Vec::<NewVideoInfo>::new();
    for packet in receiver.iter() {
        if packet.last_modified.offset().utc_minus_local() == 0 && packet.last_modified.timestamp() == 0 {
            break
        }
        for video in packet.videos {
            list.push (NewVideoInfo {
                last_modified: packet.last_modified,
                content_id: video.content_id.unwrap(),
                title: video.title.unwrap(),
                description: video.description,
                view_counter: video.view_counter.unwrap(),
                mylist_counter: video.mylist_counter.unwrap(),
                length_seconds: video.length_seconds.unwrap(),
                thumbnail_url: video.thumbnail_url,
                start_time: video.start_time.unwrap(),
                last_res_body: video.last_res_body,
                comment_counter: video.comment_counter.unwrap(),
                last_comment_time: video.last_comment_time,
                category_tags: video.category_tags,
                tags: video.tags.unwrap(),
                genre: video.genre,
            });
        }
    }
    eprintln!("writeing.....");
    bincode::serialize_into(&mut writer, &list);
}