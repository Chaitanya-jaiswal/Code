#![allow(unused)]

use crossbeam_channel::{select_biased, unbounded, Receiver, Sender};
use rand::Rng;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Duration;
use std::{fs, thread};
use wg_2024::config::Config;
use wg_2024::controller::{self, DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::*;


///Beware thy who enter: Highly repetitive code 
pub enum NodeCommand {
    SendPacket(Packet),
}

pub enum NodeEvent {
    SentPacket(Packet),
}

struct SimulationController {
    droness: HashMap<NodeId, Vec<NodeId>>,
    drones: HashMap<NodeId, Sender<DroneCommand>>,
    node_event_recv: Receiver<DroneEvent>,
    cli_ser_send: HashMap<NodeId,Sender<NodeCommand>>,
    cli_ser_recv: Receiver<NodeEvent>,
}


impl SimulationController {
    fn crash_all(&mut self, id: u8) {
        if let Some(res) = self.drones.get(&id) {
            match res.send(DroneCommand::Crash) {
                Ok(_)=>{
                    println!("Crashead");
                },
                Err(_)=>{
                    println!("Crashead");
                }
            }
            for s in self.droness.get(&id).unwrap() {
                println!("{}",s);
                match self.drones
                    .get(s)
                    .unwrap()
                    .send(DroneCommand::RemoveSender(id)) 
                {
                    Ok(p)=>{
                        println!("Sent remove_id to {}",s);
                    },
                    Err(_e)=>{
                        println!("remove_id to {} error",s);
                    }
                }
            }
        } else {
            println!("Drone does not exist");
        }
    }
}

fn parse_config(file: &str) -> Config {
    let file_str = fs::read_to_string(file).unwrap();
    toml::from_str(&file_str).unwrap()
}

fn main() {
    let config = parse_config("config.toml");
    println!("{:?}", config.drone.clone());
    println!("{:?}", config.client.clone());

    let mut dd = HashMap::new();
    let mut controller_drones = HashMap::new();
    let (node_event_send, node_event_recv) = unbounded();
    let mut cs_controller = HashMap::new();
    let (cs_send, cs_recv)=unbounded::<NodeEvent>();

    let mut packet_channels = HashMap::new();
    for drone in config.drone.iter() {
        packet_channels.insert(drone.id, unbounded());
    }
    for client in config.client.iter() {
        packet_channels.insert(client.id, unbounded());
    }
    for server in config.server.iter() {
        packet_channels.insert(server.id, unbounded());
    }

    let mut handles = Vec::new();
    
    // let mut handles_c = Vec::new();
    for drone in config.drone.into_iter() {
        let mut rng = rand::thread_rng();
        let val = rng.gen_range(0..8);
        // controller
        let (controller_drone_send, controller_drone_recv) = unbounded();
        controller_drones.insert(drone.id, controller_drone_send);
        let node_event_send = node_event_send.clone();
        // packet
        let packet_recv = packet_channels[&drone.id].1.clone();
        let packet_send = drone
            .connected_node_ids
            .clone()
            .into_iter()
            .map(|id| (id, packet_channels[&id].0.clone()))
            .collect();
        dd.insert(drone.id, drone.connected_node_ids.clone());
        handles.push(thread::spawn(move || {

            
            // let mut drone = null_pointer_drone::MyDrone::new(
            //     drone.id,
            //     node_event_send,
            //     controller_drone_recv,
            //     packet_recv,
            //     packet_send,
            //     drone.pdr,
            // );
            let val = 7;
            match val {
                0=>{
                    println!("BagelBomber Id[{}]",drone.id);
                    let mut drone = build_on_imls::<bagel_bomber::BagelBomber>(drone.id, node_event_send, controller_drone_recv, packet_recv, packet_send, drone.pdr);
                    drone.run();
                },
                1=>{
                    println!("BetteCallDrone Id[{}]",drone.id);
                    let mut drone = build_on_imls::<drone_bettercalldrone::BetterCallDrone>(drone.id, node_event_send, controller_drone_recv, packet_recv, packet_send, drone.pdr);
                    drone.run();
                },
                2=>{
                    println!("RustRoveri Id[{}]",drone.id);
                    let mut drone = build_on_imls::<rust_roveri::drone::RustRoveri>(drone.id, node_event_send, controller_drone_recv, packet_recv, packet_send, drone.pdr);
                    drone.run();
                },
                3=>{
                    println!("GetDroned Id[{}]",drone.id);
                    let mut drone = build_on_imls::<getdroned::GetDroned>(drone.id, node_event_send, controller_drone_recv, packet_recv, packet_send, drone.pdr);
                    drone.run();
                },
                4=>{
                    println!("C++Enjoyers Id[{}]",drone.id);
                    let mut drone = build_on_imls::<ap2024_unitn_cppenjoyers_drone::CppEnjoyersDrone>(drone.id, node_event_send, controller_drone_recv, packet_recv, packet_send, drone.pdr);
                    drone.run();
                },
                5=>{
                    println!("D.R.O.N.E Id[{}]",drone.id);
                    let mut drone = build_on_imls::<d_r_o_n_e_drone::MyDrone>(drone.id, node_event_send, controller_drone_recv, packet_recv, packet_send, drone.pdr);
                    drone.run();
                },
                6=>{
                    println!("NNP Id[{}]",drone.id);
                    let mut drone = build_on_imls::<null_pointer_drone::MyDrone>(drone.id, node_event_send, controller_drone_recv, packet_recv, packet_send, drone.pdr);
                    drone.run();
                },
                7=>{
                    println!("Rustafarian Id[{}]",drone.id);
                    let mut drone = build_on_imls::<rustafarian_drone::RustafarianDrone>(drone.id, node_event_send, controller_drone_recv, packet_recv, packet_send, drone.pdr);
                    drone.run();
                },
                8=>{
                    println!("GameOfDrones Id[{}]",drone.id);
                    let mut drone = build_on_imls::<game_of_drones::GameOfDrones>(drone.id, node_event_send, controller_drone_recv, packet_recv, packet_send, drone.pdr);
                    drone.run();
                },
                _=>{
                    println!("BetterCallDrone2 Id[{}]",drone.id);
                    let mut drone = build_on_imls::<drone_bettercalldrone::BetterCallDrone>(drone.id, node_event_send, controller_drone_recv, packet_recv, packet_send, drone.pdr);
                    drone.run();
                }
            }

            // println!("{}  {:?}", drone.id, drone.packet_send.clone());

            // wg_2024::drone::Drone::run(&mut drone);
        }));
    }

    // let mut clients = Vec::new();
    for drone in config.client.into_iter() {
        // controller
        let (controller_drone_send, controller_drone_recv) = unbounded::<NodeCommand>();
        cs_controller.insert(drone.id, controller_drone_send);
        let cs_send = cs_send.clone();
        // packet
        let packet_recv = packet_channels[&drone.id].1.clone();
        let packet_send = drone
            .connected_drone_ids
            .into_iter()
            .map(|id| (id, packet_channels[&id].0.clone()))
            .collect();

        
        handles.push(thread::spawn(move|| {
            let mut drone = Client {
                id: drone.id,
                controller_recv: controller_drone_recv,
                controller_send: cs_send,
                packet_recv,
                packet_send,
                flood_ids: HashSet::new(),
            };
            drone.run();
        }));
    }

    // let mut servers = Vec::new();

    for drone in config.server.into_iter() {
        // controller
        let (controller_drone_send, controller_drone_recv) = unbounded();
        cs_controller.insert(drone.id, controller_drone_send);
        let cs_send = cs_send.clone();
        // packet
        let packet_recv = packet_channels[&drone.id].1.clone();
        let packet_send = drone
            .connected_drone_ids
            .into_iter()
            .map(|id| (id, packet_channels[&id].0.clone()))
            .collect();

        handles.push(thread::spawn(move || {
            let mut drone = Server {
                id: drone.id,
                controller_recv: controller_drone_recv,
                controller_send: cs_send,
                packet_recv,
                packet_send,
                flood_ids: HashSet::new(),
            };
            drone.run();
        }));
    }

    let mut controller = SimulationController {
        droness: dd,
        drones: controller_drones,
        node_event_recv,
        cli_ser_send: cs_controller,
        cli_ser_recv: cs_recv,
    };

    // controller.cli_ser_send.get(&5).unwrap().send(NodeCommand::SendPacket(Packet::new_flood_request(SourceRoutingHeader::with_first_hop([5,1].to_vec()), 1, FloodRequest::initialize(1, 5, NodeType::Client))));
    // thread::sleep(Duration::from_secs(4));
    // controller.cli_ser_send.get(&6).unwrap().send(NodeCommand::SendPacket(Packet::new_flood_request(SourceRoutingHeader::with_first_hop([6,2].to_vec()), 2, FloodRequest::initialize(2, 6, NodeType::Server))));
    // thread::sleep(Duration::from_secs(4));
    // 
    controller.cli_ser_send.get(&5).unwrap().send(NodeCommand::SendPacket(Packet::new_fragment(SourceRoutingHeader::with_first_hop([5,1,4,2,6].to_vec()),3, Fragment::from_string(1, 10, "Hello_World_!".to_string()))));
    thread::sleep(Duration::from_secs(4));
    
    controller.crash_all(4);
    
    thread::sleep(Duration::from_secs(4));
    controller.cli_ser_send.get(&5).unwrap().send(NodeCommand::SendPacket(Packet::new_fragment(SourceRoutingHeader::with_first_hop([5,1,4,2,6].to_vec()),3, Fragment::from_string(1, 10, "Hello_World_!".to_string()))));
    
    match controller.node_event_recv.recv() {
        Ok(e)=>{
            match e {
                DroneEvent::PacketSent(epi)=>{
                 //   println!("{:?}",epi)
                },
                _=>{}
            }
        },
        _=>{}
    }

    while let Some(handle ) = handles.pop() {
        handle.join();
    }

}

#[derive(Clone)]
pub struct Client {
    pub id: NodeId,
    pub controller_send: Sender<NodeEvent>,
    pub controller_recv: Receiver<NodeCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub flood_ids: HashSet<u64>,
}

#[derive(Clone)]
pub struct Server {
    pub id: NodeId,
    pub controller_send: Sender<NodeEvent>,
    pub controller_recv: Receiver<NodeCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub flood_ids: HashSet<u64>,
}

impl Client {
    fn run(&mut self) {
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
                        println!("Received:");
                        println!();
                        thread::sleep(Duration::from_secs(1));
                        if let Ok(packet) = packet_res {
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
                                PacketType::Nack(n)=>{
                                    println!("{:?} to Client {}",n,self.id);
                                },
                                PacketType::Ack(_)=>{println!("Ack")},
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
                                _=>{}
                                }
                        }
                    },
                }
            }
        }
}

impl Server {
    fn run(&mut self) {
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


fn build_on_imls<T:Drone> (id: NodeId, controller_send:Sender<DroneEvent> , controller_recv: Receiver<DroneCommand>,
                  packet_recv: Receiver<Packet>, packet_send: HashMap<u8, Sender<Packet>>, pdr: f32)->T {
    T::new(id, controller_send, controller_recv, packet_recv, packet_send, pdr)
}