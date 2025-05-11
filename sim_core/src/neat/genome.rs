use crate::config::Config as SimConfig;
use rand::{thread_rng, Rng, seq::SliceRandom};
use std::collections::HashMap;
use super::config::EvolutionConfig;
use super::onnx_exporter;

/// A node in the network
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    Input,
    Hidden,
    Output,
}

/// A node in the network
#[derive(Clone, Debug)]
pub struct NodeGene {
    pub id: usize,
    pub node_type: NodeType,
}

/// A connection with innovation number
#[derive(Clone, Debug)]
pub struct ConnGene {
    pub in_node: usize,
    pub out_node: usize,
    pub weight: f32,
    pub enabled: bool,
    pub innovation: usize,
}

/// A genome: lists of nodes & connections and its fitness
#[derive(Clone, Debug)]
pub struct Genome {
    pub nodes: Vec<NodeGene>,
    pub conns: Vec<ConnGene>,
    /// Accumulated fitness of this genome
    pub fitness: f32,
}

impl Genome {
    /// Create an initial minimal genome
    pub fn new() -> Self {
        Genome { nodes: Vec::new(), conns: Vec::new(), fitness: 0.0 }
    }

    /// Initialize as minimal fully-connected network
    pub fn initialize(&mut self, sim_cfg: &SimConfig, evo_cfg: &EvolutionConfig) {
        // inputs: [self_hp, self_shield] + per-enemy (dx,dy,hp,shield) + per-ally (dx,dy,hp,shield) + per-wreck (dx,dy,pool)
        let input_size = 2
            + sim_cfg.nearest_k_enemies * 4
            + sim_cfg.nearest_k_allies * 4
            + sim_cfg.nearest_k_wrecks * 3;
        let output_size = 3;
        self.nodes.clear();
        self.conns.clear();
        // input nodes
        for i in 0..input_size {
            self.nodes.push(NodeGene { id: i as usize, node_type: NodeType::Input });
        }
        // output nodes
        for j in 0..output_size {
            self.nodes.push(NodeGene { id: input_size as usize + j as usize, node_type: NodeType::Output });
        }
        // full connect inputs→outputs
        let mut rng = thread_rng();
        let mut innov = 0;
        for in_node in 0..input_size {
            for out_node in input_size..(input_size + output_size) {
                let w = rng.gen_range(-1.0..1.0);
                self.conns.push(ConnGene { in_node: in_node as usize, out_node: out_node as usize, weight: w, enabled: true, innovation: innov });
                innov += 1;
            }
        }
    }

    /// Mutate the genome by adding node or connection
    pub fn mutate(&mut self, cfg: &EvolutionConfig) {
        let mut rng = thread_rng();
        // Add connection mutation
        if rng.gen_bool(cfg.mutation_add_conn_rate as f64) {
            for _ in 0..100 {
                let in_gene = self.nodes.choose(&mut rng).unwrap();
                let out_gene = self.nodes.choose(&mut rng).unwrap();
                if in_gene.node_type == NodeType::Output
                    || out_gene.node_type == NodeType::Input
                    || in_gene.id == out_gene.id
                {
                    continue;
                }
                if self.conns.iter().any(|c| c.in_node == in_gene.id && c.out_node == out_gene.id) {
                    continue;
                }
                let innov = self.conns.len();
                let weight = rng.gen_range(-1.0..1.0);
                self.conns.push(ConnGene { in_node: in_gene.id, out_node: out_gene.id, weight, enabled: true, innovation: innov });
                break;
            }
        }
        // Add node mutation
        if rng.gen_bool(cfg.mutation_add_node_rate as f64) {
            // pick a random enabled connection to split
            let enabled_idxs: Vec<usize> = self.conns.iter().enumerate()
                .filter_map(|(i, c)| if c.enabled { Some(i) } else { None }).collect();
            if let Some(&idx) = enabled_idxs.choose(&mut rng) {
                // clone and disable the connection
                let old_conn = self.conns[idx].clone();
                self.conns[idx].enabled = false;
                // new hidden node
                let new_id = self.nodes.iter().map(|n| n.id).max().unwrap() + 1;
                self.nodes.push(NodeGene { id: new_id, node_type: NodeType::Hidden });
                // split connection into two
                let innov1 = self.conns.len();
                self.conns.push(ConnGene { in_node: old_conn.in_node, out_node: new_id, weight: 1.0, enabled: true, innovation: innov1 });
                let innov2 = self.conns.len();
                self.conns.push(ConnGene { in_node: new_id, out_node: old_conn.out_node, weight: old_conn.weight, enabled: true, innovation: innov2 });
            }
        }
    }

