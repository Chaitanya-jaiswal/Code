#![allow(dead_code)]
use std::collections::{HashMap, HashSet, VecDeque};

use toml::{self};
use wg_2024::{
    config::Config,
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::{NodeType, Packet},
};

trait Factory <T: Drone>{
    
    fn build(
        id: NodeId,
        command_recv: crossbeam_channel::Receiver<DroneCommand>,
        command_send: crossbeam_channel::Sender<DroneEvent>,
        packet_recv: crossbeam_channel::Receiver<Packet>,
        packet_send: HashMap<NodeId, crossbeam_channel::Sender<Packet>>,
        pdr: f32,
    ) -> T;
    fn check_neighbors_id(current: NodeId, neighbors: &Vec<NodeId>) -> bool;
    fn check_pdr(pdr: f32) -> bool;
    fn check_initializer(path_to_file: &str) -> bool;
}



enum DroneFactory
{
    BagelBomber,
    Rustafarian,
    DrOnes,
    RustEze,
    DRONE,
    GetDroned,
    NullPointerPatrol,
    BetterCallDrone,
    RustRoveri,
    CppEnjoyers,
}

fn build<T>(
    id: NodeId,
    command_recv: crossbeam_channel::Receiver<DroneCommand>,
    command_send: crossbeam_channel::Sender<DroneEvent>,
    packet_recv: crossbeam_channel::Receiver<Packet>,
    packet_send: HashMap<NodeId, crossbeam_channel::Sender<Packet>>,
    pdr: f32,
) -> T 
where T: Drone{
    T::new(
        id,
        command_send,
        command_recv,
        packet_recv,
        packet_send,
        pdr,
    )
}

fn check_neighbors_id(current: NodeId, neighbors: &Vec<NodeId>) -> bool {
    neighbors.into_iter().all(|f| *f != current)
        && (neighbors.iter().copied().collect::<HashSet<_>>().len() == neighbors.len())
}

fn check_pdr(pdr: f32) -> bool {
    pdr >= 0.0 && pdr <= 1.00
}

