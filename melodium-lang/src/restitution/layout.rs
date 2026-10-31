//! Layout of the connections of a treatment body.
//!
//! Connections form a graph whose nodes are the instanciated treatments, plus `Self` as the
//! source giving the treatment inputs and `Self` as the destination receiving its outputs.
//! The layout draws that graph as text, depending only on the graph and never on the order
//! connections were declared or added in:
//!
//! - each node gets a rank, the length of the longest path reaching it, cycles being broken
//!   at the node where they close;
//! - each node is entered once, through its anchor: the connection coming from its
//!   predecessor of highest rank, so that a node is drawn after everything it receives from;
//! - every connection is drawn under its source node, the first one without attributes
//!   continuing the line that entered the node (`a.x -> node.in,out -> b.y`), the others
//!   starting their own lines at the column of the node, all of them reaching their
//!   destinations on the same column;
//! - a node whose connections would not fit within the width starts its own lines instead.
//!
//! Drawn this way, and leaving width and attributes aside, a node stands at the hop of its
//! line matching its rank, and the number of lines is the lowest possible for lines crossing
//! each node at most once.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// End of a connection, from the point of view of the treatment holding it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Node {
    /// `Self` as source, giving the inputs of the treatment.
    Inputs,
    /// Instanciated treatment, by its name.
    Instance(String),
    /// `Self` as destination, receiving the outputs of the treatment.
    Outputs,
}

impl Node {
    fn name(&self) -> &str {
        match self {
            Node::Inputs | Node::Outputs => "Self",
            Node::Instance(name) => name,
        }
    }
}

/// Connection to lay out.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Link {
    pub source: Node,
    pub output: String,
    pub destination: Node,
    pub input: String,
    /// Attributes of the connection, ordered by name.
    pub attributes: Vec<(String, String)>,
}

/// Laid out connections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Layout {
    /// Lines of text, indentation and attributes included.
    pub lines: Vec<String>,
    /// Names of the instanciated treatments, in the order they first appear in lines.
    pub order: Vec<String>,
}

/// Lays out `links`, lines starting with `indent` spaces and kept within `width` characters
/// unless a single connection is longer on its own.
pub fn layout(mut links: Vec<Link>, indent: usize, width: usize) -> Layout {
    links.sort();
    let mut graph = Graph::new(links);
    graph.rank();
    graph.anchor();

    let mut painter = Painter {
        graph: &graph,
        indent,
        width,
        lines: Vec::new(),
        deferred: VecDeque::new(),
        order: Vec::new(),
        mentioned: BTreeSet::new(),
    };
    for root in graph.roots() {
        painter.group(&root, None, " ".repeat(indent));
        while let Some(node) = painter.deferred.pop_front() {
            painter.group(&node, None, " ".repeat(indent));
        }
    }

    Layout {
        lines: painter.lines,
        order: painter.order,
    }
}

struct Graph {
    links: Vec<Link>,
    nodes: BTreeSet<Node>,
    outgoing: BTreeMap<Node, Vec<usize>>,
    incoming: BTreeMap<Node, Vec<usize>>,
    /// Links closing cycles, left out of ranks and anchors.
    back: Vec<bool>,
    rank: BTreeMap<Node, usize>,
    /// Link entering each node.
    anchor: BTreeMap<Node, usize>,
}

impl Graph {
    fn new(links: Vec<Link>) -> Self {
        let mut nodes = BTreeSet::new();
        let mut outgoing: BTreeMap<Node, Vec<usize>> = BTreeMap::new();
        let mut incoming: BTreeMap<Node, Vec<usize>> = BTreeMap::new();
        for (index, link) in links.iter().enumerate() {
            nodes.insert(link.source.clone());
            nodes.insert(link.destination.clone());
            outgoing.entry(link.source.clone()).or_default().push(index);
            incoming
                .entry(link.destination.clone())
                .or_default()
                .push(index);
        }
        Self {
            back: vec![false; links.len()],
            links,
            nodes,
            outgoing,
            incoming,
            rank: BTreeMap::new(),
            anchor: BTreeMap::new(),
        }
    }

    fn outgoing(&self, node: &Node) -> &[usize] {
        self.outgoing.get(node).map(|l| l.as_slice()).unwrap_or(&[])
    }

