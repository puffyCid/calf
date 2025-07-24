use calf::{
    calf::{CalfReader, CalfReaderAction, QcowInfo},
    format::{header::CalfHeader, level::CalfLevel},
};
use std::{
    env,
    fs::File,
    io::{BufReader, Read, Seek},
    path::Path,
};

fn main() {
    println!("Lets get some basic QCOW info!\n");

    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        let path = &args[1];
        if Path::new(path).is_file() {
            qcow_info(path);
        } else {
            println!("This is not a file")
        }
    } else {
        println!("Require QCOW input file!!")
    }
}

fn qcow_info(path: &str) {
    let reader = File::open(path).unwrap();
    let buf = BufReader::new(reader);
    let mut calf = CalfReader { fs: buf };
    let header = calf.header().unwrap();

    println!(
        "Version: {} - OS Size: {} - Compression Support: {:?}",
        header.version, header.size, header.compression_method
    );

    let size = header.level_one_table_ref * 8;
    let levels = calf.levels(&header.level_one_table_offset, &size).unwrap();
    println!("{}", header.level_one_table_offset);
    println!("size: {size}, levels count: {}", levels.len());
    let info = QcowInfo {
        header: calf.header().unwrap(),
        level1_table: calf.level1_entries().unwrap(),
    };
    let mut os_reader = calf.os_reader(&info).unwrap();
    println!("{}", os_reader.get_os_size());
    let mut buf = vec![0; 4096];

    os_reader.seek(std::io::SeekFrom::Start(165871616)).unwrap();
    os_reader.read_exact(&mut buf).unwrap();

    println!("sector bytes: {buf:?}");
    panic!("TODO: Try NTFS crate and see if u can read it?");
}