fn check_initializer(path_to_file: &str) -> bool {
    let config_data =
        std::fs::read_to_string(path_to_file).expect("Unable to read config file");
    // having our structs implement the Deserialize trait allows us to use the toml::from_str function to deserialize the config file into each of them
    let config: Config = toml::from_str(&config_data).expect("Unable to parse TOML");
    let mut current;
    let mut last = 0;
    let mut res = true;
    for drone in config.drone {
        current = drone.id;
        if check_neighbors_id(current, &drone.connected_node_ids) {
            if check_pdr(drone.pdr) {
                if current != last {
                    last = drone.id;
                } else {
                    res = false;
                }
            } else {
                res = false;
            }
        } else {
            res = false;
        }
    }
    if res {
        for client in config.client {
            current = client.id;
            if check_neighbors_id(current, &client.connected_drone_ids) {
                if current != last {
                    last = client.id;
                } else {
                    res = false;
                }
            } else {
                res = false;
            }
        }
        if res {
            for server in config.server {
                current = server.id;
                if check_neighbors_id(current, &server.connected_drone_ids) {
                    if current != last {
                        last = server.id;
                    } else {
                        res = false;
                    }
                } else {
                    res = false;
                }
            }
        }
    }
    res
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_init() {
        assert_eq!(
            check_initializer(
                "/home/stefano/Desktop/GoD/Code/Test/test_drone_2/config.toml"
            ),
            true
        );
    }

    #[test]
    fn test_pdr() {
        for pdr in 0..100 {
            assert_eq!(
                check_pdr((pdr / 100) as f32),
                true
            );
        }
    }

    #[test]
    fn test_neigbors() {
        let neighbors: Vec<u8> = [2, 3, 4, 5].to_vec();
        let neighbors_not: Vec<u8> = [2, 3, 4, 3].to_vec();
        assert_eq!(
            check_neighbors_id(1, &neighbors),
            true
        );
        assert_eq!(
            check_neighbors_id(4, &neighbors),
            false
        );
        assert_eq!(
            check_neighbors_id(1, &neighbors_not),
            false
        );
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Node {
    value: NodeId,
    node_type: NodeType, 
    // pdr: f32, //only if the type is drone.
    pub adjacents: Vec<(NodeId,NodeType)>,
}

impl Node {
    fn new(value: NodeId, node_type: NodeType)->Self{
        Self { value , node_type, adjacents: Vec::new() }
    }

    fn add_adjacents (&mut self,id: NodeId, node_type: NodeType) {
        if !self.adjacents.contains(&(id,node_type)){
            self.adjacents.push((id,node_type));
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Topology {
    nodes: HashMap<NodeId,Node>
}

impl Topology {
    fn new()->Self{
        Self { nodes: HashMap::new() }
    }

    fn update_topology(&mut self, initiator: (NodeId,NodeType), mut path_trace: Vec<(NodeId,NodeType)>){
        let mut path_trace_init = Vec::new(); 
        if !path_trace.contains(&initiator) {
            path_trace_init.push(initiator);
            path_trace_init.append(&mut path_trace);
            path_trace.append(&mut path_trace_init);
        } 
        let len = path_trace.len()-1;
        for value in 0..len+1 {
            if let Some(node) =self.nodes.get_mut(&path_trace[value].0){
                if value != len{
                    node.add_adjacents(path_trace[value+1].0,path_trace[value+1].1);
                }
                if value!=0{
                    node.add_adjacents(path_trace[value-1].0,path_trace[value-1].1);
                }
            } else {
                let mut node = Node::new(path_trace[value].0,path_trace[value].1);
                if value != len{
                    node.add_adjacents(path_trace[value+1].0,path_trace[value+1].1);
                }
                if value!=0{
                    node.add_adjacents(path_trace[value-1].0,path_trace[value-1].1);
                }
                self.nodes.insert(node.value,node);    
            }
        }
    }

    pub fn shortest_path(&self, src: NodeId, dst: NodeId) -> Option<Vec<NodeId>> {
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();
        let mut predecessors = HashMap::new();

        // Initialize BFS
        visited.insert(src, true);
        queue.push_back(src);

        while let Some(current) = queue.pop_front() {
            if current == dst {
                // Reconstruct the path
                let mut path = Vec::new();
                let mut node = dst;
                while let Some(&pred) = predecessors.get(&node) {
                    path.push(node);
                    node = pred;
                }
                path.push(src);
                path.reverse();
                return Some(path);
            }

            // Explore neighbors
            if let Some(node) = self.nodes.get(&current) {
                for &(neighbor, _) in &node.adjacents {
                    if !visited.contains_key(&neighbor) {
                        visited.insert(neighbor, true);
                        predecessors.insert(neighbor, current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        // If we reach here, no path was found
        None
    }


    pub fn all_shortest_paths(&self, src: NodeId, dst: NodeId) -> Vec<Vec<NodeId>> {
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();
        let mut predecessors = HashMap::new();
        let mut distances = HashMap::new();

        // Initialize BFS
        visited.insert(src, true);
        queue.push_back(src);
        distances.insert(src, 0);

        while let Some(current) = queue.pop_front() {
            if let Some(node) = self.nodes.get(&current) {
                for &(neighbor, _) in &node.adjacents {
                    // If the neighbor hasn't been visited or is at the same distance as the shortest path
                    let current_distance = distances[&current] + 1;
                    if !distances.contains_key(&neighbor) {
                        distances.insert(neighbor, current_distance);
                        queue.push_back(neighbor);
                    }

                    if distances[&neighbor] == current_distance {
                        predecessors
                            .entry(neighbor)
                            .or_insert_with(Vec::new)
                            .push(current);
                    }
                }
            }
        }

        // Reconstruct all paths from src to dst
        let mut paths = Vec::new();
        if distances.get(&dst).is_some() {
            let mut path = Vec::new();
            self.reconstruct_paths(&predecessors, &mut paths, &mut path, src, dst);
        }

        paths
    }

    /// Helper function to reconstruct all paths using DFS
    fn reconstruct_paths(
        &self,
        predecessors: &HashMap<NodeId, Vec<NodeId>>,
        paths: &mut Vec<Vec<NodeId>>,
        path: &mut Vec<NodeId>,
        src: NodeId,
        current: NodeId,
    ) {
        path.push(current);

        if current == src {
            let mut complete_path = path.clone();
            complete_path.reverse();
            paths.push(complete_path);
        } else if let Some(preds) = predecessors.get(&current) {
            for &pred in preds {
                self.reconstruct_paths(predecessors, paths, path, src, pred);
            }
        }

        path.pop();
    }
}





#[cfg(test)]
mod testq {
    use wg_2024::packet::NodeType;

    use crate::Topology;

    #[test]
    fn check_up_top ( ) {
        let path_trace = [(1,NodeType::Drone),(2,NodeType::Drone),(3,NodeType::Drone),(4,NodeType::Drone)].to_vec();
        let path_trace_1 = [(2,NodeType::Drone),(4,NodeType::Drone)].to_vec();
        
        let mut top = Topology::new();

        top.update_topology((0,NodeType::Client), path_trace);
        top.update_topology((0,NodeType::Client), path_trace_1);

        println!("{:?}",top);

        let mut top_test = Topology::new();
        top_test.nodes.insert(0, crate::Node { value: 0, node_type: NodeType::Client, adjacents: [(1,NodeType::Drone),(2,NodeType::Drone)].to_vec() });
        top_test.nodes.insert(1, crate::Node { value: 1, node_type: NodeType::Drone, adjacents: [(0,NodeType::Client),(2,NodeType::Drone)].to_vec() });
        top_test.nodes.insert(2, crate::Node { value: 2, node_type: NodeType::Drone, adjacents: [(0,NodeType::Client),(1,NodeType::Drone),(3,NodeType::Drone),(4,NodeType::Drone)].to_vec() });
        top_test.nodes.insert(3, crate::Node { value: 3, node_type: NodeType::Drone, adjacents: [(2,NodeType::Drone),(4,NodeType::Drone)].to_vec() });
        top_test.nodes.insert(4, crate::Node { value: 4, node_type: NodeType::Drone, adjacents: [(3,NodeType::Drone)].to_vec() });


        for nodes in top_test.nodes {
            let node  = top.nodes.get_mut(&nodes.0).unwrap();
            assert_eq!(nodes.0,node.value);
            assert_eq!(nodes.1.node_type,node.node_type);
            for adj in nodes.1.adjacents {
                assert_eq!(node.adjacents.contains(&adj),true);
            }
        }

        let paths = top.all_shortest_paths(0, 4);

        println!("{:?}",paths);
        assert_eq!([[1,2,3].to_vec()].to_vec(),paths);
    }
}

