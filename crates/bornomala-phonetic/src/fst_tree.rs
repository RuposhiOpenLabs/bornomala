use fst::raw::{Fst, Node};

#[derive(Clone)]
pub struct FstTree<D: AsRef<[u8]>> {
    fst: Fst<D>,
}

impl<D: AsRef<[u8]>> FstTree<D> {
    pub fn from_fst(data: D) -> Self {
        let fst = Fst::new(data).expect("Failed to initialize FST from binary bytes");
        Self { fst }
    }

    pub fn match_longest_common_prefix<'a>(&self, prefix: &'a str) -> (&'a str, &'a str, bool) {
        let mut iter = prefix.chars();
        let mut index = 0;
        let mut node = self.fst.root();

        while let Some(c) = iter.next() {
            match node.find_input(c as u8) {
                Some(addr) => {
                    node = self.fst.node(node.transition_addr(addr));
                    index += c.len_utf8();
                }
                None => break,
            }
        }

        let (matched_prefix, remaining) = prefix.split_at(index);
        (matched_prefix, remaining, node.is_final())
    }

    pub fn matching_node<'a>(&'a self, word: &str) -> Option<FstNode<'a, D>> {
        let mut iter = word.chars();
        let mut node = self.fst.root();

        while let Some(c) = iter.next() {
            match node.find_input(c as u8) {
                Some(addr) => {
                    node = self.fst.node(node.transition_addr(addr));
                }
                None => return None,
            }
        }

        Some(FstNode {
            fst: &self.fst,
            node,
            word: word.to_string(),
        })
    }
}

#[derive(Clone)]
pub struct FstNode<'a, D: AsRef<[u8]>> {
    fst: &'a Fst<D>,
    node: Node<'a>,
    word: String,
}

impl<'a, D: AsRef<[u8]>> FstNode<'a, D> {
    pub fn get_matching_node(&self, suffix: &str) -> Option<FstNode<'a, D>> {
        let mut iter = suffix.chars();
        let mut node = self.node;

        while let Some(c) = iter.next() {
            match node.find_input(c as u8) {
                Some(addr) => {
                    node = self.fst.node(node.transition_addr(addr));
                }
                None => return None,
            }
        }

        let word = self.word.clone() + suffix;
        Some(FstNode {
            fst: self.fst,
            node,
            word,
        })
    }

    pub fn get_word(self) -> Option<String> {
        if self.node.is_final() {
            Some(self.word)
        } else {
            None
        }
    }
}
