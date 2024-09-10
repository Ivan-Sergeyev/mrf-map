#![allow(dead_code)]

/// todo: docs
pub trait Hypergraph<NodeData, HyperedgeData> {
    type NodeIndex;
    type HyperedgeIndex;

    fn new() -> Self;
    fn with_capacity(node_capacity: usize, hyperedge_capacity: usize) -> Self;
    fn from_node_data(node_data: Vec<NodeData>, hyperedge_capacity: usize) -> Self;

    fn num_nodes(&self) -> usize;
    fn num_hyperedges(&self) -> usize;

    fn hyperedge_endpoints(&self, hyperedge_idx: Self::HyperedgeIndex) -> &Vec<Self::NodeIndex>;

    fn node_data(&self, node_idx: Self::NodeIndex) -> &NodeData;
    fn hyperedge_data(&self, hyperedge_idx: Self::HyperedgeIndex) -> &HyperedgeData;

    fn node_data_mut(&mut self, node_idx: Self::NodeIndex) -> &mut NodeData;
    fn hyperedge_data_mut(&mut self, hyperedge_idx: Self::HyperedgeIndex) -> &HyperedgeData;

    fn add_node(&mut self, node_data: NodeData) -> Self::NodeIndex;
    fn add_hyperedge(
        &mut self,
        endpoints: Vec<Self::NodeIndex>,
        hyperedge_data: HyperedgeData,
    ) -> Self::HyperedgeIndex;

    fn iter_node_indices(&self) -> impl Iterator<Item = Self::NodeIndex>;
    fn iter_hyperedge_indices(&self) -> impl Iterator<Item = Self::HyperedgeIndex>;

    fn iter_incident_hyperedge_indices(
        &self,
        node_idx: Self::NodeIndex,
    ) -> impl Iterator<Item = Self::HyperedgeIndex>;

    // todo: check if a hyperedge already exists
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

impl<NodeData, HyperedgeData> Hypergraph<NodeData, HyperedgeData>
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

    fn from_node_data(node_data: Vec<NodeData>, hyperedge_capacity: usize) -> Self {
        UndirectedHypergraph {
            nodes: node_data
                .into_iter()
                .map(|data| Node {
                    data: data,
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

    fn hyperedge_endpoints(&self, hyperedge_idx: usize) -> &Vec<usize> {
        &self.hyperedges[hyperedge_idx].endpoints
    }

    fn node_data(&self, node_idx: usize) -> &NodeData {
        &self.nodes[node_idx].data
    }

    fn hyperedge_data(&self, hyperedge_idx: usize) -> &HyperedgeData {
        &self.hyperedges[hyperedge_idx].data
    }

    fn node_data_mut(&mut self, node_idx: usize) -> &mut NodeData {
        &mut self.nodes[node_idx].data
    }

    fn hyperedge_data_mut(&mut self, hyperedge_idx: usize) -> &HyperedgeData {
        &mut self.hyperedges[hyperedge_idx].data
    }

    fn add_node(&mut self, node_data: NodeData) -> usize {
        self.nodes.push(Node {
            data: node_data,
            adjacent_hyperedges: Vec::new(),
        });
        self.num_nodes() - 1
    }

    fn add_hyperedge(&mut self, endpoints: Vec<usize>, hyperedge_data: HyperedgeData) -> usize {
        assert!(endpoints.iter().all(|&node| node < self.num_nodes()));
        let new_hyperedge_index = self.num_hyperedges();
        for &node in &endpoints {
            self.nodes[node]
                .adjacent_hyperedges
                .push(new_hyperedge_index);
        }
        self.hyperedges.push(Hyperedge {
            data: hyperedge_data,
            endpoints: endpoints,
        });
        new_hyperedge_index
    }

    fn iter_node_indices(&self) -> impl Iterator<Item = usize> {
        0..self.num_nodes()
    }

    fn iter_hyperedge_indices(&self) -> impl Iterator<Item = usize> {
        0..self.num_hyperedges()
    }

    fn iter_incident_hyperedge_indices(
        &self,
        node_idx: Self::NodeIndex,
    ) -> impl Iterator<Item = usize> {
        self.nodes[node_idx]
            .adjacent_hyperedges
            .iter()
            .map(|&hyperedge_idx| hyperedge_idx)
    }
}

#[cfg(test)]
mod tests {
    // todo: tests
}
