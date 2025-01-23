use audio::AudioSource;
use codecs::png::PngDecoder;
use io::Reader;
use render::{render_resource::Extent3d, texture::ImageFormat};
use wg_2024::{packet::*,network::*};
use std::{collections::{HashMap,HashSet}, io::Cursor, mem::swap, ops::Deref, sync::Arc, thread, time::Duration};
use bevy::*;
use image::*;


///TODO::
/// -Specific comments on what the code does
/// -impl Fragmentation and Assembler for ContentRequest and ChatRequest
/// -Message structure with generic type T: Fragmentation + Assembler ??
/// -Review on auxiliary fuctions




// Trait to handle message fragmentation
//      Every impl Fragemntation has a diff recognition bit, that is the first element
//      of the vector of the message's bytes. It will be used then to be the first fragment
//      so that when reconstructing a message the types can be inferred.

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
}

// Function to check if all fragments are present
fn check_wholeness(fragments: &mut Vec<Fragment>) -> bool {
    let size = fragments[0].total_n_fragments; // Total number of fragments
    let mut count = 0;
    for i in 1..size+1 {
        count += i as u64; // Sum of expected fragment indices
    }
    let mut check_count = 0;
    for fr in fragments {
        check_count += fr.fragment_index; // Sum of actual fragment indices
    }
    check_count == count // Verify completeness
}

// Trait to assemble fragments into the original message
pub trait Assembler<T: Fragmentation<T>> {
    fn assemble(fragments: &mut Vec<Fragment>) -> Result<T, String>;
}

// Implementation of Fragmentation for String
impl Fragmentation<String> for String {
    fn fragment(message: String) -> Vec<u8> {
        let mut vec = [1].to_vec();
        vec.append(&mut message.into_bytes()); // Convert the string into bytes
        vec
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
                if fr.fragment_index != 1 { 
                    for i in 0..fr.length {
                        vec.push(fr.data[i as usize]); // Collect fragment data
                    }
                }
            }
            Ok(String::from_utf8(vec).expect("Something is wrong with the assembler")) // Reconstruct string
        }
    }
}

// Implementation of Fragmentation for Bevy's AudioSource
impl Fragmentation<bevy::audio::AudioSource> for AudioSource {
    fn fragment(message: bevy::audio::AudioSource) -> Vec<u8> {
        let mut vec = [2].to_vec();
        vec.append(&mut message.bytes.to_vec()); // Extract bytes from AudioSource
        vec
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
                if fr.fragment_index != 1 {
                    for i in 0..fr.length {
                        vec.push(fr.data[i as usize]); // Collect fragment data
                    }
                }
            }
            Ok(AudioSource { bytes: Arc::from(vec) }) // Create new AudioSource
        }
    }
}

