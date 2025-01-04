use audio::AudioSource;
use codecs::png::PngDecoder;
use io::Reader;
use render::{render_resource::Extent3d, texture::ImageFormat};
use wg_2024::{packet::*,network::*};
use std::{collections::{HashMap,HashSet}, io::Cursor, mem::swap, ops::Deref, sync::Arc, thread, time::Duration};
use bevy::*;
use image::*;


// Trait to handle message fragmentation
pub trait Fragmentation<T> {
    fn fragment(message: T) -> Vec<u8>; // Fragment a message into bytes
}

// Helper function to sort fragments by their index
fn sort_by_fragment_index(fragments: &mut Vec<Fragment>) {
    let len = fragments.len();
    for i in 0..len {
        for j in 0..len {
            if fragments[i].fragment_index < fragments[j].fragment_index {
                let tmp = fragments[i].clone();
                fragments[i] = fragments[j].clone();
                fragments[j] = tmp;
            }
        }
    }
    // println!("{:?}", fragments); // Debug output for sorted fragments
}

// Function to check if all fragments are present
fn check_wholeness(fragments: &mut Vec<Fragment>) -> bool {
    let size = fragments[0].total_n_fragments; // Total number of fragments
    let mut count = 0;
    for i in 0..size {
        count += i as u64; // Sum of expected fragment indices
    }
    let mut check_count = 0;
    for fr in fragments {
        check_count += fr.fragment_index; // Sum of actual fragment indices
    }
    // println!("{}  {}", check_count, count); // Debug output for counts
    check_count == count // Verify completeness
}

// Trait to assemble fragments into the original message
pub trait Assembler<T: Fragmentation<T>> {
    fn assemble(fragments: &mut Vec<Fragment>) -> Result<T, String>;
}

// Implementation of Fragmentation for String
impl Fragmentation<String> for String {
    fn fragment(message: String) -> Vec<u8> {
        message.into_bytes() // Convert the string into bytes
    }
}

// Implementation of Assembler for String
impl Assembler<String> for String {
    fn assemble(fragments: &mut Vec<Fragment>) -> Result<String, String> {
        sort_by_fragment_index(fragments); // Sort fragments
        if !check_wholeness(fragments) {
            return Err("Missing one or more fragments. Cannot reconstruct the message".to_string());
        } else {
            let mut vec = Vec::new();
            for fr in fragments {
                for i in 0..fr.length {
                    vec.push(fr.data[i as usize]); // Collect fragment data
                }
            }
            Ok(String::from_utf8(vec).expect("Something is wrong with the assembler")) // Reconstruct string
        }
    }
}

// Implementation of Fragmentation for Bevy's AudioSource
impl Fragmentation<bevy::audio::AudioSource> for AudioSource {
    fn fragment(message: bevy::audio::AudioSource) -> Vec<u8> {
        message.bytes.to_vec() // Extract bytes from AudioSource
    }
}

// Implementation of Assembler for Bevy's AudioSource
impl Assembler<bevy::audio::AudioSource> for AudioSource {
    fn assemble(fragments: &mut Vec<Fragment>) -> Result<bevy::audio::AudioSource, String> {
        sort_by_fragment_index(fragments); // Sort fragments
        if !check_wholeness(fragments) {
            return Err("Missing one or more fragments. Cannot reconstruct the message".to_string());
        } else {
            let mut vec = Vec::new();
            for fr in fragments {
                for i in 0..fr.length {
                    vec.push(fr.data[i as usize]); // Collect fragment data
                }
            }
            Ok(AudioSource { bytes: Arc::from(vec) }) // Create new AudioSource
        }
    }
}

// Implementation of Fragmentation for Bevy's Image
impl Fragmentation<image::DynamicImage> for image::DynamicImage {
    fn fragment(message: image::DynamicImage) -> Vec<u8> {
        let mut data = Vec::new();
        message.write_to(&mut Cursor::new(&mut data), image::ImageFormat::Png).unwrap(); // Extract data from Image
        data
    }
}

