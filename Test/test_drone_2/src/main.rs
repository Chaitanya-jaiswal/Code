#![allow(unused)]

use crossbeam_channel::{select_biased, unbounded, Receiver, Sender};
use game_of_drones::GameOfDrones;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::time::Duration;
use std::{fs, thread};
use wg_2024::config::Config;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::*;


struct SimulationController {
    droness: HashMap<NodeId,Vec<NodeId>>,
    drones: HashMap<NodeId, Sender<DroneCommand>>,
    node_event_recv: Receiver<DroneEvent>,

}

impl SimulationController {
    fn crash_all(&mut self, id: u8) {
        if let Some(res) = self.drones.get_key_value(&id){
            for s in self.droness.get(&id) {
                for ss in s{
                    self.drones.get(ss).unwrap().send(DroneCommand::RemoveSender(id));
                }
            }
            self.drones.get(&id).unwrap().send(DroneCommand::Crash);
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
    println!("{:?}",config.drone.clone());
    println!("{:?}",config.client.clone());

    let mut dd  = HashMap::new();
    let mut controller_drones = HashMap::new();
    let (node_event_send, node_event_recv) = unbounded();

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
    let mut handles_c= Vec::new();
    for drone in config.drone.into_iter() {
        // controller
        let (controller_drone_send, controller_drone_recv) = unbounded();
        controller_drones.insert(drone.id, controller_drone_send);
        let node_event_send = node_event_send.clone();
        // packet
        let packet_recv = packet_channels[&drone.id].1.clone();
        let packet_send = drone
            .connected_node_ids.clone()
            .into_iter()
            .map(|id| (id, packet_channels[&id].0.clone()))
            .collect();
        dd.insert(drone.id, drone.connected_node_ids.clone());
        handles.push(thread::spawn(move || {
            let mut drone = GameOfDrones::new( 
                drone.id,
                node_event_send,
                controller_drone_recv,
                packet_recv,
                packet_send,
                drone.pdr,
            );
            println!("{}  {:?}",drone.id,drone.packet_send.clone());

            drone.run();
        }));
    }


    // let mut clients = Vec::new();
    for drone in config.client.into_iter() {
        // controller
        let (controller_drone_send, controller_drone_recv) = unbounded();
        controller_drones.insert(drone.id, controller_drone_send);
        let node_event_send = node_event_send.clone();
        // packet
        let packet_recv = packet_channels[&drone.id].1.clone();
        let packet_send = drone
            .connected_drone_ids
            .into_iter()
            .map(|id| (id, packet_channels[&id].0.clone()))
            .collect();

        let mut drone = Client {
            id: drone.id,
            controller_recv: controller_drone_recv,
            controller_send: node_event_send,
            packet_recv,
            packet_send,
            flood_id: 1
        };
        handles_c.push(thread::spawn( || drone ));
    }

    // let mut servers = Vec::new();

    for drone in config.server.into_iter() {
        // controller
        let (controller_drone_send, controller_drone_recv) = unbounded();
        controller_drones.insert(drone.id, controller_drone_send);
        let node_event_send = node_event_send.clone();
        // packet
        let packet_recv = packet_channels[&drone.id].1.clone();
        let packet_send = drone
            .connected_drone_ids
            .into_iter()
            .map(|id| (id, packet_channels[&id].0.clone()))
            .collect();

        handles.push(thread::spawn(move ||  {
            let mut drone = Server {
                id: drone.id,
                controller_recv: controller_drone_recv,
                controller_send: node_event_send,
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
    };
    

    // let c1 = clients[0].clone();

    let res = Packet { 
        pack_type: PacketType::MsgFragment(Fragment { fragment_index: 1, total_n_fragments: 1, length: 1, data: [0;128] }), 
        routing_header: SourceRoutingHeader{hop_index:1,hops: [5,1,2,3].to_vec()}, 
        session_id: 1 };

    // if let Some(_r) = res {
        // println!("Sent Packet");
    // } else {
        // eprintln!("Error")
    // }

    thread::sleep(Duration::from_secs(2));
    
    // // let s1 = &servers[0];
    // let res1 = s1.packet_recv.recv_timeout(Duration::from_secs(1));
    // if let Ok(p) = res1{
    //     println!("{:?}",p);
    // } else {
    //     eprintln!("Fuck");
    // }

    
    // let res =Packet { 
    //     pack_type: PacketType::Ack(Ack{fragment_index:0}), 
    //     routing_header: SourceRoutingHeader{hop_index:1,hops: [4,3,2,6].to_vec()}, 
    //     session_id: 1 };

    
    

    match handles_c.pop() {
        Some(c)=>{
            match c.join() {
                Ok(mut cc)=>{
                    if cc.id == 5{
                        cc.packet_send.get(&1).unwrap().send(res);
                        println!("Waiting to receive");
                        thread::sleep(Duration::from_secs(3));
                        println!("Trying to receive");
                        cc.run();
                    } else {
                    }
                },
                _=>{}
            }
        },
        None=>{}
    }
    
    
    // Topology::flooding(clients[1].clone());

    // for v in &Topology::flooding_receiving(c1.clone()){
    //     println!("V:{:?}",v);
    // }

    //controller.crash_all(7);
    
    // match handles_c.pop() {
    //     Some(c)=>{
    //         match c.join() {
    //             Ok(mut cc)=>{
    //                 if cc.id == 5{
    //                     cc.packet_send.get(&1).unwrap().send(
    //                         Packet {
    //                             pack_type: PacketType::FloodRequest(FloodRequest {
    //                                 flood_id:1,
    //                                 initiator_id: cc.id,
    //                                 path_trace: [(5,NodeType::Client)].to_vec()
    //                             }),
    //                             routing_header: SourceRoutingHeader{
    //                                 hop_index:1,
    //                                 hops: Vec::new(),
    //                             },
    //                             session_id: 1
    //                         }
    //                     );
    //                     println!("Waiting to receive");
    //                     thread::sleep(Duration::from_secs(3));
    //                     println!("Trying to receive");
    //                     cc.run();
    //                 } else {
    //                 }
    //             },
    //             _=>{}
    //         }
    //     },
    //     None=>{}
    // }
     
    // while let Ok(p) = controller.node_event_recv.recv_timeout(Duration::from_secs(2)){
    //     match p {
    //         DroneEvent::PacketSent(pp) => {
    //             match pp.clone().pack_type{
    //                 PacketType::FloodResponse(f) => {
    //                     println!("{:?} mm",pp.clone()); 
    //                 },
    //                 _=>{}
    //             }
    //         },
    //         _=>{}
    //     }
        
    // }
    thread::sleep(Duration::from_secs(1));
    
    while let Some(handle) = handles.pop() {
        handle.join().unwrap();
    }
}

#[derive(Clone)]
pub struct Client { 
    pub id: NodeId,
    pub controller_send: Sender<DroneEvent>,
    pub controller_recv: Receiver<DroneCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub flood_id: u64
}

#[derive(Clone)]
pub struct Server {
    pub id: NodeId,
    pub controller_send: Sender<DroneEvent>,
    pub controller_recv: Receiver<DroneCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
    pub flood_ids: HashSet<u64>,
}


pub type NodeRef = Rc<RefCell<(NodeId,NodeType)>>;

pub struct Topology {
    topology: Vec<NodeRef>
}

impl Topology {
    pub fn flooding(c: Client){
            if let Ok(rr)= c.packet_send.get(&1).unwrap().send(Packet { 
                pack_type: PacketType::FloodRequest(FloodRequest{flood_id:1,initiator_id:c.id,path_trace: [(c.id,NodeType::Client)].to_vec()}),
                routing_header: SourceRoutingHeader { hop_index: 1, hops: [].to_vec() }, session_id: 1 
            }) {
                println!("FLOOD SENT");
            }
    }

    pub fn flooding_receiving(c: Client)->Vec<Vec<(u8,NodeType)>>{
        let mut vec = Vec::new();
        while let Some(r) = c.packet_recv.recv().ok(){
            match r.pack_type {
                PacketType::FloodResponse(f)=>{
                    vec.push(f.path_trace.clone());
                    println!("Yee");
                },
                _=>{println!("Wrong")}
            }
            println!("Loop");
            thread::sleep(Duration::from_secs(1));
            if c.packet_recv.is_empty(){
                break;
            }
        }
        println!("Loopnt");
        vec
    }
}

impl Client { 
    fn run (&mut self) {
        loop {
            loop {
                select_biased! {
                    recv(self.controller_recv) -> command_res => {
                        if let Ok(command) = command_res {
                            match command {
                                _=>{}
                            }
                        }
                    },
                    recv(self.packet_recv) -> packet_res => {
                        if let Ok(packet) = packet_res {
                            match packet.pack_type {
                                PacketType::FloodResponse(f)=>{
                                    println!("Yee {:?}",f.path_trace.clone());
                                },
                                PacketType::Nack(n)=>{
                                    println!("{:?}",n);
                                },
                                    _=>{println!("Wrong")}
                                }
                        }
                    },
                }
            }
        }
    }
}

impl Server {
    fn run(&mut self){
        loop {
            select_biased! {
                recv(self.controller_recv) -> command_res => {
                    if let Ok(command) = command_res {
                        match command {
                            _=>{}
                        }
                    }
                },
                recv(self.packet_recv) -> packet_res => {
                    if let Ok(packet) = packet_res {
                        match packet.clone().pack_type {
                            PacketType::FloodResponse(f)=>{
                                println!("Yee {:?}",f.path_trace.clone());
                            },
                            PacketType::FloodRequest(mut f)=>{
                                if let Some(_id) = self.flood_ids.get(&f.flood_id){
                                    f.path_trace.push((self.id,NodeType::Server));
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
                                    let next_hop = packet_t.clone().routing_header.hops[packet_t.clone().routing_header.hop_index];
                                    self.packet_send.get(&next_hop).unwrap().send(packet_t.clone());
                                }
                                else {
                                    f.path_trace.push((self.id,NodeType::Server));
                                    self.flood_ids.insert(f.flood_id);
                                    
                                    let prev_hop = f.path_trace[f.path_trace.len()-2].0;
                                    for send_to in self.packet_send.clone().into_iter(){
                                        if send_to.0 != prev_hop {
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

                                            send_to.1.send(packet_r.clone());
                                        }
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