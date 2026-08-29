pub(crate) struct AdvisoryUnionFindV1 {
    parents: Vec<usize>,
    ranks: Vec<u8>,
}

impl AdvisoryUnionFindV1 {
    pub(crate) fn new(length: usize) -> Self {
        Self {
            parents: (0..length).collect(),
            ranks: vec![0; length],
        }
    }

    pub(crate) fn find(&mut self, mut item: usize) -> usize {
        let mut root = item;
        while self.parents[root] != root {
            root = self.parents[root];
        }
        while self.parents[item] != item {
            let parent = self.parents[item];
            self.parents[item] = root;
            item = parent;
        }
        root
    }

    pub(crate) fn union(&mut self, left: usize, right: usize) {
        let mut left_root = self.find(left);
        let mut right_root = self.find(right);
        if left_root == right_root {
            return;
        }
        if self.ranks[left_root] < self.ranks[right_root] {
            std::mem::swap(&mut left_root, &mut right_root);
        }
        self.parents[right_root] = left_root;
        if self.ranks[left_root] == self.ranks[right_root] {
            self.ranks[left_root] = self.ranks[left_root].saturating_add(1);
        }
    }
}
