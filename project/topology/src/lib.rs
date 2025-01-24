use std::{collections::{HashMap, HashSet, VecDeque}, fs, thread::{self, JoinHandle}};

use rand::*;
use serde::{Deserialize, Serialize};
use wg_2024::{
    config::Config,
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::{NodeType, Packet},
};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Node {
    value: NodeId,
    node_type: NodeType, 
    // pdr: f32, //only if the type is drone.
    pub adjacents: Vec<(NodeId,NodeType)>,
}
#[derive(Debug, Clone,Serialize,Deserialize)]
pub enum  ServerType{
    ChatServer,
    WebServer
}
impl Node {
    pub fn new(value: NodeId, node_type: NodeType)->Self{
        Self { value , node_type, adjacents: Vec::new() }
    }

    pub fn add_adjacents (&mut self,id: NodeId, node_type: NodeType) {
        if !self.adjacents.contains(&(id,node_type)){
            self.adjacents.push((id,node_type));
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Topology {
    nodes: HashMap<NodeId,Node>
}

impl Topology {
    pub fn new()->Self{
        Self { nodes: HashMap::new() }
    }

    pub fn update_topology(&mut self, initiator: (NodeId,NodeType), mut path_trace: Vec<(NodeId,NodeType)>){
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
        assert_eq!([[0,2,4].to_vec()].to_vec(),paths);
    }
}

