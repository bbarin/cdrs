use std::sync::Mutex;

use super::LoadBalancingStrategy;
use std::borrow::Borrow;

pub struct RoundRobinSync<N> {
    cluster: Vec<N>,
    prev_idx: Mutex<usize>,
}

impl<N> RoundRobinSync<N> {
    pub fn new() -> Self {
        RoundRobinSync {
            prev_idx: Mutex::new(0),
            cluster: vec![],
        }
    }
}

impl<N> From<Vec<N>> for RoundRobinSync<N> {
    fn from(cluster: Vec<N>) -> RoundRobinSync<N> {
        RoundRobinSync {
            prev_idx: Mutex::new(0),
            cluster: cluster,
        }
    }
}

impl<N> LoadBalancingStrategy<N> for RoundRobinSync<N> {
    fn init(&mut self, cluster: Vec<N>) {
        self.cluster = cluster;
    }

    /// Returns next node from a cluster
    fn next(&self) -> Option<&N> {
        let mut prev_idx = self.prev_idx.lock();
        if let Ok(ref mut mtx) = prev_idx {
            let next_idx = (**mtx + 1) % self.cluster.len();
            **mtx = next_idx;
            self.cluster.get(next_idx)
        } else {
            return None;
        }
    }

    fn get_all_nodes(&self) -> &Vec<N> {
        self.cluster.borrow()
    }

    fn remove_node<F>(&mut self, filter: F)
    where
        F: FnMut(&N) -> bool,
    {
        if let Some(i) = self.cluster.iter().position(filter) {
            self.cluster.remove(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn next_round_robin() {
        let nodes = vec!["a", "b", "c"];
        let nodes_c = nodes.clone();
        let load_balancer = RoundRobinSync::from(nodes);
        for i in 0..10 {
            assert_eq!(&nodes_c[(i + 1) % 3], load_balancer.next().unwrap());
        }
    }

    #[test]
    fn remove_from_round_robin() {
        let nodes = vec!["a", "b"];
        let mut load_balancer = RoundRobinSync::from(nodes);
        assert_eq!(&"b", load_balancer.next().unwrap());

        load_balancer.remove_node(|n| n == &"a");
        assert_eq!(&"b", load_balancer.next().unwrap());
    }
}
