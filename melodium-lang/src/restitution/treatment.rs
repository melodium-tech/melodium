use super::{describe_type, value::value};
use itertools::Itertools;
use melodium_common::descriptor::{
    Attribuable, Documented, Generics, Identified, Identifier, Parameterized,
    Treatment as TreatmentDescriptor,
};
use melodium_engine::design::{Connection, Treatment as TreatmentDesign, IO};
use std::collections::{BTreeMap, HashMap, HashSet};

pub struct Treatment {
    design: TreatmentDesign,
    uses: Vec<Identifier>,
}

impl Treatment {
    pub fn new(design: TreatmentDesign) -> Self {
        let mut uses = design.descriptor.upgrade().unwrap().uses();

        uses.retain(|id| id != design.descriptor.upgrade().unwrap().identifier());

        Self { design, uses }
    }

    pub fn design(&self) -> &TreatmentDesign {
        &self.design
    }

    pub fn uses(&self) -> &Vec<Identifier> {
        &self.uses
    }

    pub fn implementation(&self, names: &BTreeMap<Identifier, String>) -> String {
        let descriptor = self.design.descriptor.upgrade().unwrap();

        let mut implementation = if descriptor.documentation().trim().is_empty() {
            String::new()
        } else {
            format!(
                "/**\n{}\n*/\n",
                descriptor
                    .documentation()
                    .lines()
                    .map(|l| format!("\t{l}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        for (name, attribute) in descriptor.attributes() {
            implementation.push_str("#[");
            implementation.push_str(name);
            implementation.push_str("(");
            implementation.push_str(&attribute);
            implementation.push_str(")]\n");
        }

        implementation.push_str("treatment ");
        implementation.push_str(descriptor.identifier().name());

        if !descriptor.generics().is_empty() {
            implementation.push('<');

            implementation.push_str(
                &descriptor
                    .generics()
                    .iter()
                    .map(|generic| {
                        if generic.traits.is_empty() {
                            generic.name.clone()
                        } else {
                            format!(
                                "{}: {}",
                                generic.name,
                                generic
                                    .traits
                                    .iter()
                                    .map(|tr| tr.to_string())
                                    .collect::<Vec<_>>()
                                    .join(" + ")
                            )
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            );

            implementation.push('>');
        }

        if !descriptor.models().is_empty() {
            implementation.push_str("[");

            implementation.push_str(
                &descriptor
                    .models()
                    .iter()
                    .sorted_by_key(|(k, _)| *k)
                    .map(|(name, model)| {
                        format!("{name}: {id}", id = names.get(model.identifier()).unwrap())
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            );

            implementation.push_str("]");
        }

        implementation.push_str("(");

        implementation.push_str(
            &descriptor
                .parameters()
                .iter()
                .sorted_by_key(|(k, _)| *k)
                .map(|(_, param)| {
                    format!(
                        "{attributes}\n    {variability} {name}: {param}{default}",
                        variability = param.variability(),
                        attributes = param
                            .attributes()
                            .iter()
                            .map(|(name, attribute)| format!("\n    #[{name}({attribute})]"))
                            .collect::<Vec<_>>()
                            .join(""),
                        name = param.name(),
                        param = describe_type(param.described_type(), names),
                        default = param
                            .default()
                            .as_ref()
                            .map(|v| format!(" = {}", value(&v.into(), names, 1)))
                            .unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join(","),
        );

        if !descriptor.parameters().is_empty() {
            implementation.push_str("\n");
        }

        implementation.push_str(")\n");

        for (_, context) in descriptor.contexts().iter().sorted_by_key(|(k, _)| *k) {
            implementation.push_str("  require ");
            implementation.push_str(names.get(context.identifier()).unwrap());
            implementation.push_str("\n");
        }

        for (_, input) in descriptor.inputs().iter().sorted_by_key(|(k, _)| *k) {
            for (name, attribute) in input.attributes() {
                implementation.push_str("  #[");
                implementation.push_str(name);
                implementation.push_str("(");
                implementation.push_str(&attribute);
                implementation.push_str(")]\n");
            }
            implementation.push_str("  input ");
            implementation.push_str(input.name());
            implementation.push_str(": ");
            implementation.push_str(&input.flow().to_string());
            implementation.push_str("<");
            implementation.push_str(&describe_type(input.described_type(), names));
            implementation.push_str(">\n");
        }

        for (_, output) in descriptor.outputs().iter().sorted_by_key(|(k, _)| *k) {
            for (name, attribute) in output.attributes() {
                implementation.push_str("  #[");
                implementation.push_str(name);
                implementation.push_str("(");
                implementation.push_str(&attribute);
                implementation.push_str(")]\n");
            }
            implementation.push_str("  output ");
            implementation.push_str(output.name());
            implementation.push_str(": ");
            implementation.push_str(&output.flow().to_string());
            implementation.push_str("<");
            implementation.push_str(&describe_type(output.described_type(), names));
            implementation.push_str(">\n");
        }

        for (_, model) in self
            .design
            .model_instanciations
            .iter()
            .sorted_by_key(|(k, _)| *k)
        {
            for (name, attribute) in model.attributes() {
                implementation.push_str("  #[");
                implementation.push_str(name);
                implementation.push_str("(");
                implementation.push_str(&attribute);
                implementation.push_str(")]\n");
            }
            implementation.push_str("  model ");
            implementation.push_str(&model.name);
            implementation.push_str(": ");
            implementation.push_str(
                names
                    .get(model.descriptor.upgrade().unwrap().identifier())
                    .unwrap(),
            );

            implementation.push_str("(");

            implementation.push_str(
                &model
                    .parameters
                    .iter()
                    .sorted_by_key(|(k, _)| *k)
                    .map(|(_, param)| {
                        format!(
                            "{name} = {value}",
                            name = param.name,
                            value = value(&param.value, names, 1)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
            );

            implementation.push_str(")\n");
        }

        implementation.push_str("{\n");

        for (_, instanciation) in self.design.treatments.iter().sorted_by_key(|(k, _)| *k) {
            let descriptor = instanciation.descriptor.upgrade().unwrap();

            for (name, attribute) in instanciation.attributes() {
                implementation.push_str("    #[");
                implementation.push_str(name);
                implementation.push_str("(");
                implementation.push_str(&attribute);
                implementation.push_str(")]\n");
            }
            implementation.push_str("    ");
            implementation.push_str(&instanciation.name);

            let treatment_name = names.get(descriptor.identifier()).unwrap();
            if treatment_name != &instanciation.name {
                implementation.push_str(": ");
                implementation.push_str(treatment_name);
            }

            if !descriptor.generics().is_empty() && !instanciation.generics.is_empty() {
                implementation.push('<');

                implementation.push_str(
                    &descriptor
                        .generics()
                        .iter()
                        .map(|generic| {
                            instanciation
                                .generics
                                .get(&generic.name)
                                .map(|desc_type| describe_type(desc_type, names))
                                .unwrap_or_else(|| "_".to_string())
                        })
                        .collect::<Vec<_>>()
                        .join(", "),
                );

                implementation.push('>');
            }

            if !instanciation.models.is_empty() {
                implementation.push_str("[");
                implementation.push_str(
                    &instanciation
                        .models
                        .iter()
                        .sorted_by_key(|(k, _)| *k)
                        .map(|(name, model)| format!("{name} = {model}"))
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                implementation.push_str("]");
            }

            implementation.push_str("(");

            implementation.push_str(
                &instanciation
                    .parameters
                    .iter()
                    .sorted_by_key(|(k, _)| *k)
                    .map(|(_, param)| {
                        format!(
                            "\n        {name} = {value}",
                            name = param.name,
                            value = value(&param.value, names, 3)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(","),
            );
            if !instanciation.parameters.is_empty() {
                implementation.push_str("\n    ");
            }
            implementation.push_str(")\n");
        }

        implementation.push_str("\n");

        implementation.push_str(&Self::render_connections(&self.design.connections));

        implementation.push_str("}\n\n");

        implementation
    }

    fn io_name(io: &IO) -> &str {
        match io {
            IO::Sequence() => "Self",
            IO::Treatment(name) => name,
        }
    }

    // Key identifying one side of a connection: (treatment_name, port_name).
    // Used as map keys — must be cheaply comparable.
    fn src_key(conn: &Connection) -> (String, String) {
        (
            Self::io_name(&conn.output_treatment).to_string(),
            conn.output_name.clone(),
        )
    }

    fn dst_key(conn: &Connection) -> (String, String) {
        (
            Self::io_name(&conn.input_treatment).to_string(),
            conn.input_name.clone(),
        )
    }

    fn render_connections(connections: &[Connection]) -> String {
        if connections.is_empty() {
            return String::new();
        }

        // out_edges: (src_treatment, src_output) -> sorted list of connection indices
        let mut out_edges: HashMap<(String, String), Vec<usize>> = HashMap::new();

        for (i, conn) in connections.iter().enumerate() {
            out_edges.entry(Self::src_key(conn)).or_default().push(i);
        }

        // Sort each fan-out list alphabetically by (receiver_treatment, receiver_input)
        for list in out_edges.values_mut() {
            list.sort_by(|&a, &b| {
                Self::dst_key(&connections[a]).cmp(&Self::dst_key(&connections[b]))
            });
        }

        // A connection is an inline continuation (non-root) iff:
        //   - it has no attributes
        //   - its (src_treatment, src_output) has exactly one outgoing connection (itself)
        //   - some attribute-free connection arrives at its src_treatment
        //     (meaning src is an intermediate node, not a source)
        let is_inline_continuation = |idx: usize| -> bool {
            let conn = &connections[idx];
            if !conn.attributes().is_empty() {
                return false;
            }
            let fan_out = out_edges
                .get(&Self::src_key(conn))
                .map(|v| v.len())
                .unwrap_or(0);
            if fan_out != 1 {
                return false;
            }
            let src_name = Self::io_name(&conn.output_treatment);
            connections
                .iter()
                .any(|p| p.attributes().is_empty() && Self::io_name(&p.input_treatment) == src_name)
        };

        // Roots: connections that start a new chain line.
        // Ordered: Self-sourced first, then by (src_treatment, src_output, dst_treatment, dst_input).
        let mut roots: Vec<usize> = (0..connections.len())
            .filter(|&i| !is_inline_continuation(i))
            .collect();

        roots.sort_by(|&a, &b| {
            let ca = &connections[a];
            let cb = &connections[b];
            let a_self = matches!(ca.output_treatment, IO::Sequence());
            let b_self = matches!(cb.output_treatment, IO::Sequence());
            b_self
                .cmp(&a_self)
                .then_with(|| Self::src_key(ca).cmp(&Self::src_key(cb)))
                .then_with(|| Self::dst_key(ca).cmp(&Self::dst_key(cb)))
        });

        let mut visited: HashSet<usize> = HashSet::new();
        let mut output = String::new();

        for root_idx in roots {
            if visited.contains(&root_idx) {
                continue;
            }
            Self::render_chain(
                root_idx,
                connections,
                &out_edges,
                &mut visited,
                &mut output,
                "    ",
                0,
            );
        }

        // Safety net: emit any connection missed by the greedy walk as a plain line.
        for i in 0..connections.len() {
            if !visited.contains(&i) {
                let conn = &connections[i];
                for (name, attribute) in conn.attributes() {
                    output.push_str("    #[");
                    output.push_str(name);
                    output.push('(');
                    output.push_str(attribute);
                    output.push_str(")]\n");
                }
                output.push_str("    ");
                output.push_str(Self::io_name(&conn.output_treatment));
                output.push('.');
                output.push_str(&conn.output_name);
                output.push_str(" -> ");
                output.push_str(Self::io_name(&conn.input_treatment));
                output.push('.');
                output.push_str(&conn.input_name);
                output.push('\n');
                visited.insert(i);
            }
        }

        output
    }

    // Renders one chain starting at `start_idx`.
    // `align_col`: when >0, pad the source name to this width (fan-out group continuation).
    fn render_chain(
        start_idx: usize,
        connections: &[Connection],
        out_edges: &HashMap<(String, String), Vec<usize>>,
        visited: &mut HashSet<usize>,
        output: &mut String,
        indent: &str,
        align_col: usize,
    ) {
        if visited.contains(&start_idx) {
            return;
        }

        let conn = &connections[start_idx];
        for (name, attribute) in conn.attributes() {
            output.push_str(indent);
            output.push_str("#[");
            output.push_str(name);
            output.push('(');
            output.push_str(attribute);
            output.push_str(")]\n");
        }

        let src_part = format!(
            "{}.{}",
            Self::io_name(&conn.output_treatment),
            conn.output_name
        );

        let mut chain_line = if align_col > 0 && src_part.len() < align_col {
            format!("{:<width$}", src_part, width = align_col)
        } else {
            src_part
        };

        let mut current_idx = start_idx;

        loop {
            visited.insert(current_idx);
            let cur = &connections[current_idx];
            let dst_name = Self::io_name(&cur.input_treatment);

            chain_line.push_str(" -> ");
            chain_line.push_str(dst_name);
            chain_line.push('.');
            chain_line.push_str(&cur.input_name);

            // Collect all output ports of dst_name that have outgoing connections
            let successors: Vec<(String, Vec<usize>)> = out_edges
                .iter()
                .filter(|((t, _), _)| t == dst_name)
                .map(|((_, port), idxs)| (port.clone(), idxs.clone()))
                .sorted_by_key(|(port, _)| port.clone())
                .collect();

            if successors.is_empty() {
                break;
            }

            if successors.len() == 1 {
                let (ref out_port, ref idxs) = successors[0];
                if idxs.len() == 1 && connections[idxs[0]].attributes().is_empty() {
                    // Single unambiguous continuation — extend inline
                    chain_line.push(',');
                    chain_line.push_str(out_port);
                    current_idx = idxs[0];
                    continue;
                } else {
                    // Single port but fan-out or attribute break
                    chain_line.push(',');
                    chain_line.push_str(out_port);
                    output.push_str(indent);
                    output.push_str(&chain_line);
                    output.push('\n');
                    let fan_src = format!("{}.{}", dst_name, out_port);
                    let idxs = idxs.clone();
                    Self::render_fanout_group(
                        &fan_src,
                        &idxs,
                        connections,
                        out_edges,
                        visited,
                        output,
                        indent,
                    );
                    return;
                }
            } else {
                // Multiple output ports — end chain, render each as a group
                output.push_str(indent);
                output.push_str(&chain_line);
                output.push('\n');
                for (out_port, idxs) in &successors {
                    let fan_src = format!("{}.{}", dst_name, out_port);
                    Self::render_fanout_group(
                        &fan_src,
                        idxs,
                        connections,
                        out_edges,
                        visited,
                        output,
                        indent,
                    );
                }
                return;
            }
        }

        output.push_str(indent);
        output.push_str(&chain_line);
        output.push('\n');
    }

    // Renders all branches of a fan-out group, with their source names padded to the
    // same column so arrows align visually.
    fn render_fanout_group(
        src_port: &str,
        idxs: &[usize],
        connections: &[Connection],
        out_edges: &HashMap<(String, String), Vec<usize>>,
        visited: &mut HashSet<usize>,
        output: &mut String,
        indent: &str,
    ) {
        let src_len = src_port.len();
        let branch_indices: Vec<usize> = idxs
            .iter()
            .filter(|&&i| !visited.contains(&i))
            .copied()
            .collect();

        for idx in branch_indices {
            if visited.contains(&idx) {
                continue;
            }
            let conn = &connections[idx];
            for (name, attribute) in conn.attributes() {
                output.push_str(indent);
                output.push_str(&" ".repeat(src_len + 1));
                output.push_str("#[");
                output.push_str(name);
                output.push('(');
                output.push_str(attribute);
                output.push_str(")]\n");
            }
            Self::render_chain(
                idx,
                connections,
                out_edges,
                visited,
                output,
                indent,
                src_len,
            );
        }
    }
}