// Implementation of Fragmentatio for images(for now just png)
impl Fragmentation<image::DynamicImage> for image::DynamicImage {
    fn fragment(message: image::DynamicImage) -> Vec<u8> {
        let mut vec = [3].to_vec();
        let mut data = Vec::new();
        message.write_to(&mut Cursor::new(&mut data), image::ImageFormat::Png).unwrap(); // Extract data from Image
        vec.append(&mut data);
        vec
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
            if fragment.fragment_index != 1 {
                combined_data.extend_from_slice(&fragment.data[..fragment.length as usize]);
            }
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

#[derive(Debug,PartialEq, Eq, Clone, Copy)]
pub enum DefaultsRequest {
    LOGIN,              //client logs to chat web_server
    REGISTER,           //client register to chat web_server
    GETALLTEXT,         //request all text file inside of content web_server
    GETALLMEDIALINKS,   //request all media links insede of content web_server
    SETUNAVAILABLE,     //set client unavailable for chatting inside of chat web_server
    SETAVAILABLE,       //set client available for chatting inside of chat web_server
    GETALLAVAILABLE,    //get all client available for chatting
}

impl Fragmentation<DefaultsRequest> for DefaultsRequest{
    fn fragment(message: DefaultsRequest) -> Vec<u8> {
        match message {
            DefaultsRequest::LOGIN => {
                vec![4,0]
            },
            DefaultsRequest::REGISTER => {
                vec![4,1]
            },
            DefaultsRequest::GETALLTEXT => {
                vec![4,2]
            },
            DefaultsRequest::GETALLMEDIALINKS => {
                vec![4,3]
            },
            DefaultsRequest::GETALLAVAILABLE => {
                vec![4,4]
            },
            DefaultsRequest::SETAVAILABLE => {
                vec![4,5]
            },
            DefaultsRequest::SETUNAVAILABLE => {
                vec![4,6]
            }
        }
    }
} 

impl Assembler<DefaultsRequest> for DefaultsRequest {
    fn assemble(fragments: &mut Vec<Fragment>) -> Result<DefaultsRequest, String> {
        if fragments.len()>2{
            Err("Lenght of default requests must be 2".to_string())
        } else {
            //match the second fragment first bit for recognition.
            match fragments[1].data[0] {
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


pub enum ContentRequest {
    GETTEXT(String),    //get specific text file, String is the path inside the assets directory
    GETMEDIA(String),   //get specific media, String is the path inside of the assets directory
}


pub enum ChatRequests<T: Fragmentation<T>+Assembler<T>> {
    SENDTO(NodeId,T),  //send to specific client to simulate chat behaviour
}
//Do we need content and chat responses?
//Or do we use session id for responding ?

fn slice_to_array(slice: &[u8], len: usize) -> [u8; 128] {
    let mut res: [u8; 128] = [0; 128];
    for i in 0..len {
        res[i] = slice[i];
    }
    res
}

// Serialize data into fragments
pub fn serialize(datas: Vec<u8>) -> Vec<Fragment> {
    let (f0, data) = datas.split_at(1);
    let len = data.len();
    let mut iter = data.chunks(128); // Split data into chunks of 128 bytes
    let mut vec = Vec::new();
    let mut size = ((len / 128)+1) as u64;
    let last = (len % 128) as u64;
    if last != 0 {
        size += 1; // Adjust total size for remaining data
    }

    let frag_0 = Fragment{
        fragment_index:1,
        total_n_fragments:size,
        data: slice_to_array(f0,1),
        length: 1
    };
    vec.push(frag_0);
    let mut i = 2;
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
            data: slice_to_array(data, last as usize),
            length: last as u8,
        });
    }
    vec
}

#[cfg(test)]
mod test {

    use super::*;
    
    // Test string fragmentation
    #[test]
    fn test1() {
        let string = "hello".to_string();
        let ser = <String as Fragmentation<String>>::fragment(string);
        let ast = [1,104, 101, 108, 108, 111].to_vec();
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
            fragment_index: 2,
            total_n_fragments: 2,
            length: 5,
            data: ast,
        };
        let ser = serialize(fra);
        eprintln!("{:?}\n{:?}", fr, ser);

        for f in ser {
            if f.fragment_index!=1 {
                assert_eq!(f.data, fr.data);
                assert_eq!(f.fragment_index, fr.fragment_index);
                assert_eq!(f.length, fr.length);
                assert_eq!(f.total_n_fragments, fr.total_n_fragments);
            }
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
        let fr0 = Fragment{fragment_index:1,total_n_fragments:4,length:128,data:[0;128]};
        let fr1 = Fragment::from_string(2, 4, "Hello".to_string());
        let fr2 = Fragment::from_string(3, 4, " World!\n".to_string());
        let fr3 = Fragment::from_string(4, 4, "Modefeckers!".to_string());
        let mut test_sub = vec![fr2, fr3, fr1, fr0];

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

    #[test]
    fn test6 () {
        let def_req = DefaultsRequest::LOGIN;
        let def_bytes = <DefaultsRequest as Fragmentation<DefaultsRequest>>::fragment(def_req);
        let mut def_frag = serialize(def_bytes);
        if def_frag[0].fragment_index == 1 && def_frag[0].data[0] == 4 {
            let assembly = <DefaultsRequest as Assembler<DefaultsRequest>>::assemble(&mut def_frag);
            if let Ok(res) = assembly.clone()  {
                println!("{:?}", res);
            } else {
                eprintln!("Something went wrong {:?}",assembly.clone().err());
            }
            assert_eq!(assembly.clone().ok().unwrap(),def_req);
        } else {
            eprintln!("Fragmentation went very wrong");
        }
    }
}
