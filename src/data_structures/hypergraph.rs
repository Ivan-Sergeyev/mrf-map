#![allow(dead_code)]

use std::fmt::Debug;

use log::debug;

pub trait Hypergraph<NodeData, HyperedgeData>
where
    NodeData: Debug,
    HyperedgeData: Debug,
{
    type NodeIndex;
    type HyperedgeIndex;

    fn new() -> Self;
    fn with_capacity(node_capacity: usize, hyperedge_capacity: usize) -> Self;
    fn from_node_data(data: Vec<NodeData>, hyperedge_capacity: usize) -> Self;

    fn add_node(&mut self, data: NodeData) -> Self::NodeIndex;
    fn add_hyperedge(
        &mut self,
        endpoints: Vec<Self::NodeIndex>,
        data: HyperedgeData,
    ) -> Self::HyperedgeIndex;

    fn num_nodes(&self) -> usize;
    fn nodes_iter(&self) -> impl Iterator<Item = Self::NodeIndex>;
    fn node_data(&self, node: Self::NodeIndex) -> &NodeData;
    fn node_data_mut(&mut self, node: Self::NodeIndex) -> &mut NodeData;
    fn incident_hyperedges(
        &self,
        node: Self::NodeIndex,
    ) -> impl Iterator<Item = Self::HyperedgeIndex>;

    fn num_hyperedges(&self) -> usize;
    fn hyperedges_iter(&self) -> impl Iterator<Item = Self::HyperedgeIndex>;
    fn hyperedge_endpoints(&self, hyperedge: Self::HyperedgeIndex) -> &Vec<Self::NodeIndex>;
    fn hyperedge_data(&self, hyperedge: Self::HyperedgeIndex) -> &HyperedgeData;
    fn hyperedge_data_mut(&mut self, hyperedge: Self::HyperedgeIndex) -> &HyperedgeData;
    // feature todo: check if a hyperedge already exists
}

pub struct Node<NodeData, HyperedgeIndex> {
    data: NodeData,
    adjacent_hyperedges: Vec<HyperedgeIndex>,
}

pub struct Hyperedge<HyperedgeData, NodeIndex> {
    data: HyperedgeData,
    endpoints: Vec<NodeIndex>,
}

pub struct UndirectedHypergraph<NodeData, HyperedgeData> {
    nodes: Vec<Node<NodeData, usize>>,
    hyperedges: Vec<Hyperedge<HyperedgeData, usize>>,
}

impl<NodeData: Debug, HyperedgeData: Debug> Hypergraph<NodeData, HyperedgeData>
    for UndirectedHypergraph<NodeData, HyperedgeData>
{
    type NodeIndex = usize;
    type HyperedgeIndex = usize;

    fn new() -> Self {
        UndirectedHypergraph {
            nodes: Vec::new(),
            hyperedges: Vec::new(),
        }
    }

    fn with_capacity(node_capacity: usize, hyperedge_capacity: usize) -> Self {
        UndirectedHypergraph {
            nodes: Vec::with_capacity(node_capacity),
            hyperedges: Vec::with_capacity(hyperedge_capacity),
        }
    }

    fn from_node_data(data: Vec<NodeData>, hyperedge_capacity: usize) -> Self {
        UndirectedHypergraph {
            nodes: data
                .into_iter()
                .map(|data| Node {
                    data,
                    adjacent_hyperedges: Vec::new(),
                })
                .collect(),
            hyperedges: Vec::with_capacity(hyperedge_capacity),
        }
    }

    fn num_nodes(&self) -> usize {
        self.nodes.len()
    }

    fn num_hyperedges(&self) -> usize {
        self.hyperedges.len()
    }

    fn hyperedge_endpoints(&self, hyperedge: usize) -> &Vec<usize> {
        &self.hyperedges[hyperedge].endpoints
    }

    fn node_data(&self, node: usize) -> &NodeData {
        &self.nodes[node].data
    }

    fn hyperedge_data(&self, hyperedge: usize) -> &HyperedgeData {
        &self.hyperedges[hyperedge].data
    }

    fn node_data_mut(&mut self, node: usize) -> &mut NodeData {
        &mut self.nodes[node].data
    }

    fn hyperedge_data_mut(&mut self, hyperedge: usize) -> &HyperedgeData {
        &mut self.hyperedges[hyperedge].data
    }

    fn add_node(&mut self, data: NodeData) -> usize {
        debug!("Add node with data {:?}", data);
        self.nodes.push(Node {
            data: data,
            adjacent_hyperedges: Vec::new(),
        });
        self.num_nodes() - 1
    }

    fn add_hyperedge(&mut self, endpoints: Vec<usize>, data: HyperedgeData) -> usize {
        debug!(
            "Add hyperedge with endpoints {:?} and data {:?}",
            endpoints, data
        );

        assert!(endpoints.iter().all(|node| *node < self.num_nodes()));
        let new_hyperedge_index = self.num_hyperedges();
        for endpoint in &endpoints {
            self.nodes[*endpoint]
                .adjacent_hyperedges
                .push(new_hyperedge_index);
        }
        self.hyperedges.push(Hyperedge { data, endpoints });
        new_hyperedge_index
    }

    fn nodes_iter(&self) -> impl Iterator<Item = usize> {
        0..self.num_nodes()
    }

    fn hyperedges_iter(&self) -> impl Iterator<Item = usize> {
        0..self.num_hyperedges()
    }

    fn incident_hyperedges(&self, node: Self::NodeIndex) -> impl Iterator<Item = usize> {
        self.nodes[node]
            .adjacent_hyperedges
            .iter()
            .map(|hyperedge| *hyperedge)
    }
}

#[cfg(test)]
mod tests {
    // todo: tests
}
