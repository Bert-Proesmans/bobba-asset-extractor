use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{prelude::*, BufReader, Seek, SeekFrom};
use std::iter::IntoIterator;
use std::path::Path;


use flate2::read::ZlibDecoder;
use glob::glob;
use quick_xml::de::{from_reader, DeError};
use reqwest::{header, StatusCode, Url};
use serde::Deserialize;
// use swf_parser::swf_types::{ImageType, Tag};
// use swf_parser::{parse_swf, SwfParseError};
use png;
use swf::*;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename = "furnidata")]
struct FurnitureData {
    #[serde(rename = "roomitemtypes")]
    room_item_types: RoomItemTypes,
    #[serde(rename = "wallitemtypes")]
    wall_item_types: WallItemTypes,
}

#[derive(Debug, Deserialize, PartialEq)]
struct RoomItemTypes {
    #[serde(rename = "furnitype")]
    items: Vec<FurnitureType>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct WallItemTypes {
    #[serde(rename = "furnitype")]
    items: Vec<FurnitureType>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct FurnitureType {
    id: u32,
    #[serde(rename = "classname")]
    class_name: String,
    name: String,
    description: String,
    revision: u32,
    #[serde(rename = "canstandon")]
    can_stand_on: Option<bool>,
}


mod cli;
mod error;
mod runtime;
mod asset_metadata;
mod asset_extraction;

use slog::Drain;
use slog_async;
use slog_term;
use pipeliner::Pipeline;

fn main() -> Result<(), error::ExtractorError> {
    let root_logger = {
        let std_out_decorator = slog_term::PlainDecorator::new(std::io::stdout());
        let std_err_decorator = slog_term::PlainDecorator::new(std::io::stderr());

        let std_out_drain = {
            let drain = slog_term::CompactFormat::new(std_out_decorator)
                .use_utc_timestamp()
                .build();
            // NOTE; Define a minimum and maximum filter so we split messages between drains!
            let drain = slog::LevelFilter::new(drain, slog::Level::Trace);
            slog::Filter::new(drain, |record: &slog::Record| match record.level() {
                slog::Level::Error | slog::Level::Critical => false,
                _ => true,
            })
        };

        let std_err_drain = {
            let drain = slog_term::FullFormat::new(std_err_decorator)
                .use_utc_timestamp()
                .use_original_order()
                .build();
            slog::LevelFilter::new(drain, slog::Level::Error)
        };

        let drain = slog::Duplicate(std_out_drain.fuse(), std_err_drain.fuse());
        let drain = slog_async::Async::new(drain.fuse())
            .thread_name(String::from("Logging"))
            .build();
        slog::Logger::root(drain.fuse(), slog::o!())
    };

    let io_thread_count = 50;
    let cpu_thread_count = 4;
    let options = cli::get_cli()?;

    slog::info!(root_logger, "Program initialized");
    slog::error!(root_logger, "Program initialized");

    options.zones
        .clone()
        .with_threads(io_thread_count)
        .map(|x| {
            asset_metadata::prepare_folders();
            asset_metadata::download_index_data();
        })
        .with_threads(io_thread_count)
        .map(|x| asset_extraction::download_asset_packs())
        .with_threads(cpu_thread_count)
        .map(|x| asset_extraction::extract_asset_packs());

    // let DATA_DIR = "./data";
    // let FURNI_FILE = "./data/furniture_data.xml";
    // let FURNI_BASE_PATH = "./data/furniture/";
    // let FURNI_EXTRACTED_PATH = "./data/extracted";
    // fs::create_dir_all(DATA_DIR)?;
    // fs::create_dir_all(FURNI_BASE_PATH)?;
    // fs::create_dir_all(FURNI_EXTRACTED_PATH)?;

    // println!("I require furniture data, either a local path or URL is fine!");
    // let mut furni_string = String::new();
    // io::stdin().read_line(&mut furni_string)?;
    // // WARN; String includes the newline!
    // let furni_string = furni_string.trim().trim_matches(|c| c == '\'' || c == '"');

    // let furni_file_open_result = OpenOptions::new().read(true).open(furni_string);
    // let furni_file = match furni_file_open_result {
    //     Ok(file) => file,
    //     Err(open_error) => {
    //         println!(
    //             "[DBG] Local file couldn't be opened due to error: {:?}\nAttempting download",
    //             open_error
    //         );

    //         let mut response = http_client.get(furni_string).send()?.error_for_status()?;
    //         if !response.status().is_success() {
    //             Err(BobbaError::FurniDownload(response.status()))?;
    //         }

    //         let mut furni_file = OpenOptions::new()
    //             .read(true)
    //             .write(true)
    //             .create(true)
    //             .truncate(true)
    //             .open(FURNI_FILE)?;

    //         if response.copy_to(&mut furni_file)? == 0 {
    //             Err(BobbaError::FurniDownload(StatusCode::NO_CONTENT))?;
    //         }
    //         furni_file.seek(SeekFrom::Start(0))?;
    //         println!("Furni file downloaded to {}", FURNI_FILE);
    //         furni_file
    //     }
    // };

    // let furniture_structures: FurnitureData = from_reader(BufReader::new(furni_file))?;
    // let room_furniture = furniture_structures.room_item_types.items.into_iter();
    // let wall_furniture = furniture_structures.wall_item_types.items.into_iter();

    // let furniture = room_furniture.chain(wall_furniture).fold(
    //     BTreeMap::<u32, FurnitureType>::new(),
    //     |mut acc, item| {
    //         acc.entry(item.id).or_insert(item);
    //         acc
    //     },
    // );

    // let MEDIA_URL_BASE = Url::parse("http://images.habbo.com/dcr/hof_furni/")?;
    // for (furni_id, furni_struct) in furniture {
    //     if furni_struct.revision == 0 {
    //         println!(
    //             "SKIP; Furniture `{}`: specific revision {}!",
    //             furni_struct.name, furni_struct.revision,
    //         );
    //         continue;
    //     }

    //     let revision = furni_struct.revision.to_string();
    //     let asset_name = match furni_struct.class_name.split('*').next() {
    //         Some(value) => value,
    //         None => unreachable!(), // "There is always a string value present!"
    //     };
    //     let furni_file_name = format!("{base_name}.swf", base_name = asset_name);
    //     let mut furni_url = MEDIA_URL_BASE.clone();
    //     furni_url
    //         .path_segments_mut()
    //         .expect("No static base path!")
    //         .extend(&[&revision, &furni_file_name]);
    //     let furni_local_path = Path::new(FURNI_BASE_PATH).join(&furni_file_name);
    //     if furni_local_path.exists() {
    //         println!(
    //             "SKIP; Furniture `{}`: Already downloaded!",
    //             furni_struct.name
    //         );
    //         continue;
    //     }

    //     let mut response = http_client.get(furni_url).send()?;
    //     if !response.status().is_success() {
    //         // Err(BobbaError::FurniDownload(response.status()))?;
    //         println!("FAIL; Furniture `{}`", furni_struct.name);
    //     }

    //     let mut furni_file = OpenOptions::new()
    //         .write(true)
    //         .create(true)
    //         .truncate(true)
    //         .open(furni_local_path)?;

    //     if response.copy_to(&mut furni_file)? == 0 {
    //         // Err(BobbaError::FurniDownload(StatusCode::NO_CONTENT))?;
    //         println!("FAIL; Furniture `{}`", furni_struct.name);
    //     }

    //     println!("OK; Furniture `{}`", furni_struct.name);
    // }

    // let swf_blob_glob = Path::new(FURNI_BASE_PATH).join("**/*.swf");
    // let swf_blobs_glob_str = swf_blob_glob.to_str().expect("Invalid string characters!");
    // println!("Extracting from all {:?}", swf_blobs_glob_str);
    // let swf_blobs_iterator = glob(swf_blobs_glob_str).expect("Invalid Glob!");
    // for swf_blob_result in swf_blobs_iterator {
    //     let swf_blob_path = swf_blob_result?;
    //     let swf_basename = swf_blob_path.file_stem().expect("Path should have a stem!");
    //     let swf_blob = fs::read(&swf_blob_path)?;

    //     let mut asset_map = BTreeMap::new();
    //     let extracted_asset_path = Path::new(FURNI_EXTRACTED_PATH).join(&swf_basename);
    //     fs::create_dir_all(&extracted_asset_path)?;

    //     // let swf_movie = parse_swf(&swf_blob).map_err(BobbaError::SwfParseError)?;
    //     let swf_movie = match read_swf(&swf_blob[..]) {
    //         Ok(result) => result,
    //         Err(error) => {
    //             println!("Error reading SWF file `{:?}`: {:}", swf_blob_path, error);
    //             continue;
    //         }
    //     };
    //     for tag in swf_movie.tags.iter() {
    //         match tag {
    //             Tag::SymbolClass(symbol_class) => {
    //                 for asset in symbol_class.iter() {
    //                     // NOTE; len() + 1 because the trimming ends with an additional underscore
    //                     let trimmed_asset_name: String = asset
    //                         .class_name
    //                         .chars()
    //                         .skip(swf_basename.len() + 1)
    //                         .collect();
    //                     asset_map
    //                         .entry(asset.id as u16)
    //                         .or_insert(trimmed_asset_name);
    //                     println!(
    //                         "Detected file name: {} at idx {}, for furniture `{:?}`",
    //                         asset.class_name, asset.id, swf_basename
    //                     );
    //                 }
    //             }
    //             _ => continue,
    //         }
    //     }

    //     for tag in swf_movie.tags.iter() {
    //         match tag {
    //             // NOTE; Tag order is not defined! We might have to split this in two passes!
    //             Tag::DefineBinaryData {
    //                 id: asset_id,
    //                 data: blob,
    //             } => {
    //                 let asset_id = *asset_id as u16;
    //                 let blob_file_stem = asset_map.get(&asset_id).ok_or({
    //                     let max_data = std::cmp::min(blob.len(), 100);
    //                     let err_string = format!(
    //                         "BINARY DATA: Asset id ({}) {} was not registered! Bytes: {:x?}",
    //                         asset_id,
    //                         0, // blob_stem_id
    //                         &blob[0..max_data]
    //                     );
    //                     BobbaError::Other(err_string)
    //                 })?;

    //                 let blob_file_name = format!("{}.xml", blob_file_stem);
    //                 let blob_destination_file = extracted_asset_path.join(blob_file_name);
    //                 if blob_destination_file.exists() {
    //                     println!("Skipping");
    //                     break;
    //                 }
    //                 fs::write(blob_destination_file, &blob)?;
    //             }
    //             Tag::DefineBitsLossless(bitmap) => {
    //                 let asset_id = bitmap.id as u16;
    //                 let mut data_buffer = &bitmap.data;
    //                 let blob_file_stem = asset_map.get(&asset_id).ok_or({
    //                     let max_data = std::cmp::min(bitmap.data.len(), 100);
    //                     let err_string = format!(
    //                         "BITMAP: Asset id ({}) {} was not registered! Bytes: {:x?}",
    //                         asset_id,
    //                         0, // blob_stem_id
    //                         &bitmap.data[0..max_data]
    //                     );
    //                     BobbaError::Other(err_string)
    //                 })?;

    //                 let mut decompressed_image_buffer = Vec::new();
    //                 let mut decoder = ZlibDecoder::new(&bitmap.data[..]);
    //                 match decoder.read_to_end(&mut decompressed_image_buffer) {
    //                     Result::Ok(_) => {
    //                         let buffer_copy: Vec<u8> = decompressed_image_buffer[..]
    //                             .chunks(4)
    //                             // NOTE; There is a missing IntoIterator implementation for slice/array that prevents
    //                             // us from simply calling [..].into_iter().
    //                             // We require an intermediate Vector, otherwise we're stuck with Iterator<Item=&u8>
    //                             .flat_map(|data| vec![data[1], data[2], data[3], data[0]])
    //                             .collect();
    //                         decompressed_image_buffer.clear();
    //                         {
    //                             let mut png_encoder = png::Encoder::new(
    //                                 &mut decompressed_image_buffer,
    //                                 bitmap.width as u32,
    //                                 bitmap.height as u32,
    //                             );
    //                             png_encoder.set_color({
    //                                 match bitmap.format {
    //                                     BitmapFormat::Rgb32 => png::ColorType::RGBA,
    //                                     _ => unimplemented!("Unknown bit depth!"),
    //                                 }
    //                             });
    //                             png_encoder.set_depth({
    //                                 match bitmap.format {
    //                                     BitmapFormat::Rgb32 => png::BitDepth::Eight,
    //                                     _ => unimplemented!("Unknown bit depth!"),
    //                                 }
    //                             });
    //                             let mut png_writer = png_encoder.write_header()?;
    //                             png_writer.write_image_data(&buffer_copy)?;
    //                         }
    //                         data_buffer = &decompressed_image_buffer;
    //                     }
    //                     Result::Err(err) => {
    //                         println!("ERROR: skipping");
    //                         continue;
    //                     }
    //                 };

    //                 let blob_file_name = format!("{}.{}", blob_file_stem, "png");
    //                 let blob_destination_file = extracted_asset_path.join(blob_file_name);
    //                 if blob_destination_file.exists() {
    //                     println!("Skipping");
    //                     break;
    //                 }
    //                 fs::write(blob_destination_file, &data_buffer)?;
    //             }
    //             // Tag::DefineBitmap(bitmap) => {
    //             //     let asset_id = bitmap.id;
    //             //     let mut data_buffer = &bitmap.data;
    //             //     let blob_file_stem = asset_map.get(&asset_id).ok_or({
    //             //         let err_string = format!(
    //             //             "BITMAP: Asset id ({}) {} was not registered! Bytes: {:x?}",
    //             //             asset_id,
    //             //             0, // blob_stem_id
    //             //             &bitmap.data[0..=100]
    //             //         );
    //             //         BobbaError::Other(err_string)
    //             //     })?;

    //             //     let mut decoded_image_buffer = Vec::new();
    //             //     let blob_file_extension = match bitmap.media_type {
    //             //         ImageType::Png => "png",
    //             //         ImageType::SwfLossless2 => {
    //             //             let mut decoder = ZlibDecoder::new(&bitmap.data[..]);
    //             //             match decoder.read(&mut decoded_image_buffer) {
    //             //                 Result::Ok(_) => {}
    //             //                 Result::Err(err) => {
    //             //                     println!("ERROR: skipping");
    //             //                     continue;
    //             //                 }
    //             //             }
    //             //             data_buffer = &decoded_image_buffer;
    //             //             "unknown"
    //             //         }
    //             //         // WARN; We have no access to the extended headers!
    //             //         // We're making a decoding guess here!
    //             //         x => {
    //             //             println!(
    //             //                 "BITMAP: Asset `{}`, unimplemented variant {:?}",
    //             //                 blob_file_stem, x
    //             //             );
    //             //             "unknown"
    //             //         }
    //             //         // unimplemented!(
    //             //         // "BITMAP: Index {}, Unknown media type {:?}! Bytes: {:x?}",
    //             //         // asset_id,
    //             //         // bitmap.media_type,
    //             //         // &bitmap.data[0..=100]
    //             //     };

    //             //     let blob_file_name = format!("{}.{}", blob_file_stem, blob_file_extension);
    //             //     let blob_destination_file = extracted_asset_path.join(blob_file_name);
    //             //     fs::write(blob_destination_file, &data_buffer)?;
    //             // }
    //             _ => continue,
    //         }
    //     }
    // }

    Ok(())
}
