use crate::Packet;
use std::io::{Write, BufWriter};
use std::sync::mpsc::Receiver;
use structs::NewVideoInfo;

pub(crate) fn run<W: Write>(receiver: Receiver<Packet>, writer: W) {
    let mut writer = BufWriter::new(writer);
    for packet in receiver.iter() {
        for video in packet.videos {
            bincode::serialize_into(&mut writer, &NewVideoInfo {
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
            }).unwrap();
        }
    }
}