// Implementation of Assembler for Bevy's Image
impl Assembler<image::DynamicImage> for image::DynamicImage {
    fn assemble(fragments: &mut Vec<Fragment>) -> Result<image::DynamicImage, String> {
        // Sort fragments by index
        fragments.sort_by_key(|fr| fr.fragment_index);

        // Check if all fragments are present
        if !check_wholeness(fragments) {
            return Err("Missing one or more fragments. Cannot reconstruct the message.".to_string());
        }

        // Combine data from fragments
        let mut combined_data = Vec::new();
        for fragment in fragments.iter() {
            combined_data.extend_from_slice(&fragment.data[..fragment.length as usize]);
        }

        // Split the combined data into image bytes and dimensions
        if combined_data.len() < 2 {
            return Err("Insufficient data to reconstruct the image.".to_string());
        }

        let reader = PngDecoder::new(Cursor::new(combined_data)).expect("Error in decoder");
        let res = image::DynamicImage::from_decoder(reader);
        // Decode the image

        match res {
            Ok(image) => Ok(image),
            Err(_) => Err("Failed to reconstruct the image from fragments.".to_string()),
        }
    }
}


// Helper function to convert a slice to a fixed-size array
fn slice_to_array(slice: &[u8], len: usize) -> [u8; 128] {
    let mut res: [u8; 128] = [0; 128];
    for i in 0..len {
        res[i] = slice[i];
    }
    res
}

// Serialize data into fragments
pub fn serialize(data: Vec<u8>) -> Vec<Fragment> {
    let len = data.len();
    let mut iter = data.chunks(128); // Split data into chunks of 128 bytes
    let mut vec = Vec::new();
    let mut size = (len / 128) as u64;
    let last = (len % 128) as u64;
    if last != 0 {
        size += 1; // Adjust total size for remaining data
    }
    let mut i = 1;
    let mut j = 128;
    if len > 128 {
        loop {
            if j < len {
                let fragment_data = iter.next().unwrap();
                vec.push(Fragment {
                    fragment_index: i,
                    total_n_fragments: size,
                    data: slice_to_array(fragment_data, fragment_data.len()),
                    length: fragment_data.len() as u8,
                });
                i += 1;
                j += 128;
            } else {
                let fragment_data = iter.next().unwrap();
                vec.push(Fragment {
                    fragment_index: i,
                    total_n_fragments: size,
                    data: slice_to_array(fragment_data, fragment_data.len()),
                    length: fragment_data.len() as u8,
                });
                break;
            }
        }
    } else {
        vec.push(Fragment {
            fragment_index: i,
            total_n_fragments: size,
            data: slice_to_array(data.as_slice(), last as usize),
            length: last as u8,
        });
    }
    vec
}

// Unit tests for the implemented functionality
pub enum DefaultsRequest {
    LOGIN,
    REGISTER,
    GETALLTEXT,
    GETALLMEDIALINKS,
    SETUNAVAILABLE,
    SETAVAILABLE,
    GETALLAVAILABLE,
}

impl Fragmentation<DefaultsRequest> for DefaultsRequest{
    fn fragment(message: DefaultsRequest) -> Vec<u8> {
        match message {
            DefaultsRequest::LOGIN => {
                vec![0]
            },
            DefaultsRequest::REGISTER => {
                vec![1]
            },
            DefaultsRequest::GETALLTEXT => {
                vec![2]
            },
            DefaultsRequest::GETALLMEDIALINKS => {
                vec![3]
            },
            DefaultsRequest::GETALLAVAILABLE => {
                vec![4]
            },
            DefaultsRequest::SETAVAILABLE => {
                vec![5]
            },
            DefaultsRequest::SETUNAVAILABLE => {
                vec![6]
            }
        }
    }
} 