    fn incoming(&self, node: &Node) -> &[usize] {
        self.incoming.get(node).map(|l| l.as_slice()).unwrap_or(&[])
    }

    /// Links entering `node` that are not closing a cycle.
    fn forward_incoming<'a>(&'a self, node: &Node) -> impl Iterator<Item = usize> + 'a {
        self.incoming(node)
            .iter()
            .copied()
            .filter(move |l| !self.back[*l])
    }

    /// Ranks every node, marking as back links the ones closing cycles.
    fn rank(&mut self) {
        loop {
            let mut progress = true;
            while progress {
                progress = false;
                for node in &self.nodes {
                    if self.rank.contains_key(node) {
                        continue;
                    }
                    let mut rank = Some(0);
                    for link in self.forward_incoming(node) {
                        rank = match (rank, self.rank.get(&self.links[link].source)) {
                            (Some(rank), Some(source_rank)) => Some(rank.max(source_rank + 1)),
                            _ => None,
                        };
                    }
                    if let Some(rank) = rank {
                        self.rank.insert(node.clone(), rank);
                        progress = true;
                    }
                }
            }

            let remaining = self
                .nodes
                .iter()
                .filter(|node| !self.rank.contains_key(node))
                .cloned()
                .collect::<Vec<_>>();
            if remaining.is_empty() {
                break;
            }

            let breaking = self.breaking_node(&remaining);
            for link in self.incoming(&breaking).to_vec() {
                if !self.rank.contains_key(&self.links[link].source) {
                    self.back[link] = true;
                }
            }
        }
    }

    /// Chooses the node at which to break the cycles among `remaining` nodes.
    ///
    /// The node is taken from a group of nodes all reaching each other that nothing else
    /// remaining reaches, so that the links marked as back links stay within that group.
    /// Within it, a node already reached from ranked nodes comes first, then the one
    /// receiving the most links from its own group (where its cycles close, as a server
    /// connection receiving the responses to the requests it gives), then the first by name.
    fn breaking_node(&self, remaining: &[Node]) -> Node {
        let remaining_set = remaining.iter().collect::<BTreeSet<_>>();
        let reach = remaining
            .iter()
            .map(|node| {
                let mut reached = BTreeSet::new();
                let mut stack = vec![node];
                while let Some(current) = stack.pop() {
                    for link in self.outgoing(current) {
                        let destination = &self.links[*link].destination;
                        if !self.back[*link]
                            && remaining_set.contains(destination)
                            && reached.insert(destination)
                        {
                            stack.push(destination);
                        }
                    }
                }
                (node, reached)
            })
            .collect::<BTreeMap<_, _>>();

        remaining
            .iter()
            .filter(|node| {
                remaining
                    .iter()
                    .all(|other| !reach[other].contains(node) || reach[node].contains(other))
            })
            .min_by_key(|node| {
                let entered = self
                    .forward_incoming(node)
                    .any(|link| self.rank.contains_key(&self.links[link].source));
                let closing = self
                    .forward_incoming(node)
                    .filter(|link| {
                        let source = &self.links[*link].source;
                        reach.get(source).map(|r| r.contains(node)).unwrap_or(false)
                            && reach[node].contains(source)
                    })
                    .count();
                (!entered, std::cmp::Reverse(closing), (*node).clone())
            })
            .cloned()
            .unwrap_or_else(|| remaining[0].clone())
    }

    /// Chooses the anchor of every node, as the link coming from its predecessor of highest
    /// rank, the first of them in order on equality.
    fn anchor(&mut self) {
        for node in &self.nodes {
            if *node == Node::Outputs {
                continue;
            }
            if let Some(link) = self.forward_incoming(node).max_by_key(|link| {
                (
                    self.rank[&self.links[*link].source],
                    std::cmp::Reverse(*link),
                )
            }) {
                self.anchor.insert(node.clone(), link);
            }
        }
    }

    /// Nodes starting lines: `Self` inputs, nodes receiving nothing, then nodes receiving only
    /// links closing cycles.
    fn roots(&self) -> Vec<Node> {
        let starting =
            |node: &&Node| !self.outgoing(node).is_empty() && !self.anchor.contains_key(*node);
        let mut roots = Vec::new();
        if !self.outgoing(&Node::Inputs).is_empty() {
            roots.push(Node::Inputs);
        }
        roots.extend(
            self.nodes
                .iter()
                .filter(starting)
                .filter(|node| **node != Node::Inputs && self.incoming(node).is_empty())
                .cloned(),
        );
        roots.extend(
            self.nodes
                .iter()
                .filter(starting)
                .filter(|node| !self.incoming(node).is_empty())
                .cloned(),
        );
        roots
    }
}

