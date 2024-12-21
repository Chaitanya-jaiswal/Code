use wg_2024::{packet::*,network::*};
use std::collections::{HashMap,HashSet};
use crossbeam_channel::*;
use controller::*;

#[derive(Clone)]
pub struct Server {
    pub id: NodeId,
    pub controller_send: Sender<NodeEvent>,
    pub controller_recv: Receiver<NodeCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub flood_ids: HashSet<u64>,
}

impl Server {
    pub fn run(&mut self) {
        loop {
            select_biased! {
                recv(self.controller_recv) -> command_res => {
                    if let Ok(command) = command_res {
                        match command {
                            NodeCommand::SendPacket(p)=>{
                                self.packet_send.get(&p.routing_header.hops[p.routing_header.hop_index]).unwrap().send(p.clone());
                            }
                        }
                    }
                },
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(mut packet) = packet_res {
                        println!("Received:");
                        println!();
                        match packet.clone().pack_type {
                            PacketType::FloodResponse(mut f)=>{
                                if f.path_trace.clone()[0].0!=self.id{
                                    f.path_trace.reverse();
                                    f.path_trace.push((self.id,NodeType::Client));
                                    f.path_trace.reverse();
                                    println!("Yeee {:?}",f.path_trace);
                                } else {
                                    println!("Fuck");
                                    println!("Yee {:?}",f.path_trace);
                                }
                            },
                            PacketType::FloodRequest(mut f)=>{
                                f.path_trace.push((self.id,NodeType::Server));
                                if let Some(_id) = self.flood_ids.get(&f.flood_id){
                                    let mut packet_t: Packet = Packet {
                                        pack_type: PacketType::FloodResponse(FloodResponse{
                                            flood_id: f.flood_id,
                                            path_trace: f.path_trace.clone(),
                                        }),
                                        routing_header: SourceRoutingHeader{
                                            hop_index: 1,
                                            hops: f.path_trace.clone().into_iter().map(|f| f.0).collect::<Vec<u8>>(),
                                        },
                                        session_id: packet.session_id,
                                    };
                                    packet_t.routing_header.hops.reverse();
                                    if let Some(destination) = packet_t.routing_header.hops.last() {
                                        if *destination != f.initiator_id {
                                            packet_t.routing_header.hops.push(f.initiator_id);
                                        }
                                    }
                                    let next_hop = packet_t.clone().routing_header.hops[packet_t.clone().routing_header.hop_index];
                                    self.packet_send.get(&next_hop).unwrap().send(packet_t.clone());
                                }
                                else {
                                    // f.path_trace.push((self.id,NodeType::Server));
                                    self.flood_ids.insert(f.flood_id);
                                    if self.packet_send.clone().len() > 1 {
                                        let prev_hop = f.path_trace[f.path_trace.len()-2].0;
                                        for send_to in self.packet_send.clone().into_iter(){
                                            let sub ;
                                            if f.path_trace[0].0 != f.initiator_id
                                                && f.path_trace.clone().len() < 2
                                            {
                                                sub = 1;
                                            } else {
                                                sub = 2;
                                            }
                                            if send_to.0
                                            != f.path_trace.clone()[f.path_trace.clone().len() - sub].0{
                                                let packet_r = Packet {
                                                    pack_type: PacketType::FloodRequest(FloodRequest{
                                                        initiator_id: f.initiator_id,
                                                        flood_id: f.flood_id,
                                                        path_trace: f.path_trace.clone(),
                                                    }),
                                                    routing_header: SourceRoutingHeader {
                                                        hop_index: 1,
                                                        hops: [].to_vec(),
                                                    },
                                                    session_id:packet.clone().session_id,
                                                };

                                                send_to.1.send(packet_r.clone()).ok();
                                            }
                                        }
                                    }
                                    else {
                                        let mut packet_t: Packet = Packet {
                                        pack_type: PacketType::FloodResponse(FloodResponse{
                                            flood_id: f.flood_id,
                                            path_trace: f.path_trace.clone(),
                                        }),
                                        routing_header: SourceRoutingHeader{
                                            hop_index: 1,
                                            hops: f.path_trace.clone().into_iter().map(|f| f.0).collect::<Vec<u8>>(),
                                        },
                                        session_id: packet.session_id,
                                        };
                                        packet_t.routing_header.hops.reverse();
                                        if let Some(destination) = packet_t.routing_header.hops.last() {
                                            if *destination != f.initiator_id {
                                                packet_t.routing_header.hops.push(f.initiator_id);
                                            }
                                        }
                                        let next_hop = packet_t.clone().routing_header.hops[packet_t.clone().routing_header.hop_index];

                                        println!("{:?} {} {}",packet_t.routing_header.clone(),self.id, next_hop);

                                        self.packet_send.get(&next_hop).unwrap().send(packet_t.clone());
                                    }
                                }
                            },
                            PacketType::MsgFragment(m)=>{
                                println!("{:?}",m);
                                break;
                            },
                            _=>{}
                        }
                    }
                },
            }
        }
    }
}