impl Assembler<DefaultsRequest> for DefaultsRequest {
    fn assemble(fragments: &mut Vec<Fragment>) -> Result<DefaultsRequest, String> {
        if fragments.len()!=1{
            Err("Lenght of default requests must be 1".to_string())
        } else {
            match fragments[0].data[0] {
                0=>{
                    Ok(DefaultsRequest::LOGIN)
                },
                1=>{
                    Ok(DefaultsRequest::REGISTER)
                },
                2=>{
                    Ok(DefaultsRequest::GETALLTEXT)
                },
                3=>{
                    Ok(DefaultsRequest::GETALLMEDIALINKS)
                },
                4=>{
                    Ok(DefaultsRequest::GETALLAVAILABLE)
                },
                5=>{
                    Ok(DefaultsRequest::SETAVAILABLE)
                },
                6=>{
                    Ok(DefaultsRequest::SETUNAVAILABLE)
                },
                _=>{
                    Err("Default request identifier does not match".to_string())
                }
            }
        }
    }
}


#[cfg(test)]
mod test {

    use super::*;
    
    // Test string fragmentation
    #[test]
    fn test1() {
        let string = "hello".to_string();
        let ser = <String as Fragmentation<String>>::fragment(string);
        let ast = [104, 101, 108, 108, 111].to_vec();
        eprintln!("{:?}\n{:?}", ast, ser);
        assert_eq!(ast, ser);
    }

    // Test serialization of string fragments
    #[test]
    fn test2() {
        let string = "hello".to_string();
        let fra = <String as Fragmentation<String>>::fragment(string);

        let mut ast = [0; 128];
        ast[0] = 104;
        ast[1] = 101;
        ast[2] = 108;
        ast[3] = 108;
        ast[4] = 111;

        let fr = Fragment {
            fragment_index: 1,
            total_n_fragments: 1,
            length: 5,
            data: ast,
        };
        let ser = serialize(fra);
        eprintln!("{:?}\n{:?}", fr, ser);

        for f in ser {
            assert_eq!(f.data, fr.data);
            assert_eq!(f.fragment_index, fr.fragment_index);
            assert_eq!(f.length, fr.length);
            assert_eq!(f.total_n_fragments, fr.total_n_fragments);
        }
    }

    // Test assembly of string fragments
    #[test]
    fn test3() {
        let dd = <String as Fragmentation<String>>::fragment("Hello".to_string());
        let mut dis = serialize(dd);
        let ass = <String as Assembler<String>>::assemble(&mut dis);
        if let Ok(rs) = ass {
            assert_eq!("Hello".to_string(), rs)
        } else {
            eprintln!("{:?}", ass.err());
            assert_eq!(1, 2)
        }
    }

    // Test sorting of fragments by index
    #[test]
    fn test4() {
        let fr1 = Fragment::from_string(1, 2, "Hello".to_string());
        let fr2 = Fragment::from_string(2, 2, " World!\n".to_string());
        let fr3 = Fragment::from_string(3, 2, "Modefeckers!".to_string());
        let mut test_sub = vec![fr2, fr3, fr1];

        sort_by_fragment_index(&mut test_sub);
        for i in 1..test_sub.len() + 1 {
            assert_eq!(i, test_sub[i - 1].fragment_index as usize);
        }
    }

    #[test]
    fn test5 () {
        let img = image::open("/home/stefano/Downloads/drone_image.png").expect("Failed to open image");
        
        let mut frags = <image::DynamicImage as Fragmentation<image::DynamicImage>>::fragment(img.clone());
        let mut series = serialize(frags.clone());
        let assembly: Result<DynamicImage, String> = <DynamicImage as Assembler<DynamicImage>>::assemble(&mut series);
        if let Ok(ass)= assembly.clone(){
            println!("N_frag :{}\nDimension of reconstructed image{:?}\n Dim original h:{}__w:{}",frags.clone().len(),ass.dimensions(),img.height(),img.width());
        } else {
            println!("{:?}",assembly.clone().err());
        }
        assert_eq!(img,assembly.clone().ok().unwrap());
    }
}
