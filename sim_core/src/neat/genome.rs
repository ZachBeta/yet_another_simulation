use crate::config::Config as SimConfig;
use rand::{thread_rng, Rng, seq::SliceRandom};
use std::collections::HashMap;

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

use super::config::EvolutionConfig;

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
        let mut conn_map_f: HashMap<usize, &ConnGene> = fitter.conns.iter().map(|c| (c.innovation, c)).collect();
        let mut conn_map_w: HashMap<usize, &ConnGene> = weaker.conns.iter().map(|c| (c.innovation, c)).collect();
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config as SimConfig;

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
}
