mod web_server;

use wg_2024::{packet::*, network::*};
use std::collections::{HashMap,HashSet};
use std::string::ToString;
use crossbeam_channel::*;
use controller::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use wg_2024::network::NodeId;
use topology::*;

#[derive(Debug, Clone)]
pub struct Message<M: DroneSend> {
    pub source_id: NodeId,
    pub session_id: u64,
    pub content: M,
}

pub trait DroneSend: Serialize + DeserializeOwned {
    fn stringify(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    fn from_string(raw: String) -> Result<Self, String> {
        serde_json::from_str(raw.as_str()).map_err(|e| e.to_string())
    }
}

pub trait Request: DroneSend {}
pub trait Response: DroneSend {}

// ReqServerType,
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextRequest {
    TextList,
    Text(u64),
}

impl DroneSend for TextRequest {}
impl Request for TextRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaRequest {
    MediaList,
    Media(u64),
}

impl DroneSend for MediaRequest {}
impl Request for MediaRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatRequest {
    ClientList,
    Register(NodeId),
    SendMessage {
        from: NodeId,
        to: NodeId,
        message: String,
    },
}

impl DroneSend for ChatRequest {}
impl Request for ChatRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextResponse {
    TextList(Vec<u64>),
    Text(String),
    NotFound,
}

impl DroneSend for TextResponse {}
impl Response for TextResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaResponse {
    MediaList(Vec<u64>),
    Media(Vec<u8>), // should we use some other type?
}

impl DroneSend for MediaResponse {}
impl Response for MediaResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatResponse {
    ClientList(Vec<NodeId>),
    MessageFrom { from: NodeId, message: Vec<u8> },
    MessageSent,
}

impl DroneSend for ChatResponse {}
impl Response for ChatResponse {}

type Req=String;

pub trait Server {

    type RequestType: Request; //
    type ResponseType: Response;

    fn compose_message(
        source_id: NodeId,
        session_id: u64,
        raw_content: String
    ) -> Result<Message<Self::RequestType>, String> {
        let content = Self::RequestType::from_string(raw_content)?;
        Ok(Message {
            session_id,
            source_id,
            content,
        })
    }

    fn on_request_arrived(&mut self, source_id: NodeId, session_id: u64, raw_content: String) {
        if raw_content == "ServerType" {
            let _server_type = Self::get_sever_type();
            // send response
            return;
        }
        match Self::compose_message(source_id, session_id, raw_content) {
            Ok(message) => {
                let response = self.handle_request(message.content);
                self.send_response(response);
            }
            Err(str) => panic!("{}", str),
        }
    }

    fn send_response(&mut self, _response: Self::ResponseType) {
        // send response
    }

    fn handle_request(&mut self, request: Self::RequestType) -> Self::ResponseType;