struct Painter<'a> {
    graph: &'a Graph,
    indent: usize,
    width: usize,
    lines: Vec<String>,
    /// Nodes whose connections start their own lines once the current root is drawn.
    deferred: VecDeque<Node>,
    order: Vec<String>,
    mentioned: BTreeSet<String>,
}

impl<'a> Painter<'a> {
    fn mention(&mut self, node: &Node) {
        if let Node::Instance(name) = node {
            if self.mentioned.insert(name.clone()) {
                self.order.push(name.clone());
            }
        }
    }

    /// Draws the links leaving `node`, whose name is written at the end of `line`.
    ///
    /// `input` is the input `node` is entered through on that line, none when the line starts
    /// with `node`, then repeated for every link it gives.
    fn group(&mut self, node: &Node, input: Option<&str>, line: String) {
        let graph = self.graph;
        let column = width(&line);
        let name = node.name();
        self.mention(node);

        let links = graph.outgoing(node);
        let continuation = input.and_then(|_| {
            links
                .iter()
                .copied()
                .find(|link| graph.links[*link].attributes.is_empty())
        });
        let head = |link: usize| match (input, Some(link) == continuation) {
            (Some(input), true) => format!("{name}.{input},{}", graph.links[link].output),
            _ => format!("{name}.{}", graph.links[link].output),
        };
        let destination = |link: usize| {
            let link = &graph.links[link];
            format!("{}.{}", link.destination.name(), link.input)
        };

        let heads_width = links.iter().map(|l| width(&head(*l))).max().unwrap_or(0);
        let destinations_column = column + heads_width + 4;

        if let Some(input) = input {
            let destinations_width = links
                .iter()
                .map(|l| width(&destination(*l)))
                .max()
                .unwrap_or(0);
            if links.is_empty()
                || (column > self.indent && destinations_column + destinations_width > self.width)
            {
                self.lines.push(format!("{line}{name}.{input}"));
                if !links.is_empty() {
                    self.deferred.push_back(node.clone());
                }
                return;
            }
            if continuation.is_none() {
                self.lines.push(format!("{line}{name}.{input}"));
            }
        }

        let arrow = |head: &str| {
            format!(
                " {}> ",
                "-".repeat(destinations_column - column - width(head) - 3)
            )
        };

        if let Some(link) = continuation {
            let head = head(link);
            self.hop(link, format!("{line}{head}{}", arrow(&head)));
        }
        let margin = " ".repeat(column);
        for link in links.iter().copied().filter(|l| Some(*l) != continuation) {
            for (name, attribute) in &graph.links[link].attributes {
                self.lines.push(format!("{margin}#[{name}({attribute})]"));
            }
            let head = head(link);
            self.hop(link, format!("{margin}{head}{}", arrow(&head)));
        }
    }

    /// Ends `line` with the destination of `link`, continuing through it if `link` enters it.
    fn hop(&mut self, link: usize, line: String) {
        let graph = self.graph;
        let destination = &graph.links[link].destination;
        let input = &graph.links[link].input;
        if graph.anchor.get(destination) == Some(&link) {
            self.group(destination, Some(input), line);
        } else {
            self.mention(destination);
            self.lines
                .push(format!("{line}{}.{input}", destination.name()));
        }
    }
}

