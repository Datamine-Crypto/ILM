use ilm_core::base::words::answer_words;
use ilm_core::cases::Case;
use ilm_core::tools::call::Lessons;
use ilm_core::tools::graph::GRAPH_TOOL;

struct Mix {
    state: u64,
}

impl Mix {
    fn below(&mut self, n: usize) -> usize {
        self.state = (self.state * 1103515245 + 12345) % 2147483648;
        ((self.state >> 8) as usize) % n
    }
}

fn has_loop(edges: &[(usize, usize)], nodes: usize) -> bool {
    let mut parent: Vec<usize> = (0..nodes).collect();
    let top = |parent: &Vec<usize>, mut at: usize| {
        while parent[at] != at {
            at = parent[at];
        }
        at
    };
    for (one, other) in edges {
        let (first, second) = (top(&parent, *one), top(&parent, *other));
        if first == second {
            return true;
        }
        parent[first] = second;
    }
    false
}

fn graphs(count: usize, seed: u64) -> Vec<Case> {
    let mut mix = Mix { state: seed };
    let mut out = Vec::new();
    while out.len() < count {
        let nodes = 5 + mix.below(8);
        let wanted = 3 + mix.below(nodes);
        let mut edges: Vec<(usize, usize)> = Vec::new();
        while edges.len() < wanted {
            let (one, other) = (mix.below(nodes), mix.below(nodes));
            if one != other && !edges.contains(&(one, other)) && !edges.contains(&(other, one)) {
                edges.push((one, other));
            }
        }
        let mut sentences = vec![format!("The towns are numbered from 0 to {}, and the roads are:", nodes - 1)];
        sentences.extend(edges.iter().map(|(one, other)| format!("road {one} meets {other}")));
        let answer = if has_loop(&edges, nodes) { "Yes, a round trip exists." } else { "No round trip exists." };
        out.push(Case { sentences, question: "Can one drive a round trip?".to_string(), answer: answer.to_string() });
    }
    out
}

#[test]
fn graph_reads_a_loop_of_the_told_links() {
    let lessons = graphs(120, 9);
    let model = (GRAPH_TOOL.learn)(Lessons::of(&lessons), 1).expect("the stories hold links");
    let read = graphs(100, 41);
    let right = read.iter().filter(|case| model.answer(&case.sentences, &case.question).words == answer_words(&case.answer)).count();
    assert_eq!(right, read.len());
}

#[test]
fn graph_reads_no_kind_without_links() {
    let lessons: Vec<Case> = (0..30).map(|at| Case { sentences: vec![format!("Anna has {at} roads.")], question: "Can one drive a round trip?".to_string(), answer: "No round trip exists.".to_string() }).collect();
    assert!((GRAPH_TOOL.learn)(Lessons::of(&lessons), 1).is_err());
}