    /// Crossover two parents to produce a child
    pub fn crossover(
        parent1: &Genome,
        parent2: &Genome,
        cfg: &EvolutionConfig,
    ) -> Genome {
        let mut rng = thread_rng();
        // Determine fitter and weaker parents
        let (fitter, weaker) = if parent1.fitness >= parent2.fitness {
            (parent1, parent2)
        } else {
            (parent2, parent1)
        };
        let mut child = Genome::new();
        // Merge nodes
        let mut node_map: HashMap<usize, NodeGene> = HashMap::new();
        for n in &fitter.nodes {
            node_map.insert(n.id, n.clone());
        }
        for n in &weaker.nodes {
            node_map.entry(n.id).or_insert_with(|| n.clone());
        }
        child.nodes = node_map.values().cloned().collect();
        // Merge connections by innovation
        let conn_map_f: HashMap<usize, &ConnGene> = fitter.conns.iter().map(|c| (c.innovation, c)).collect();
        let conn_map_w: HashMap<usize, &ConnGene> = weaker.conns.iter().map(|c| (c.innovation, c)).collect();
        let mut all_innovs: Vec<usize> = conn_map_f.keys().cloned().chain(conn_map_w.keys().cloned()).collect();
        all_innovs.sort_unstable();
        all_innovs.dedup();
        for innov in all_innovs {
            if let Some(&g1) = conn_map_f.get(&innov) {
                if let Some(&g2) = conn_map_w.get(&innov) {
                    // Matching gene: randomly choose
                    if rng.gen_bool(0.5) {
                        child.conns.push(g1.clone());
                    } else {
                        child.conns.push(g2.clone());
                    }
                } else {
                    // Disjoint/excess from fitter
                    child.conns.push(g1.clone());
                }
            }
            // Genes only in weaker parent are skipped
        }
        child
    }

    /// Feed-forward evaluation given sensor inputs
    pub fn feed_forward(&self, inputs: &[f32]) -> Vec<f32> {
        // map input node values
        let mut values: HashMap<usize, f32> = HashMap::new();
        let mut input_nodes: Vec<&NodeGene> = self.nodes.iter().filter(|n| n.node_type == NodeType::Input).collect();
        input_nodes.sort_by_key(|n| n.id);
        assert_eq!(input_nodes.len(), inputs.len(), "Input length mismatch");
        for (n, &v) in input_nodes.iter().zip(inputs.iter()) {
            values.insert(n.id, v);
        }
        // hidden nodes
        let mut hidden_nodes: Vec<&NodeGene> = self.nodes.iter().filter(|n| n.node_type == NodeType::Hidden).collect();
        hidden_nodes.sort_by_key(|n| n.id);
        for n in hidden_nodes {
            let sum: f32 = self.conns.iter().filter(|c| c.enabled && c.out_node == n.id)
                .map(|c| values.get(&c.in_node).cloned().unwrap_or(0.0) * c.weight).sum();
            values.insert(n.id, sum.tanh());
        }
        // output nodes
        let mut output_nodes: Vec<&NodeGene> = self.nodes.iter().filter(|n| n.node_type == NodeType::Output).collect();
        output_nodes.sort_by_key(|n| n.id);
        let mut outputs = Vec::new();
        for n in output_nodes {
            let sum: f32 = self.conns.iter().filter(|c| c.enabled && c.out_node == n.id)
                .map(|c| values.get(&c.in_node).cloned().unwrap_or(0.0) * c.weight).sum();
            outputs.push(sum.tanh());
        }
        outputs
    }

    /// Decompose into strictly-layered structure: input->hidden?->output
    pub fn layers(&self) -> Vec<Layer> {
        // Collect node IDs by type
        let mut input_ids = self.nodes.iter().filter(|n| n.node_type == NodeType::Input).map(|n| n.id).collect::<Vec<_>>();
        let mut hidden_ids = self.nodes.iter().filter(|n| n.node_type == NodeType::Hidden).map(|n| n.id).collect::<Vec<_>>();
        let mut output_ids = self.nodes.iter().filter(|n| n.node_type == NodeType::Output).map(|n| n.id).collect::<Vec<_>>();
        input_ids.sort_unstable(); hidden_ids.sort_unstable(); output_ids.sort_unstable();
        let mut layers = Vec::new();
        if !hidden_ids.is_empty() {
            layers.push(Layer::new(&input_ids, &hidden_ids, &self.conns));
            layers.push(Layer::new(&hidden_ids, &output_ids, &self.conns));
        } else {
            layers.push(Layer::new(&input_ids, &output_ids, &self.conns));
        }
        layers
    }

    /// Number of inputs (first layer) for ONNX export
    pub fn input_size(&self) -> usize {
        self.layers().first().map(|l| l.input_size()).unwrap_or(0)
    }
    /// Number of outputs (last layer) for ONNX export
    pub fn output_size(&self) -> usize {
        self.layers().last().map(|l| l.output_size()).unwrap_or(0)
    }

    /// Export this genome to ONNX bytes
    pub fn to_onnx(&self) -> Vec<u8> {
        onnx_exporter::export_genome(self)
    }
} // end impl Genome