fn width(text: &str) -> usize {
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{Script, Treatment};

    fn node(name: &str, inputs: bool) -> Node {
        match name {
            "Self" if inputs => Node::Inputs,
            "Self" => Node::Outputs,
            name => Node::Instance(name.to_string()),
        }
    }

    /// Links from `a.x -> b.y` notations, attributes given as `#name(value) a.x -> b.y`.
    fn links(connections: &[&str]) -> Vec<Link> {
        connections
            .iter()
            .map(|connection| {
                let (attributes, connection) = match connection.strip_prefix('#') {
                    Some(rest) => {
                        let (attribute, connection) = rest.split_once(' ').unwrap();
                        let (name, value) =
                            attribute.trim_end_matches(')').split_once('(').unwrap();
                        (vec![(name.to_string(), value.to_string())], connection)
                    }
                    None => (Vec::new(), *connection),
                };
                let (source, destination) = connection.split_once(" -> ").unwrap();
                let (source, output) = source.split_once('.').unwrap();
                let (destination, input) = destination.split_once('.').unwrap();
                Link {
                    source: node(source, true),
                    output: output.to_string(),
                    destination: node(destination, false),
                    input: input.to_string(),
                    attributes,
                }
            })
            .collect()
    }

    /// Links read back from `lines` by the parser of the language.
    fn parse(lines: &[String]) -> Vec<Link> {
        let script = Script::build(&format!("treatment t()\n{{\n{}\n}}\n", lines.join("\n")))
            .unwrap_or_else(|err| panic!("{err:?} in\n{}", lines.join("\n")));
        let treatment: &Treatment = &script.treatments[0];
        let mut links = treatment
            .connections
            .iter()
            .map(|connection| Link {
                source: node(&connection.name_start_point.string, true),
                output: connection.name_data_out.as_ref().unwrap().string.clone(),
                destination: node(&connection.name_end_point.string, false),
                input: connection.name_data_in.as_ref().unwrap().string.clone(),
                attributes: connection
                    .annotations
                    .as_ref()
                    .map(|annotations| {
                        let mut attributes = annotations
                            .annotations
                            .iter()
                            .filter_map(|annotation| annotation.as_attribute())
                            .collect::<Vec<_>>();
                        attributes.sort();
                        attributes
                    })
                    .unwrap_or_default(),
            })
            .collect::<Vec<_>>();
        links.sort();
        links
    }

    fn check(connections: &[&str], width: usize) -> Vec<String> {
        let links = links(connections);
        let layout = layout(links.clone(), 4, width);
        let mut expected = links.clone();
        expected.sort();
        assert_eq!(
            parse(&layout.lines),
            expected,
            "{}",
            layout.lines.join("\n")
        );

        let mut reversed = links;
        reversed.reverse();
        assert_eq!(layout, super::layout(reversed, 4, width));

        layout.lines
    }

    fn lines(text: &str) -> Vec<String> {
        text.lines().map(|l| l.to_string()).collect()
    }

    #[test]
    fn chain() {
        assert_eq!(
            check(
                &[
                    "parse.lines -> write.data",
                    "Self.data -> parse.text",
                    "write.done -> Self.result",
                ],
                100
            ),
            lines("    Self.data -> parse.text,lines -> write.data,done -> Self.result")
        );
    }

    #[test]
    fn fan_out() {
        assert_eq!(
            check(
                &[
                    "connection.data -> trigger.stream",
                    "trigger.start -> status.trigger",
                    "trigger.start -> headers.trigger",
                    "trigger.start -> logConn.trigger",
                    "status.emit -> Self.status",
                    "headers.emit -> Self.headers",
                ],
                100
            ),
            lines(
                "    connection.data -> trigger.stream,start -> headers.trigger,emit -> Self.headers
                       trigger.start --------> logConn.trigger
                       trigger.start --------> status.trigger,emit -> Self.status"
            )
        );
    }

    #[test]
    fn several_outputs() {
        assert_eq!(
            check(
                &["A.out -> B.in", "B.first -> C.in", "B.second -> D.in"],
                100
            ),
            lines(
                "    A.out -> B.in,first -> C.in
             B.second ---> D.in"
            )
        );
    }

    #[test]
    fn fan_in_enters_from_the_highest_rank() {
        assert_eq!(
            check(
                &[
                    "Self.text -> count.stream",
                    "count.count -> trigger.stream",
                    "trigger.last -> toBytes.value",
                    "toBytes.data -> stream.block",
                    "stream.stream -> write.data",
                    "trigger.end -> emit.trigger",
                    "emit.emit -> write.path",
                    "write.done -> Self.done",
                ],
                140
            ),
            lines(
                "    Self.text -> count.stream,count -> trigger.stream,end -> emit.trigger,emit -> write.path
                                       trigger.last -------> toBytes.value,data -> stream.block,stream -> write.data,done -> Self.done"
            )
        );
    }

    #[test]
    fn self_groups_are_aligned() {
        assert_eq!(
            check(
                &[
                    "Self.label -> log.label",
                    "Self.message -> log.message",
                    "step.started -> Self.started",
                    "step.error -> Self.error",
                    "Self.trigger -> step.trigger",
                ],
                100
            ),
            lines(
                "    Self.label ---> log.label
    Self.message -> log.message
    Self.trigger -> step.trigger,error -> Self.error
                    step.started -------> Self.started"
            )
        );
    }

    #[test]
    fn attributes_start_their_own_lines() {
        assert_eq!(
            check(
                &[
                    "Self.data -> a.in",
                    "#note(\"x\") a.out -> b.in",
                    "a.out -> c.in",
                    "#note(\"y\") Self.other -> c.other",
                ],
                100
            ),
            lines(
                "    Self.data --> a.in,out -> c.in
                  #[note(\"x\")]
                  a.out ----> b.in
    #[note(\"y\")]
    Self.other -> c.other"
            )
        );
    }

    #[test]
    fn cycles_start_where_they_close() {
        assert_eq!(
            check(
                &[
                    "startup.trigger -> log.trigger",
                    "connection.data -> trigger.stream",
                    "trigger.start -> status.trigger",
                    "status.emit -> connection.status",
                    "connection.data -> decode.data",
                    "decode.text -> encode.text",
                    "encode.data -> connection.data",
                ],
                100
            ),
            lines(
                "    startup.trigger -> log.trigger
    connection.data -> decode.data,text -> encode.text,data -> connection.data
    connection.data -> trigger.stream,start -> status.trigger,emit -> connection.status"
            )
        );
    }

    #[test]
    fn loops_enter_from_outside() {
        assert_eq!(
            check(
                &["Self.in -> b.in", "b.out -> c.in", "c.out -> b.again"],
                100
            ),
            lines("    Self.in -> b.in,out -> c.in,out -> b.again")
        );
    }

    #[test]
    fn width_moves_groups_to_their_own_lines() {
        assert_eq!(
            check(
                &[
                    "Self.data -> first.input",
                    "first.output -> second.input",
                    "second.output -> third.input",
                    "second.other -> fourth.input",
                ],
                60
            ),
            lines(
                "    Self.data -> first.input,output -> second.input
    second.other --> fourth.input
    second.output -> third.input"
            )
        );
    }

    #[test]
    fn self_to_self() {
        assert_eq!(
            check(&["Self.a -> Self.b", "Self.c -> x.c"], 100),
            lines(
                "    Self.a -> Self.b
    Self.c -> x.c"
            )
        );
    }

    /// Random graphs, possibly cyclic, with fan-ins, fan-outs and attributes, all read back
    /// identically whatever order their links are given in.
    #[test]
    fn random_graphs_read_back() {
        let mut seed: u64 = 0x5eed;
        let mut next = |bound: u64| {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (seed >> 33) % bound
        };
        for _ in 0..500 {
            let nodes = 2 + next(10);
            let count = 1 + next(25);
            let mut links = Vec::new();
            for _ in 0..count {
                let source = next(nodes + 1);
                let destination = next(nodes + 1);
                links.push(Link {
                    source: if source == nodes {
                        Node::Inputs
                    } else {
                        Node::Instance(format!("n{source}"))
                    },
                    output: format!("o{}", next(3)),
                    destination: if destination == nodes {
                        Node::Outputs
                    } else {
                        Node::Instance(format!("n{destination}"))
                    },
                    input: format!("i{}", next(3)),
                    attributes: if next(6) == 0 {
                        vec![("a".to_string(), "1".to_string())]
                    } else {
                        Vec::new()
                    },
                });
            }
            let width = 30 + next(90) as usize;
            let laid_out = layout(links.clone(), 4, width);
            let mut expected = links.clone();
            expected.sort();
            assert_eq!(
                parse(&laid_out.lines),
                expected,
                "{}",
                laid_out.lines.join("\n")
            );

            let mut shuffled = links;
            for i in (1..shuffled.len()).rev() {
                shuffled.swap(i, next(i as u64 + 1) as usize);
            }
            assert_eq!(laid_out, layout(shuffled, 4, width));
        }
    }
}
