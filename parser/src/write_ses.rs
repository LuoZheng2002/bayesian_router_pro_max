use crate::dsn_struct::{DsnStruct, Library, Network, Shape};
use shared::pcb_problem::{FixedTrace, PcbSolution};
use shared::trace_path::Via;
use shared::vec2::FixedVec2;
use std::collections::HashMap;
use std::fs::File;
// use std::io::{Result, Write};
// use std::fmt::Result;
use std::fmt::Write;

fn generate_placement(dsn: &DsnStruct) -> Result<String, String> {
    let mut ses = String::new();

    writeln!(ses, "  (placement").unwrap();
    writeln!(
        ses,
        "    (resolution {} 1)",
        dsn.resolution.unit, // dsn.resolution.value
    ).unwrap();
    for component in &dsn.placement.components {
        writeln!(ses, "    (component \"{}\"", component.name).unwrap();

        for inst in &component.instances {
            writeln!(
                ses,
                "      (place {} {:.6} {:.6} {} {:.6})",
                inst.reference,
                inst.position.x,
                inst.position.y,
                inst.placement_layer.as_str(),
                inst.rotation
            ).unwrap();
        }
        writeln!(ses, "    )\n").unwrap();
    }
    writeln!(ses, "  )\n").unwrap();
    Ok(ses)
}

pub struct ViaSES {
    name: String,
    shape: String,
    through_hole: bool,
    diameter: f32,
}

impl ViaSES {
    fn to_ses_string(&self, layers: &[String]) -> String {
        let shape = &self.shape;
        let dia_int = self.diameter.round() as i32;
        let mut s = format!("      (padstack \"{}\"\n", self.name);

        if self.through_hole {
            for layer in layers {
                s += &format!(
                    "        (shape\n          ({} {} {} 0 0)\n        )\n",
                    shape, layer, dia_int
                );
            }
        } else {
            let layer = &layers[0];
            s += &format!(
                "        (shape\n          ({} {} {} 0 0)\n        )\n",
                shape, layer, dia_int
            );
        }

        s += &format!("        (attach off)\n      )\n");

        s
    }
}

fn via_info(dsn: &DsnStruct) -> Vec<ViaSES> {
    dsn.library
        .pad_stacks
        .iter()
        .filter_map(|(name, pad)| {
            if name.starts_with("Via") {
                if let Shape::Circle { diameter } = pad.shape {
                    Some(ViaSES {
                        name: name.clone(),
                        shape: "circle".to_string(),
                        through_hole: pad.through_hole,
                        diameter,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

fn extract_fixed_vec2(v: &FixedVec2) -> (f32, f32) {
    (v.x.to_num::<f32>(), v.y.to_num::<f32>())
}

fn find_via_name(netname: &String, dsn: &DsnStruct) -> Option<String> {
    for netclass in dsn.network.netclasses.values() {
        if netclass.net_names.contains(netname) {
            return Some(netclass.via_name.clone());
        }
    }
    None
}

fn generate_network(
    dsn: &DsnStruct,
    solution: &PcbSolution,
    layers: &Vec<String>,
    vias: &Vec<ViaSES>,
) -> Result<String, String> {
    // This function will generate the network information based on the PcbProblem and PcbSolution
    // The implementation will depend on the specific requirements of the network format
    let mut ses = String::new();
    let scale_down_factor = solution.scale_down_factor;

    let mut nets: HashMap<&String, Vec<&FixedTrace>> = HashMap::new();
    for trace in solution.determined_traces.values() {
        nets.entry(&trace.net_name.0).or_default().push(trace);
    }

    for (net_name, traces) in nets {
        writeln!(ses, "  (net \"{}\"", net_name).unwrap();
        let via_name = find_via_name(&net_name, &dsn).unwrap_or("default_via".to_string());

        for trace in traces {
            for via in &trace.trace_path.vias {
                let (x, y) = extract_fixed_vec2(&via.position);
                writeln!(ses, "    (via {} {} {})", via_name, x * scale_down_factor, y * scale_down_factor).unwrap();
            }
            for segment in &trace.trace_path.segments {
                let (start_x, start_y) = extract_fixed_vec2(&segment.start);
                let (end_x, end_y) = extract_fixed_vec2(&segment.end);
                let layer_name = layers[segment.layer].as_str();
                writeln!(
                    ses,
                    "        (wire\n          (path {} {}\n            {} {}\n            {} {}))",
                    layer_name, // 0 = front, highest = back
                    segment.width * scale_down_factor,
                    start_x * scale_down_factor,
                    start_y * scale_down_factor,
                    end_x * scale_down_factor,
                    end_y * scale_down_factor
                ).unwrap();
            }
        }
        writeln!(ses, "    )").unwrap();
    }
    writeln!(ses, "  )").unwrap();
    Ok(ses)
}

pub fn write_ses_to_string(dsn: &DsnStruct, solution: &PcbSolution) -> Result<String, String> {
    // This function will convert the PcbProblem and PcbSolution into a SES format string
    // The implementation will depend on the specific requirements of the SES format
    // let mut ses = File::create(output.to_string() + ".ses")?;
    let mut ses = String::new();
    // let ses = &mut ses;

    let layer_names: Vec<String> = dsn.get_layer_names();

    writeln!(ses, "(session bayesian_router_output.ses)").unwrap();
    writeln!(ses, "  (base_design dont_know.dsn)").unwrap();

    let placement = generate_placement(&dsn)?;
    writeln!(ses, "{}", placement).unwrap();

    writeln!(ses, "  (was_is").unwrap();
    writeln!(ses, "  )").unwrap();
    writeln!(ses, "  (routes").unwrap();
    writeln!(
        ses,
        "    (resolution {} 1)",
        dsn.resolution.unit// , dsn.resolution.value
    ).unwrap();
    writeln!(ses, "    (parser").unwrap();
    writeln!(ses, "      (host_cad \"KiCad's Pcbnew\")").unwrap();
    writeln!(ses, "      (host_version 9.0.2)").unwrap();
    writeln!(ses, "    )").unwrap();

    // via
    writeln!(ses, "    (library_out").unwrap();
    let vias = via_info(&dsn);
    for via in &vias {
        let via_str = via.to_ses_string(&layer_names);
        write!(ses, "{}", via_str).unwrap();
    }
    writeln!(ses, "    )").unwrap();

    // net
    writeln!(ses, "    (network_out").unwrap();

    let network = generate_network(&dsn, &solution, &layer_names, &vias)?;
    writeln!(ses, "{}", network).unwrap();
    writeln!(ses, "  )").unwrap();
    writeln!(ses, ")").unwrap();
    Ok(ses)
}
