#![allow(unused)]

use crossbeam_channel::{select_biased, unbounded, Receiver, Sender};
use game_of_drones::GameOfDrones;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;
use std::{fs, thread};
use wg_2024::config::Config;
use wg_2024::controller::{DroneCommand, DroneEvent};
use wg_2024::drone::Drone;
use wg_2024::network::{NodeId, SourceRoutingHeader};
use wg_2024::packet::*;


struct SimulationController {
    drones: HashMap<NodeId, Sender<DroneCommand>>,
    node_event_recv: Receiver<DroneEvent>,
}

impl SimulationController {
    fn crash_all(&mut self) {
        for (_, sender) in self.drones.iter() {
            sender.send(DroneCommand::Crash).unwrap();
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
    for drone in config.drone.into_iter() {
        // controller
        let (controller_drone_send, controller_drone_recv) = unbounded();
        controller_drones.insert(drone.id, controller_drone_send);
        let node_event_send = node_event_send.clone();
        // packet
        let packet_recv = packet_channels[&drone.id].1.clone();
        let packet_send = drone
            .connected_node_ids
            .into_iter()
            .map(|id| (id, packet_channels[&id].0.clone()))
            .collect();

        handles.push(thread::spawn(move || {
            let mut drone = GameOfDrones::new( 
                drone.id,
                node_event_send,
                controller_drone_recv,
                packet_recv,
                packet_send,
                drone.pdr,
            );

            drone.run();
        }));
    }


    let mut clients = Vec::new();
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
        clients.push(drone);
    }

    let mut servers = Vec::new();

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

        let mut drone = Server {
            id: drone.id,
            controller_recv: controller_drone_recv,
            controller_send: node_event_send,
            packet_recv,
            packet_send,
        };

        servers.push(drone);
    }


    let mut controller = SimulationController {
        drones: controller_drones,
        node_event_recv,
    };
    

    let c1 = clients[0].clone();

    // let res = c1.packet_send.get(&3).unwrap().send(Packet { 
    //     pack_type: PacketType::Ack(Ack{fragment_index:0}), 
    //     routing_header: SourceRoutingHeader{hop_index:1,hops: [4,3,1,2,6].to_vec()}, 
    //     session_id: 1 }
    // ).ok();

    // if let Some(_r) = res {
    //     println!("Sent Packet");
    // } else {
    //     eprintln!("Error")
    // }

    // thread::sleep(Duration::from_secs(2));
    
    // let s1 = &servers[0];
    // let res1 = s1.packet_recv.recv_timeout(Duration::from_secs(1));
    // if let Ok(p) = res1{
    //     println!("{:?}",p);
    // } else {
    //     eprintln!("Fuck");
    // }

    
    // let res = c1.packet_send.get(&3).unwrap().send(Packet { 
    //     pack_type: PacketType::Ack(Ack{fragment_index:0}), 
    //     routing_header: SourceRoutingHeader{hop_index:1,hops: [4,3,2,6].to_vec()}, 
    //     session_id: 1 }
    // ).ok();

    // if let Some(_r) = res {
    //     println!("Sent Packet");
    // } else {
    //     eprintln!("Error")
    // }

    // let ress = s1.packet_recv.recv_timeout(Duration::from_secs(4)).ok();
    // if let Some(p) = ress{
    //     println!("{:?}",p);
    // } else {
    //     eprintln!("Fuck");
    // }

    
    
    Topology::flooding(clients[1].clone());

    for v in &Topology::flooding_receiving(c1.clone()){
        println!("V:{:?}",v);
    }

    while let Ok(p) = controller.node_event_recv.recv(){
        println!("{:?} mm",p);
        // 
    }
    thread::sleep(Duration::from_secs(1));
    
    controller.crash_all();
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
        while let Ok(r) = c.packet_recv.recv_timeout(Duration::from_secs(2)){
            match r.pack_type {
                PacketType::FloodResponse(f)=>{
                    println!("{:?} ",f);
                    vec.push(f.path_trace.clone());
                },
                _=>{println!("Wrong")}
            }
            println!("Loop");
        }
        println!("Loopnt");
        vec
    }
}