/// A strictly-layered feed-forward network layer
pub struct Layer {
    pub input_ids: Vec<usize>,
    pub output_ids: Vec<usize>,
    pub weights: Vec<f32>,  // row-major [out_dim, in_dim]
    pub biases: Vec<f32>,   // len = out_dim
}

impl Layer {
    /// Build a layer from node id lists and connections
    pub fn new(input_ids: &[usize], output_ids: &[usize], conns: &[ConnGene]) -> Self {
        let in_dim = input_ids.len();
        let out_dim = output_ids.len();
        let mut weights = vec![0.0f32; in_dim * out_dim];
        let biases = vec![0.0f32; out_dim]; // NEAT has no bias nodes
        for c in conns.iter().filter(|c| c.enabled
            && input_ids.contains(&c.in_node)
            && output_ids.contains(&c.out_node)) {
            let i = output_ids.iter().position(|&id| id == c.out_node).unwrap();
            let j = input_ids.iter().position(|&id| id == c.in_node).unwrap();
            weights[i * in_dim + j] = c.weight;
        }
        Layer { input_ids: input_ids.to_vec(), output_ids: output_ids.to_vec(), weights, biases }
    }
    pub fn input_size(&self) -> usize { self.input_ids.len() }
    pub fn output_size(&self) -> usize { self.output_ids.len() }
    pub fn weight_bytes(&self) -> Vec<u8> {
        self.weights.iter().flat_map(|f| f.to_le_bytes()).collect()
    }
    pub fn bias_bytes(&self) -> Vec<u8> {
        self.biases.iter().flat_map(|f| f.to_le_bytes()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config as SimConfig;
    use prost::Message;

    #[test]
    fn test_mutate_add_connection_and_node() {
        let sim_cfg = SimConfig::default();
        let mut evo_cfg = EvolutionConfig::default();
        evo_cfg.mutation_add_conn_rate = 1.0;
        evo_cfg.mutation_add_node_rate = 1.0;
        let mut genome = Genome::new();
        genome.initialize(&sim_cfg, &evo_cfg);
        let initial_nodes = genome.nodes.len();
        let initial_conns = genome.conns.len();
        genome.mutate(&evo_cfg);
        assert!(genome.nodes.len() > initial_nodes, "Node count did not increase");
        assert!(genome.conns.len() > initial_conns, "Conn count did not increase");
    }

    #[test]
    fn test_layers_direct() {
        let genome = Genome {
            nodes: vec![
                NodeGene { id: 0, node_type: NodeType::Input },
                NodeGene { id: 1, node_type: NodeType::Input },
                NodeGene { id: 2, node_type: NodeType::Output },
            ],
            conns: vec![
                ConnGene { in_node: 0, out_node: 2, weight: 1.23, enabled: true, innovation: 0 },
                ConnGene { in_node: 1, out_node: 2, weight: 4.56, enabled: true, innovation: 1 },
            ],
            fitness: 0.0,
        };
        let layers = genome.layers();
        assert_eq!(layers.len(), 1);
        let layer = &layers[0];
        assert_eq!(layer.input_ids, vec![0, 1]);
        assert_eq!(layer.output_ids, vec![2]);
        assert_eq!(layer.weights, vec![1.23, 4.56]);
        assert_eq!(layer.biases, vec![0.0]);
    }

    #[test]
    fn test_layers_with_hidden() {
        let genome = Genome {
            nodes: vec![
                NodeGene { id: 0, node_type: NodeType::Input },
                NodeGene { id: 1, node_type: NodeType::Hidden },
                NodeGene { id: 2, node_type: NodeType::Output },
            ],
            conns: vec![
                ConnGene { in_node: 0, out_node: 1, weight: 7.89, enabled: true, innovation: 0 },
                ConnGene { in_node: 1, out_node: 2, weight: 0.12, enabled: true, innovation: 1 },
            ],
            fitness: 0.0,
        };
        let layers = genome.layers();
        assert_eq!(layers.len(), 2);
        let l0 = &layers[0];
        assert_eq!(l0.input_ids, vec![0]);
        assert_eq!(l0.output_ids, vec![1]);
        assert_eq!(l0.weights, vec![7.89]);
        assert_eq!(l0.biases, vec![0.0]);
        let l1 = &layers[1];
        assert_eq!(l1.input_ids, vec![1]);
        assert_eq!(l1.output_ids, vec![2]);
        assert_eq!(l1.weights, vec![0.12]);
        assert_eq!(l1.biases, vec![0.0]);
    }

    #[test]
    fn test_export_to_onnx_simple() {
        let mut genome = Genome::new();
        let sim_cfg = SimConfig::default();
        let evo_cfg = EvolutionConfig::default();
        genome.initialize(&sim_cfg, &evo_cfg);
        let bytes = genome.to_onnx();
        assert!(!bytes.is_empty(), "ONNX output should not be empty");
        let model = prost::Message::decode(bytes.as_slice()).unwrap();
        assert_eq!(model, "neat_model");
    }
}