    fn get_sever_type() -> ServerType;

}


// #[derive(Clone)]
// pub struct Server {
//     pub id: NodeId,
//     pub controller_send: Sender<NodeEvent>,
//     pub controller_recv: Receiver<NodeCommand>,
//     pub packet_recv: Receiver<Packet>,
//     pub packet_send: HashMap<NodeId, Sender<Packet>>,
//     pub flood_ids: HashSet<u64>,
// }
//
// impl Server {
//     pub fn run(&mut self) {
//         loop {
//             select_biased! {
//                 recv(self.controller_recv) -> command_res => {
//                     if let Ok(command) = command_res {
//                         match command {
//                             NodeCommand::SendPacket(p)=>{
//                                 self.packet_send.get(&p.routing_header.hops[p.routing_header.hop_index]).unwrap().send(p.clone());
//                             }
//                         }
//                     }
//                 },
//                 recv(self.packet_recv) -> packet_res => {
//                     if let Ok(mut packet) = packet_res {
//                         println!("Received:");
//                         println!();
//                         match packet.clone().pack_type {
//                             PacketType::FloodResponse(mut f)=>{
//                                 if f.path_trace.clone()[0].0!=self.id{
//                                     f.path_trace.reverse();
//                                     f.path_trace.push((self.id,NodeType::Client));
//                                     f.path_trace.reverse();
//                                     println!("Yeee {:?}",f.path_trace);
//                                 } else {
//                                     println!("Fuck");
//                                     println!("Yee {:?}",f.path_trace);
//                                 }
//                             },
//                             PacketType::FloodRequest(mut f)=>{
//                                 f.path_trace.push((self.id,NodeType::Server));
//                                 if let Some(_id) = self.flood_ids.get(&f.flood_id){
//                                     let mut packet_t: Packet = Packet {
//                                         pack_type: PacketType::FloodResponse(FloodResponse{
//                                             flood_id: f.flood_id,
//                                             path_trace: f.path_trace.clone(),
//                                         }),
//                                         routing_header: SourceRoutingHeader{
//                                             hop_index: 1,
//                                             hops: f.path_trace.clone().into_iter().map(|f| f.0).collect::<Vec<u8>>(),
//                                         },
//                                         session_id: packet.session_id,
//                                     };
//                                     packet_t.routing_header.hops.reverse();
//                                     if let Some(destination) = packet_t.routing_header.hops.last() {
//                                         if *destination != f.initiator_id {
//                                             packet_t.routing_header.hops.push(f.initiator_id);
//                                         }
//                                     }
//                                     let next_hop = packet_t.clone().routing_header.hops[packet_t.clone().routing_header.hop_index];
//                                     self.packet_send.get(&next_hop).unwrap().send(packet_t.clone());
//                                 }
//                                 else {
//                                     // f.path_trace.push((self.id,NodeType::Server));
//                                     self.flood_ids.insert(f.flood_id);
//                                     if self.packet_send.clone().len() > 1 {
//                                         let prev_hop = f.path_trace[f.path_trace.len()-2].0;
//                                         for send_to in self.packet_send.clone().into_iter(){
//                                             let sub ;
//                                             if f.path_trace[0].0 != f.initiator_id
//                                                 && f.path_trace.clone().len() < 2
//                                             {
//                                                 sub = 1;
//                                             } else {
//                                                 sub = 2;
//                                             }
//                                             if send_to.0
//                                             != f.path_trace.clone()[f.path_trace.clone().len() - sub].0{
//                                                 let packet_r = Packet {
//                                                     pack_type: PacketType::FloodRequest(FloodRequest{
//                                                         initiator_id: f.initiator_id,
//                                                         flood_id: f.flood_id,
//                                                         path_trace: f.path_trace.clone(),
//                                                     }),
//                                                     routing_header: SourceRoutingHeader {
//                                                         hop_index: 1,
//                                                         hops: [].to_vec(),
//                                                     },
//                                                     session_id:packet.clone().session_id,
//                                                 };
//
//                                                 send_to.1.send(packet_r.clone()).ok();
//                                             }
//                                         }
//                                     }
//                                     else {
//                                         let mut packet_t: Packet = Packet {
//                                         pack_type: PacketType::FloodResponse(FloodResponse{
//                                             flood_id: f.flood_id,
//                                             path_trace: f.path_trace.clone(),
//                                         }),
//                                         routing_header: SourceRoutingHeader{
//                                             hop_index: 1,
//                                             hops: f.path_trace.clone().into_iter().map(|f| f.0).collect::<Vec<u8>>(),
//                                         },
//                                         session_id: packet.session_id,
//                                         };
//                                         packet_t.routing_header.hops.reverse();
//                                         if let Some(destination) = packet_t.routing_header.hops.last() {
//                                             if *destination != f.initiator_id {
//                                                 packet_t.routing_header.hops.push(f.initiator_id);
//                                             }
//                                         }
//                                         let next_hop = packet_t.clone().routing_header.hops[packet_t.clone().routing_header.hop_index];
//
//                                         println!("{:?} {} {}",packet_t.routing_header.clone(),self.id, next_hop);
//
//                                         self.packet_send.get(&next_hop).unwrap().send(packet_t.clone());
//                                     }
//                                 }
//                             },
//                             PacketType::MsgFragment(m)=>{
//                                 println!("{:?}",m);
//                                 break;
//                             },
//                             _=>{}
//                         }
//                     }
//                 },
//             }
//         }
//     }
// }